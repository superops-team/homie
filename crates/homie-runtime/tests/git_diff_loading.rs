use std::fs;
use std::path::Path;
use std::process::Command;

use homie_proto::SessionDiffBase;
use homie_runtime::{DiffRowKind, load_git_diff};
use tempfile::TempDir;

#[test]
fn loads_tracked_and_untracked_changes() {
    let temp = TempDir::new().expect("tempdir");
    let repo = temp.path();
    init_repo(repo);
    fs::write(repo.join("tracked.txt"), "after\n").expect("tracked");
    fs::write(repo.join("untracked.txt"), "new\n").expect("untracked");

    let snapshot = load_git_diff(repo, SessionDiffBase::DefaultBranch).expect("diff");

    assert_eq!(snapshot.files, 2);
    assert_eq!(snapshot.additions, 2);
    assert_eq!(snapshot.deletions, 1);
    assert!(
        snapshot
            .rows
            .iter()
            .any(|row| { row.kind == DiffRowKind::File && row.text == "untracked.txt" })
    );
}

#[test]
fn head_comparison_excludes_committed_branch_changes() {
    let temp = TempDir::new().expect("tempdir");
    let repo = temp.path();
    init_repo(repo);
    run_git(repo, ["checkout", "-q", "-b", "feature"]);
    fs::write(repo.join("tracked.txt"), "committed feature\n").expect("tracked");
    run_git(repo, ["add", "tracked.txt"]);
    run_git(repo, ["commit", "-q", "-m", "feature"]);

    let clean = load_git_diff(repo, SessionDiffBase::Head).expect("clean diff");
    assert_eq!(clean.files, 0);
    assert_eq!(clean.base_ref.as_deref(), Some("HEAD"));

    fs::write(repo.join("tracked.txt"), "working change\n").expect("tracked");
    let dirty = load_git_diff(repo, SessionDiffBase::Head).expect("dirty diff");
    assert_eq!(dirty.files, 1);
    assert_eq!(dirty.additions, 1);
    assert_eq!(dirty.deletions, 1);
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
        "git failed stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
