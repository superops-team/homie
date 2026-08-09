use std::io::Write;
use std::process::{Command, Stdio};

use serde_json::Value;
use tempfile::TempDir;

mod support;

#[test]
fn sibling_release_is_refused_and_target_survives() {
    let temp = TempDir::new().expect("tempdir");
    let _runtime = support::RuntimeGuard::new(temp.path());
    let root = create_session(temp.path(), "Root");
    let sibling_a = spawn_child(temp.path(), &root, "Sibling A");
    let sibling_b = spawn_child(temp.path(), &root, "Sibling B");

    let response = release_response(temp.path(), &sibling_a, &sibling_b);
    assert_release_denied(response);
    assert_snapshot_status(temp.path(), &sibling_b, "running");
}

#[test]
fn unrelated_release_is_refused_and_target_survives() {
    let temp = TempDir::new().expect("tempdir");
    let _runtime = support::RuntimeGuard::new(temp.path());
    let root_a = create_session(temp.path(), "Root A");
    let root_b = create_session(temp.path(), "Root B");

    let response = release_response(temp.path(), &root_a, &root_b);
    assert_release_denied(response);
    assert_snapshot_status(temp.path(), &root_b, "running");
}

fn assert_release_denied(response: Value) {
    assert_eq!(response["error"]["code"], -32000);
    assert!(
        response["error"]["message"]
            .as_str()
            .unwrap_or_default()
            .contains("only release a direct child"),
        "response={response}"
    );
}

fn assert_snapshot_status(data_dir: &std::path::Path, session_id: &str, expected: &str) {
    let output = Command::new(env!("CARGO_BIN_EXE_homie"))
        .args([
            "session",
            "snapshot",
            "--data-dir",
            data_dir.to_str().unwrap(),
            "--id",
            session_id,
        ])
        .output()
        .expect("session snapshot");
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let payload = serde_json::from_slice::<Value>(&output.stdout).expect("json");
    assert_eq!(payload["status"]["status"], expected);
}

fn release_response(data_dir: &std::path::Path, caller: &str, target: &str) -> Value {
    mcp_roundtrip(
        data_dir,
        caller,
        &serde_json::json!({
            "jsonrpc": "2.0",
            "id": "call",
            "method": "tools/call",
            "params": {
                "name": "release_agent",
                "arguments": { "sessionId": target }
            }
        })
        .to_string(),
    )
}

fn spawn_child(data_dir: &std::path::Path, parent: &str, title: &str) -> String {
    let response = mcp_roundtrip(
        data_dir,
        parent,
        &serde_json::json!({
            "jsonrpc": "2.0",
            "id": "call",
            "method": "tools/call",
            "params": {
                "name": "spawn_agent",
                "arguments": { "cwd": data_dir, "title": title }
            }
        })
        .to_string(),
    );
    let text = response["result"]["content"][0]["text"]
        .as_str()
        .expect("text");
    serde_json::from_str::<Value>(text).expect("payload")["session"]["id"]
        .as_str()
        .expect("id")
        .to_string()
}

fn create_session(data_dir: &std::path::Path, title: &str) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_homie"))
        .args([
            "session",
            "create",
            "--data-dir",
            data_dir.to_str().unwrap(),
            "--workspace",
            data_dir.to_str().unwrap(),
            "--title",
            title,
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

fn mcp_roundtrip(data_dir: &std::path::Path, session_id: &str, line: &str) -> Value {
    let mut child = Command::new(env!("CARGO_BIN_EXE_homie"))
        .arg("mcp-stdio")
        .arg("--data-dir")
        .arg(data_dir)
        .arg("--session-id")
        .arg(session_id)
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
