use crate::utils::Result;
use git2::{Branch, Remote, Repository};
use std::path::PathBuf;

pub struct RepoRemoteBranchData {
    pub dirty_files: Vec<PathBuf>,
    pub dirty_files_ignored: Vec<PathBuf>,
    pub commits_ahead: usize,
    pub commits_behind: usize,
}

pub struct RepoRemoteBranch<'a> {
    pub repo: &'a Repository,
    pub remote: Option<Remote<'a>>,
    pub branch: Branch<'a>,
}

impl<'a> RepoRemoteBranch<'a> {
    pub fn new(
        repo: &'a Repository,
        remote: Option<Remote<'a>>,
        branch: Branch<'a>,
    ) -> RepoRemoteBranch<'a> {
        RepoRemoteBranch {
            repo,
            remote,
            branch,
        }
    }

    // pub fn get_data(&self) -> Result<RepoRemoteBranchData> {
    //     let dirty_files = vec![];
    //     let commits_ahead = 0;
    //     let commits_behind = 0;
    //     Ok(RepoRemoteBranchData {
    //         dirty_files,
    //         commits_ahead,
    //         commits_behind,
    //     })
    // }
}
