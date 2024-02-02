use crate::Error;
use git2::{Remote, Status, StatusEntry};

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
) -> Result<Vec<RemoteWrapper>, Error> {
    let mut result = vec![];
    for remote_str in repo.remotes()?.iter() {
        let remote_str = remote_str.ok_or("Remote name is not a valid UTF-8 string")?;
        let mut remote = repo.find_remote(remote_str)?;
        if connect {
            remote.connect_auth(git2::Direction::Fetch, Some(get_remote_callbacks()?), None)?;
        }
        result.push(RemoteWrapper(remote));
    }
    Ok(result)
}

pub(crate) struct StatusEntryWrapper<'a>(StatusEntry<'a>);
impl<'a> std::fmt::Display for StatusEntryWrapper<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.path().ok_or(std::fmt::Error {})?.fmt(f)
    }
}

pub(crate) enum RemoteStatus {
    LocalAhead,
    LocalBehind,
    UpToDate,
}

impl RemoteStatus {
    pub(crate) fn is_clean(&self) -> bool {
        match self {
            RemoteStatus::UpToDate => true,
            _ => false,
        }
    }
}

pub(crate) struct RemoteWrapper<'a>(Remote<'a>);
impl<'a> std::fmt::Display for RemoteWrapper<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.name().unwrap())
    }
}

pub(crate) fn is_remote_up_to_date(
    repo: &git2::Repository,
    mut remote: RemoteWrapper,
) -> Result<RemoteStatus, Error> {
    let mut remote = remote.0;
    remote.update_tips(None, false, git2::AutotagOption::Unspecified, None)?;
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

pub(crate) fn safe_to_delete(status: Status) -> bool {
    status.is_ignored()
        || !(status.is_wt_new()
            || status.is_wt_modified()
            || status.is_index_new()
            || status.is_index_modified()
            || status.is_conflicted())
}
