use serde_json::Value;
use std::process::Command;

mod support;

#[test]
fn persists_permission_request_to_session_snapshot() {
    let temp = tempfile::tempdir().expect("tempdir");
    let _runtime = support::RuntimeGuard::new(temp.path());
    let session_id = create_session(temp.path());
    let payload = serde_json::json!({
        "session_id": session_id,
        "tool_name": "Bash",
        "tool_input": {
            "command": "deploy --token=example-token-value"
        }
    })
    .to_string();

    let hook = Command::new(env!("CARGO_BIN_EXE_homie"))
        .args([
            "hook",
            "--data-dir",
            temp.path().to_str().unwrap(),
            "PermissionRequest",
            &payload,
        ])
        .output()
        .expect("hook");
    assert!(
        hook.status.success(),
        "hook stderr={}",
        String::from_utf8_lossy(&hook.stderr)
    );
    let hook_json: Value = serde_json::from_slice(&hook.stdout).expect("hook json");
    assert_eq!(hook_json["needsInput"]["kind"], "approval");
    assert!(
        !String::from_utf8_lossy(&hook.stdout).contains("example-token-value"),
        "hook output leaked secret"
    );

    let snapshot = snapshot(temp.path(), &session_id);
    assert_eq!(snapshot["status"]["status"], "needs_input");
    assert_eq!(snapshot["status"]["needsInput"]["kind"], "approval");
    assert_eq!(snapshot["status"]["needsInput"]["toolName"], "Bash");
    assert!(
        !snapshot["status"]["needsInput"]["summary"]
            .as_str()
            .unwrap_or_default()
            .contains("example-token-value")
    );
}

fn create_session(data_dir: &std::path::Path) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_homie"))
        .args([
            "session",
            "create",
            "--data-dir",
            data_dir.to_str().unwrap(),
            "--workspace",
            data_dir.to_str().unwrap(),
            "--title",
            "Hook Runtime",
            "--json",
        ])
        .output()
        .expect("session create");
    assert!(
        output.status.success(),
        "create stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice::<Value>(&output.stdout).expect("json")["id"]
        .as_str()
        .expect("id")
        .to_string()
}

fn snapshot(data_dir: &std::path::Path, session_id: &str) -> Value {
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
        .expect("snapshot");
    assert!(
        output.status.success(),
        "snapshot stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("snapshot json")
}
