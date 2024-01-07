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
use git2::{Repository, Status, StatusEntry};
use maplit::{hashmap};
use serde::{Serialize, Deserialize};
use serde_json;
use multipeek::multipeek;
use colored::Colorize;
use rand::{thread_rng, Rng};
use rand::seq::SliceRandom;


extern crate inquire;

type Error = Box<dyn std::error::Error>;

/// Move files to Trash.
#[derive(StructOpt)]
#[structopt(name = "lazy-git-clean")]
struct Opt {

    /// Repo folders to process
    #[structopt(name = "REPOS", parse(from_os_str))]
    repos: Vec<PathBuf>
}

fn inquire_select<'a, T>(prompt: &str, options: &'a Vec<(&str, T)>) -> &'a T {
    let options_str = options.iter().map(|x| x.0).collect::<Vec<_>>();
    let selected = inquire::Select::new(prompt,options_str).prompt().unwrap().to_string();
    let selected = options.iter().filter(|x| x.0 == selected).next().unwrap();
    &selected.1
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

struct RepoStatus{
    remotes_clean: bool,
    files_clean: bool
}


impl RepoStatus {
    fn is_clean(&self) -> bool {
        self.remotes_clean && self.files_clean
    }

    fn is_clean_str(&self) -> &'static str {
        if self.is_clean() {
            "clean"
        } else {
            "not clean"
        }
    }

}
fn get_repo_status_verbose(repo: &Repository) -> Result<RepoStatus, Error> {
    let mut remotes = git_utils::get_all_remotes(&repo, true)?;
    let remotes_status : Vec<Result<bool, Error>> = remotes.into_iter().map(|x| git_utils::is_remote_up_to_date(&repo, x)).collect();
    let remotes_clean = remotes_status.iter().all(|x| *x.as_ref().ok().unwrap_or(&false));
    println!("{} Remotes up to date", bool_to_checkmark(remotes_clean));

    let unsafe_to_delete = |status : Status| {
        !status.is_ignored() &&
            (status.is_wt_new() || status.is_wt_modified() || status.is_index_new() || status.is_index_modified() || status.is_conflicted())
    };

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

fn which(bin: &str) -> bool {
    Command::new("which").arg(bin).stdout(Stdio::null()).status().unwrap().success()
}

fn process_repo(path: &Path) -> Result<bool, Error> {
    let repo = Repository::open(path)?;
    let mut changed = true;
    let mut status = get_repo_status_verbose(&repo)?;
    let mut used_lazygit = false;
    let mut first_run = true;

    while changed {
        changed = false;
        if !first_run{
            status = get_repo_status_verbose(&repo)?;
        }
        first_run = false;
        if status.is_clean() {
            break;
        }
        if !used_lazygit && which("lazygit") {
            used_lazygit = true;
            if !inquire::Confirm::new("Do you want to use lazygit?").prompt().unwrap() {
                continue;
            }
            let mut cmd = Command::new("lazygit");
            cmd.current_dir(path);
            let _ = cmd.status()?;
            changed = true;
            continue;
        }
    }

    let answer_first = inquire::Confirm::new(&format!("Repo is {}. Do you want to delete it?", status.is_clean_str())).prompt().unwrap();
    if !answer_first{
        return Ok(false);
    }
    if !status.is_clean() {
        let answer_second = inquire::Confirm::new("Repo is not clean. Do you want to delete it anyway?").prompt().unwrap();
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
        println!("Processing {path}", path=path.to_str().unwrap());
        match process_repo(path) {
            Ok(true) => {
                trash::delete(path).unwrap();
            }
            Ok(false) => {}
            Err(e) => {
                eprintln!("{}", e.to_string().red());
                if inquire::Confirm::new("Do you want to delete repository anyway?").prompt().unwrap() {
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