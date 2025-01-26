mod branch;
mod git_utils;
mod remote;
mod repo;
mod utils;

fn main() {
    let repo = git2::Repository::open_from_env().unwrap();
    let branches = repo.branches(None).unwrap();
    for branch in branches {
        let (branch, branch_type) = branch.unwrap();
        let branch_name = branch.name().unwrap().unwrap().to_string();
        println!("{}, {:?}", branch_name, branch_type);
    }
}
