use std::io::Write;
use std::process::{Command, Stdio};

use serde_json::Value;
use tempfile::TempDir;

mod support;

#[test]
fn mcp_spawn_agent_stamps_parent_and_list_children_returns_child() {
    let temp = TempDir::new().expect("tempdir");
    let _runtime = support::RuntimeGuard::new(temp.path());
    let parent_id = create_session(temp.path(), "Parent");

    let spawned = mcp_tool(
        temp.path(),
        &parent_id,
        "spawn_agent",
        serde_json::json!({
            "cwd": temp.path(),
            "title": "Child"
        }),
    );
    let child_id = spawned["session"]["id"].as_str().expect("child id");

    let children = mcp_tool(
        temp.path(),
        &parent_id,
        "list_children",
        serde_json::json!({}),
    );
    let rows = children["children"].as_array().expect("children");
    assert_eq!(children["count"], 1);
    assert!(rows.iter().any(|row| {
        row["id"] == child_id
            && row["title"] == "Child"
            && row["parentSessionId"] == parent_id
            && row["relation"] == "child"
    }));
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

fn mcp_tool(data_dir: &std::path::Path, session_id: &str, name: &str, arguments: Value) -> Value {
    let response = mcp_roundtrip(
        data_dir,
        session_id,
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
