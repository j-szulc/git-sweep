mod cli_utils;
mod git_utils;

use crate::cli_utils::{bool_to_checkmark, print_subsection};
use crate::git_utils::safe_to_delete;
use colored::Colorize;
use git2::Repository;
use rand::thread_rng;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::vec;
use structopt::StructOpt;
use trash;

pub(crate) type Error = Box<dyn std::error::Error>;

/// Move files to Trash.
#[derive(StructOpt)]
#[structopt(name = "lazy-git-clean")]
struct Opt {
    /// Repo folders to process
    #[structopt(name = "REPOS", parse(from_os_str))]
    repos: Vec<PathBuf>,
}

fn is_repo_clean_verbose(repo: &Repository) -> Result<bool, Error> {
    let remotes = git_utils::get_all_remotes(&repo, true)?;
    let (remotes_clean, remotes_clean_msg) = cli_utils::check_verbose(remotes, |remote| {
        Ok(git_utils::is_remote_up_to_date(&repo, remote)?.is_clean())
    });
    print_subsection(remotes_clean_msg, 5, 4);

    let statuses = repo.statuses(None)?;
    let mut unsafe_files = statuses.iter()
        .filter(|x| unsafe_to_delete(x.status()))
        .map(|x| x.path().unwrap().to_string())
        .collect::<Vec<_>>();
    unsafe_files.shuffle(&mut thread_rng());


    let mut ignored_files = statuses.iter()
        .filter(|x| x.status().is_ignored())
        .map(|x| x.path().unwrap().to_string())
        .collect::<Vec<_>>();
    ignored_files.shuffle(&mut thread_rng());

    println!("{} Not ignored files clean", bool_to_checkmark(unsafe_files.is_empty()));
    print_subsection(unsafe_files.iter(), 5, 4);
    println!("✅ The following ignored files will be deleted:");
    print_subsection(ignored_files, 5, 4);

    Ok(RepoStatus{
        remotes_clean,
        files_clean: unsafe_files.is_empty()
    })
}

fn process_repo(path: &Path) -> Result<bool, Error> {
    let repo = Repository::open(path)?;
    let mut changed = true;
    let mut status = get_repo_status_verbose(&repo)?;
    let mut used_lazygit = false;
    let mut first_run = true;

    while changed {
        changed = false;
        if !first_run {
            status = get_repo_status_verbose(&repo)?;
        }
        first_run = false;
        if status.is_clean() {
            break;
        }
        if !used_lazygit && which("lazygit") {
            used_lazygit = true;
            if !inquire::Confirm::new("Do you want to use lazygit?")
                .prompt()
                .unwrap()
            {
                continue;
            }
            let mut cmd = Command::new("lazygit");
            cmd.current_dir(path);
            let _ = cmd.status()?;
            changed = true;
            continue;
        }
    }

    let answer_first = inquire::Confirm::new(&format!(
        "Repo is {}. Do you want to delete it?",
        status.is_clean_str()
    ))
    .prompt()
    .unwrap();
    if !answer_first {
        return Ok(false);
    }
    if !status.is_clean() {
        let answer_second =
            inquire::Confirm::new("Repo is not clean. Do you want to delete it anyway?")
                .prompt()
                .unwrap();
        if !answer_second {
            return Ok(false);
        }
    }

    Ok(true)
}
fn main() {
    let opt = Opt::from_args();
    for repo in opt.repos {
        let path = repo.as_path();
        println!("Processing {path}", path = path.to_str().unwrap());
        match process_repo(path) {
            Ok(true) => {
                trash::delete(path).unwrap();
            }
            Ok(false) => {}
            Err(e) => {
                eprintln!("{}", e.to_string().red());
                if inquire::Confirm::new("Do you want to delete repository anyway?")
                    .prompt()
                    .unwrap()
                {
                    if inquire::Confirm::new("Are you sure?").prompt().unwrap() {
                        trash::delete(path).unwrap();
                    }
                } else {
                    break;
                }
            }
        }
    }
}
