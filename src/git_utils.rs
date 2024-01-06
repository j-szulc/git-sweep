use std::path::{Path, PathBuf};
use git2::{Remote, Status};
use crate::Error;

fn get_remote_callbacks<'a>() -> Result<git2::RemoteCallbacks<'a>, Error> {
    let mut callbacks = git2::RemoteCallbacks::new();
    callbacks.credentials(move |url, username_from_url, allowed_types| {
        let username = username_from_url.unwrap_or("git");
        git2::Cred::ssh_key_from_agent(username)
    });
    Ok(callbacks)
}

pub(crate) fn get_all_remotes(repo: &git2::Repository, connect: bool) -> Result<Vec<Remote>, Error> {
    let mut result = vec![];
    for remote_str in repo.remotes()?.iter() {
        let remote_str = remote_str.ok_or("Remote name is not a valid UTF-8 string")?;
        let mut remote = repo.find_remote(remote_str)?;
        if connect {
            remote.connect_auth(git2::Direction::Fetch, Some(get_remote_callbacks()?), None)?;
        }
        result.push(remote);
    }
    Ok(result)
}

pub(crate) fn is_remote_up_to_date(repo: &git2::Repository, mut remote: Remote) -> Result<bool, Error> {
    remote.update_tips(None, false, git2::AutotagOption::Unspecified, None)?;
    let remote_head = remote.list()?.iter().filter(|x| x.name() == "HEAD").next().ok_or("Remote HEAD not found")?.oid();

    let default_branch = remote.default_branch()?;
    let default_branch = default_branch.as_str().unwrap();

    let local_head = repo.refname_to_id(default_branch)?;
    Ok(local_head == remote_head)
}
