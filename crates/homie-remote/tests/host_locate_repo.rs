use std::fs;
use std::path::Path;

use homie_remote::{discover_repo_origin, locate_repo};
use tempfile::TempDir;

#[test]
fn locates_matching_candidate_by_origin() {
    let temp = TempDir::new().expect("tempdir");
    let source = temp.path().join("source");
    let clone = temp.path().join("clone");
    let other = temp.path().join("other");
    write_git_config(&source, "git@example.invalid:acme/app.git");
    write_git_config(&clone, "git@example.invalid:acme/app.git");
    write_git_config(&other, "git@example.invalid:acme/other.git");

    assert_eq!(
        discover_repo_origin(&source)
            .expect("source origin")
            .as_deref(),
        Some("git@example.invalid:acme/app.git")
    );

    let result = locate_repo(Some(&source), None, &[other, clone.clone()]).expect("locate");
    assert_eq!(result.path.as_deref(), Some(clone.to_str().unwrap()));
    assert_eq!(
        result.origin_url.as_deref(),
        Some("git@example.invalid:acme/app.git")
    );
}

#[test]
fn returns_origin_without_path_when_repo_is_not_cloned() {
    let temp = TempDir::new().expect("tempdir");
    let source = temp.path().join("source");
    let other = temp.path().join("other");
    write_git_config(&source, "git@example.invalid:acme/app.git");
    write_git_config(&other, "git@example.invalid:acme/other.git");

    let result = locate_repo(Some(&source), None, &[other]).expect("locate");
    assert_eq!(result.path, None);
    assert_eq!(
        result.origin_url.as_deref(),
        Some("git@example.invalid:acme/app.git")
    );
}

#[test]
fn returns_empty_result_when_cwd_has_no_origin() {
    let temp = TempDir::new().expect("tempdir");
    let source = temp.path().join("source");
    fs::create_dir_all(&source).expect("source");

    let result = locate_repo(Some(&source), None, &[]).expect("locate");
    assert_eq!(result.path, None);
    assert_eq!(result.origin_url, None);
}

#[test]
fn follows_linked_worktree_gitdir_for_origin() {
    let temp = TempDir::new().expect("tempdir");
    let worktree = temp.path().join("worktree");
    let gitdir = temp.path().join("main/.git/worktrees/worktree");
    fs::create_dir_all(&worktree).expect("worktree");
    fs::create_dir_all(&gitdir).expect("gitdir");
    fs::write(
        worktree.join(".git"),
        format!("gitdir: {}\n", gitdir.display()),
    )
    .expect("gitfile");
    fs::write(
        gitdir.join("config"),
        "[remote \"origin\"]\n\turl = git@example.invalid:acme/linked.git\n",
    )
    .expect("config");

    assert_eq!(
        discover_repo_origin(&worktree)
            .expect("linked origin")
            .as_deref(),
        Some("git@example.invalid:acme/linked.git")
    );
}

fn write_git_config(root: &Path, origin: &str) {
    let git = root.join(".git");
    fs::create_dir_all(&git).expect("git dir");
    fs::write(
        git.join("config"),
        format!("[remote \"origin\"]\n\turl = {origin}\n"),
    )
    .expect("git config");
}
