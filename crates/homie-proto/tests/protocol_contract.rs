use homie_proto::{
    AccountCatalogResult, AccountInstallation, AccountLoginStartParams, AccountProfile,
    AccountProfileParams, AccountSetDefaultParams, AccountUpsertParams, BlobChunk, BlobHasParams,
    BlobHasResult, BlobPutParams, BlobReadParams, CheckpointFile, CheckpointIdParams,
    CheckpointManifest, CheckpointManifestParams, CheckpointPrepareParams, CheckpointStageResult,
    ControlMessage, ErrorEnvelope, EventCursor, EventName, EventsSubscribeRequest,
    EventsWaitRequest, HostEntry, HostLocateRepoParams, HostLocateRepoResult, HostNodeConfig,
    HostSyncPrefsParams, HostSyncPrefsResult, HostsConfig, InstallationStatus, LoginChallenge,
    LoginInputParams, LoginMode, LoginSessionParams, Method, MoveAbortParams, MoveCommitParams,
    MovePhase, MoveRecord, NodeCapability, NodeHelloParams, NodeHelloResult, NodeMethod,
    NodeStatusResult, PrefsSyncToolReport, ProviderCallParams, ProviderCallResult, ProviderKind,
    RemoteConfig, RequestId, SessionAttachRequest, SessionDiffBase, SessionId, SessionKillRequest,
    SessionReadDiffRequest, SessionReadDiffResult, SessionResizeRequest, SessionSendTextRequest,
    SessionSpawnRequest, SessionStatus, TransferMode, UsageEvent, UsageQueryParams,
    UsageQueryResult, UsageSource, UsageTotals, UsageValueKind,
};
use serde_json::json;
use std::collections::BTreeMap;

#[test]
fn method_catalog_covers_reference_parity_surface() {
    let methods = Method::ALL;
    for expected in [
        Method::HELLO,
        Method::STATE_SNAPSHOT,
        Method::CLIENT_SET_ACTIVE,
        Method::AGENT_READINESS,
        Method::SESSION_SPAWN,
        Method::SESSION_LIST,
        Method::SESSION_ATTACH,
        Method::SESSION_SEND_TEXT,
        Method::SESSION_RESIZE,
        Method::SESSION_READ_SCREEN,
        Method::SESSION_READ_SCROLLBACK_CELLS,
        Method::SESSION_READ_DIFF,
        Method::SESSION_ARCHIVE,
        Method::SESSION_HISTORY,
        Method::WORKTREE_CREATE,
        Method::EVENTS_SUBSCRIBE,
        Method::HOOK_REPORT,
        Method::TEST_RUN,
        Method::BROWSER_ACT,
        Method::LLM_VIRTUAL_KEY_ISSUE,
        Method::AGENT_PROFILE_LIST,
        Method::TASK_UPDATE,
        Method::MEMORY_WRITE_CANDIDATE,
    ] {
        assert!(methods.contains(&expected), "missing method {expected}");
    }
}

#[test]
fn event_resume_request_dtos_preserve_cursor_contract() {
    let subscribe: EventsSubscribeRequest =
        serde_json::from_value(json!({"afterSeq": 41, "eventFilter": ["session.output"]}))
            .expect("subscribe");
    assert_eq!(subscribe.after_seq, 41);
    assert_eq!(subscribe.event_filter, vec!["session.output"]);
    assert_eq!(
        serde_json::to_value(&subscribe).expect("subscribe json"),
        json!({"afterSeq": 41, "eventFilter": ["session.output"]})
    );

    let wait: EventsWaitRequest = serde_json::from_value(json!({"afterSeq": 9})).expect("wait");
    assert_eq!(wait.after_seq, 9);
    assert_eq!(wait.timeout_ms, 30_000);
    assert!(wait.event_filter.is_empty());

    let cursor = EventCursor { next_seq: 42 };
    assert_eq!(
        serde_json::to_value(cursor).expect("cursor"),
        json!({"nextSeq": 42})
    );
}

#[test]
fn host_locate_repo_round_trips_diri_spelling() {
    let params = HostLocateRepoParams {
        host: Some("forge".to_string()),
        origin_url: Some("git@example.invalid:acme/app.git".to_string()),
        session_id: Some(SessionId::from("s_1")),
    };
    assert_eq!(
        serde_json::to_value(&params).expect("params json"),
        json!({
            "host": "forge",
            "originURL": "git@example.invalid:acme/app.git",
            "sessionID": "s_1"
        })
    );

    let found: HostLocateRepoResult = serde_json::from_value(json!({
        "path": "/home/user/app",
        "originURL": "git@example.invalid:acme/app.git"
    }))
    .expect("found result");
    assert_eq!(found.path.as_deref(), Some("/home/user/app"));
    assert_eq!(
        found.origin_url.as_deref(),
        Some("git@example.invalid:acme/app.git")
    );

    let missing: HostLocateRepoResult = serde_json::from_value(json!({})).expect("missing result");
    assert_eq!(missing, HostLocateRepoResult::default());
}

#[test]
fn host_sync_prefs_round_trips_diri_wire() {
    let params = HostSyncPrefsParams {
        host: "forge".to_string(),
    };
    assert_eq!(
        serde_json::to_value(&params).expect("params"),
        json!({"host": "forge"})
    );

    let report: HostSyncPrefsResult = serde_json::from_value(json!({
        "tools": [
            {"tool": "claude", "ok": true, "synced": ["CLAUDE.md", "commands"]},
            {"tool": "codex", "ok": false, "synced": [], "error": "rsync is not installed on Forge"},
            {"tool": "opencode", "ok": true, "synced": []}
        ]
    }))
    .expect("result");
    assert_eq!(report.tools.len(), 3);
    assert_eq!(
        report.tools[0],
        PrefsSyncToolReport {
            tool: "claude".to_string(),
            ok: true,
            synced: vec!["CLAUDE.md".to_string(), "commands".to_string()],
            error: None,
        }
    );
    assert_eq!(
        report.tools[1].error.as_deref(),
        Some("rsync is not installed on Forge")
    );

    let encoded = serde_json::to_value(&report).expect("encoded");
    assert!(encoded["tools"][0].get("error").is_none());
    assert_eq!(
        encoded["tools"][1]["error"],
        "rsync is not installed on Forge"
    );
    assert_eq!(encoded["tools"][2]["synced"], json!([]));
}

#[test]
fn host_catalog_and_remote_config_match_diri_wire() {
    let hosts: HostsConfig = serde_json::from_value(json!({
        "hosts": [
            {
                "id": "forge",
                "name": "Forge",
                "ssh": "cristi@forge",
                "defaultCwd": "~/code",
                "node": {
                    "endpoint": "tcp://100.64.0.2:7337",
                    "tokenFile": "~/.config/dirijor/forge.token",
                    "nodeId": "node-forge"
                }
            },
            {
                "id": "studio",
                "name": "Studio Mac",
                "ssh": "studio.local"
            }
        ]
    }))
    .expect("hosts");
    assert_eq!(hosts.hosts.len(), 2);
    assert_eq!(hosts.hosts[0].display_name(), "Forge");
    assert_eq!(hosts.hosts[0].default_cwd.as_deref(), Some("~/code"));
    assert_eq!(
        hosts.hosts[0]
            .node
            .as_ref()
            .map(|node| node.endpoint.as_str()),
        Some("tcp://100.64.0.2:7337")
    );
    assert_eq!(
        hosts.host("studio").expect("studio").display_name(),
        "Studio Mac"
    );

    let minimal: HostsConfig =
        serde_json::from_value(json!({"hosts": [{"id": "builder", "ssh": "root@1.2.3.4"}]}))
            .expect("minimal");
    let builder = minimal.host("builder").expect("builder");
    assert_eq!(builder.display_name(), "builder");
    assert_eq!(builder.default_cwd, None);
    assert_eq!(builder.node, None);

    let empty: HostsConfig = serde_json::from_value(json!({})).expect("empty");
    assert!(empty.hosts.is_empty());

    let encoded = serde_json::to_value(HostsConfig {
        hosts: vec![HostEntry {
            id: "forge".to_string(),
            name: Some("Forge".to_string()),
            ssh: "cristi@forge".to_string(),
            default_cwd: Some("~/code".to_string()),
            node: Some(HostNodeConfig {
                endpoint: "tcp://100.64.0.2:7337".to_string(),
                token_file: "~/.config/dirijor/forge.token".to_string(),
                node_id: Some("node-forge".to_string()),
            }),
        }],
    })
    .expect("encoded");
    assert_eq!(encoded["hosts"][0]["defaultCwd"], "~/code");
    assert_eq!(
        encoded["hosts"][0]["node"]["tokenFile"],
        "~/.config/dirijor/forge.token"
    );

    let current: RemoteConfig = serde_json::from_value(json!({
        "port": 48620,
        "bindHost": "100.101.102.103",
        "token": "secret",
        "forwardAnyPort": false
    }))
    .expect("remote current");
    assert_eq!(current.port, 48_620);
    assert_eq!(current.bind_host.as_deref(), Some("100.101.102.103"));
    assert_eq!(current.forward_any_port, Some(false));

    let legacy: RemoteConfig =
        serde_json::from_value(json!({"port": 48620, "token": "secret"})).expect("legacy");
    assert_eq!(legacy.bind_host, None);
    assert_eq!(legacy.forward_any_port, None);
}

#[test]
fn node_hello_and_usage_match_diri_wire() {
    assert_eq!(NodeMethod::HELLO, "node.hello");
    assert_eq!(NodeMethod::USAGE_RECORD, "usage.record");
    assert_eq!(NodeCapability::FLEET_USAGE, "usage-ledger.v1");

    let hello_params = NodeHelloParams::new("homie-test", "example-token");
    assert_eq!(
        serde_json::to_value(&hello_params).expect("hello params"),
        json!({
            "proto": 1,
            "build": "homie-test",
            "token": "example-token"
        })
    );

    let hello = NodeHelloResult {
        proto: 1,
        control_proto: 1,
        build: "homie-node".to_string(),
        node_id: "node-forge".to_string(),
        display_name: "Forge".to_string(),
        os: "linux".to_string(),
        arch: "x86_64".to_string(),
        capabilities: vec![NodeCapability::FLEET_USAGE.to_string()],
    };
    let hello_json = serde_json::to_value(&hello).expect("hello result");
    assert_eq!(hello_json["controlProto"], 1);
    assert!(!hello_json.to_string().contains("secret"));
    assert!(!hello_json.to_string().contains("token"));

    let status = NodeStatusResult {
        node: hello,
        started_at: 1_800_000_000,
        accounts: 2,
        active_logins: 1,
        pending_moves: 0,
    };
    let status_json = serde_json::to_value(&status).expect("status");
    assert_eq!(status_json["startedAt"], 1_800_000_000i64);
    assert_eq!(status_json["activeLogins"], 1);

    let mut defaults = BTreeMap::new();
    defaults.insert(ProviderKind::Claude, "work".to_string());
    defaults.insert(ProviderKind::Codex, "personal".to_string());
    assert_eq!(
        serde_json::to_value(&defaults).expect("provider map"),
        json!({"claude": "work", "codex": "personal"})
    );

    let event = UsageEvent {
        id: "usage-1".to_string(),
        occurred_at: 1_800_000_001,
        provider: ProviderKind::Codex,
        profile_id: Some("personal".to_string()),
        session_id: Some("thread-1".to_string()),
        model: Some("codex".to_string()),
        input_tokens: 100,
        output_tokens: 20,
        cache_read_tokens: 10,
        cache_write_tokens: 0,
        estimated_usd: Some(0.001),
        billed_usd: None,
        value_kind: UsageValueKind::EstimatedApiEquivalent,
        source: UsageSource::Transcript,
    };
    let event_json = serde_json::to_value(&event).expect("usage event");
    assert_eq!(event_json["occurredAt"], 1_800_000_001i64);
    assert_eq!(event_json["provider"], "codex");
    assert_eq!(event_json["valueKind"], "estimatedApiEquivalent");
    assert_eq!(event_json["source"], "transcript");
    assert!(event_json.get("billedUsd").is_none());

    let query = UsageQueryParams {
        from: Some(1),
        to: Some(2),
        provider: Some(ProviderKind::Claude),
        profile_id: Some("work".to_string()),
        session_id: None,
    };
    assert_eq!(
        serde_json::to_value(&query).expect("query"),
        json!({
            "from": 1,
            "to": 2,
            "provider": "claude",
            "profileId": "work"
        })
    );

    let mut by_provider = BTreeMap::new();
    by_provider.insert(
        ProviderKind::Codex,
        UsageTotals {
            input_tokens: 100,
            output_tokens: 20,
            cache_read_tokens: 10,
            cache_write_tokens: 0,
            estimated_usd: 0.001,
            billed_usd: 0.0,
            events: 1,
        },
    );
    let result = UsageQueryResult {
        node_id: "node-forge".to_string(),
        totals: UsageTotals {
            input_tokens: 100,
            output_tokens: 20,
            cache_read_tokens: 10,
            cache_write_tokens: 0,
            estimated_usd: 0.001,
            billed_usd: 0.0,
            events: 1,
        },
        by_provider,
        authoritative_billing_available: false,
        updated_at: 1_800_000_002,
    };
    let result_json = serde_json::to_value(&result).expect("usage result");
    assert_eq!(result_json["nodeId"], "node-forge");
    assert_eq!(result_json["byProvider"]["codex"]["events"], 1);
}

#[test]
fn node_checkpoint_move_match_diri_wire() {
    let prepare = CheckpointPrepareParams {
        session_id: "session-1".to_string(),
        provider: ProviderKind::Claude,
        profile_id: "work".to_string(),
        workspace_root: "/repo".to_string(),
        provider_session_id: Some("claude-session".to_string()),
        mode: TransferMode::Move,
    };
    assert_eq!(
        serde_json::to_value(&prepare).expect("prepare"),
        json!({
            "sessionId": "session-1",
            "provider": "claude",
            "profileId": "work",
            "workspaceRoot": "/repo",
            "providerSessionId": "claude-session",
            "mode": "move"
        })
    );

    let source_file = CheckpointFile {
        path: "src/main.rs".to_string(),
        digest: "sha256:abc".to_string(),
        size: 12,
        unix_mode: Some(0o644),
    };
    let manifest = CheckpointManifest {
        version: 1,
        checkpoint_id: "cp-1".to_string(),
        source_node_id: "node-a".to_string(),
        session_id: "session-1".to_string(),
        provider: ProviderKind::Claude,
        profile_id: "work".to_string(),
        workspace_root: "/repo".to_string(),
        provider_session_id: None,
        mode: TransferMode::Fork,
        created_at: 1_800_000_000,
        files: vec![source_file.clone()],
        provider_state: Some(CheckpointFile {
            path: ".claude/session.jsonl".to_string(),
            digest: "sha256:def".to_string(),
            size: 99,
            unix_mode: None,
        }),
        excluded: vec!["target".to_string()],
    };
    let encoded_manifest = serde_json::to_value(&manifest).expect("manifest");
    assert_eq!(encoded_manifest["checkpointId"], "cp-1");
    assert_eq!(encoded_manifest["sourceNodeId"], "node-a");
    assert_eq!(
        encoded_manifest["providerState"]["path"],
        ".claude/session.jsonl"
    );
    assert_eq!(encoded_manifest["files"][0]["unixMode"], 0o644);
    assert_eq!(
        serde_json::to_value(CheckpointManifestParams {
            manifest: manifest.clone()
        })
        .expect("manifest params")["manifest"]["checkpointId"],
        "cp-1"
    );
    assert_eq!(
        serde_json::to_value(CheckpointIdParams {
            checkpoint_id: "cp-1".to_string()
        })
        .expect("id params"),
        json!({"checkpointId": "cp-1"})
    );

    assert_eq!(
        serde_json::to_value(BlobHasParams {
            digests: vec!["sha256:abc".to_string()]
        })
        .expect("has"),
        json!({"digests": ["sha256:abc"]})
    );
    assert_eq!(
        serde_json::to_value(BlobHasResult {
            missing: vec!["sha256:def".to_string()]
        })
        .expect("has result"),
        json!({"missing": ["sha256:def"]})
    );
    assert_eq!(
        serde_json::to_value(BlobReadParams {
            digest: "sha256:abc".to_string(),
            offset: 1,
            max_bytes: 4,
        })
        .expect("read"),
        json!({"digest": "sha256:abc", "offset": 1, "maxBytes": 4})
    );
    assert_eq!(
        serde_json::to_value(BlobChunk {
            digest: "sha256:abc".to_string(),
            offset: 1,
            hex: "6869".to_string(),
            eof: true,
        })
        .expect("chunk"),
        json!({"digest": "sha256:abc", "offset": 1, "hex": "6869", "eof": true})
    );
    assert_eq!(
        serde_json::to_value(BlobPutParams {
            digest: "sha256:abc".to_string(),
            offset: 1,
            hex: "6869".to_string(),
            eof: true,
        })
        .expect("put"),
        json!({"digest": "sha256:abc", "offset": 1, "hex": "6869", "eof": true})
    );
    assert_eq!(
        serde_json::to_value(CheckpointStageResult {
            checkpoint_id: "cp-1".to_string(),
            quarantine_path: "/tmp/cp-1".to_string(),
            ready: true,
        })
        .expect("stage"),
        json!({"checkpointId": "cp-1", "quarantinePath": "/tmp/cp-1", "ready": true})
    );

    assert_eq!(
        serde_json::to_value(MoveCommitParams {
            checkpoint_id: "cp-1".to_string(),
            target_node_id: "node-b".to_string(),
            lease_id: "lease-1".to_string(),
        })
        .expect("commit"),
        json!({"checkpointId": "cp-1", "targetNodeId": "node-b", "leaseId": "lease-1"})
    );
    assert_eq!(
        serde_json::to_value(MoveAbortParams {
            checkpoint_id: "cp-1".to_string(),
            reason: "cancelled".to_string(),
        })
        .expect("abort"),
        json!({"checkpointId": "cp-1", "reason": "cancelled"})
    );
    let move_record = MoveRecord {
        checkpoint_id: "cp-1".to_string(),
        session_id: "session-1".to_string(),
        source_node_id: "node-a".to_string(),
        target_node_id: Some("node-b".to_string()),
        phase: MovePhase::TargetReady,
        lease_id: Some("lease-1".to_string()),
        reason: None,
        updated_at: 1_800_000_001,
    };
    let move_json = serde_json::to_value(&move_record).expect("move record");
    assert_eq!(move_json["targetNodeId"], "node-b");
    assert_eq!(move_json["phase"], "targetReady");
    assert!(move_json.get("reason").is_none());
}

#[test]
fn node_account_login_match_diri_wire() {
    let profile = AccountProfile {
        id: "work".to_string(),
        provider: ProviderKind::Claude,
        label: "Work".to_string(),
        email: Some("me@example.invalid".to_string()),
        organization: None,
        tags: vec!["primary".to_string()],
        created_at: 1,
        updated_at: 2,
    };
    let installation = AccountInstallation {
        profile_id: "work".to_string(),
        provider: ProviderKind::Claude,
        node_id: "node-forge".to_string(),
        status: InstallationStatus::Ready,
        config_home: "/home/me/.claude".to_string(),
        identity: Some("me@example.invalid".to_string()),
        plan: Some("max".to_string()),
        last_error: None,
        checked_at: Some(3),
    };
    let mut defaults = BTreeMap::new();
    defaults.insert(ProviderKind::Claude, "work".to_string());
    let catalog = AccountCatalogResult {
        profiles: vec![profile],
        installations: vec![installation],
        defaults,
    };
    let catalog_json = serde_json::to_value(&catalog).expect("catalog");
    assert_eq!(catalog_json["profiles"][0]["provider"], "claude");
    assert_eq!(catalog_json["installations"][0]["status"], "ready");
    assert_eq!(catalog_json["defaults"], json!({"claude": "work"}));
    assert!(catalog_json["profiles"][0].get("organization").is_none());
    assert!(catalog_json["installations"][0].get("lastError").is_none());

    let upsert = AccountUpsertParams {
        id: "personal".to_string(),
        provider: ProviderKind::Codex,
        label: "Personal".to_string(),
        email: None,
        organization: None,
        tags: vec![],
    };
    assert_eq!(
        serde_json::to_value(&upsert).expect("upsert"),
        json!({"id": "personal", "provider": "codex", "label": "Personal", "tags": []})
    );
    assert_eq!(
        serde_json::to_value(AccountProfileParams {
            profile_id: "work".to_string()
        })
        .expect("profile params"),
        json!({"profileId": "work"})
    );
    assert_eq!(
        serde_json::to_value(AccountSetDefaultParams {
            provider: ProviderKind::Codex,
            profile_id: "personal".to_string(),
        })
        .expect("default params"),
        json!({"provider": "codex", "profileId": "personal"})
    );

    let login_start = AccountLoginStartParams {
        profile_id: "work".to_string(),
        mode: LoginMode::Browser,
    };
    assert_eq!(
        serde_json::to_value(&login_start).expect("login start"),
        json!({"profileId": "work", "mode": "browser"})
    );
    let challenge = LoginChallenge {
        login_id: "login-1".to_string(),
        profile_id: "work".to_string(),
        kind: LoginMode::DeviceCode,
        verification_url: Some("https://example.invalid/device".to_string()),
        user_code: Some("ABCD-EFGH".to_string()),
        output: "Open the URL".to_string(),
        complete: false,
        success: false,
        error: None,
    };
    let challenge_json = serde_json::to_value(&challenge).expect("challenge");
    assert_eq!(challenge_json["loginId"], "login-1");
    assert_eq!(
        challenge_json["verificationUrl"],
        "https://example.invalid/device"
    );
    assert!(challenge_json.get("error").is_none());
    assert_eq!(
        serde_json::to_value(LoginSessionParams {
            login_id: "login-1".to_string()
        })
        .expect("login session"),
        json!({"loginId": "login-1"})
    );
    assert_eq!(
        serde_json::to_value(LoginInputParams {
            login_id: "login-1".to_string(),
            text: "123456".to_string(),
        })
        .expect("login input"),
        json!({"loginId": "login-1", "text": "123456"})
    );

    let call = ProviderCallParams {
        profile_id: "work".to_string(),
        method: "account.status".to_string(),
        params: json!({"verbose": true}),
    };
    assert_eq!(
        serde_json::to_value(&call).expect("provider call"),
        json!({"profileId": "work", "method": "account.status", "params": {"verbose": true}})
    );
    let result = ProviderCallResult {
        provider: ProviderKind::Claude,
        method: "account.status".to_string(),
        result: json!({"ok": true}),
    };
    assert_eq!(
        serde_json::to_value(&result).expect("provider result"),
        json!({"provider": "claude", "method": "account.status", "result": {"ok": true}})
    );
}

#[test]
fn session_read_diff_uses_diri_base64_wire() {
    let request = SessionReadDiffRequest {
        session_id: SessionId::from("s_1"),
        base: Some(SessionDiffBase::Head),
    };
    assert_eq!(
        serde_json::to_value(&request).expect("request"),
        json!({"sessionID": "s_1", "base": "head"})
    );

    let result = SessionReadDiffResult {
        patch: b"diff --git a/a b/a\n".to_vec(),
        repo_root: "/repo".to_string(),
        truncated: false,
        base_ref: Some("HEAD".to_string()),
    };
    let value = serde_json::to_value(&result).expect("result");
    assert_eq!(value["patch"], "ZGlmZiAtLWdpdCBhL2EgYi9hCg==");
    assert_eq!(value["baseRef"], "HEAD");
    let decoded: SessionReadDiffResult = serde_json::from_value(value).expect("decoded");
    assert_eq!(decoded.patch, b"diff --git a/a b/a\n");
}

#[test]
fn session_runtime_request_dtos_are_camel_case_contracts() {
    let spawn = SessionSpawnRequest {
        cwd: "/tmp/work".to_string(),
        title: Some("Work".to_string()),
        parent_session_id: Some(SessionId::from("parent-1")),
    };
    assert_eq!(
        serde_json::to_value(&spawn).expect("spawn"),
        json!({
            "cwd": "/tmp/work",
            "title": "Work",
            "parentSessionId": "parent-1"
        })
    );

    let attach: SessionAttachRequest =
        serde_json::from_value(json!({"sessionId": "s1", "outputOffset": 7})).expect("attach");
    assert_eq!(attach.session_id, SessionId::from("s1"));
    assert_eq!(attach.output_offset, 7);
    assert_eq!(attach.max_bytes, 8192);

    let send = SessionSendTextRequest {
        session_id: SessionId::from("s1"),
        text: "hello".to_string(),
        submit: true,
    };
    assert_eq!(
        serde_json::to_value(&send).expect("send"),
        json!({"sessionId": "s1", "text": "hello", "submit": true})
    );

    let resize = SessionResizeRequest {
        session_id: SessionId::from("s1"),
        cols: 100,
        rows: 30,
    };
    assert_eq!(
        serde_json::to_value(&resize).expect("resize"),
        json!({"sessionId": "s1", "cols": 100, "rows": 30})
    );

    let kill = SessionKillRequest {
        session_id: SessionId::from("s1"),
    };
    assert_eq!(
        serde_json::to_value(&kill).expect("kill"),
        json!({"sessionId": "s1"})
    );
}

#[test]
fn event_catalog_covers_reference_parity_surface() {
    let events = EventName::ALL;
    for expected in [
        EventName::RUNTIME_READY,
        EventName::SESSION_UPDATED,
        EventName::SESSION_STATUS,
        EventName::SESSION_NEEDS_INPUT,
        EventName::SESSION_OUTPUT,
        EventName::SESSION_ARTIFACT,
        EventName::WORKTREE_CREATED,
        EventName::LLM_REQUEST_COMPLETED,
        EventName::TOOL_CALL_FAILED,
        EventName::METRICS_WRITE_FAILED,
        EventName::EVENTS_DROPPED,
    ] {
        assert!(events.contains(&expected), "missing event {expected}");
    }
}

#[test]
fn control_messages_round_trip_with_camel_case_fields() {
    let request = ControlMessage::request(
        RequestId::from(7),
        Method::SESSION_SEND_TEXT,
        json!({
            "sessionID": "session_1",
            "text": "hello",
            "submit": true
        }),
    );
    let encoded = serde_json::to_string(&request).expect("serialize request");
    assert!(encoded.contains("\"type\":\"request\""));
    assert!(encoded.contains("\"requestId\":7"));

    let decoded: ControlMessage = serde_json::from_str(&encoded).expect("decode request");
    assert_eq!(decoded, request);

    let response = ControlMessage::success(RequestId::from(7), json!({"ok": true}));
    let decoded_response: ControlMessage =
        serde_json::from_str(&serde_json::to_string(&response).expect("serialize response"))
            .expect("decode response");
    assert_eq!(decoded_response, response);

    let event = ControlMessage::event(
        EventName::SESSION_UPDATED,
        42,
        json!({"id": "session_1", "status": "running"}),
    );
    let decoded_event: ControlMessage =
        serde_json::from_str(&serde_json::to_string(&event).expect("serialize event"))
            .expect("decode event");
    assert_eq!(decoded_event, event);
}

#[test]
fn unknown_session_status_decodes_without_failing() {
    let status: SessionStatus = serde_json::from_str("\"brand_new_state\"").expect("status");
    assert_eq!(
        status,
        SessionStatus::Unknown("brand_new_state".to_string())
    );
    assert_eq!(
        serde_json::to_string(&status).expect("serialize"),
        "\"brand_new_state\""
    );
}

#[test]
fn error_envelope_preserves_safe_details() {
    let error = ErrorEnvelope::new("permission_denied", "not allowed", false)
        .with_detail("scope", "session")
        .with_detail("sessionId", SessionId::from("session_1").to_string());
    let encoded = serde_json::to_value(&error).expect("serialize error");
    assert_eq!(encoded["code"], "permission_denied");
    assert_eq!(encoded["retryable"], false);
    assert_eq!(encoded["details"]["scope"], "session");
    assert_eq!(encoded["details"]["sessionId"], "session_1");
}
