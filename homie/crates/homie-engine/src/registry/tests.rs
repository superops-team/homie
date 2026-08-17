use super::*;
use homie_proto::{AgentKind, DateMillis, ProjectId, Resumability, SessionId, TitleSource};

use super::persisted::repair_persisted_agent_title;

fn record(id: &str) -> SessionRecord {
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
        status: SessionStatus::Starting,
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

fn engine() -> Arc<ManifestEngine> {
    let dir = crate::detect::bundled_manifest_dir()
        .canonicalize()
        .expect("manifests");
    let (engine, _) = ManifestEngine::load_dir(&dir).expect("load");
    Arc::new(engine)
}

#[test]
fn state_round_trips_through_the_swift_file_shape() {
    let temp = tempfile::tempdir().expect("temp");
    let state_file = temp.path().join("state.json");

    let mut registry = Registry::new(engine(), &state_file);
    registry.records.insert("s_1".into(), record("s_1"));
    registry.persist().expect("persist");

    // The shape on disk is what the reference implementation expects.
    let raw: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&state_file).expect("read")).expect("parse");
    assert_eq!(raw["version"], 1);
    assert!(raw["sessions"].is_array());
    assert!(raw["projects"].is_array());
    assert_eq!(raw["sessions"][0]["id"], "s_1");

    let mut reloaded = Registry::new(engine(), &state_file);
    assert_eq!(reloaded.load().expect("load"), 1);
    assert_eq!(reloaded.records()[0].id.0, "s_1");
}

#[test]
fn split_store_dry_run_migration_writes_nothing() {
    let temp = tempfile::tempdir().expect("temp");
    let state_file = temp.path().join("state.json");
    let split_root = temp.path().join("split");
    let state = PersistedState::current(
        vec![record("s_1"), record("s_2")],
        vec![serde_json::json!({"id": "p", "root": "/tmp", "name": "tmp"})],
    );
    std::fs::write(&state_file, serde_json::to_vec(&state).expect("encode")).expect("write");

    let report =
        migrate_envelope_to_split(&state_file, &split_root, true).expect("dry-run migration");
    assert!(report.dry_run);
    assert_eq!(report.project_count, 1);
    assert_eq!(report.session_count, 2);
    assert_eq!(report.backup_path, None);
    assert!(!split_root.join("projects.json").exists());
    assert!(!split_root.join("sessions").exists());
    assert!(state_file.exists());
}

#[test]
fn split_store_migration_preserves_backup_and_loads_records() {
    let temp = tempfile::tempdir().expect("temp");
    let state_file = temp.path().join("state.json");
    let split_root = temp.path().join("split");
    let mut first = record("s_1");
    first.title = "first".into();
    let mut second = record("s_2");
    second.title = "second".into();
    let state = PersistedState::current(
        vec![first, second],
        vec![serde_json::json!({"id": "p", "root": "/tmp", "name": "tmp"})],
    );
    std::fs::write(&state_file, serde_json::to_vec(&state).expect("encode")).expect("write");

    let report =
        migrate_envelope_to_split(&state_file, &split_root, false).expect("apply migration");
    assert!(!report.dry_run);
    assert_eq!(report.project_count, 1);
    assert_eq!(report.session_count, 2);
    assert!(
        report
            .backup_path
            .as_ref()
            .is_some_and(|path| path.exists())
    );
    assert!(state_file.exists(), "source envelope remains in place");

    let split = SplitJsonStore::new(&split_root);
    assert_eq!(split.load_projects().expect("projects").len(), 1);
    let sessions = split.load_sessions().expect("sessions");
    assert_eq!(sessions.len(), 2);
    assert_eq!(sessions[0].id.0, "s_1");
    assert_eq!(sessions[1].id.0, "s_2");
}

#[test]
fn split_store_quarantines_one_corrupt_session_file() {
    let temp = tempfile::tempdir().expect("temp");
    let split = SplitJsonStore::new(temp.path());
    split.save_session(&record("s_1")).expect("save s_1");
    split.save_session(&record("s_2")).expect("save s_2");
    let bad = temp.path().join("sessions").join("s_bad.json");
    std::fs::write(&bad, b"{ not json").expect("write bad");

    let sessions = split.load_sessions().expect("load sessions");
    assert_eq!(sessions.len(), 2);
    assert!(sessions.iter().any(|session| session.id.0 == "s_1"));
    assert!(sessions.iter().any(|session| session.id.0 == "s_2"));
    assert!(!bad.exists());
    assert!(
        temp.path()
            .join("sessions")
            .join("s_bad.json.corrupt")
            .exists()
    );
}

#[test]
fn loading_repairs_same_path_sessions_into_host_scoped_projects() {
    let temp = tempfile::tempdir().expect("temp");
    let state_file = temp.path().join("state.json");
    let mut forge = record("forge");
    forge.cwd = "/srv/app".into();
    forge.host = Some("forge".into());
    let mut build = record("build");
    build.cwd = "/srv/app".into();
    build.host = Some("build".into());
    let state = PersistedState::current(vec![forge, build], Vec::new());
    std::fs::write(&state_file, serde_json::to_vec(&state).expect("encode")).expect("write");

    let mut registry = Registry::new(engine(), &state_file);
    registry.load().expect("load");
    let records = registry.records();
    assert_ne!(records[0].project_id, records[1].project_id);
    assert_eq!(registry.projects_raw().len(), 2);
}

/// Older records stored `projectID` as the raw directory path instead of a
/// hashed id. Load recomputes identity, so those are repaired in place
/// rather than left as a second, path-shaped namespace — and records that
/// already carry a hashed id keep it, so an existing sidebar does not
/// fragment into duplicate project rows.
#[test]
fn loading_repairs_path_shaped_project_ids_and_leaves_hashed_ones_alone() {
    let temp = tempfile::tempdir().expect("temp");
    let state_file = temp.path().join("state.json");
    let root = "/workspace/app";

    let mut legacy = record("legacy");
    legacy.cwd = root.into();
    legacy.project_id = ProjectId(root.to_owned());
    let mut hashed = record("hashed");
    hashed.cwd = root.into();
    hashed.project_id = session_project_id(root, None);
    let expected = hashed.project_id.clone();

    let state = PersistedState::current(vec![legacy, hashed], Vec::new());
    std::fs::write(&state_file, serde_json::to_vec(&state).expect("encode")).expect("write");

    let mut registry = Registry::new(engine(), &state_file);
    registry.load().expect("load");
    let records = registry.records();
    assert!(
        records.iter().all(|record| record.project_id == expected),
        "both records should share one repaired project identity: {:?}",
        records
            .iter()
            .map(|record| &record.project_id)
            .collect::<Vec<_>>()
    );
    assert_eq!(
        registry.projects_raw().len(),
        1,
        "the repair must not leave a second project row behind"
    );
}

#[test]
fn loading_keeps_a_linked_worktree_under_its_project_root() {
    let temp = tempfile::tempdir().expect("temp");
    let state_file = temp.path().join("state.json");
    let project_root = "/workspace/app";
    let project_id = session_project_id(project_root, None);
    let mut worktree = record("worktree");
    worktree.cwd = "/workspace/app-feature".into();
    worktree.worktree_path = Some(worktree.cwd.clone());
    worktree.project_id = project_id.clone();
    let state = PersistedState::current(
        vec![worktree],
        vec![serde_json::json!({
            "id": project_id.0,
            "root": project_root,
            "name": "app"
        })],
    );
    std::fs::write(&state_file, serde_json::to_vec(&state).expect("encode")).expect("write");

    let mut registry = Registry::new(engine(), &state_file);
    registry.load().expect("load");
    let loaded = registry.records().pop().expect("record");
    assert_eq!(loaded.project_id, session_project_id(project_root, None));
    assert_eq!(registry.projects_raw().len(), 1);
}

/// An exited record whose agent had named its conversation is the case
/// every Resume affordance gates on, and each of them checks for
/// `Resumable` — a record left on `Live` reads to all of them as "cannot
/// be resumed" and the button is never drawn.
#[test]
fn a_conversation_that_outlived_its_session_reports_resumable() {
    let temp = tempfile::tempdir().expect("temp");
    let mut registry = Registry::new(engine(), temp.path().join("state.json"));

    let mut dead = record("s_dead");
    dead.kind = AgentKind::CLAUDE_CODE;
    dead.agent_session_id = Some("conv-1".into());
    dead.resumability = Resumability::Live;
    dead.status = SessionStatus::Exited(homie_proto::ExitInfo {
        reason: homie_proto::ExitReason::Exited,
        code: Some(255),
        signal: None,
    });
    registry.records.insert("s_dead".into(), dead);

    assert_eq!(
        registry.record("s_dead").expect("record").resumability,
        Resumability::Resumable
    );
}

/// The machine-death case. Holders die with the Mac, so the records they
/// were reporting for come back saying `Working` with nobody behind them.
/// Left alone they read as running to every consumer: the app dials a
/// socket that will never answer and spins "Reconnecting terminal…"
/// forever, and no Resume is offered because the session still looks live.
#[test]
fn a_local_session_whose_holder_died_with_the_machine_is_reaped_into_resumable() {
    let temp = tempfile::tempdir().expect("temp");
    let mut registry = Registry::new(engine(), temp.path().join("state.json"));

    let mut orphan = record("s_orphan");
    orphan.kind = AgentKind::CLAUDE_CODE;
    orphan.agent_session_id = Some("conv-1".into());
    orphan.resumability = Resumability::Live;
    orphan.status = SessionStatus::Working;
    registry.records.insert("s_orphan".into(), orphan);

    // No holder sockets: exactly what an empty holders dir looks like
    // after the machine that owned them went down.
    let holders_dir = temp.path().join("holders");
    std::fs::create_dir_all(&holders_dir).expect("holders dir");
    let holder = HolderConfig {
        holders_dir,
        executable: temp.path().join("homie-holder"),
    };
    assert!(registry.restore(&holder, temp.path()).is_empty());

    let reaped = registry.record("s_orphan").expect("record");
    assert!(matches!(reaped.status, SessionStatus::Exited(_)));
    assert_eq!(reaped.resumability, Resumability::Resumable);
}

/// Remote sessions live in tmux on another machine: they outlive this
/// daemon and this Mac, so the reap pass must not touch them. Marking one
/// exited would strand still-running work behind a Resume button that
/// starts a second agent on top of the first.
#[test]
fn a_remote_session_survives_the_reap() {
    let temp = tempfile::tempdir().expect("temp");
    let mut registry = Registry::new(engine(), temp.path().join("state.json"));

    let mut remote = record("s_remote");
    remote.kind = AgentKind::CLAUDE_CODE;
    remote.host = Some("forge".into());
    remote.status = SessionStatus::Working;
    registry.records.insert("s_remote".into(), remote);

    let holders_dir = temp.path().join("holders");
    std::fs::create_dir_all(&holders_dir).expect("holders dir");
    let holder = HolderConfig {
        holders_dir,
        executable: temp.path().join("homie-holder"),
    };
    registry.restore(&holder, temp.path());

    assert_eq!(
        registry.record("s_remote").expect("record").status,
        SessionStatus::Working
    );
}

/// Without a conversation id there is nothing to re-enter, and offering
/// Resume would only produce an agent that fails to launch.
#[test]
fn an_exited_session_with_no_conversation_id_is_not_resumable() {
    let temp = tempfile::tempdir().expect("temp");
    let mut registry = Registry::new(engine(), temp.path().join("state.json"));

    let mut dead = record("s_dead");
    dead.kind = AgentKind::CLAUDE_CODE;
    dead.resumability = Resumability::Live;
    dead.status = SessionStatus::Exited(homie_proto::ExitInfo {
        reason: homie_proto::ExitReason::Exited,
        code: Some(0),
        signal: None,
    });
    registry.records.insert("s_dead".into(), dead);

    assert_eq!(
        registry.record("s_dead").expect("record").resumability,
        Resumability::NotResumable
    );
}

/// A running session keeps saying `Live`: resumability only becomes a
/// question once the agent is gone.
#[test]
fn a_running_session_keeps_reporting_live() {
    let temp = tempfile::tempdir().expect("temp");
    let mut registry = Registry::new(engine(), temp.path().join("state.json"));

    let mut running = record("s_live");
    running.kind = AgentKind::CLAUDE_CODE;
    running.agent_session_id = Some("conv-1".into());
    running.resumability = Resumability::Live;
    running.status = SessionStatus::Idle;
    registry.records.insert("s_live".into(), running);

    assert_eq!(
        registry.record("s_live").expect("record").resumability,
        Resumability::Live
    );
}

/// Interop against the state file the reference implementation actually maintains.
///
/// Ignored by default because it needs a real one. Point
/// `HOMIE_INTEROP_STATE` at a **copy** — never at the live file, which the
/// running daemon rewrites:
///
/// ```sh
/// cp "~/Library/Application Support/Homie/state.json" /tmp/state.json
/// HOMIE_INTEROP_STATE=/tmp/state.json cargo test -p homie-engine -- --ignored
/// ```
#[test]
#[ignore = "needs HOMIE_INTEROP_STATE pointing at a copy of a Swift-written state.json"]
fn reads_the_state_file_the_swift_daemon_wrote() {
    let Ok(raw) = std::env::var("HOMIE_INTEROP_STATE") else {
        eprintln!("skipped: HOMIE_INTEROP_STATE is not set");
        return;
    };
    let path = PathBuf::from(raw);
    let original: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).expect("read")).expect("parse");
    let session_count = original["sessions"].as_array().map_or(0, Vec::len);
    let project_count = original["projects"].as_array().map_or(0, Vec::len);
    assert!(session_count > 0, "pick a state file with sessions in it");

    let temp = tempfile::tempdir().expect("temp");
    let working = temp.path().join("state.json");
    std::fs::copy(&path, &working).expect("copy");

    let mut registry = Registry::new(engine(), &working);
    assert_eq!(
        registry.load().expect("the real state file must parse"),
        session_count,
        "every session record should survive the round trip"
    );

    // Writing it back must not lose anything the reference implementation owns.
    registry.persist().expect("persist");
    let rewritten: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&working).expect("read")).expect("parse");
    assert_eq!(rewritten["version"], 1);
    assert_eq!(
        rewritten["projects"].as_array().map_or(0, Vec::len),
        project_count,
        "projects this engine does not model must be carried through"
    );
    assert_eq!(
        rewritten["sessions"].as_array().map_or(0, Vec::len),
        session_count
    );
}

#[test]
fn a_missing_state_file_is_a_fresh_start_not_an_error() {
    let temp = tempfile::tempdir().expect("temp");
    let mut registry = Registry::new(engine(), temp.path().join("absent.json"));
    assert_eq!(registry.load().expect("load"), 0);
}

#[test]
fn an_unparseable_state_file_is_quarantined_rather_than_overwritten() {
    // Treating a corrupt file as a fresh install would erase every session
    // record on the next write.
    let temp = tempfile::tempdir().expect("temp");
    let state_file = temp.path().join("state.json");
    std::fs::write(&state_file, b"{ not json").expect("write");

    let mut registry = Registry::new(engine(), &state_file);
    let error = registry.load().expect_err("corrupt state must be an error");
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);

    assert!(
        temp.path().join("state.json.corrupt").exists(),
        "the unreadable file should still be recoverable by hand"
    );
}

#[test]
fn unknown_projects_survive_a_write() {
    // Additive fields outside the minimal Project model are not discarded.
    let temp = tempfile::tempdir().expect("temp");
    let state_file = temp.path().join("state.json");
    std::fs::write(
        &state_file,
        br#"{"version":1,"projects":[{"id":"p1","name":"keep me"}],"sessions":[]}"#,
    )
    .expect("write");

    let mut registry = Registry::new(engine(), &state_file);
    registry.load().expect("load");
    registry.persist().expect("persist");

    let raw: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&state_file).expect("read")).expect("parse");
    assert_eq!(raw["projects"][0]["name"], "keep me");
}

#[test]
fn project_identity_includes_the_execution_host() {
    let local = session_project_id("/workspace/app", None);
    let forge = session_project_id("/workspace/app", Some("forge"));
    let build = session_project_id("/workspace/app", Some("build"));
    assert_ne!(local, forge);
    assert_ne!(forge, build);
    assert_eq!(forge, session_project_id("/workspace/app", Some("forge")));
}

#[test]
fn live_claude_metadata_promotes_the_generated_conversation_title() {
    let temp = tempfile::tempdir().expect("temp");
    let transcript = temp.path().join("conversation.jsonl");
    std::fs::write(
        &transcript,
        "{\"type\":\"user\",\"message\":{\"content\":\"vague prompt\"}}\n\
         {\"type\":\"ai-title\",\"aiTitle\":\"Repair remote session recovery\"}\n",
    )
    .expect("write transcript");
    let mut registry = Registry::new(engine(), temp.path().join("state.json"));
    let mut session = record("claude");
    session.kind = AgentKind::CLAUDE_CODE;
    session.title = "vague prompt".to_owned();
    session.title_source = TitleSource::FirstPrompt;
    registry.insert_record(session);

    assert!(registry.apply_hook_metadata(
        "claude",
        &crate::hooks::HookMetadata {
            transcript_path: Some(transcript.to_string_lossy().into_owned()),
            ..crate::hooks::HookMetadata::default()
        }
    ));

    let updated = registry.record("claude").expect("record");
    assert_eq!(updated.title, "Repair remote session recovery");
    assert_eq!(updated.title_source, TitleSource::AgentProvided);
}

#[test]
fn pty_titles_are_filtered_fallbacks_and_never_override_user_renames() {
    let view = SessionView {
        id: "claude".to_owned(),
        status: SessionStatus::Working,
        needs_input: None,
        title: Some("Repair remote attach".to_owned()),
        title_source: Some(TitleSource::AgentProvided),
        tail_offset: 0,
        exited: false,
    };
    let mut provisional = record("claude");
    provisional.kind = AgentKind::CLAUDE_CODE;
    fold_session_view(&mut provisional, &view);
    assert_eq!(provisional.title, "Repair remote attach");
    assert_eq!(provisional.title_source, TitleSource::AgentProvided);

    let mut renamed = record("renamed");
    renamed.kind = AgentKind::CLAUDE_CODE;
    renamed.title = "My fixed title".to_owned();
    renamed.title_source = TitleSource::UserRename;
    fold_session_view(&mut renamed, &view);
    assert_eq!(renamed.title, "My fixed title");

    let mut first_prompt = record("first-prompt");
    first_prompt.kind = AgentKind::CODEX;
    first_prompt.title = "Initial vague request".to_owned();
    first_prompt.title_source = TitleSource::FirstPrompt;
    fold_session_view(&mut first_prompt, &view);
    assert_eq!(first_prompt.title, "Repair remote attach");
    assert_eq!(first_prompt.title_source, TitleSource::AgentProvided);

    let mut captured_prompt = record("captured-prompt");
    captured_prompt.kind = AgentKind::CODEX;
    let prompt_view = SessionView {
        title: Some("Implement terminal IME".to_owned()),
        title_source: Some(TitleSource::FirstPrompt),
        ..view.clone()
    };
    fold_session_view(&mut captured_prompt, &prompt_view);
    assert_eq!(captured_prompt.title, "Implement terminal IME");
    assert_eq!(captured_prompt.title_source, TitleSource::FirstPrompt);

    let mut generic = record("generic");
    generic.kind = AgentKind::CODEX;
    generic.cwd = "/work/homie".to_owned();
    let generic_view = SessionView {
        title: Some("homie".to_owned()),
        ..view
    };
    fold_session_view(&mut generic, &generic_view);
    assert_eq!(generic.title_source, TitleSource::Placeholder);

    let mut decorated = record("decorated");
    decorated.kind = AgentKind::CLAUDE_CODE;
    let decorated_view = SessionView {
        title: Some("✳ Claude Code".to_owned()),
        ..generic_view
    };
    fold_session_view(&mut decorated, &decorated_view);
    assert_eq!(decorated.title_source, TitleSource::Placeholder);

    decorated.title = "✳ Claude Code".to_owned();
    decorated.title_source = TitleSource::AgentProvided;
    assert!(repair_persisted_agent_title(&mut decorated));
    assert_eq!(decorated.title, AgentKind::CLAUDE_CODE_ID);
    assert_eq!(decorated.title_source, TitleSource::Placeholder);
}
