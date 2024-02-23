mod git_utils;

use colored::Colorize;
use git2::Repository;
use rand::seq::SliceRandom;
use std::fmt::Display;
use std::path::{Path, PathBuf};
use structopt::StructOpt;
use trash;
use crate::git_utils::is_local_dirty;

type Error = Box<dyn std::error::Error>;

/// Move files to Trash.
#[derive(StructOpt)]
#[structopt(name = "lazy-git-clean")]
struct Opt {
    /// Repo folders to process
    #[structopt(name = "REPOS", parse(from_os_str))]
    repos: Vec<PathBuf>,
}

fn bool_to_checkmark(b: bool) -> &'static str {
    if b {
        "✅"
    } else {
        "❌"
    }
}

fn process_repo(path: &Path) -> Result<bool, Error> {
    let repo = Repository::open(path)?;
    let mut msgs = vec![];
    if is_local_dirty(&repo)? {
        msgs.push("Dirty local index".to_string());
    }
    let remotes = git_utils::get_all_remotes(&repo, true)?;
    for remote in remotes {
        let name_str = remote.name().unwrap_or("unnamed remote").to_string();
        let status = git_utils::is_remote_up_to_date(&repo, remote);
        match status {
            Ok(git_utils::RemoteStatus::LocalAhead) => {
                msgs.push(format!("Ahead of {}", name_str));
            }
            Err(e) => {
                msgs.push(format!("Error: {}", e));
            }
            _ => {}
        }
    }
    println!(
        "{checkmark} {path} {reason}",
        checkmark = bool_to_checkmark(msgs.is_empty()),
        path = path.to_str().unwrap(),
        reason = msgs.join(", ")
    );

    Ok(msgs.is_empty())
}
fn main() -> Result<(), Error>{
    let opt = Opt::from_args();
    for repo in opt.repos {
        let path = repo.as_path();
        process_repo(path)?;
        //     Ok(true) => {
        //         trash::delete(path).unwrap();
        //     }
        //     Ok(false) => {}
        //     Err(e) => {
        //         eprintln!("{}", e.to_string().red());
        //         if inquire::Confirm::new("Do you want to delete repository anyway?")
        //             .prompt()
        //             .unwrap()
        //         {
        //             if inquire::Confirm::new("Are you sure?").prompt().unwrap() {
        //                 trash::delete(path).unwrap();
        //             }
        //         } else {
        //             break;
        //         }
        //     }
        // }
    }
    Ok(())
}
