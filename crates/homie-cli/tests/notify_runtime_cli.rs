use serde_json::Value;
use std::process::Command;

mod support;

#[test]
fn codex_notify_turn_complete_persists_idle_status() {
    let temp = tempfile::tempdir().expect("tempdir");
    let _runtime = support::RuntimeGuard::new(temp.path());
    let session_id = create_session(temp.path());
    let payload = serde_json::json!({
        "type": "agent-turn-complete",
        "thread-id": session_id,
        "input-messages": ["Implement feature"]
    })
    .to_string();

    let notify = Command::new(env!("CARGO_BIN_EXE_homie"))
        .args([
            "notify",
            "--data-dir",
            temp.path().to_str().unwrap(),
            &payload,
        ])
        .output()
        .expect("notify");
    assert!(
        notify.status.success(),
        "notify stderr={}",
        String::from_utf8_lossy(&notify.stderr)
    );

    let snapshot = snapshot(temp.path(), &session_id);
    assert_eq!(snapshot["status"]["status"], "idle");
    assert_eq!(snapshot["status"]["turnCompleted"], true);
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
            "Notify Runtime",
            "--json",
        ])
        .output()
        .expect("session create");
    assert!(output.status.success());
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
