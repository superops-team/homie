mod support;

use std::time::Duration;

use homie_client::{ClientOptions, HomieClient};
use homie_proto::model::{
    ArtifactScan, PortListRow, ScannedHistoryEntry, SessionParentResult, SessionStatusReport,
    SessionSummary, WorktreeOverviewResult,
};
use homie_proto::paths::RuntimeEndpoint;
use homie_proto::transport::{AckResult, ClientRole};
use homie_proto::{
    HostLocateRepoParams, HostLocateRepoResult, Method, SessionDiffBase, SessionHistoryRequest,
    SessionReadDiffResult, SessionResumeFromHistoryRequest, SessionStatus, WorktreeCreateRequest,
    WorktreeInfo, WorktreeListRequest, WorktreeRemoveRequest,
};
use serde_json::json;

use support::MockSocket;

const METHODS: &[&str] = &[
    Method::SESSION_SPAWN,
    Method::SESSION_LIST,
    Method::SESSION_STATUS,
    Method::SESSION_ARTIFACTS,
    Method::SESSION_PORTS,
    Method::SESSION_SET_PARENT,
    Method::SESSION_LIST_CHILDREN,
    Method::SESSION_PARENT,
    Method::SESSION_HISTORY,
    Method::SESSION_RESUME_FROM_HISTORY,
    Method::SESSION_READ_DIFF,
    Method::HOST_LOCATE_REPO,
    Method::WORKTREE_LIST,
    Method::WORKTREE_CREATE,
    Method::WORKTREE_REMOVE,
    Method::WORKTREE_OVERVIEW,
    Method::HOOK_REPORT,
];

#[tokio::test]
async fn typed_facade_uses_exact_methods_and_protocol_owned_dtos() {
    let socket = MockSocket::bind();
    let endpoint = socket.endpoint().to_path_buf();
    let server = tokio::spawn(async move {
        let mut peer = socket.accept(METHODS, &[], "daemon-a").await;
        for expected in METHODS {
            let request = peer.read_request().await;
            assert_eq!(&request.method, expected);
            if *expected == Method::SESSION_STATUS {
                assert_eq!(request.params, json!({"sessionId": "session-1"}));
            }
            peer.respond_ok(request.message_id, response_for(expected))
                .await;
        }
    });
    let client = HomieClient::connect(options(endpoint))
        .await
        .expect("connect");

    client
        .spawn_shell(std::path::Path::new("/tmp/workspace"), Some("title"))
        .await
        .expect("spawn");
    client.list_sessions().await.expect("list");
    client.status_report("session-1").await.expect("status");
    client
        .scan_session_artifacts("session-1")
        .await
        .expect("artifacts");
    client.list_ports().await.expect("ports");
    client
        .set_session_parent("session-1", "parent-1")
        .await
        .expect("set parent");
    client
        .list_child_sessions("parent-1")
        .await
        .expect("children");
    client.parent_session_id("session-1").await.expect("parent");
    client
        .session_history(SessionHistoryRequest {
            claude_root: "/tmp/claude".to_string(),
            codex_root: "/tmp/codex".to_string(),
            tracked: Vec::new(),
        })
        .await
        .expect("history");
    client
        .resume_from_history(SessionResumeFromHistoryRequest {
            agent_kind: "codex".to_string(),
            external_id: "external-1".to_string(),
            cwd: "/tmp/workspace".to_string(),
            title: None,
        })
        .await
        .expect("resume");
    client
        .read_diff("session-1", SessionDiffBase::Head)
        .await
        .expect("diff");
    client
        .locate_repo(HostLocateRepoParams::default())
        .await
        .expect("locate");
    client
        .worktree_list(WorktreeListRequest {
            repo_path: "/tmp/repo".to_string(),
        })
        .await
        .expect("worktree list");
    client
        .worktree_create(WorktreeCreateRequest {
            repo_path: "/tmp/repo".to_string(),
            branch: Some("feature".to_string()),
            base: None,
        })
        .await
        .expect("worktree create");
    client
        .worktree_remove(WorktreeRemoveRequest {
            repo_path: "/tmp/repo".to_string(),
            worktree_path: "/tmp/repo-feature".to_string(),
            force: false,
        })
        .await
        .expect("worktree remove");
    client.worktree_overview().await.expect("overview");
    client
        .report_turn_complete("session-1")
        .await
        .expect("hook report");

    client.close().await.expect("close");
    server.await.expect("server");
}

fn options(endpoint: std::path::PathBuf) -> ClientOptions {
    ClientOptions {
        endpoint: RuntimeEndpoint::new(endpoint).expect("absolute endpoint"),
        role: ClientRole::Cli,
        connect_timeout: Duration::from_secs(1),
        request_timeout: Duration::from_millis(50),
    }
}

fn response_for(method: &str) -> serde_json::Value {
    match method {
        Method::SESSION_SPAWN | Method::SESSION_RESUME_FROM_HISTORY => {
            serde_json::to_value(session()).expect("session")
        }
        Method::SESSION_LIST | Method::SESSION_LIST_CHILDREN => {
            serde_json::to_value(vec![session()]).expect("sessions")
        }
        Method::SESSION_STATUS => serde_json::to_value(SessionStatusReport {
            status: SessionStatus::Running,
            needs_input: None,
            turn_completed: false,
            screen_lines: Vec::new(),
            screen_observation: None,
        })
        .expect("status"),
        Method::SESSION_ARTIFACTS => serde_json::to_value(ArtifactScan {
            artifacts: Vec::new(),
            ports: Vec::new(),
        })
        .expect("artifacts"),
        Method::SESSION_PORTS => serde_json::to_value(Vec::<PortListRow>::new()).expect("ports"),
        Method::SESSION_PARENT => serde_json::to_value(SessionParentResult {
            parent_session_id: Some("parent-1".to_string()),
        })
        .expect("parent"),
        Method::SESSION_HISTORY => {
            serde_json::to_value(Vec::<ScannedHistoryEntry>::new()).expect("history")
        }
        Method::SESSION_READ_DIFF => serde_json::to_value(SessionReadDiffResult {
            patch: Vec::new(),
            repo_root: "/tmp/repo".to_string(),
            truncated: false,
            base_ref: Some("HEAD".to_string()),
        })
        .expect("diff"),
        Method::HOST_LOCATE_REPO => serde_json::to_value(HostLocateRepoResult {
            path: Some("/tmp/repo".to_string()),
            origin_url: None,
        })
        .expect("repo"),
        Method::WORKTREE_LIST => serde_json::to_value(vec![worktree()]).expect("worktrees"),
        Method::WORKTREE_CREATE => serde_json::to_value(worktree()).expect("worktree"),
        Method::WORKTREE_OVERVIEW => serde_json::to_value(WorktreeOverviewResult {
            entries: Vec::new(),
        })
        .expect("overview"),
        Method::SESSION_SET_PARENT | Method::WORKTREE_REMOVE | Method::HOOK_REPORT => {
            serde_json::to_value(AckResult { ok: true }).expect("ack")
        }
        other => panic!("unexpected method {other}"),
    }
}

fn session() -> SessionSummary {
    SessionSummary {
        id: "session-1".to_string(),
        title: "title".to_string(),
        status: "running".to_string(),
        workspace: "/tmp/workspace".to_string(),
        agent_profile_id: "agent".to_string(),
        runtime_id: "runtime".to_string(),
        llm_profile_id: "llm".to_string(),
        permission_profile_id: "permission".to_string(),
    }
}

fn worktree() -> WorktreeInfo {
    WorktreeInfo {
        path: "/tmp/repo-feature".to_string(),
        branch: Some("feature".to_string()),
        is_bare: false,
        is_detached: false,
        is_prunable: false,
    }
}
