use std::fs;
use std::os::unix::fs::PermissionsExt as _;
use std::process::Command;

use serde_json::Value;
use tempfile::TempDir;

#[test]
fn agent_readiness_cli_projects_descriptor_binary_availability() {
    let temp = TempDir::new().expect("tempdir");
    let descriptors = temp.path().join("descriptors");
    let bin = temp.path().join("bin");
    fs::create_dir_all(&descriptors).expect("descriptors");
    fs::create_dir_all(&bin).expect("bin");
    write_descriptor(&descriptors, "codex", "Codex", Some("codex"));
    write_descriptor(&descriptors, "claude-code", "Claude Code", Some("claude"));
    write_descriptor(&descriptors, "shell", "Shell", None);
    let codex = bin.join("codex");
    fs::write(&codex, "#!/bin/sh\nexit 0\n").expect("codex");
    fs::set_permissions(&codex, fs::Permissions::from_mode(0o755)).expect("chmod");

    let output = Command::new(env!("CARGO_BIN_EXE_homie"))
        .args([
            "agent",
            "readiness",
            "--descriptor-dir",
            descriptors.to_str().unwrap(),
            "--bin-dir",
            bin.to_str().unwrap(),
            "--json",
        ])
        .output()
        .expect("homie agent readiness");
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: Value = serde_json::from_slice(&output.stdout).expect("json");
    let agents = value["agents"].as_array().expect("agents");
    let codex = agents
        .iter()
        .find(|agent| agent["id"] == "codex")
        .expect("codex");
    assert_eq!(codex["available"], true);
    assert_eq!(codex["path"], bin.join("codex").to_str().unwrap());
    let claude = agents
        .iter()
        .find(|agent| agent["id"] == "claude-code")
        .expect("claude");
    assert_eq!(claude["available"], false);
    assert!(agents.iter().all(|agent| agent["id"] != "shell"));
}

fn write_descriptor(root: &std::path::Path, id: &str, name: &str, binary: Option<&str>) {
    let mut agent = serde_json::json!({
        "displayName": name,
        "shortLabel": id,
        "statusAuthority": "process"
    });
    if let Some(binary) = binary {
        agent["binary"] = serde_json::json!(binary);
    }
    let manifest = serde_json::json!({
        "schemaVersion": 2,
        "id": id,
        "version": "test",
        "statusModel": "processOnly",
        "agent": agent,
        "rules": []
    });
    fs::write(
        root.join(format!("{id}.json")),
        serde_json::to_vec_pretty(&manifest).expect("manifest"),
    )
    .expect("write descriptor");
}
