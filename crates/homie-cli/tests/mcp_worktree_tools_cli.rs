use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

use serde_json::Value;
use tempfile::TempDir;

mod support;

#[test]
fn mcp_creates_lists_and_removes_real_worktree() {
    let temp = TempDir::new().expect("tempdir");
    let _runtime = support::RuntimeGuard::new(temp.path());
    let repo = temp.path().join("repo");
    init_repo(&repo);

    let created = mcp_tool(
        temp.path(),
        "create_worktree",
        serde_json::json!({
            "repo": repo,
            "branch": "feature/mcp",
            "base": "HEAD"
        }),
    );
    let worktree_path = created["path"].as_str().expect("worktree path");
    assert!(Path::new(worktree_path).is_dir());
    assert_eq!(created["branch"], "feature/mcp");

    let listed = mcp_tool(
        temp.path(),
        "list_worktrees",
        serde_json::json!({ "repo": repo }),
    );
    assert!(
        listed["worktrees"]
            .as_array()
            .expect("worktrees")
            .iter()
            .any(|entry| entry["path"] == worktree_path),
        "created worktree missing from list: {listed}"
    );

    let removed = mcp_tool(
        temp.path(),
        "remove_worktree",
        serde_json::json!({
            "repo": repo,
            "path": worktree_path,
            "force": true
        }),
    );
    assert_eq!(removed["ok"], true);
    assert_eq!(removed["path"], worktree_path);
    assert!(!Path::new(worktree_path).exists());
}

#[test]
fn missing_parameters_return_invalid_params() {
    let temp = TempDir::new().expect("tempdir");
    let _runtime = support::RuntimeGuard::new(temp.path());
    let create = mcp_error(
        temp.path(),
        "create_worktree",
        serde_json::json!({ "branch": "missing/repo" }),
    );
    assert_eq!(create["error"]["code"], -32602);
    assert!(
        create["error"]["message"]
            .as_str()
            .unwrap_or_default()
            .contains("repo is required")
    );

    let remove = mcp_error(
        temp.path(),
        "remove_worktree",
        serde_json::json!({ "repo": temp.path() }),
    );
    assert_eq!(remove["error"]["code"], -32602);
    assert!(
        remove["error"]["message"]
            .as_str()
            .unwrap_or_default()
            .contains("path is required")
    );
}

fn mcp_tool(data_dir: &std::path::Path, name: &str, arguments: Value) -> Value {
    let response = mcp_response(data_dir, name, arguments);
    let text = response["result"]["content"][0]["text"]
        .as_str()
        .expect("tool text");
    serde_json::from_str(text).expect("tool payload")
}

fn mcp_error(data_dir: &std::path::Path, name: &str, arguments: Value) -> Value {
    mcp_response(data_dir, name, arguments)
}

fn mcp_response(data_dir: &std::path::Path, name: &str, arguments: Value) -> Value {
    mcp_roundtrip(
        data_dir,
        &serde_json::json!({
            "jsonrpc": "2.0",
            "id": "call",
            "method": "tools/call",
            "params": {
                "name": name,
                "arguments": arguments
            }
        })
        .to_string(),
    )
}

fn mcp_roundtrip(data_dir: &std::path::Path, line: &str) -> Value {
    let mut child = Command::new(env!("CARGO_BIN_EXE_homie"))
        .arg("mcp-stdio")
        .arg("--data-dir")
        .arg(data_dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("spawn mcp");
    {
        let stdin = child.stdin.as_mut().expect("stdin");
        stdin.write_all(line.as_bytes()).expect("line");
        stdin.write_all(b"\n").expect("newline");
    }
    let output = child.wait_with_output().expect("output");
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("json")
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
