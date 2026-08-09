use homie_storage::{
    CreateSession, HistoryEntryUpsert, ProjectUpsert, RecordUsage, SessionCoreMetadataUpdate,
    StorageConfig, StorageError, UsageQuery, WorktreeUpsert, open_or_create,
};
use serde_json::json;

fn open_migrated_storage() -> homie_storage::Storage {
    let temp = tempfile::tempdir().expect("tempdir");
    let storage = open_or_create(StorageConfig {
        data_dir: temp.keep(),
    })
    .expect("open storage");
    storage.migrate().expect("migrate");
    storage
}

fn seed_default_session(
    storage: &homie_storage::Storage,
    title: &str,
) -> homie_storage::SessionSummary {
    storage.seed_defaults().expect("seed defaults");
    storage
        .create_session(CreateSession {
            workspace: std::env::temp_dir().join("homie-storage-test"),
            title: Some(title.to_string()),
        })
        .expect("create session")
}

#[test]
fn storage_schema_inventory_covers_diri_phase_one() {
    let storage = open_migrated_storage();
    let inventory = storage.schema_inventory().expect("schema inventory");

    for table in [
        "preferences",
        "projects",
        "worktrees",
        "sessions",
        "history_entries",
        "model_pricing",
        "pricing_snapshots",
        "usage_records",
        "usage_scan_files",
        "usage_hourly_rollups",
    ] {
        assert!(inventory.has_table(table), "missing table {table}");
    }

    let sessions = inventory.table("sessions").expect("sessions table");
    for column in [
        "project_id",
        "worktree_path",
        "git_branch",
        "title_source",
        "agent_session_id",
        "transcript_path",
        "needs_input_kind",
        "needs_input_payload_json",
        "resumability",
        "parent_session_id",
        "pinned",
        "archived_at",
        "remote_active",
        "host_id",
        "foreground_agent_kind",
        "memory_bytes",
    ] {
        assert!(sessions.has_column(column), "sessions missing {column}");
    }

    let history = inventory.table("history_entries").expect("history table");
    for column in ["external_id", "title_source", "metadata_json"] {
        assert!(
            history.has_column(column),
            "history_entries missing {column}"
        );
    }
    assert!(history.has_unique_index(&["agent_kind", "external_id"]));

    let worktrees = inventory.table("worktrees").expect("worktrees table");
    for column in ["head_sha", "is_bare", "is_detached", "is_prunable"] {
        assert!(worktrees.has_column(column), "worktrees missing {column}");
    }
    assert!(worktrees.has_unique_index(&["path"]));
    assert!(worktrees.has_unique_index(&["project_id", "branch"]));

    let usage = inventory.table("usage_records").expect("usage table");
    for column in [
        "cache_write_5m_tokens",
        "cache_write_1h_tokens",
        "billed_cost",
        "value_kind",
        "source",
        "source_event_id",
    ] {
        assert!(usage.has_column(column), "usage_records missing {column}");
    }
    assert!(usage.has_unique_index(&["provider_id", "source", "source_event_id"]));
}

#[test]
fn history_repository_upserts_lists_and_tracks_entries() {
    let storage = open_migrated_storage();
    let session = seed_default_session(&storage, "Tracked");

    let original = storage
        .upsert_history_entry(HistoryEntryUpsert {
            agent_kind: "codex".to_string(),
            external_id: "thread-1".to_string(),
            cwd: "/work/a".into(),
            title: Some("First prompt".to_string()),
            title_source: "first_prompt".to_string(),
            transcript_path: "/tmp/thread-1.jsonl".into(),
            last_active_at: 10,
            created_at: Some(1),
            cwd_exists: true,
            metadata: json!({"source":"codex"}),
        })
        .expect("insert history");

    let updated = storage
        .upsert_history_entry(HistoryEntryUpsert {
            agent_kind: "codex".to_string(),
            external_id: "thread-1".to_string(),
            cwd: "/work/a".into(),
            title: Some("Updated title".to_string()),
            title_source: "agent_title".to_string(),
            transcript_path: "/tmp/thread-1.jsonl".into(),
            last_active_at: 30,
            created_at: Some(1),
            cwd_exists: true,
            metadata: json!({"source":"codex","seen":true}),
        })
        .expect("update history");
    assert_eq!(original.id, updated.id);
    assert_eq!(updated.title.as_deref(), Some("Updated title"));

    storage
        .upsert_history_entry(HistoryEntryUpsert {
            agent_kind: "claude_code".to_string(),
            external_id: "claude-1".to_string(),
            cwd: "/work/b".into(),
            title: Some("Claude".to_string()),
            title_source: "ai_title".to_string(),
            transcript_path: "/tmp/claude-1.jsonl".into(),
            last_active_at: 20,
            created_at: Some(2),
            cwd_exists: false,
            metadata: json!({}),
        })
        .expect("insert second history");

    storage
        .mark_history_entry_tracked("codex", "thread-1", &session.id)
        .expect("track history");

    let entries = storage.list_history_entries(10).expect("list history");
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].external_id, "thread-1");
    assert_eq!(
        entries[0].tracked_session_id.as_deref(),
        Some(session.id.as_str())
    );
    assert_eq!(entries[1].external_id, "claude-1");
}

#[test]
fn project_worktree_repository_enforces_identity_and_lists_by_project() {
    let storage = open_migrated_storage();
    let session = seed_default_session(&storage, "Worktree");

    let project = storage
        .upsert_project(ProjectUpsert {
            root_path: "/repo/main".into(),
            display_name: Some("main".to_string()),
            remote_origin: Some("git@example.invalid:repo/main.git".to_string()),
            pinned_order: Some(1),
        })
        .expect("insert project");
    let same_project = storage
        .upsert_project(ProjectUpsert {
            root_path: "/repo/main".into(),
            display_name: Some("main renamed".to_string()),
            remote_origin: None,
            pinned_order: Some(2),
        })
        .expect("update project");
    assert_eq!(project.id, same_project.id);
    assert_eq!(same_project.display_name.as_deref(), Some("main renamed"));

    let worktree = storage
        .upsert_worktree(WorktreeUpsert {
            project_id: project.id.clone(),
            session_id: Some(session.id.clone()),
            path: "/repo/main-feature".into(),
            branch: Some("feature/storage".to_string()),
            head_sha: Some("abc123".to_string()),
            is_bare: false,
            is_detached: false,
            is_prunable: true,
            dirty: true,
            merged: false,
            stale_suggestion: true,
        })
        .expect("insert worktree");
    assert_eq!(worktree.session_id.as_deref(), Some(session.id.as_str()));
    assert!(worktree.is_prunable);
    assert!(worktree.dirty);

    let duplicate_branch = storage.upsert_worktree(WorktreeUpsert {
        project_id: project.id.clone(),
        session_id: None,
        path: "/repo/other-feature".into(),
        branch: Some("feature/storage".to_string()),
        head_sha: None,
        is_bare: false,
        is_detached: false,
        is_prunable: false,
        dirty: false,
        merged: false,
        stale_suggestion: false,
    });
    assert!(
        duplicate_branch.is_err(),
        "branch identity should be unique per project"
    );

    let worktrees = storage
        .list_worktrees_for_project(&project.id)
        .expect("list worktrees");
    assert_eq!(worktrees.len(), 1);
    assert_eq!(worktrees[0].path, "/repo/main-feature");
}

#[test]
fn session_core_metadata_round_trips_diri_record_subset() {
    let storage = open_migrated_storage();
    let parent = seed_default_session(&storage, "Parent");
    let child = storage
        .create_session(CreateSession {
            workspace: "/repo/main".into(),
            title: Some("Child".to_string()),
        })
        .expect("create child");
    let project = storage
        .upsert_project(ProjectUpsert {
            root_path: "/repo/main".into(),
            display_name: Some("main".to_string()),
            remote_origin: None,
            pinned_order: None,
        })
        .expect("project");

    storage
        .connection()
        .execute(
            "INSERT INTO hosts(id, name, ssh, default_cwd, node_endpoint, node_token_file, node_id, created_at, updated_at)
             VALUES ('host_1', 'Remote', 'remote.example', '/repo/main', NULL, NULL, 'node_1', 1, 1)",
            [],
        )
        .expect("host fixture");

    storage
        .update_session_core_metadata(
            &child.id,
            SessionCoreMetadataUpdate {
                project_id: Some(project.id.clone()),
                worktree_path: Some("/repo/main-feature".into()),
                git_branch: Some("feature/storage".to_string()),
                title_source: "agent_title".to_string(),
                agent_session_id: Some("thread-child".to_string()),
                transcript_path: Some("/tmp/thread-child.jsonl".into()),
                needs_input_kind: Some("permission".to_string()),
                needs_input_payload: json!({"reason":"approval_required"}),
                resumability: "resumable".to_string(),
                parent_session_id: Some(parent.id.clone()),
                pinned: true,
                archived_at: Some(1_800_000_000),
                remote_active: true,
                host_id: Some("host_1".to_string()),
                foreground_agent_kind: Some("claude_code".to_string()),
                memory_bytes: Some(42_000_000),
            },
        )
        .expect("update metadata");

    let metadata = storage
        .session_core_metadata(&child.id)
        .expect("load metadata");
    assert_eq!(metadata.project_id.as_deref(), Some(project.id.as_str()));
    assert_eq!(
        metadata.worktree_path.as_deref(),
        Some("/repo/main-feature")
    );
    assert_eq!(metadata.git_branch.as_deref(), Some("feature/storage"));
    assert_eq!(metadata.title_source, "agent_title");
    assert_eq!(metadata.agent_session_id.as_deref(), Some("thread-child"));
    assert_eq!(
        metadata.transcript_path.as_deref(),
        Some("/tmp/thread-child.jsonl")
    );
    assert_eq!(metadata.needs_input_kind.as_deref(), Some("permission"));
    assert_eq!(metadata.needs_input_payload["reason"], "approval_required");
    assert_eq!(metadata.resumability, "resumable");
    assert_eq!(
        metadata.parent_session_id.as_deref(),
        Some(parent.id.as_str())
    );
    assert!(metadata.pinned);
    assert!(metadata.remote_active);
    assert_eq!(metadata.host_id.as_deref(), Some("host_1"));
    assert_eq!(
        metadata.foreground_agent_kind.as_deref(),
        Some("claude_code")
    );
    assert_eq!(metadata.memory_bytes, Some(42_000_000));
}

#[test]
fn usage_repository_deduplicates_source_events_and_queries_totals() {
    let storage = open_migrated_storage();
    let session = seed_default_session(&storage, "Usage");

    let first = RecordUsage {
        request_id: "req-1".to_string(),
        session_id: Some(session.id.clone()),
        agent_profile_id: session.agent_profile_id.clone(),
        runtime_id: session.runtime_id.clone(),
        provider_id: "provider_local_placeholder".to_string(),
        llm_profile_id: session.llm_profile_id.clone(),
        model: "gpt-4o-mini".to_string(),
        request_kind: "chat".to_string(),
        status: "ok".to_string(),
        input_tokens: 100,
        output_tokens: 20,
        cached_input_tokens: 10,
        cache_read_tokens: 10,
        cache_write_tokens: 5,
        cache_write_5m_tokens: 3,
        cache_write_1h_tokens: 2,
        reasoning_tokens: 7,
        unit_price_input: Some("1.0".to_string()),
        unit_price_output: Some("2.0".to_string()),
        currency: Some("USD".to_string()),
        pricing_snapshot_id: None,
        estimated_cost: Some("0.0002".to_string()),
        billed_cost: None,
        first_token_latency_ms: Some(40),
        total_latency_ms: Some(300),
        started_at: 100,
        completed_at: 101,
        safe_error_code: None,
        value_kind: "estimated_api_equivalent".to_string(),
        source: "transcript".to_string(),
        source_event_id: "event-1".to_string(),
    };

    assert!(storage.record_usage(first.clone()).expect("insert usage"));
    assert!(!storage.record_usage(first).expect("dedupe usage"));

    assert!(
        storage
            .record_usage(usage_fixture(&session, "req-2", "event-2", 50, 10, 4, 1))
            .expect("insert second usage")
    );

    let totals = storage
        .query_usage_totals(UsageQuery {
            session_id: Some(session.id.clone()),
            provider_id: Some("provider_local_placeholder".to_string()),
            model: Some("gpt-4o-mini".to_string()),
            from: Some(0),
            to: Some(1_000),
        })
        .expect("query usage");

    assert_eq!(totals.events, 2);
    assert_eq!(totals.input_tokens, 150);
    assert_eq!(totals.output_tokens, 30);
    assert_eq!(totals.cache_read_tokens, 14);
    assert_eq!(totals.cache_write_tokens, 6);
    assert_eq!(totals.cache_write_5m_tokens, 3);
    assert_eq!(totals.cache_write_1h_tokens, 3);
    assert_eq!(totals.reasoning_tokens, 7);
    assert_eq!(totals.total_tokens, 200);
    assert!((totals.estimated_cost - 0.0003).abs() < 1e-12);
    assert!((totals.billed_cost - 0.00011).abs() < 1e-12);
    assert!(totals.authoritative_billing_available);
}

#[test]
fn usage_repository_handles_empty_queries_and_rejects_negative_tokens() {
    let storage = open_migrated_storage();
    let session = seed_default_session(&storage, "Usage negative");

    let empty = storage
        .query_usage_totals(UsageQuery {
            session_id: Some("missing".to_string()),
            ..UsageQuery::default()
        })
        .expect("empty query");
    assert_eq!(empty.events, 0);
    assert_eq!(empty.total_tokens, 0);
    assert!(!empty.authoritative_billing_available);

    let error = storage
        .record_usage(usage_fixture(
            &session,
            "req-negative",
            "event-negative",
            -1,
            0,
            0,
            0,
        ))
        .expect_err("negative usage is rejected");
    assert!(matches!(error, StorageError::InvalidInput(_)));

    let mut missing_source = usage_fixture(&session, "req-empty", "", 1, 0, 0, 0);
    missing_source.source_event_id.clear();
    let error = storage
        .record_usage(missing_source)
        .expect_err("source event id is required");
    assert!(matches!(error, StorageError::InvalidInput(_)));

    let after_reject = storage
        .query_usage_totals(UsageQuery::default())
        .expect("query after reject");
    assert_eq!(after_reject.events, 0);
}

fn usage_fixture(
    session: &homie_storage::SessionSummary,
    request_id: &str,
    source_event_id: &str,
    input_tokens: i64,
    output_tokens: i64,
    cache_read_tokens: i64,
    cache_write_tokens: i64,
) -> RecordUsage {
    RecordUsage {
        request_id: request_id.to_string(),
        session_id: Some(session.id.clone()),
        agent_profile_id: session.agent_profile_id.clone(),
        runtime_id: session.runtime_id.clone(),
        provider_id: "provider_local_placeholder".to_string(),
        llm_profile_id: session.llm_profile_id.clone(),
        model: "gpt-4o-mini".to_string(),
        request_kind: "chat".to_string(),
        status: "ok".to_string(),
        input_tokens,
        output_tokens,
        cached_input_tokens: 0,
        cache_read_tokens,
        cache_write_tokens,
        cache_write_5m_tokens: 0,
        cache_write_1h_tokens: 1,
        reasoning_tokens: 0,
        unit_price_input: Some("1.0".to_string()),
        unit_price_output: Some("2.0".to_string()),
        currency: Some("USD".to_string()),
        pricing_snapshot_id: None,
        estimated_cost: Some("0.0001".to_string()),
        billed_cost: Some("0.00011".to_string()),
        first_token_latency_ms: Some(20),
        total_latency_ms: Some(200),
        started_at: 200,
        completed_at: 201,
        safe_error_code: None,
        value_kind: "authoritative_billed".to_string(),
        source: "transcript".to_string(),
        source_event_id: source_event_id.to_string(),
    }
}
