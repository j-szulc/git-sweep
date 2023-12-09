use std::collections::HashMap;
use std::fmt::Error;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use structopt::StructOpt;
use trash;
use std::process::{Command, Stdio};
use serde::{Serialize, Deserialize};
use serde_json;
extern crate inquire;

/// Move files to Trash.
#[derive(StructOpt)]
#[structopt(name = "lazy-git-clean")]
struct Opt {

    /// Ignore config files
    #[structopt(short = "-f", long = "--overwrite-config")]
    overwrite_config: bool,

    /// Repo folders to process
    #[structopt(name = "REPOS", parse(from_os_str))]
    repos: Vec<PathBuf>
}


#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
enum RepoConfig {
    ReadWrite,
    ReadOnly,
    LeaveAloneForNow,
    LeaveAloneForever,
    ContinueReadWriteForNow,
    ContinueReadOnlyForNow,
}

impl RepoConfig{
    fn is_leave_alone(&self) -> bool {
        match self {
            RepoConfig::LeaveAloneForNow | RepoConfig::LeaveAloneForever => true,
            _ => false
        }
    }
    fn is_temporary(&self) -> bool {
        match self {
            RepoConfig::LeaveAloneForNow | RepoConfig::ContinueReadWriteForNow | RepoConfig::ContinueReadOnlyForNow => true,
            _ => false
        }
    }
    fn is_read_only(&self) -> bool {
        match self {
            RepoConfig::ReadOnly | RepoConfig::ContinueReadOnlyForNow => true,
            _ => false
        }
    }
}

fn check_repo_clean(path: &Path, repo_config: RepoConfig) -> Result<(), String> {
    let mut errors = Vec::new();

    let git_status_out = Command::new("git").arg("status").arg("--porcelain").current_dir(path).output().unwrap();
    if git_status_out.stdout.len() > 0 {
        // print error
        errors.push(String::from_utf8(git_status_out.stdout).unwrap());
    }

    // Check if remote up to date
    if !repo_config.is_read_only() {
        let git_push_out = Command::new("git").arg("push").arg("--dry-run").current_dir(path).output().unwrap();
        let git_push_stderr = String::from_utf8(git_push_out.stderr).unwrap();
        if git_push_out.stdout.len() > 0 || git_push_stderr != "Everything up-to-date\n" {
            // print error
            let git_push_stdout = String::from_utf8(git_push_out.stdout).unwrap();
            errors.push(format!("Remote not up to date. Stdout: {}, Stderr: {}", git_push_stdout, git_push_stderr));
        }
    }


    if errors.len() > 0 {
        return Err(errors.join("\n"));
    }
    Ok(())
}

fn check_repo_clean_verbose(path: &Path, repo_config: RepoConfig) -> bool{
    let path_display = path.display();
    match check_repo_clean(path, repo_config) {
        Ok(()) => {
            eprintln!("{path_display}: Repo is clean");
            true
        },
        Err(e) => {
            eprintln!("{path_display}: Repo is not clean");
            eprintln!("{}", e);
            false
        }
    }
}

fn load_repo_config(path: &Path, opt_overwrite_config: bool) -> Result<RepoConfig, Error> {
    let repo_config_path = path.join(".git/lazy-git-clean.json");

    if !opt_overwrite_config{
        match File::open(repo_config_path.clone()) {
            Ok(mut file) => {
                let mut contents = String::new();
                file.read_to_string(&mut contents).unwrap();
                return Ok(serde_json::from_str(&contents).unwrap());
            },
            Err(_) => {}
        }
    }

    let options = vec![
        ("Read/Write (default)", RepoConfig::ReadWrite),
        ("Read Only (no push permissions)", RepoConfig::ReadOnly),
        ("Leave alone for now", RepoConfig::LeaveAloneForNow),
        ("Leave alone forever", RepoConfig::LeaveAloneForever),
        ("Continue Read/Write for now", RepoConfig::ContinueReadWriteForNow),
        ("Continue Read Only for now", RepoConfig::ContinueReadOnlyForNow)
    ];
    let options_str = options.iter().map(|(k,_)| *k).collect::<Vec<_>>();
    let options_hashmap : HashMap<String, RepoConfig> = options.iter().map(|(k,v)| (k.to_string(), *v)).collect();

    let selected = inquire::Select::new("Repo mode?",options_str).prompt().unwrap();
    let repo_config = options_hashmap.get(selected).unwrap();

    if !repo_config.is_temporary() {
        let mut file = File::create(repo_config_path).unwrap();
        file.write_all(serde_json::to_string(repo_config).unwrap().as_bytes()).unwrap();
    }

    Ok(*repo_config)
}

fn main() {
    // Check if lazygit installed
    if !Command::new("which").arg("lazygit").stdout(Stdio::null()).status().unwrap().success() {
        eprintln!("lazy-git-clean requires lazygit to be installed");
        std::process::exit(1);
    }
    let opt = Opt::from_args();
    for repo in opt.repos {
        let path = repo.as_path();
        let path_display = path.display();

        // First checks

        eprintln!("Processing {path_display}");

        if !path.exists() {
            // print error
            eprintln!("{path_display}: No such file or directory");
            continue;
        }
        if !path.join(".git").exists() {
            // print error
            eprintln!("{path_display}: Not a git repository");
            match inquire::Confirm::new("Delete?").with_default(true).prompt(){
                Ok(true) => {},
                Ok(false) => continue,
                Err(e) => {
                    eprintln!("Error: {e}");
                    std::process::exit(1);
                }
            }
            trash::delete(path).unwrap();
            continue;
        }

        let ls_remote_status = Command::new("git").arg("ls-remote").current_dir(path).status().unwrap();
        if !ls_remote_status.success() {
            // print error
            eprintln!("{path_display}: Remote not available, exit code {}", ls_remote_status);
            std::process::exit(1);
        }

        let repo_config = load_repo_config(path, opt.overwrite_config).unwrap();
        if repo_config.is_leave_alone() {
            eprintln!("{path_display}: Repo is set to leave alone");
            check_repo_clean_verbose(path, repo_config);
            continue;
        }

        // Check if repo is clean
        if check_repo_clean(path, repo_config).is_err() {
            let status = Command::new("lazygit").arg("--path").arg(path).status().unwrap();
            if !status.success() {
                // print error
                eprintln!("{path_display}: lazygit failed with exit code {status}");
                continue;
            }
        }

        match inquire::Confirm::new("Delete?").with_default(true).prompt(){
            Ok(true) => {},
            Ok(false) => continue,
            Err(e) => {
                eprintln!("Error: {e}");
                std::process::exit(1);
            }
        }

        if !check_repo_clean_verbose(path, repo_config) {
            eprintln!("REPO IS STILL NOT CLEAN!");
            match inquire::Confirm::new("DELETE ANYWAY?").with_default(false).prompt(){
                Ok(true) => {},
                Ok(false) => continue,
                Err(e) => {
                    eprintln!("Error: {e}");
                    std::process::exit(1);
                }
            }
        }

        trash::delete(path).unwrap();
    }
}