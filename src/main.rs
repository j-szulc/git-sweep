mod git_utils;

use std::collections::HashSet;
use colored::Colorize;
use git2::Repository;
use std::fmt::Display;
use std::hash::Hash;
use std::path::{Path, PathBuf};
use rayon::iter::{ParallelIterator, IntoParallelIterator};
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

fn multiselect<'a, T, V>(prompt: &str, pairs: &'a Vec<(T, V)>) -> Result<Vec<&'a V>, Error>
where T: Display + Eq + Hash{
    let items: Vec<&T> = pairs.iter().map(|(t, _)| t).collect();
    let result = inquire::MultiSelect::new(prompt, items).prompt()?;
    let result: HashSet<&T> = HashSet::from_iter(result);
    let mut out = vec![];
    for (item, value) in pairs {
        if result.contains(item) {
            out.push(value);
        }
    }
    Ok(out)
}

fn process_repo(path: &Path) -> Result<(bool, String), Error> {
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
    let result = format!(
        "{checkmark} {path} {reason}",
        checkmark = bool_to_checkmark(msgs.is_empty()),
        path = path.to_str().unwrap(),
        reason = msgs.join(", ")
    );
    Ok((msgs.is_empty(), result))
}

fn main() -> Result<(), Error>{
    let opt = Opt::from_args();
    let process_results : Vec<(PathBuf, bool)> = opt.repos.into_par_iter().map(|repo| {
        let path = repo.as_path();
        let clean = match process_repo(path) {
            Ok((clean, msg)) => {
                println!("{}", msg);
                clean
            }
            Err(e) => {
                eprintln!("{}: {}", path.to_string_lossy().red(), e);
                false
            }
        };
        (repo, clean)
    }).collect();
    let safe_to_delete : Vec<(&str, &PathBuf)> = process_results.iter().filter(|(_, clean)| *clean).map(|(path, _)| (path.to_str().unwrap(), path)).collect();
    let result = multiselect("Select repos to delete", &safe_to_delete)?;
    let result : Vec<&Path> = result.iter().map(|p| p.as_path()).collect();
    for path in result {
        trash::delete(path)?;
    }
    Ok(())
}
