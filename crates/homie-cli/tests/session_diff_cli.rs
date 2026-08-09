use std::fs;
use std::path::Path;
use std::process::Command;

use serde_json::Value;
use tempfile::TempDir;

mod support;

#[test]
fn session_diff_cli_loads_real_git_diff() {
    let temp = TempDir::new().expect("tempdir");
    let data_dir = temp.path().join("data");
    let _runtime = support::RuntimeGuard::new(&data_dir);
    let repo = temp.path().join("repo");
    fs::create_dir_all(&repo).expect("repo");
    init_repo(&repo);
    let session_id = create_session(&data_dir, &repo);

    fs::write(repo.join("tracked.txt"), "after\n").expect("tracked");
    fs::write(repo.join("untracked.txt"), "new\n").expect("untracked");

    let diff = homie_json([
        "session",
        "diff",
        "--data-dir",
        data_dir.to_str().unwrap(),
        "--id",
        &session_id,
        "--base",
        "default-branch",
    ]);
    assert_eq!(diff["sessionId"], session_id);
    assert_eq!(diff["files"], 2);
    assert_eq!(diff["additions"], 2);
    assert_eq!(diff["deletions"], 1);
    assert!(
        diff["patchText"]
            .as_str()
            .unwrap_or_default()
            .contains("untracked.txt")
    );
}

fn create_session(data_dir: &Path, repo: &Path) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_homie"))
        .args([
            "session",
            "create",
            "--data-dir",
            data_dir.to_str().unwrap(),
            "--workspace",
            repo.to_str().unwrap(),
            "--title",
            "Diff CLI",
            "--json",
        ])
        .output()
        .expect("session create");
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice::<Value>(&output.stdout).expect("json")["id"]
        .as_str()
        .expect("id")
        .to_string()
}

fn homie_json<const N: usize>(args: [&str; N]) -> Value {
    let output = Command::new(env!("CARGO_BIN_EXE_homie"))
        .args(args)
        .output()
        .expect("homie");
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("json")
}

fn init_repo(repo: &Path) {
    run_git(repo, ["init", "-q", "-b", "main"]);
    run_git(repo, ["config", "user.email", "test@example.invalid"]);
    run_git(repo, ["config", "user.name", "Homie Test"]);
    fs::write(repo.join("tracked.txt"), "before\n").expect("tracked");
    run_git(repo, ["add", "tracked.txt"]);
    run_git(repo, ["commit", "-q", "-m", "init"]);
}

fn run_git<const N: usize>(cwd: &Path, args: [&str; N]) {
    let output = Command::new("/usr/bin/git")
        .args(args)
        .current_dir(cwd)
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()
        .expect("git");
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}
