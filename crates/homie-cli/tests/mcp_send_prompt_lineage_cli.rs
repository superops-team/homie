use std::io::Write;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use serde_json::Value;
use tempfile::TempDir;

mod support;

#[test]
fn sibling_send_prompt_is_attributed() {
    let temp = TempDir::new().expect("tempdir");
    let _runtime = support::RuntimeGuard::new(temp.path());
    let parent = create_session(temp.path(), "Parent");
    let left = spawn_child(temp.path(), &parent, "Left");
    let right = spawn_child(temp.path(), &parent, "Right");

    let sent = mcp_tool(
        temp.path(),
        &left,
        "send_prompt",
        serde_json::json!({
            "sessionId": right,
            "text": "hello sibling",
            "submit": false
        }),
    );
    assert_eq!(sent["ok"], true);
    assert_eq!(sent["relation"], "sibling");
    assert_eq!(sent["attributed"], true);

    let output = wait_output(temp.path(), &right, "message from id:");
    assert!(output.contains("hello sibling"));
    assert!(output.contains(&left));
}

#[test]
fn self_send_prompt_is_rejected() {
    let temp = TempDir::new().expect("tempdir");
    let _runtime = support::RuntimeGuard::new(temp.path());
    let caller = create_session(temp.path(), "Caller");
    let response = mcp_roundtrip(
        temp.path(),
        &caller,
        &serde_json::json!({
            "jsonrpc": "2.0",
            "id": "call",
            "method": "tools/call",
            "params": {
                "name": "send_prompt",
                "arguments": {
                    "sessionId": caller,
                    "text": "loop"
                }
            }
        })
        .to_string(),
    );
    assert_eq!(response["error"]["code"], -32000);
    assert!(
        response["error"]["message"]
            .as_str()
            .unwrap_or_default()
            .contains("cannot target the calling session")
    );
}

fn spawn_child(data_dir: &std::path::Path, parent: &str, title: &str) -> String {
    let spawned = mcp_tool(
        data_dir,
        parent,
        "spawn_agent",
        serde_json::json!({
            "cwd": data_dir,
            "title": title
        }),
    );
    spawned["session"]["id"]
        .as_str()
        .expect("child")
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

fn wait_output(data_dir: &std::path::Path, session_id: &str, needle: &str) -> String {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let value = mcp_tool(
            data_dir,
            session_id,
            "read_output",
            serde_json::json!({ "sessionId": session_id }),
        );
        let output = value["outputText"].as_str().unwrap_or_default().to_string();
        if output.contains(needle) {
            return output;
        }
        assert!(
            Instant::now() < deadline,
            "timed out waiting for {needle}: {output}"
        );
        thread::sleep(Duration::from_millis(100));
    }
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
