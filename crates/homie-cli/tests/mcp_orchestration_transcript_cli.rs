use std::io::Write;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use serde_json::Value;
use tempfile::TempDir;

mod support;

#[test]
fn mcp_orchestrates_child_session_transcript() {
    let temp = TempDir::new().expect("tempdir");
    let _runtime = support::RuntimeGuard::new(temp.path());
    let parent_id = create_session(temp.path(), "Parent Transcript");

    let spawned = mcp_tool(
        temp.path(),
        &parent_id,
        "spawn_agent",
        serde_json::json!({
            "cwd": temp.path(),
            "title": "Child Transcript"
        }),
    );
    let child_id = spawned["session"]["id"].as_str().expect("child id");

    let sent = mcp_tool(
        temp.path(),
        &parent_id,
        "send_prompt",
        serde_json::json!({
            "sessionId": child_id,
            "text": "echo http://localhost:6123",
            "submit": true
        }),
    );
    assert_eq!(sent["ok"], true);
    assert_eq!(sent["relation"], "child");

    wait_for_output(temp.path(), &parent_id, child_id, "localhost:6123");
    notify_done(temp.path(), child_id);

    let waited = mcp_tool(
        temp.path(),
        &parent_id,
        "wait_for_agent",
        serde_json::json!({
            "session_id": child_id,
            "until": "done",
            "timeout_s": 2
        }),
    );
    assert_eq!(waited["settled"], true);
    assert_eq!(waited["status"], "idle");

    let output = mcp_tool(
        temp.path(),
        &parent_id,
        "read_output",
        serde_json::json!({ "sessionId": child_id }),
    );
    assert!(
        output["outputText"]
            .as_str()
            .unwrap_or_default()
            .contains("http://localhost:6123"),
        "missing child output: {output}"
    );

    let artifacts = mcp_tool(
        temp.path(),
        &parent_id,
        "get_artifacts",
        serde_json::json!({ "session_id": child_id }),
    );
    assert!(
        artifacts["listeningPorts"]
            .as_array()
            .expect("listeningPorts")
            .iter()
            .any(|port| port["port"] == 6123),
        "missing listening port: {artifacts}"
    );

    let released = mcp_tool(
        temp.path(),
        &parent_id,
        "release_agent",
        serde_json::json!({ "session_id": child_id }),
    );
    assert_eq!(released["ok"], true);
    assert_snapshot_status(temp.path(), child_id, "exited");
    kill_session(temp.path(), &parent_id);
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

fn wait_for_output(data_dir: &std::path::Path, parent_id: &str, session_id: &str, needle: &str) {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let output = mcp_tool(
            data_dir,
            parent_id,
            "read_output",
            serde_json::json!({ "sessionId": session_id }),
        );
        if output["outputText"]
            .as_str()
            .unwrap_or_default()
            .contains(needle)
        {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "timed out waiting for {needle}: {output}"
        );
        thread::sleep(Duration::from_millis(100));
    }
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
