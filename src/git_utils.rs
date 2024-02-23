use crate::Error;
use git2::Remote;

fn get_remote_callbacks<'a>() -> Result<git2::RemoteCallbacks<'a>, Error> {
    let mut callbacks = git2::RemoteCallbacks::new();
    callbacks.credentials(move |_, username_from_url, _| {
        let username = username_from_url.unwrap_or("git");
        git2::Cred::ssh_key_from_agent(username)
    });
    Ok(callbacks)
}

pub(crate) fn get_all_remotes(
    repo: &git2::Repository,
    connect: bool,
) -> Result<Vec<Remote>, Error> {
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

#[derive(Debug)]
pub(crate) enum RemoteStatus {
    LocalAhead,
    LocalBehind,
    UpToDate,
}

pub(crate) fn is_remote_up_to_date(
    repo: &git2::Repository,
    mut remote: Remote,
) -> Result<RemoteStatus, Error> {
    let mut fetch_opts = git2::FetchOptions::new();
    fetch_opts.remote_callbacks(get_remote_callbacks()?);
    remote.download::<String>(&[], Some(&mut fetch_opts))?;
    remote.update_tips(None, true, git2::AutotagOption::Auto, None)?;

    let remote_head = remote
        .list()?
        .iter()
        .filter(|x| x.name() == "HEAD")
        .next()
        .ok_or("Remote HEAD not found")?
        .oid();

    let default_branch = remote.default_branch()?;
    let default_branch = default_branch.as_str().unwrap();

    let local_head = repo.refname_to_id(default_branch)?;

    if local_head == remote_head {
        return Ok(RemoteStatus::UpToDate);
    }

    let local_ahead = repo.graph_descendant_of(local_head, remote_head)?;
    let local_behind = repo.graph_descendant_of(remote_head, local_head)?;

    match (local_ahead, local_behind) {
        (true, false) => Ok(RemoteStatus::LocalAhead),
        (false, true) => Ok(RemoteStatus::LocalBehind),
        (true, true) => Err("Local commit is both ahead and behind remote!".into()),
        (false, false) => Err("Local commit is neither ahead nor behind remote!".into()),
    }
}

pub(crate) fn is_local_dirty(repo: &git2::Repository) -> Result<bool, Error> {
    let statuses = repo.statuses(None)?;
    let unsafe_to_delete = |status: git2::Status| {
        !status.is_ignored()
            && (status.is_wt_new()
                || status.is_wt_modified()
                || status.is_index_new()
                || status.is_index_modified()
                || status.is_conflicted())
    };
    Ok(statuses.iter().any(|x| unsafe_to_delete(x.status())))
}