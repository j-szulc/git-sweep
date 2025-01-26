use git2::string_array::StringArray;

use crate::{
    git_utils::get_remote_callbacks,
    remote::{self, RepoRemote},
    utils::{split_results, Result},
};

pub trait RepoExtra<'a> {
    fn get_all_remotes(&self) -> std::result::Result<RepoRemote, Box<git2::Error>>;
    // fn get_all_remotes_or_error(
    //     &self,
    // ) -> Result<(Vec<RepoRemote>, Vec<Box<dyn std::error::Error>>)>;
}

fn collect_string_arr(sa: StringArray) -> Vec<&str> {
    let it: Box<dyn Iterator<Option<&str>>> = sa.iter();
}
impl<'a> RepoExtra<'a> for git2::Repository {
    fn get_all_remotes(&self) -> std::result::Result<Vec<RepoRemote>, Box<git2::Error>> {
        // TODO: error on a per-remote basis
        let remotes_strs: StringArray = self
            .remotes()
            .iter()
            .map(|sa: &StringArray| sa.iter().collect())
            .collect();
        let result: Vec<RepoRemote> = vec![];
        for remote_str in remotes_strs {
            let remote_ = self.find_remote(remote_str)?;
        }
        Ok(result)
        // .iter()
        // .map(|remote_str| {
        //     let remote = self.find_remote(remote_str.unwrap_or(""));
        //     match remote {
        //         Ok(mut remote) => {
        //             if let Ok(callbacks) = get_remote_callbacks() {
        //                 remote
        //                     .connect_auth(git2::Direction::Fetch, Some(callbacks), None)
        //                     .ok();
        //             }
        //             Ok(RepoRemote {
        //                 repo: self,
        //                 remote: Some(remote),
        //             })
        //         }
        //         Err(e) => Err(Box::new(e)),
        //     }
        // })
        // .collect()
    }

    // fn get_all_remotes_or_error(
    //     &self,
    // ) -> Result<(Vec<RepoRemote>, Vec<Box<dyn std::error::Error>>)> {
    //     let vec_of_results: Vec<Result<RepoRemote<'_, '_>>> = self
    //         .remotes()?
    //         .iter()
    //         .map(|remote_str| -> Result<RepoRemote> {
    //             let remote = self.find_remote(remote_str.ok_or("Remote name is not valid")?);
    //             match remote {
    //                 Ok(mut remote) => {
    //                     if let Ok(callbacks) = get_remote_callbacks() {
    //                         remote.connect_auth(git2::Direction::Fetch, Some(callbacks), None);
    //                     }
    //                     Ok(RepoRemote {
    //                         repo: self,
    //                         remote: Some(remote)),
    //                     })
    //                 }
    //                 Err(e) => Err(Box::new(e)),
    //             }
    //         })
    //         .collect::<Vec<Result<RepoRemote<'a>>>>();
    //     Ok(split_results(vec_of_results))
    // }
}
