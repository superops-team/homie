use std::fs;
use std::path::Path;
use std::process::Command;

use tempfile::TempDir;

#[test]
fn host_locate_repo_cli_matches_candidate_and_reports_not_cloned() {
    let temp = TempDir::new().expect("tempdir");
    let data_dir = temp.path().join("data");
    let source = temp.path().join("source");
    let clone = temp.path().join("clone");
    let other = temp.path().join("other");
    write_git_config(&source, "git@example.invalid:acme/app.git");
    write_git_config(&clone, "git@example.invalid:acme/app.git");
    write_git_config(&other, "git@example.invalid:acme/other.git");

    let found = run_homie([
        "host",
        "locate-repo",
        "--data-dir",
        data_dir.to_str().unwrap(),
        "--cwd",
        source.to_str().unwrap(),
        "--candidate",
        other.to_str().unwrap(),
        "--candidate",
        clone.to_str().unwrap(),
    ]);
    assert_eq!(found["path"], clone.to_str().unwrap());
    assert_eq!(found["originURL"], "git@example.invalid:acme/app.git");

    let not_cloned = run_homie([
        "host",
        "locate-repo",
        "--data-dir",
        data_dir.to_str().unwrap(),
        "--origin-url",
        "git@example.invalid:acme/missing.git",
        "--candidate",
        other.to_str().unwrap(),
    ]);
    assert!(not_cloned.get("path").is_none());
    assert_eq!(
        not_cloned["originURL"],
        "git@example.invalid:acme/missing.git"
    );
}

fn run_homie<const N: usize>(args: [&str; N]) -> serde_json::Value {
    let output = Command::new(env!("CARGO_BIN_EXE_homie"))
        .args(args)
        .output()
        .expect("run homie");
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("json")
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
