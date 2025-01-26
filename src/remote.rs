use crate::branch::RepoRemoteBranch;
use crate::utils::split_results;
use git2::{Remote, Repository};
pub struct RepoRemote<'a, 'b> {
    pub repo: &'a Repository,
    pub remote: Option<Remote<'b>>,
}

impl<'a, 'b> RepoRemote<'a, 'b> {
    pub fn get_all_branches(&self) -> (Vec<RepoRemoteBranch<'a>>, Vec<git2::Error>) {
        let vec_of_results: Vec<Result<RepoRemoteBranch<'a>, git2::Error>> = self
            .repo
            .branches(None)
            .unwrap()
            .into_iter()
            .map(|branch| -> Result<RepoRemoteBranch<'a>, git2::Error> {
                let (branch, branch_type) = branch?;
                let remote = match branch_type {
                    git2::BranchType::Remote => self.remote.clone(),
                    git2::BranchType::Local => None,
                };
                Ok(RepoRemoteBranch::new(self.repo, remote, branch))
            })
            .collect();
        split_results(vec_of_results)
    }
}
