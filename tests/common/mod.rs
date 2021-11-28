use std::ffi::OsStr;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Output;

use git2::{BranchType, IndexAddOption, Oid, Repository};

pub fn generate_path_to_repo<S>(repo_name: S) -> PathBuf
where
    S: Into<String>,
{
    let repo_name: String = repo_name.into();
    let test_fixture_path = Path::new("./test_sandbox/");
    let path_to_repo = test_fixture_path.join(repo_name);
    assert!(path_to_repo.is_relative());
    path_to_repo.canonicalize().unwrap()
}

pub fn setup_git_repo<S>(repo_name: S) -> Repository
where
    S: Into<String>,
{
    let path_to_repo = generate_path_to_repo(repo_name);

    fs::remove_dir_all(&path_to_repo).ok();
    fs::create_dir_all(&path_to_repo).unwrap();

    let repo = match Repository::init(path_to_repo) {
        Ok(repo) => repo,
        Err(err) => panic!("failed to init repo: {}", err),
    };

    let mut config = repo.config().unwrap();
    config.set_str("user.name", "name").unwrap();
    config.set_str("user.email", "email").unwrap();

    repo
}

pub fn teardown_git_repo<S>(repo_name: S)
where
    S: Into<String>,
{
    let path_to_repo = generate_path_to_repo(repo_name);
    fs::remove_dir_all(&path_to_repo).ok();
}

pub fn create_branch(repo: &Repository, branch_name: &str) {
    // create branch from HEAD
    let oid = repo.head().unwrap().target().unwrap();
    let commit = repo.find_commit(oid).unwrap();

    repo.branch(branch_name, &commit, false).unwrap();
}

pub fn checkout_branch(repo: &Repository, branch_name: &str) {
    let obj = repo
        .revparse_single(&("refs/heads/".to_owned() + branch_name))
        .unwrap();

    repo.checkout_tree(&obj, None).unwrap();

    repo.set_head(&("refs/heads/".to_owned() + branch_name))
        .unwrap();
}

pub fn stage_everything(repo: &Repository) -> Oid {
    let mut index = repo.index().expect("cannot get the Index file");
    index
        .add_all(["*"].iter(), IndexAddOption::DEFAULT, None)
        .unwrap();
    index.write().unwrap();

    let mut index = repo.index().unwrap();
    // root_tree_oid
    index.write_tree().unwrap()
}

pub fn create_first_commit(repo: &Repository, root_tree_oid: Oid, message: &str) {
    let tree = repo.find_tree(root_tree_oid).unwrap();

    let author = &repo.signature().unwrap();
    let committer = &author;

    repo.commit(Some("HEAD"), author, committer, message, &tree, &[])
        .unwrap();
}

pub fn create_commit(repo: &Repository, root_tree_oid: Oid, message: &str) {
    let tree = repo.find_tree(root_tree_oid).unwrap();
    let head_id = repo.refname_to_id("HEAD").unwrap();
    let parent = repo.find_commit(head_id).unwrap();

    let author = &repo.signature().unwrap();
    let committer = &author;

    repo.commit(Some("HEAD"), author, committer, message, &tree, &[&parent])
        .unwrap();
}

pub fn first_commit_all(repo: &Repository, message: &str) {
    // stage all changes - git add -A *
    let root_tree_oid = stage_everything(repo);

    create_first_commit(repo, root_tree_oid, message);
}

pub fn commit_all(repo: &Repository, message: &str) {
    // stage all changes - git add -A *
    let root_tree_oid = stage_everything(repo);

    create_commit(repo, root_tree_oid, message);
}

pub fn delete_local_branch(repo: &Repository, branch_name: &str) {
    let mut some_branch = repo.find_branch(branch_name, BranchType::Local).unwrap();

    // Should not be able to delete branch_name if it is the current working tree
    assert!(!some_branch.is_head());

    some_branch.delete().unwrap();
}

pub fn run_test_bin_expect_err<I, T>(arguments: I) -> Output
where
    I: IntoIterator<Item = T>,
    T: AsRef<OsStr>,
{
    let output = test_bin::get_test_bin("git-chain")
        .args(arguments)
        .output()
        .expect("Failed to start git-chain");

    if output.status.success() {
        io::stdout().write_all(&output.stdout).unwrap();
        io::stderr().write_all(&output.stderr).unwrap();
    }

    assert!(!output.status.success());

    output
}

pub fn run_test_bin_expect_ok<I, T>(arguments: I) -> Output
where
    I: IntoIterator<Item = T>,
    T: AsRef<OsStr>,
{
    let output = test_bin::get_test_bin("git-chain")
        .args(arguments)
        .output()
        .expect("Failed to start git-chain");

    if !output.status.success() {
        io::stdout().write_all(&output.stdout).unwrap();
        io::stderr().write_all(&output.stderr).unwrap();
    }

    assert!(output.status.success());

    output
}
