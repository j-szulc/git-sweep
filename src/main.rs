mod git_utils;

use std::collections::HashMap;
use std::fmt::Display;
use std::fs::File;
use std::io::{Read, Write};
use std::os::macos::raw::stat;
use std::path::{Path, PathBuf};
use structopt::StructOpt;
use trash;
use std::process::{Command, Stdio};
use git2::{Status, StatusEntry};
use maplit::{hashmap};
use serde::{Serialize, Deserialize};
use serde_json;
use multipeek::multipeek;
use colored::Colorize;


extern crate inquire;

type Error = Box<dyn std::error::Error>;

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


fn inquire_select<T: Clone>(prompt: &str, options: &HashMap<&str, T>) -> T {
    let options_str = options.keys().collect::<Vec<&&str>>();
    let selected = inquire::Select::new(prompt,options_str).prompt().unwrap();
    options.get(selected).unwrap().clone()
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

    let options : HashMap<&str, RepoConfig> = hashmap!{
        "Read/Write (default)" => RepoConfig::ReadWrite,
        "Read Only (no push permissions)" => RepoConfig::ReadOnly,
        "Leave alone for now" => RepoConfig::LeaveAloneForNow,
        "Leave alone forever" => RepoConfig::LeaveAloneForever,
        "Continue Read/Write for now" => RepoConfig::ContinueReadWriteForNow,
        "Continue Read Only for now" => RepoConfig::ContinueReadOnlyForNow
    };
    let repo_config = inquire_select("Select repo config", &options);
    if !repo_config.is_temporary() {
        let mut file = File::create(repo_config_path).unwrap();
        file.write_all(serde_json::to_string(&repo_config).unwrap().as_bytes()).unwrap();
    }

    Ok(repo_config)
}

fn bool_to_checkmark(b: bool) -> &'static str {
    if b {
        "✅"
    } else {
        "❌"
    }
}

fn print_subsection<Item: Display, Container: IntoIterator<Item=Item>> (items: Container, limit: usize, indent: usize) {
    let mut items = multipeek(items.into_iter());
    let mut count = 0;
    while let Some(item) = items.next() {
        if count >= limit && items.peek_nth(2).is_some() {
            println!("{}... {} more", " ".repeat(indent), items.count());
            break;
        }
        println!("{}{}", " ".repeat(indent), item);
        count += 1;
    }
}

fn process_repo(path: &Path) -> Result<(), Error> {
    let repo = git2::Repository::open(path)?;
    let mut remotes = git_utils::get_all_remotes(&repo, true)?;
    let remotes_status : Vec<Result<bool, Error>> = remotes.into_iter().map(|x| git_utils::is_remote_up_to_date(&repo, x)).collect();
    let remotes_clean = remotes_status.iter().all(|x| *x.as_ref().ok().unwrap_or(&false));
    println!("{} Remotes up to date", bool_to_checkmark(remotes_clean));

    let unsafe_to_delete = |status : Status| {
        status.is_wt_new() || status.is_wt_modified() || status.is_index_new() || status.is_index_modified()
    };

    let statuses = repo.statuses(None)?;
    let (unsafe_files_ignored, unsafe_files_not_ignored): (Vec<StatusEntry>, Vec<StatusEntry>) =
        statuses.iter()
        .filter(|x| unsafe_to_delete(x.status()))
        .partition(|x| x.status().is_ignored());

    println!("{} Not ignored files clean", bool_to_checkmark(unsafe_files_not_ignored.is_empty()));
    print_subsection(unsafe_files_not_ignored.iter().map(|x| x.path().unwrap()), 5, 4);
    println!("✅ Ignored files clean");
    print_subsection(unsafe_files_ignored.iter().map(|x| x.path().unwrap()), 5, 4);
    Ok(())
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
        println!("Processing {path}", path=path.to_str().unwrap());
        match process_repo(path) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("{}", e.to_string().red());
            }
        }
    }
}