use std::io::Write;
use std::process::{Command, Stdio};

use serde_json::Value;
use tempfile::TempDir;

mod support;

#[test]
fn waits_for_agent_until_done() {
    let temp = TempDir::new().expect("tempdir");
    let _runtime = support::RuntimeGuard::new(temp.path());
    let session_id = create_session(temp.path(), "Wait Done");
    notify_done(temp.path(), &session_id);

    let waited = mcp_tool(
        temp.path(),
        "wait_for_agent",
        serde_json::json!({
            "session_id": session_id,
            "until": "done",
            "timeout_s": 2
        }),
    );
    assert_eq!(waited["settled"], true);
    assert_eq!(waited["timedOut"], false);
    assert_eq!(waited["sessionId"], session_id);
    assert_eq!(waited["status"], "idle");
    assert_eq!(waited["waitedFor"], "done");
}

#[test]
fn timeout_returns_current_status() {
    let temp = TempDir::new().expect("tempdir");
    let _runtime = support::RuntimeGuard::new(temp.path());
    let session_id = create_session(temp.path(), "Wait Timeout");

    let waited = mcp_tool(
        temp.path(),
        "wait_for_agent",
        serde_json::json!({
            "sessionId": session_id,
            "until": "done",
            "timeoutS": 0
        }),
    );
    assert_eq!(waited["settled"], false);
    assert_eq!(waited["timedOut"], true);
    assert_eq!(waited["sessionId"], session_id);
    assert_eq!(waited["status"], "running");
    assert_eq!(waited["waitedFor"], "done");
}

#[test]
fn waits_for_exited_agent() {
    let temp = TempDir::new().expect("tempdir");
    let _runtime = support::RuntimeGuard::new(temp.path());
    let session_id = create_session(temp.path(), "Wait Exited");
    kill_session(temp.path(), &session_id);

    let waited = mcp_tool(
        temp.path(),
        "wait_for_agent",
        serde_json::json!({
            "session_id": session_id,
            "until": "exited",
            "timeout_s": 2
        }),
    );
    assert_eq!(waited["settled"], true);
    assert_eq!(waited["timedOut"], false);
    assert_eq!(waited["sessionId"], session_id);
    assert_eq!(waited["status"], "exited");
    assert_eq!(waited["waitedFor"], "exited");
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

fn notify_done(data_dir: &std::path::Path, session_id: &str) {
    let payload = serde_json::json!({
        "type": "agent-turn-complete",
        "thread-id": session_id,
        "input-messages": ["done"]
    })
    .to_string();
    let output = Command::new(env!("CARGO_BIN_EXE_homie"))
        .args(["notify", "--data-dir", data_dir.to_str().unwrap(), &payload])
        .output()
        .expect("notify");
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn kill_session(data_dir: &std::path::Path, session_id: &str) {
    let output = Command::new(env!("CARGO_BIN_EXE_homie"))
        .args([
            "session",
            "kill",
            "--data-dir",
            data_dir.to_str().unwrap(),
            "--id",
            session_id,
        ])
        .output()
        .expect("session kill");
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn mcp_tool(data_dir: &std::path::Path, name: &str, arguments: Value) -> Value {
    let response = mcp_roundtrip(
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
    );
    let text = response["result"]["content"][0]["text"]
        .as_str()
        .expect("tool text");
    serde_json::from_str(text).expect("tool payload")
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
