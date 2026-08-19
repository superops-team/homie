use super::inject::is_claude_workspace_trust_screen;
use super::*;
use crate::detect::ManifestEngine;
use homie_proto::WIRE_VERSION;
use std::time::{Duration, Instant};

fn engine() -> Arc<ManifestEngine> {
    let dir = crate::detect::bundled_manifest_dir()
        .canonicalize()
        .expect("manifests");
    let (engine, _) = ManifestEngine::load_dir(&dir).expect("load");
    Arc::new(engine)
}

fn server(temp: &Path) -> ControlServer {
    let registry = Registry::new(engine(), temp.join("state.json"));
    ControlServer::new(Arc::new(Mutex::new(registry)), temp.join("daemon.sock"))
}

fn test_record(id: &str) -> homie_proto::SessionRecord {
    use homie_proto::*;
    SessionRecord {
        id: SessionId(id.into()),
        kind: AgentKind::SHELL,
        cwd: "/tmp".into(),
        project_id: ProjectId("p".into()),
        worktree_path: None,
        git_branch: None,
        title: "test".into(),
        title_source: TitleSource::Placeholder,
        agent_session_id: None,
        transcript_path: None,
        status: SessionStatus::Idle,
        needs_input: None,
        resumability: Resumability::NotResumable,
        parent: None,
        created_at: DateMillis(0.0),
        updated_at: DateMillis(0.0),
        last_turn_completed_at: None,
        last_seen_at: None,
        pinned: false,
        archived_at: None,
        host: None,
        remote_persistence: None,
        hibernation: None,
        memory_bytes: None,
        artifacts: None,
        pull_requests: None,
        listening_ports: None,
        foreground_agent: None,
    }
}

fn screen_contains(server: &ControlServer, session_id: &str, needle: &str) -> bool {
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        let screen = ok_of(call(
            server,
            homie_proto::Method::SESSION_READ_SCREEN,
            Some(json!({ "sessionID": session_id })),
        ));
        if screen["text"]
            .as_str()
            .is_some_and(|text| text.contains(needle))
        {
            return true;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    false
}

#[test]
fn local_shell_spawn_sets_term() {
    let temp = tempfile::tempdir().expect("temp");
    let server = server(temp.path()).with_logs_dir(temp.path().join("logs"));
    let spawned = ok_of(call(
        &server,
        homie_proto::Method::SESSION_SPAWN,
        Some(json!({
            "kind": { "shell": {} },
            "cwd": "/tmp",
            "argv": ["/bin/sh", "-c", "printf 'term=%s\\n' \"$TERM\"; sleep 30"],
        })),
    ));
    let session_id = spawned["id"].as_str().expect("session id");

    assert!(
        screen_contains(&server, session_id, "term=xterm-256color"),
        "local shell output should include TERM=xterm-256color"
    );

    let _ = ok_of(call(
        &server,
        homie_proto::Method::SESSION_KILL,
        Some(json!({ "sessionID": session_id })),
    ));
}

/// Round-trips one request through the dispatcher the way a client would.
/// Dispatches one line the way `serve` would, with a throwaway socket
/// standing in for the connection's write half.
fn handle(server: &ControlServer, line: &[u8]) -> Option<ControlMessage> {
    let (writer, _peer) = UnixStream::pair().expect("socketpair");
    server.handle_line(line, &Arc::new(Mutex::new(writer)), &mut None)
}

fn call(server: &ControlServer, method: &str, params: Option<JsonValue>) -> ControlMessage {
    let request = ControlMessage::Request {
        id: 1,
        method: method.into(),
        params,
    };
    let line = serde_json::to_vec(&request).expect("encode");
    handle(server, &line).expect("a request gets a response")
}

fn ok_of(message: ControlMessage) -> JsonValue {
    match message {
        ControlMessage::Response { result: Ok(ok), .. } => ok,
        other => panic!("expected success, got {other:?}"),
    }
}

fn err_of(message: ControlMessage) -> ControlError {
    match message {
        ControlMessage::Response {
            result: Err(error), ..
        } => error,
        other => panic!("expected an error, got {other:?}"),
    }
}

#[test]
fn hello_reports_the_protocol_and_the_engine_build() {
    let temp = tempfile::tempdir().expect("temp");
    let server = server(temp.path());
    let result = ok_of(call(
        &server,
        "hello",
        Some(json!({ "proto": WIRE_VERSION, "build": "test-client" })),
    ));

    assert_eq!(result["proto"], WIRE_VERSION);
    assert!(
        result["build"]
            .as_str()
            .is_some_and(|b| b.contains("homie-engine")),
        "the handshake should say which engine answered: {result}"
    );
    assert!(result["pid"].as_i64().is_some_and(|pid| pid > 0));
    assert_eq!(result["engineKind"], homie_proto::RUST_ENGINE_KIND);
    assert_eq!(
        result["executableHash"].as_str().map(str::len),
        Some(64),
        "the app needs a stable content identity for upgrade coordination"
    );
}

#[test]
fn client_activity_drives_pr_monitor_visibility() {
    let temp = tempfile::tempdir().expect("temp");
    let server = server(temp.path());
    assert!(server.pr_monitor_wake().foreground_active());

    let _ = ok_of(call(
        &server,
        homie_proto::Method::CLIENT_SET_ACTIVE,
        Some(json!({ "active": false })),
    ));
    assert!(!server.pr_monitor_wake().foreground_active());
}

#[test]
fn a_client_on_another_protocol_is_told_so() {
    let temp = tempfile::tempdir().expect("temp");
    let server = server(temp.path());
    let error = err_of(call(
        &server,
        "hello",
        Some(json!({ "proto": 99, "build": "future-client" })),
    ));
    assert_eq!(error.code, "version_mismatch");
}

#[test]
fn session_capabilities_are_read_only_and_default_to_unsupported() {
    let temp = tempfile::tempdir().expect("temp");
    let server = server(temp.path());

    let missing = err_of(call(
        &server,
        homie_proto::Method::SESSION_CAPABILITIES,
        Some(json!({ "sessionID": "s_missing" })),
    ));
    assert_eq!(missing.code, "not_found");

    {
        let mut registry = server.registry.lock().expect("registry");
        registry.insert_record(test_record("s_shell"));
        let mut fake = test_record("s_fake");
        fake.kind = homie_proto::AgentKind::new(crate::driver::FAKE_DRIVER_ID);
        registry.insert_record(fake);
    }

    let shell = ok_of(call(
        &server,
        homie_proto::Method::SESSION_CAPABILITIES,
        Some(json!({ "sessionID": "s_shell" })),
    ));
    assert_eq!(shell["sessionID"], json!("s_shell"));
    assert_eq!(
        shell["capabilities"],
        serde_json::to_value(homie_proto::DriverCapabilities::default()).unwrap()
    );

    let before = server
        .registry
        .lock()
        .expect("registry")
        .record("s_shell")
        .expect("record");
    assert_eq!(before.status, homie_proto::SessionStatus::Idle);

    let fake = ok_of(call(
        &server,
        homie_proto::Method::SESSION_CAPABILITIES,
        Some(json!({ "sessionID": "s_fake" })),
    ));
    assert_eq!(fake["capabilities"]["steerMessage"], json!(true));
    assert_eq!(fake["capabilities"]["cancelTurn"], json!(true));
    assert_eq!(fake["capabilities"]["modelDiscovery"], json!(true));
    assert_eq!(fake["capabilities"]["nativeResumeCursor"], json!(true));

    let after = server
        .registry
        .lock()
        .expect("registry")
        .record("s_shell")
        .expect("record");
    assert_eq!(after, before, "capability query must not mutate records");
}

#[test]
fn the_claude_manifest_declares_its_injection_mechanisms() {
    // The spawn path reads these; a manifest-parsing regression would
    // silently ship screen-detected Claudes with no MCP tools.
    let engine = engine();
    let manifest = engine.manifest("claude-code").expect("claude manifest");
    let descriptor = manifest.agent.clone().expect("agent");
    assert!(descriptor.injection.claude_hooks);
    assert!(descriptor.injection.claude_mcp);
    assert!(descriptor.session_id_flag.is_some());

    let codex = engine.manifest("codex").expect("codex manifest");
    let codex_descriptor = codex.agent.clone().expect("agent");
    assert!(
        codex_descriptor.injection.codex_notify || codex_descriptor.injection.codex_mcp,
        "codex opts into at least one shim"
    );
}

#[test]
fn resuming_an_agent_directly_executes_the_agent() {
    let temp = tempfile::tempdir().expect("temp");
    let registry = Registry::new(engine(), temp.path().join("state.json"));
    let server = ControlServer::new(
        Arc::new(Mutex::new(Registry::new(
            engine(),
            temp.path().join("server-state.json"),
        ))),
        temp.path().join("daemon.sock"),
    );

    let spec = server
        .resume_spec(&registry, "s_resume", "claude-code", "/tmp", Some("uuid-1"))
        .expect("resume spec");
    // Claude declares `returnToLoginShell`, so the agent runs inside the
    // PTY's login shell rather than as its argv[0]; the resume flags still
    // have to reach the agent itself.
    let command = spec.pty.argv.last().expect("argv");
    assert!(
        command.contains("'claude'") && command.contains("'--resume' 'uuid-1'"),
        "resume flags must reach the agent: {command:?}"
    );
}

/// An agent that dies on its own — a dropped ssh, a crash — leaves its
/// session in the registry, because only an explicit kill takes one out.
/// Resume used to read that presence as "already live", call itself a
/// no-op and hand the corpse straight back, which left a dead session
/// with no way at all to restart it.
#[test]
fn resume_relaunches_a_session_whose_agent_died_on_its_own() {
    let temp = tempfile::tempdir().expect("temp");
    // A manifest that resumes by flag, onto a binary that outlives the
    // call: `sh -c 'read line'` blocks on the PTY instead of exiting.
    let manifests = temp.path().join("manifests");
    std::fs::create_dir_all(&manifests).expect("manifests dir");
    std::fs::write(
        manifests.join("probe.json"),
        json!({
            "schemaVersion": 2,
            "id": "probe",
            "version": "test",
            "statusModel": "full",
            "agent": {
                "binary": "/bin/sh",
                "spawnArgs": ["-c", "read line"],
                "resume": { "style": "flag", "token": "--resume" },
            },
            "rules": [],
        })
        .to_string(),
    )
    .expect("write manifest");
    let (probe, _) = ManifestEngine::load_dir(&manifests).expect("load");
    let probe = Arc::new(probe);

    let registry = Arc::new(Mutex::new(Registry::new(
        Arc::clone(&probe),
        temp.path().join("state.json"),
    )));
    {
        let mut guard = registry.lock().expect("registry");
        let mut record = test_record("s_dead");
        record.kind = homie_proto::AgentKind::new("probe");
        record.agent_session_id = Some("conv-1".into());
        // `true` exits the moment it is spawned, standing in for the agent
        // that went away while the daemon kept its session.
        guard
            .spawn(
                crate::session::SessionSpec {
                    id: "s_dead".into(),
                    pty: crate::pty::PtySpec::new(vec!["/usr/bin/true".into()], "/tmp"),
                    manifest_id: "probe".into(),
                    authority: crate::session::authority_for("probe", &probe),
                    logs_dir: temp.path().join("logs"),
                    holder: None,
                    remote: None,
                    defer_launch: false,
                },
                record,
            )
            .expect("spawn");
    }
    for _ in 0..100 {
        let exited = registry
            .lock()
            .expect("registry")
            .record("s_dead")
            .is_some_and(|record| matches!(record.status, homie_proto::SessionStatus::Exited(_)));
        if exited {
            break;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    assert!(
        registry.lock().expect("registry").get("s_dead").is_some(),
        "the premise: a dead agent's session stays in the registry"
    );

    let server = ControlServer::new(Arc::clone(&registry), temp.path().join("daemon.sock"));
    let result = ok_of(call(
        &server,
        "session.resume",
        Some(json!({ "sessionID": "s_dead" })),
    ));

    assert!(
        result["status"].get("exited").is_none(),
        "resume handed back the corpse instead of relaunching: {}",
        result["status"]
    );
    assert!(
        registry
            .lock()
            .expect("registry")
            .get("s_dead")
            .is_some_and(|session| !session.view().exited),
        "the resumed session must be a live one"
    );
}

#[test]
fn listing_sessions_returns_records_and_projects() {
    // The app decodes SessionListResult { sessions, projects }; both keys
    // must be present, as the reference implementation answers.
    let temp = tempfile::tempdir().expect("temp");
    let server = server(temp.path());
    let result = ok_of(call(&server, "session.list", None));
    assert!(result["sessions"].is_array());
    assert!(result["projects"].is_array());
    // state.snapshot is the same view under another name.
    let snapshot = ok_of(call(&server, "state.snapshot", None));
    assert!(snapshot["sessions"].is_array());
}

#[test]
fn an_unimplemented_method_is_not_found_rather_than_a_dropped_connection() {
    // A client that asks for something this engine has not ported yet must
    // get a clean error, the same as an older daemon would give.
    let temp = tempfile::tempdir().expect("temp");
    let server = server(temp.path());
    let error = err_of(call(&server, "session.never_implemented", Some(json!({}))));
    assert_eq!(error.code, "not_found");
}

#[test]
fn addressing_a_session_that_does_not_exist_is_an_error() {
    // Params use the wire spelling the app sends: `sessionID`, not `id`.
    let temp = tempfile::tempdir().expect("temp");
    let server = server(temp.path());
    let error = err_of(call(
        &server,
        "session.send_text",
        Some(json!({ "sessionID": "s_missing", "text": "hi", "submit": false })),
    ));
    assert_eq!(error.code, "not_found");
}

#[test]
fn record_mutations_round_trip_over_the_wire() {
    // rename → mark_seen → archive → unarchive against a record-only
    // session (no live process needed).
    let temp = tempfile::tempdir().expect("temp");
    let registry = Arc::new(Mutex::new(Registry::new(
        engine(),
        temp.path().join("state.json"),
    )));
    registry
        .lock()
        .expect("registry")
        .insert_record(test_record("s_rec"));
    let server = ControlServer::new(registry, temp.path().join("daemon.sock"));

    let params = json!({ "sessionID": "s_rec", "title": "renamed by hand" });
    ok_of(call(&server, "session.rename", Some(params)));
    ok_of(call(
        &server,
        "session.mark_seen",
        Some(json!({ "sessionID": "s_rec" })),
    ));
    ok_of(call(
        &server,
        "session.archive",
        Some(json!({ "sessionID": "s_rec" })),
    ));

    let list = ok_of(call(&server, "session.list", None));
    let record = &list["sessions"][0];
    assert_eq!(record["title"], "renamed by hand");
    // TitleSource is numeric on the wire (Swift Int-raw enum);
    // serialize the variant rather than hardcoding its index.
    assert_eq!(
        record["titleSource"],
        serde_json::to_value(homie_proto::TitleSource::UserRename).expect("encode")
    );
    assert!(record["lastSeenAt"].is_number());
    assert!(record["archivedAt"].is_number());

    ok_of(call(
        &server,
        "session.unarchive",
        Some(json!({ "sessionID": "s_rec" })),
    ));
    let list = ok_of(call(&server, "session.list", None));
    assert!(list["sessions"][0].get("archivedAt").is_none());

    ok_of(call(
        &server,
        "session.remove",
        Some(json!({ "sessionID": "s_rec" })),
    ));
    let list = ok_of(call(&server, "session.list", None));
    assert_eq!(list["sessions"].as_array().map(Vec::len), Some(0));
}

#[test]
fn a_hook_report_folds_identity_into_the_record() {
    let temp = tempfile::tempdir().expect("temp");
    let registry = Arc::new(Mutex::new(Registry::new(
        engine(),
        temp.path().join("state.json"),
    )));
    registry
        .lock()
        .expect("registry")
        .insert_record(test_record("s_hook"));
    let server = ControlServer::new(registry, temp.path().join("daemon.sock"));

    ok_of(call(
        &server,
        "hook.report",
        Some(json!({
            "kind": "claude-hook",
            "homieSessionID": "s_hook",
            "event": "UserPromptSubmit",
            "payload": {
                "session_id": "uuid-from-hook",
                "transcript_path": "/tmp/t.jsonl",
                "prompt": "fix the flaky test in ci",
            },
        })),
    ));

    let list = ok_of(call(&server, "session.list", None));
    let record = &list["sessions"][0];
    assert_eq!(record["agentSessionID"], "uuid-from-hook");
    assert_eq!(record["transcriptPath"], "/tmp/t.jsonl");
    assert_eq!(
        record["title"], "fix the flaky test in ci",
        "the first prompt titles a placeholder session"
    );
}

#[test]
fn project_ids_are_deterministic_and_idempotent() {
    let temp = tempfile::tempdir().expect("temp");
    let server = server(temp.path());
    let first = ok_of(call(
        &server,
        "project.add",
        Some(json!({ "root": "/Users/x/code/app" })),
    ));
    let second = ok_of(call(
        &server,
        "project.add",
        Some(json!({ "root": "/Users/x/code/app" })),
    ));
    assert_eq!(first["id"], second["id"], "re-adding never duplicates");
    assert!(
        first["id"].as_str().expect("id").starts_with("p_"),
        "{first}"
    );
    assert_eq!(first["name"], "app");
    let list = ok_of(call(&server, "session.list", None));
    assert_eq!(list["projects"].as_array().map(Vec::len), Some(1));
}

#[test]
fn agent_readiness_serves_the_catalog_with_descriptors() {
    let temp = tempfile::tempdir().expect("temp");
    let server = server(temp.path());
    let result = ok_of(call(&server, "agent.readiness", None));
    let agents = result["agents"].as_array().expect("agents");
    assert!(!agents.is_empty());
    let claude = agents
        .iter()
        .find(|agent| agent["kind"] == "claude-code")
        .expect("claude in the catalog");
    assert_eq!(claude["binary"], "claude");
    assert!(
        claude["descriptor"]["injection"]["claudeHooks"]
            .as_bool()
            .unwrap_or(false),
        "the raw manifest descriptor rides along: {claude}"
    );
}

#[test]
fn a_removed_session_can_be_reopened() {
    let temp = tempfile::tempdir().expect("temp");
    let registry = Arc::new(Mutex::new(Registry::new(
        engine(),
        temp.path().join("state.json"),
    )));
    registry
        .lock()
        .expect("registry")
        .insert_record(test_record("s_gone"));
    let server = ControlServer::new(registry, temp.path().join("daemon.sock"));

    ok_of(call(
        &server,
        "session.remove",
        Some(json!({ "sessionID": "s_gone" })),
    ));
    let list = ok_of(call(&server, "session.list", None));
    assert_eq!(list["sessions"].as_array().map(Vec::len), Some(0));

    let reopened = ok_of(call(&server, "session.reopen_last", None));
    assert_eq!(reopened["id"], "s_gone");
    let list = ok_of(call(&server, "session.list", None));
    assert_eq!(list["sessions"].as_array().map(Vec::len), Some(1));

    // The stack is spent.
    let empty = err_of(call(&server, "session.reopen_last", None));
    assert_eq!(empty.code, "bad_request");
}

#[test]
fn read_diff_reports_working_changes() {
    let temp = tempfile::tempdir().expect("temp");
    let repo = temp.path().join("repo");
    std::fs::create_dir_all(&repo).expect("mkdir");
    let git = |arguments: &[&str]| {
        let status = std::process::Command::new("git")
            .args(arguments)
            .current_dir(&repo)
            .env("GIT_AUTHOR_NAME", "t")
            .env("GIT_AUTHOR_EMAIL", "t@t")
            .env("GIT_COMMITTER_NAME", "t")
            .env("GIT_COMMITTER_EMAIL", "t@t")
            .status()
            .expect("git");
        assert!(status.success(), "git {arguments:?}");
    };
    git(&["init", "-q", "-b", "main"]);
    std::fs::write(repo.join("file.txt"), "original\n").expect("write");
    git(&["add", "."]);
    git(&["commit", "-q", "-m", "root"]);
    std::fs::write(repo.join("file.txt"), "changed by the session\n").expect("write");

    let registry = Arc::new(Mutex::new(Registry::new(
        engine(),
        temp.path().join("state.json"),
    )));
    let mut record = test_record("s_diff");
    record.cwd = repo.to_string_lossy().into_owned();
    registry.lock().expect("registry").insert_record(record);
    let server = ControlServer::new(registry, temp.path().join("daemon.sock"));

    let result = ok_of(call(
        &server,
        "session.read_diff",
        Some(json!({ "sessionID": "s_diff" })),
    ));
    assert_eq!(result["truncated"], false);
    // The patch travels base64-encoded, as the reference implementation sends it.
    use base64::Engine as _;
    let patch = base64::engine::general_purpose::STANDARD
        .decode(result["patch"].as_str().expect("patch"))
        .expect("base64");
    let patch = String::from_utf8_lossy(&patch);
    assert!(
        patch.contains("changed by the session"),
        "the working change is in the patch: {patch}"
    );
}

#[test]
fn worktrees_are_managed_over_the_wire() {
    let temp = tempfile::tempdir().expect("temp");
    let repo = temp.path().join("repo");
    std::fs::create_dir_all(&repo).expect("mkdir");
    for arguments in [
        vec!["init", "-b", "main"],
        vec!["commit", "--allow-empty", "-m", "root"],
    ] {
        let status = std::process::Command::new("git")
            .args(&arguments)
            .arg("--quiet")
            .current_dir(&repo)
            .env("GIT_AUTHOR_NAME", "t")
            .env("GIT_AUTHOR_EMAIL", "t@t")
            .env("GIT_COMMITTER_NAME", "t")
            .env("GIT_COMMITTER_EMAIL", "t@t")
            .status()
            .expect("git");
        assert!(status.success(), "git {arguments:?}");
    }
    let server = server(temp.path());
    let repo_path = repo.to_string_lossy();

    let created = ok_of(call(
        &server,
        "worktree.create",
        Some(json!({ "repoPath": repo_path, "branch": "feature/x" })),
    ));
    assert_eq!(created["branch"], "feature/x");

    let list = ok_of(call(
        &server,
        "worktree.list",
        Some(json!({ "repoPath": repo_path })),
    ));
    let listed = list.as_array().expect("array");
    assert!(
        listed
            .iter()
            .any(|worktree| worktree["branch"] == "feature/x"),
        "{list}"
    );

    ok_of(call(
        &server,
        "worktree.remove",
        Some(json!({
            "repoPath": repo_path,
            "worktreePath": created["path"],
            "force": true,
        })),
    ));
    let list = ok_of(call(
        &server,
        "worktree.list",
        Some(json!({ "repoPath": repo_path })),
    ));
    assert!(
        !list
            .as_array()
            .expect("array")
            .iter()
            .any(|worktree| worktree["branch"] == "feature/x")
    );
}

#[test]
fn missing_parameters_are_rejected_before_anything_happens() {
    let temp = tempfile::tempdir().expect("temp");
    let server = server(temp.path());
    assert_eq!(
        err_of(call(&server, "session.send_text", None)).code,
        "bad_request"
    );
    assert_eq!(
        err_of(call(&server, "session.resize", Some(json!({ "id": "s" })))).code,
        "bad_request"
    );
}

#[test]
fn remote_spawn_fails_with_the_structured_transport_error() {
    let temp = tempfile::tempdir().expect("temp");
    let server = server(temp.path());
    let error = err_of(call(
        &server,
        "session.spawn",
        Some(json!({
            "kind": { "shell": {} },
            "cwd": "/tmp",
            "host": "forge",
        })),
    ));
    assert_eq!(error.code, crate::remote::TRANSPORT_UNAVAILABLE_CODE);
    assert!(
        server
            .registry
            .lock()
            .expect("registry")
            .records()
            .is_empty(),
        "an unavailable remote transport must not create a session record"
    );
}

#[test]
fn host_initialization_fails_closed_without_the_remote_transport() {
    let temp = tempfile::tempdir().expect("temp");
    homie_proto::HostsConfig {
        hosts: vec![homie_proto::HostEntry {
            id: "forge".into(),
            name: Some("Forge".into()),
            ssh: "you@forge".into(),
            default_cwd: None,
            node: None,
        }],
    }
    .save(temp.path().join("hosts.json"))
    .expect("host catalog");
    let server = server(temp.path());

    let error = err_of(call(
        &server,
        Method::HOST_INITIALIZE,
        Some(json!({ "host": "forge" })),
    ));

    assert_eq!(error.code, crate::remote::TRANSPORT_UNAVAILABLE_CODE);
}

#[test]
fn malformed_json_gets_an_error_rather_than_silence() {
    // A client waiting on a reply should learn that none is coming.
    let temp = tempfile::tempdir().expect("temp");
    let server = server(temp.path());
    let response = handle(&server, b"{ not json").expect("a response");
    assert_eq!(err_of(response).code, "bad_request");
}

#[test]
fn responses_and_events_from_a_client_are_ignored() {
    let temp = tempfile::tempdir().expect("temp");
    let server = server(temp.path());
    let event = serde_json::to_vec(&ControlMessage::Event {
        name: "session.updated".into(),
        seq: 1,
        params: json!({}),
    })
    .expect("encode");
    assert!(
        handle(&server, &event).is_none(),
        "the daemon sends events; it does not answer them"
    );
}

#[test]
fn the_socket_is_owner_only() {
    use std::os::unix::fs::PermissionsExt;
    let temp = tempfile::tempdir().expect("temp");
    let server = server(temp.path());
    let _listener = server.bind().expect("bind");

    let mode = std::fs::metadata(server.socket_path())
        .expect("stat")
        .permissions()
        .mode();
    assert_eq!(
        mode & 0o777,
        0o600,
        "the control socket can spawn processes as the user"
    );
}

#[test]
fn binding_over_a_live_socket_is_refused() {
    let temp = tempfile::tempdir().expect("temp");
    let server = server(temp.path());
    let _listener = server.bind().expect("first bind");

    let second = ControlServer::new(
        Arc::new(Mutex::new(Registry::new(
            engine(),
            temp.path().join("state.json"),
        ))),
        server.socket_path(),
    );
    let error = second
        .bind()
        .expect_err("two engines must not share a socket");
    assert_eq!(error.kind(), std::io::ErrorKind::AddrInUse);
}

#[test]
fn a_stale_socket_file_is_replaced() {
    // The daemon died without cleaning up; the next start must not be
    // blocked by the leftover file.
    let temp = tempfile::tempdir().expect("temp");
    let path = temp.path().join("daemon.sock");
    std::fs::write(&path, b"").expect("leave a stale file");

    let server = ControlServer::new(
        Arc::new(Mutex::new(Registry::new(
            engine(),
            temp.path().join("state.json"),
        ))),
        &path,
    );
    let _listener = server.bind().expect("a stale socket should be replaced");
}

#[test]
fn workspace_trust_auto_accept_is_narrowly_scoped_to_claudes_exact_picker() {
    assert!(is_claude_workspace_trust_screen(
        "1. Yes, I trust this folder\n2. No, exit"
    ));
    assert!(!is_claude_workspace_trust_screen(
        "1. Yes, allow this shell command\n2. No"
    ));
    assert!(!is_claude_workspace_trust_screen(
        "Yes, I trust this folder"
    ));
}
