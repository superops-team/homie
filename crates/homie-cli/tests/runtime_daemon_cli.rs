use std::process::Command;

use serde_json::Value;

mod support;

#[test]
fn runtime_guard_shuts_down_daemon_during_unwind() {
    let temp = tempfile::tempdir().expect("tempdir");
    let result = std::panic::catch_unwind(|| {
        let _runtime = support::RuntimeGuard::new(temp.path());
        runtime_status(temp.path());
        panic!("exercise runtime guard unwind cleanup");
    });

    assert!(result.is_err());
    assert!(!temp.path().join("runtime/daemon.sock").exists());
}

#[test]
fn runtime_status_reports_one_live_daemon_instance() {
    let temp = tempfile::tempdir().expect("tempdir");
    let _runtime = support::RuntimeGuard::new(temp.path());

    let first = runtime_status(temp.path());
    let second = runtime_status(temp.path());

    assert_eq!(first["status"], "ready");
    assert_eq!(first["runtimeProcess"], "running");
    assert_eq!(first["daemonInstanceId"], second["daemonInstanceId"]);
    assert!(first["daemonPid"].as_u64().is_some());
    assert!(first["methodCapabilities"].as_array().is_some());
}

#[test]
fn doctor_reads_durable_storage_without_fabricating_or_starting_runtime() {
    let temp = tempfile::tempdir().expect("tempdir");

    let output = homie_json([
        "doctor",
        "--data-dir",
        temp.path().to_str().expect("data dir"),
        "--json",
    ]);

    assert_eq!(output["status"], "ok");
    assert!(output.get("runtimeProcess").is_none());
    assert!(!temp.path().join("runtime/daemon.sock").exists());
}

fn runtime_status(data_dir: &std::path::Path) -> Value {
    homie_json([
        "runtime",
        "status",
        "--data-dir",
        data_dir.to_str().expect("data dir"),
        "--json",
    ])
}

fn homie_json<const N: usize>(args: [&str; N]) -> Value {
    let output = Command::new(env!("CARGO_BIN_EXE_homie"))
        .args(args)
        .output()
        .expect("run homie");
    assert!(
        output.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("JSON output")
}
