use std::io::Write;
use std::process::{Command, Stdio};

use serde_json::Value;
use tempfile::TempDir;

mod support;

#[test]
fn release_agent_refuses_parent_and_ancestor() {
    let temp = TempDir::new().expect("tempdir");
    let _runtime = support::RuntimeGuard::new(temp.path());
    let root = create_session(temp.path(), "Root");
    let child = spawn_child(temp.path(), &root, "Child");
    let grandchild = spawn_child(temp.path(), &child, "Grandchild");

    let parent_response = release_response(temp.path(), &child, &root);
    assert_eq!(parent_response["error"]["code"], -32000);
    assert!(
        parent_response["error"]["message"]
            .as_str()
            .unwrap_or_default()
            .contains("spawned you")
    );

    let ancestor_response = release_response(temp.path(), &grandchild, &root);
    assert_eq!(ancestor_response["error"]["code"], -32000);
    assert!(
        ancestor_response["error"]["message"]
            .as_str()
            .unwrap_or_default()
            .contains("spawned you")
    );
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
