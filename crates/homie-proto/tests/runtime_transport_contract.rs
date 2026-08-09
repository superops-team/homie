use homie_proto::model::{
    ArtifactKind, ArtifactScan, HolderSnapshot, HookReportRequest, ListeningPort, PortListRow,
    RuntimeEvent, RuntimeScreenObservation, ScannedHistoryEntry, SessionArtifact,
    SessionArtifactsRequest, SessionChildrenRequest, SessionParentRequest, SessionParentResult,
    SessionPortsRequest, SessionSetParentRequest, SessionSnapshot, SessionSnapshotRequest,
    SessionStatusReport, SessionStatusRequest, SessionSummary, StateSnapshot,
    WorktreeOverviewEntry, WorktreeOverviewResult,
};
use homie_proto::stream::{
    EventStreamOpen, StreamKind, StreamOpenRequest, StreamReset, StreamResetReason,
    TerminalStreamOpen,
};
use homie_proto::transport::{
    AckResult, ClientRole, DaemonStatus, DaemonStatusKind, HelloRequest, HelloResponse,
    ShutdownResult, StableErrorCode, WIRE_MAJOR, WIRE_MINOR,
};
use homie_proto::{Method, NeedsInputDetail, NeedsInputKind, NeedsInputSource, RiskHint};
use serde_json::json;

fn session_summary() -> SessionSummary {
    SessionSummary {
        id: "session-1".to_string(),
        title: "Protocol work".to_string(),
        status: "running".to_string(),
        workspace: "/repo".to_string(),
        agent_profile_id: "agent-1".to_string(),
        runtime_id: "runtime-1".to_string(),
        llm_profile_id: "llm-1".to_string(),
        permission_profile_id: "permission-1".to_string(),
    }
}

#[test]
fn hello_request_uses_camel_case_wire_fields() {
    let request = HelloRequest {
        wire_major: WIRE_MAJOR,
        wire_minor: WIRE_MINOR,
        client_name: "homie-cli".to_string(),
        client_version: "0.1.0".to_string(),
        client_role: ClientRole::Cli,
        process_id: 42,
    };

    assert_eq!(
        serde_json::to_value(request).expect("serialize hello request"),
        json!({
            "wireMajor": 1,
            "wireMinor": 0,
            "clientName": "homie-cli",
            "clientVersion": "0.1.0",
            "clientRole": "cli",
            "processId": 42
        })
    );
}

#[test]
fn hello_response_uses_exact_capability_fields() {
    let response = HelloResponse {
        wire_major: WIRE_MAJOR,
        wire_minor: WIRE_MINOR,
        daemon_build: "debug".to_string(),
        daemon_version: "0.1.0".to_string(),
        daemon_pid: 7,
        daemon_instance_id: "instance-1".to_string(),
        executable_hash: "sha256:abc".to_string(),
        method_capabilities: vec![Method::STATE_SNAPSHOT.to_string()],
        stream_capabilities: vec![StreamKind::EventsV1, StreamKind::TerminalV1],
        event_oldest_seq: 10,
        event_latest_seq: 12,
    };

    assert_eq!(
        serde_json::to_value(response).expect("serialize hello response"),
        json!({
            "wireMajor": 1,
            "wireMinor": 0,
            "daemonBuild": "debug",
            "daemonVersion": "0.1.0",
            "daemonPid": 7,
            "daemonInstanceId": "instance-1",
            "executableHash": "sha256:abc",
            "methodCapabilities": ["state.snapshot"],
            "streamCapabilities": ["events.v1", "terminal.v1"],
            "eventOldestSeq": 10,
            "eventLatestSeq": 12
        })
    );
}

#[test]
fn stable_error_codes_match_approved_protocol() {
    assert_eq!(
        StableErrorCode::ALL.map(StableErrorCode::as_str),
        [
            "method_not_found",
            "bad_request",
            "version_mismatch",
            "unauthorized",
            "unavailable",
            "timeout",
            "backpressure",
            "resync_required",
            "internal"
        ]
    );
}

#[test]
fn event_stream_open_uses_after_seq_and_filter() {
    let request = StreamOpenRequest::Events(EventStreamOpen {
        after_seq: 41,
        event_filter: vec!["session.updated".to_string()],
    });

    assert_eq!(
        serde_json::to_value(request).expect("serialize event stream open"),
        json!({
            "kind": "events.v1",
            "afterSeq": 41,
            "eventFilter": ["session.updated"]
        })
    );
}

#[test]
fn terminal_stream_open_preserves_recovery_state() {
    let request = StreamOpenRequest::Terminal(TerminalStreamOpen {
        session_id: "session-1".to_string(),
        output_offset: 4096,
        client_role: ClientRole::App,
        last_grid_sequence: Some(9),
    });

    assert_eq!(
        serde_json::to_value(request).expect("serialize terminal stream open"),
        json!({
            "kind": "terminal.v1",
            "sessionId": "session-1",
            "outputOffset": 4096,
            "clientRole": "app",
            "lastGridSequence": 9
        })
    );
}

#[test]
fn stream_reset_carries_only_relevant_recovery_position() {
    let reset = StreamReset {
        reason: StreamResetReason::EventGap,
        last_confirmed_offset: None,
        latest_seq: Some(84),
    };

    assert_eq!(
        serde_json::to_value(reset).expect("serialize stream reset"),
        json!({"reason": "event_gap", "latestSeq": 84})
    );
}

#[test]
fn state_snapshot_owns_session_wire_contract() {
    let snapshot = StateSnapshot {
        sessions: vec![session_summary()],
        event_cursor: 19,
    };

    assert_eq!(
        serde_json::to_value(snapshot).expect("serialize runtime snapshot"),
        json!({
            "sessions": [{
                "id": "session-1",
                "title": "Protocol work",
                "status": "running",
                "workspace": "/repo",
                "agentProfileId": "agent-1",
                "runtimeId": "runtime-1",
                "llmProfileId": "llm-1",
                "permissionProfileId": "permission-1"
            }],
            "eventCursor": 19
        })
    );
}

#[test]
fn session_snapshot_status_and_holder_are_protocol_owned() {
    let snapshot = SessionSnapshot {
        session: session_summary(),
        status: SessionStatusReport {
            status: homie_proto::SessionStatus::NeedsInput,
            needs_input: Some(NeedsInputDetail {
                kind: NeedsInputKind::Approval,
                source: NeedsInputSource::Hook,
                tool_name: Some("shell".to_string()),
                summary: "Approve command".to_string(),
                prompt_excerpt: None,
                options: None,
                risk_hint: RiskHint::Destructive,
                occurred_at: 12,
            }),
            turn_completed: false,
            screen_lines: vec!["Approve command?".to_string()],
            screen_observation: Some(RuntimeScreenObservation {
                state: "needs_input".to_string(),
                matched_rule_id: "approval".to_string(),
                content_seq: 8,
            }),
        },
        output_offset: 1024,
        output_text: "Approve command?".to_string(),
        holder: Some(HolderSnapshot {
            pid: Some(99),
            status: Some("running".to_string()),
            tree_size: Some(2),
            cols: Some(120),
            rows: Some(40),
            log_offset: Some(1024),
            epoch_offset: Some(0),
        }),
    };

    let encoded = serde_json::to_value(snapshot).expect("serialize session snapshot");
    assert_eq!(encoded["status"]["needsInput"]["riskHint"], "destructive");
    assert_eq!(encoded["status"]["screenObservation"]["contentSeq"], 8);
    assert_eq!(encoded["outputText"], "Approve command?");
    assert_eq!(encoded["holder"]["logOffset"], 1024);
}

#[test]
fn artifact_and_port_results_use_existing_typed_client_fields() {
    let scan = ArtifactScan {
        artifacts: vec![SessionArtifact {
            kind: ArtifactKind::Preview,
            url: "http://localhost:3000".to_string(),
            label: "Preview".to_string(),
        }],
        ports: vec![ListeningPort {
            port: 3000,
            url: "http://localhost:3000".to_string(),
        }],
    };
    let row = PortListRow {
        port: 3000,
        url: "http://localhost:3000".to_string(),
        session_id: "session-1".to_string(),
        session_title: "Protocol work".to_string(),
    };

    assert_eq!(
        serde_json::to_value(scan).expect("serialize artifact scan"),
        json!({
            "artifacts": [{
                "kind": "preview",
                "url": "http://localhost:3000",
                "label": "Preview"
            }],
            "ports": [{
                "port": 3000,
                "url": "http://localhost:3000"
            }]
        })
    );
    assert_eq!(
        serde_json::to_value(row).expect("serialize port row")["sessionTitle"],
        "Protocol work"
    );
}

#[test]
fn lineage_request_and_result_fields_are_camel_case() {
    let set_parent = SessionSetParentRequest {
        session_id: "child".to_string(),
        parent_session_id: "parent".to_string(),
    };
    let children = SessionChildrenRequest {
        parent_session_id: "parent".to_string(),
    };
    let parent = SessionParentRequest {
        session_id: "child".to_string(),
    };
    let result = SessionParentResult {
        parent_session_id: Some("parent".to_string()),
    };

    assert_eq!(
        serde_json::to_value(set_parent).expect("set parent"),
        json!({"sessionId": "child", "parentSessionId": "parent"})
    );
    assert_eq!(
        serde_json::to_value(children).expect("list children"),
        json!({"parentSessionId": "parent"})
    );
    assert_eq!(
        serde_json::to_value(parent).expect("get parent"),
        json!({"sessionId": "child"})
    );
    assert_eq!(
        serde_json::to_value(result).expect("parent result"),
        json!({"parentSessionId": "parent"})
    );
}

#[test]
fn status_and_snapshot_requests_preserve_limits() {
    let status = SessionStatusRequest {
        session_id: "session-1".to_string(),
    };
    let snapshot = SessionSnapshotRequest {
        session_id: "session-1".to_string(),
        output_offset: 20,
        max_bytes: 4096,
    };

    assert_eq!(
        serde_json::to_value(status).expect("status request"),
        json!({"sessionId": "session-1"})
    );
    assert_eq!(
        serde_json::to_value(snapshot).expect("snapshot request"),
        json!({"sessionId": "session-1", "outputOffset": 20, "maxBytes": 4096})
    );
}

#[test]
fn inventory_requests_cover_artifacts_ports_and_safe_hook_reports() {
    let artifacts = SessionArtifactsRequest {
        session_id: "session-1".to_string(),
    };
    let ports = SessionPortsRequest {
        session_id: Some("session-1".to_string()),
    };
    let hook = HookReportRequest {
        session_id: "session-1".to_string(),
        event: "permission_prompt".to_string(),
        needs_input: None,
        turn_completed: false,
    };

    assert_eq!(
        serde_json::to_value(artifacts).expect("artifacts request"),
        json!({"sessionId": "session-1"})
    );
    assert_eq!(
        serde_json::to_value(ports).expect("ports request"),
        json!({"sessionId": "session-1"})
    );
    assert_eq!(
        serde_json::to_value(hook).expect("hook report"),
        json!({
            "sessionId": "session-1",
            "event": "permission_prompt",
            "turnCompleted": false
        })
    );
}

#[test]
fn runtime_event_no_longer_depends_on_runtime_crate() {
    let event = RuntimeEvent {
        seq: 5,
        event: "session.updated".to_string(),
        session_id: Some("session-1".to_string()),
        status: Some("running".to_string()),
    };

    assert_eq!(
        serde_json::to_value(event).expect("serialize runtime event"),
        json!({
            "seq": 5,
            "event": "session.updated",
            "sessionId": "session-1",
            "status": "running"
        })
    );
}

#[test]
fn scanned_history_entry_uses_strings_for_wire_paths() {
    let entry = ScannedHistoryEntry {
        agent_kind: "codex".to_string(),
        external_id: "thread-1".to_string(),
        cwd: "/repo".to_string(),
        title: Some("Fix transport".to_string()),
        title_source: "first_prompt".to_string(),
        transcript_path: "/tmp/thread-1.jsonl".to_string(),
        last_active_at: 100,
        created_at: Some(90),
        cwd_exists: true,
    };

    assert_eq!(
        serde_json::to_value(entry).expect("serialize history entry"),
        json!({
            "agentKind": "codex",
            "externalId": "thread-1",
            "cwd": "/repo",
            "title": "Fix transport",
            "titleSource": "first_prompt",
            "transcriptPath": "/tmp/thread-1.jsonl",
            "lastActiveAt": 100,
            "createdAt": 90,
            "cwdExists": true
        })
    );
}

#[test]
fn daemon_status_is_authoritative_and_capability_aware() {
    let status = DaemonStatus {
        status: DaemonStatusKind::Ready,
        daemon_pid: 7,
        daemon_instance_id: "instance-1".to_string(),
        daemon_version: "0.1.0".to_string(),
        method_capabilities: vec![Method::STATE_SNAPSHOT.to_string()],
        stream_capabilities: vec![StreamKind::EventsV1],
        event_oldest_seq: 10,
        event_latest_seq: 12,
    };

    assert_eq!(
        serde_json::to_value(status).expect("serialize daemon status"),
        json!({
            "status": "ready",
            "daemonPid": 7,
            "daemonInstanceId": "instance-1",
            "daemonVersion": "0.1.0",
            "methodCapabilities": ["state.snapshot"],
            "streamCapabilities": ["events.v1"],
            "eventOldestSeq": 10,
            "eventLatestSeq": 12
        })
    );
}

#[test]
fn acknowledgement_results_have_stable_wire_fields() {
    assert_eq!(
        serde_json::to_value(AckResult { ok: true }).expect("serialize ack"),
        json!({"ok": true})
    );
    assert_eq!(
        serde_json::to_value(ShutdownResult { acknowledged: true })
            .expect("serialize shutdown ack"),
        json!({"acknowledged": true})
    );
}

#[test]
fn worktree_overview_result_is_protocol_owned() {
    let result = WorktreeOverviewResult {
        entries: vec![WorktreeOverviewEntry {
            project_root: "/repo".to_string(),
            path: "/repo".to_string(),
            branch: Some("main".to_string()),
            session_id: Some("session-1".to_string()),
            session_status: Some("running".to_string()),
            dirty: false,
            merged: false,
            age_days: 0,
            stale_suggestion: false,
        }],
    };

    assert_eq!(
        serde_json::to_value(result).expect("serialize worktree overview")["entries"][0]["sessionId"],
        "session-1"
    );
}

#[test]
fn method_constants_cover_existing_typed_client_behavior() {
    assert_eq!(
        [
            Method::DAEMON_PREPARE_SHUTDOWN,
            Method::DAEMON_SHUTDOWN,
            Method::SESSION_SNAPSHOT,
            Method::SESSION_STATUS,
            Method::SESSION_ARTIFACTS,
            Method::SESSION_PORTS,
            Method::SESSION_SET_PARENT,
            Method::SESSION_LIST_CHILDREN,
            Method::SESSION_PARENT,
            Method::WORKTREE_OVERVIEW,
            Method::HOOK_REPORT,
        ],
        [
            "daemon.prepare_shutdown",
            "daemon.shutdown",
            "session.snapshot",
            "session.status",
            "session.artifacts",
            "session.ports",
            "session.set_parent",
            "session.list_children",
            "session.parent",
            "worktree.overview",
            "hook.report",
        ]
    );
}
