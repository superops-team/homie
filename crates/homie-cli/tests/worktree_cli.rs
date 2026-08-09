use std::fs;
use std::path::Path;
use std::process::Command;

use serde_json::Value;
use tempfile::TempDir;

mod support;

#[test]
fn worktree_cli_creates_lists_and_removes_real_worktree() {
    let temp = TempDir::new().expect("tempdir");
    let data_dir = temp.path().join("data");
    let _runtime = support::RuntimeGuard::new(&data_dir);
    let repo = temp.path().join("repo");
    init_repo(&repo);

    let created = homie_json([
        "worktree",
        "create",
        "--data-dir",
        data_dir.to_str().unwrap(),
        "--repo",
        repo.to_str().unwrap(),
        "--branch",
        "feature/cli",
        "--base",
        "HEAD",
        "--json",
    ]);
    let worktree_path = created["path"].as_str().expect("worktree path");
    assert!(Path::new(worktree_path).is_dir());
    assert_eq!(created["branch"], "feature/cli");

    let listed = homie_json([
        "worktree",
        "list",
        "--data-dir",
        data_dir.to_str().unwrap(),
        "--repo",
        repo.to_str().unwrap(),
        "--json",
    ]);
    assert!(
        listed["worktrees"]
            .as_array()
            .expect("worktrees")
            .iter()
            .any(|entry| entry["path"] == worktree_path)
    );

    let removed = homie_json([
        "worktree",
        "remove",
        "--data-dir",
        data_dir.to_str().unwrap(),
        "--repo",
        repo.to_str().unwrap(),
        "--path",
        worktree_path,
        "--force",
        "--json",
    ]);
    assert_eq!(removed["ok"], true);
    assert_eq!(removed["path"], worktree_path);
    assert!(!Path::new(worktree_path).exists());
}

fn homie_json<const N: usize>(args: [&str; N]) -> Value {
    let output = Command::new(env!("CARGO_BIN_EXE_homie"))
        .args(args)
        .output()
        .expect("run homie");
    assert!(
        output.status.success(),
        "homie failed stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("homie json")
}

fn init_repo(repo: &Path) {
    fs::create_dir_all(repo).expect("repo dir");
    run_git(repo, ["init", "-q", "-b", "main"]);
    run_git(repo, ["config", "user.email", "test@example.invalid"]);
    run_git(repo, ["config", "user.name", "Homie Test"]);
    fs::write(repo.join("README.md"), "hello\n").expect("readme");
    run_git(repo, ["add", "."]);
    run_git(repo, ["commit", "-q", "-m", "init"]);
}

fn run_git<const N: usize>(cwd: &Path, args: [&str; N]) {
    let output = Command::new("/usr/bin/git")
        .args(args)
        .current_dir(cwd)
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()
        .expect("run git");
    assert!(
        output.status.success(),
        "git failed stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
