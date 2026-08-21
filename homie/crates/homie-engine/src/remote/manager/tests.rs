use super::*;
use crate::remote::bootstrap::{PackagedArtifact, RemoteTarget};
use crate::remote::executor::ProcessExecutor;
use std::collections::HashMap;
use std::fs;
use std::io::Write as _;
use std::os::unix::ffi::OsStrExt as _;
use std::os::unix::fs::{OpenOptionsExt as _, PermissionsExt};
use std::path::PathBuf;
use std::sync::Arc;

use homie_proto::HostEntry;
use homie_proto::remote_pty::{
    HelperProbe, PHASE_ONE_HELPER_CAPABILITIES, PersistenceCapability, ProtocolVersion,
};
use sha2::{Digest, Sha256};

#[test]
fn json_line_parser_ignores_bounded_shell_noise() {
    let parsed: HelperProbe = parse_json_line(
        b"welcome from rc\n{\"protocol\":{\"major\":1,\"minor\":1},\"buildId\":\"b\",\"artifactSha256\":\"h\",\"target\":\"t\",\"os\":\"o\",\"arch\":\"a\",\"supported\":true,\"holderAvailable\":true,\"capabilities\":[]}\n",
    )
    .expect("parse");
    assert_eq!(parsed.build_id, "b");
}

#[test]
fn json_line_parser_ignores_trailing_shell_noise() {
    let parsed: HelperProbe = parse_json_line(
        b"{\"protocol\":{\"major\":1,\"minor\":1},\"buildId\":\"b\",\"artifactSha256\":\"h\",\"target\":\"t\",\"os\":\"o\",\"arch\":\"a\",\"supported\":true,\"holderAvailable\":true,\"capabilities\":[]}\nlogout noise\n",
    )
    .expect("parse");
    assert_eq!(parsed.build_id, "b");
}

#[test]
fn artifact_manifest_rejects_parent_traversal() {
    let temporary = tempfile::tempdir().expect("temp");
    let manifest = temporary.path().join("manifest.json");
    fs::write(
        &manifest,
        br#"{"protocolMajor":1,"buildId":"b","artifacts":[{"target":"aarch64-apple-darwin","path":"../escape","length":1,"sha256":"00"}]}"#,
    )
    .expect("manifest");
    assert!(ArtifactCatalog::from_manifest(&manifest).is_err());
}

#[test]
fn release_manifest_requires_all_supported_targets() {
    let temporary = tempfile::tempdir().expect("temp");
    let artifact = temporary.path().join("helper");
    fs::write(&artifact, b"helper").expect("artifact");
    let hash = hex_sha256(b"helper");
    let manifest = temporary.path().join("manifest.json");
    fs::write(
        &manifest,
        format!(
            r#"{{"protocolMajor":1,"buildId":"b","artifacts":[{{"target":"aarch64-apple-darwin","path":"helper","length":6,"sha256":"{hash}"}}]}}"#
        ),
    )
    .expect("manifest");
    let error = ArtifactCatalog::from_manifest(&manifest).expect_err("incomplete catalog");
    assert!(error.to_string().contains("missing required target"));
}

#[test]
fn persistence_probe_surfaces_all_three_capability_outcomes() {
    assert_eq!(
        classify_persistence(true, false),
        PersistenceCapability::NativeDetach
    );
    assert_eq!(
        classify_persistence(false, true),
        PersistenceCapability::UserSupervisor
    );
    assert_eq!(
        classify_persistence(false, false),
        PersistenceCapability::NonPersistent
    );
}

#[test]
fn helper_probe_without_directory_management_is_rejected_before_rpc() {
    let temporary = tempfile::tempdir().expect("temp");
    let path = temporary.path().join("helper");
    fs::write(&path, b"helper").expect("artifact");
    let artifact = PackagedArtifact {
        target: RemoteTarget::MacosAarch64,
        protocol_major: homie_proto::remote_pty::PROTOCOL_MAJOR,
        build_id: "legacy-build".into(),
        length: 6,
        sha256: hex_sha256(b"helper"),
        path,
    };
    let probe = HelperProbe {
        protocol: ProtocolVersion::CURRENT,
        build_id: artifact.build_id.clone(),
        artifact_sha256: artifact.sha256.clone(),
        target: artifact.target.artifact_name().into(),
        os: "macos".into(),
        arch: "aarch64".into(),
        supported: true,
        holder_available: true,
        capabilities: PHASE_ONE_HELPER_CAPABILITIES
            .iter()
            .copied()
            .filter(|capability| {
                *capability != homie_proto::remote_pty::RemoteCapability::DirectoryList
            })
            .collect(),
    };

    let error = verify_required_helper_probe(&artifact, &probe)
        .expect_err("a legacy Helper must fail before directories RPC");
    assert!(error.to_string().contains("directory-list"));
}

#[test]
fn long_control_directories_use_a_short_stable_owner_path() {
    let requested = PathBuf::from("/very-long").join("segment".repeat(20));
    let first = normalized_control_dir(&requested);
    let second = normalized_control_dir(&requested);
    assert_eq!(first, second);
    assert!(first.starts_with("/tmp"));
    assert!(first.as_os_str().as_bytes().len() < MAX_CONTROL_DIRECTORY_BYTES);
    assert_ne!(first, normalized_control_dir(&requested.join("different")));
}

#[cfg(unix)]
#[test]
fn fake_ssh_bootstrap_uploads_activates_and_then_reuses_exact_build() {
    let temporary = tempfile::tempdir().expect("temp");
    let remote_home = temporary.path().join("remote-home");
    fs::create_dir(&remote_home).expect("remote home");
    // Use a fixed supported target instead of leaking the developer
    // machine's architecture into this fake-host bootstrap test.
    let target = RemoteTarget::MacosAarch64;
    let artifact_path = temporary.path().join("homie-remote-fixture");
    let artifact_script = format!(
        "#!/bin/sh\ncase \"$1\" in\nprobe) printf '%s\\n' '{{\"protocol\":{{\"major\":1,\"minor\":2}},\"buildId\":\"test-build\",\"artifactSha256\":\"'$TEST_ARTIFACT_SHA'\",\"target\":\"{}\",\"os\":\"test\",\"arch\":\"test\",\"supported\":true,\"holderAvailable\":true,\"capabilities\":[\"full-snapshot\",\"incremental-grid\",\"process-exit\",\"signal\",\"controller-lease\",\"scrollback\",\"session-management\",\"environment-capture\",\"directory-list\",\"persistence-probe\",\"atomic-activation\"]}}';;\nactivate) final=$(dirname \"$0\")/homie-remote; ln \"$0\" \"$final\" 2>/dev/null || true; rm -f \"$0\"; exec \"$final\" probe --format=json;;\n*) exit 64;;\nesac\n",
        target.artifact_name()
    );
    fs::write(&artifact_path, artifact_script).expect("artifact");
    fs::set_permissions(&artifact_path, fs::Permissions::from_mode(0o700)).expect("artifact mode");
    let artifact_sha = hex_sha256(&fs::read(&artifact_path).expect("artifact bytes"));
    let upload_log = temporary.path().join("uploads.log");

    let fake_ssh = temporary.path().join("ssh");
    let mut fake = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .mode(0o700)
        .open(&fake_ssh)
        .expect("fake ssh");
    writeln!(
        fake,
        "#!/bin/sh\nexport HOME='{}'\nexport TEST_ARTIFACT_SHA='{}'\nfor last; do :; done\ncase \"$last\" in *'cat >'*) printf 'upload\\n' >> '{}';; esac\ncase \"$last\" in\n  *'__HOMIE_PLATFORM_V1__'*) printf '__HOMIE_PLATFORM_V1__\\0Darwin\\0aarch64\\0%s\\0' \"$HOME\";;\n  *) exec /bin/sh -c \"$last\";;\nesac",
        remote_home.display(),
        artifact_sha,
        upload_log.display()
    )
    .expect("fake script");
    drop(fake);

    let artifact = PackagedArtifact {
        target,
        protocol_major: 1,
        build_id: "test-build".into(),
        length: fs::metadata(&artifact_path).expect("metadata").len(),
        sha256: artifact_sha,
        path: artifact_path,
    };
    artifact.verify().expect("artifact verifies");
    let catalog = ArtifactCatalog {
        artifacts: HashMap::from([(target, artifact)]),
    };
    let manager = RemoteManager::new(
        ProcessExecutor::new(&fake_ssh),
        catalog,
        temporary.path().join("control"),
    )
    .expect("manager");
    let host = HostEntry {
        id: "fixture".into(),
        name: None,
        ssh: "fake-host".into(),
        default_cwd: None,
        node: None,
    };
    let barrier = Arc::new(std::sync::Barrier::new(4));
    let installs = (0..4)
        .map(|_| {
            let manager = manager.clone();
            let host = host.clone();
            let barrier = Arc::clone(&barrier);
            std::thread::spawn(move || {
                barrier.wait();
                manager.ensure_helper(&host)
            })
        })
        .collect::<Vec<_>>();
    let installed = installs
        .into_iter()
        .map(|thread| thread.join().expect("bootstrap thread").expect("bootstrap"))
        .collect::<Vec<_>>();
    let first = &installed[0];
    assert!(
        installed
            .iter()
            .all(|helper| helper.build_id == first.build_id)
    );
    assert_eq!(first.build_id, "test-build");
    let final_path = remote_home.join(".cache/homie/bin/protocol-1/test-build/homie-remote");
    assert!(final_path.is_file());
    let second = manager.ensure_helper(&host).expect("idempotent bootstrap");
    assert_eq!(second.build_id, first.build_id);
    let uploads_before_reinstall = fs::read_to_string(&upload_log)
        .expect("upload log")
        .lines()
        .count();
    let reinstalled = manager
        .reinstall_helper(&host)
        .expect("forced verified reinstall");
    assert_eq!(reinstalled.build_id, first.build_id);
    let uploads_after_reinstall = fs::read_to_string(&upload_log)
        .expect("upload log")
        .lines()
        .count();
    assert_eq!(
        uploads_after_reinstall,
        uploads_before_reinstall + 1,
        "reinstall must stage the packaged bytes even when the exact build exists"
    );
    let after_reinstall = manager.ensure_helper(&host).expect("version-gated reuse");
    assert_eq!(after_reinstall.build_id, first.build_id);
    assert_eq!(
        fs::read_to_string(&upload_log)
            .expect("upload log")
            .lines()
            .count(),
        uploads_after_reinstall,
        "a normal version check must reuse the exact verified build"
    );
    assert!(
        fs::read_dir(final_path.parent().expect("parent"))
            .expect("version dir")
            .all(|entry| !entry
                .expect("entry")
                .file_name()
                .to_string_lossy()
                .starts_with(".tmp-"))
    );
}

fn hex_sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
