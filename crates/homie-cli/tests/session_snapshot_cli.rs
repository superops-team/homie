use serde_json::Value;
use std::process::Command;

mod support;

#[test]
fn session_create_and_snapshot_use_runtime_backed_session() {
    let temp = tempfile::tempdir().expect("tempdir");
    let _runtime = support::RuntimeGuard::new(temp.path());
    let create = Command::new(env!("CARGO_BIN_EXE_homie"))
        .arg("session")
        .arg("create")
        .arg("--data-dir")
        .arg(temp.path())
        .arg("--workspace")
        .arg(temp.path())
        .arg("--title")
        .arg("Snapshot CLI")
        .arg("--json")
        .output()
        .expect("run homie session create");
    assert!(
        create.status.success(),
        "create command failed: stdout={} stderr={}",
        String::from_utf8_lossy(&create.stdout),
        String::from_utf8_lossy(&create.stderr)
    );
    let created: Value = serde_json::from_slice(&create.stdout).expect("create json");
    let session_id = created["id"].as_str().expect("session id");

    let output = Command::new(env!("CARGO_BIN_EXE_homie"))
        .arg("session")
        .arg("snapshot")
        .arg("--data-dir")
        .arg(temp.path())
        .arg("--id")
        .arg(session_id)
        .output()
        .expect("run homie snapshot");

    assert!(
        output.status.success(),
        "snapshot command failed: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let json: Value = serde_json::from_slice(&output.stdout).expect("json output");
    assert_eq!(json["session"]["id"], session_id);
    assert_eq!(json["status"]["status"], "running");
    assert_eq!(json["holder"]["status"], "running");
    assert!(json["holder"]["pid"].as_u64().is_some());
    assert!(
        json["outputText"]
            .as_str()
            .expect("output text")
            .contains("$ "),
        "expected live shell prompt in snapshot: {json}"
    );

    let kill = Command::new(env!("CARGO_BIN_EXE_homie"))
        .arg("session")
        .arg("kill")
        .arg("--data-dir")
        .arg(temp.path())
        .arg("--id")
        .arg(session_id)
        .output()
        .expect("run homie session kill");
    assert!(
        kill.status.success(),
        "kill command failed: stdout={} stderr={}",
        String::from_utf8_lossy(&kill.stdout),
        String::from_utf8_lossy(&kill.stderr)
    );
}
