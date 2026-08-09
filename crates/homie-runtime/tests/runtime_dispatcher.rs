use std::future::Future;
use std::path::Path;
use std::pin::Pin;
use std::process::Command;
use std::sync::Arc;

use homie_proto::model::{SessionChildrenRequest, SessionParentRequest, SessionSetParentRequest};
use homie_proto::{
    EventsWaitRequest, Method, SessionDiffBase, SessionKillRequest, SessionReadDiffResult,
};
use homie_runtime::dispatcher::{
    ActorRequest, AsyncWaitHandler, RuntimeDispatcher, RuntimeLongRunningExecutor, RuntimeResponse,
};
use homie_runtime::long_running::LongRunningLane;
use homie_runtime::runtime_actor::{
    RuntimeActor, RuntimeCall, RuntimeReply, RuntimeSupervisorBackend, ServiceResult,
};
use homie_runtime::terminal_stream::{
    RuntimeTerminalBackend, TerminalBackend, TerminalSourceDescriptor, TerminalStreamError,
};
use homie_runtime::{RuntimeConfig, RuntimeSupervisor};
use homie_storage::{CreateSession, ProjectUpsert, StorageConfig, open_or_create};
use serde_json::{Value, json};

#[test]
fn production_backend_runs_session_list_through_the_actor_owner() {
    let temp = tempfile::tempdir().expect("tempdir");
    let supervisor = RuntimeSupervisor::open(RuntimeConfig {
        data_dir: temp.path().to_path_buf(),
    })
    .expect("runtime");
    let actor =
        RuntimeActor::spawn(RuntimeSupervisorBackend::new(supervisor)).expect("spawn actor");

    let result = actor
        .handle()
        .try_call(RuntimeCall::Invoke(ActorRequest::SessionList))
        .expect("submit")
        .blocking_recv()
        .expect("reply")
        .expect("session list");

    assert_eq!(
        result,
        RuntimeReply::Response(RuntimeResponse::Sessions(Vec::new()))
    );
    actor.shutdown().expect("shutdown");
}

#[test]
fn production_backend_prepare_shutdown_checkpoints_sqlite_wal() {
    let temp = tempfile::tempdir().expect("tempdir");
    let data_dir = temp.path().join("data");
    let supervisor = RuntimeSupervisor::open(RuntimeConfig {
        data_dir: data_dir.clone(),
    })
    .expect("runtime");
    supervisor
        .storage()
        .connection()
        .execute_batch(
            "PRAGMA wal_autocheckpoint = 0;
             CREATE TABLE durable_prepare_probe (value INTEGER NOT NULL);
             INSERT INTO durable_prepare_probe (value) VALUES (1);",
        )
        .expect("write WAL");
    let wal_path = data_dir.join("homie.sqlite-wal");
    assert!(
        std::fs::metadata(&wal_path).expect("WAL metadata").len() > 0,
        "fixture must contain WAL frames"
    );
    let actor =
        RuntimeActor::spawn(RuntimeSupervisorBackend::new(supervisor)).expect("spawn actor");

    actor
        .handle()
        .prepare_shutdown()
        .expect("prepare command")
        .blocking_recv()
        .expect("prepare reply")
        .expect("prepare shutdown");

    assert_eq!(std::fs::metadata(wal_path).expect("WAL metadata").len(), 0);
    actor.shutdown().expect("shutdown");
}

#[test]
fn production_backend_executes_actor_owned_lineage_operations() {
    let temp = tempfile::tempdir().expect("tempdir");
    let storage = open_or_create(StorageConfig {
        data_dir: temp.path().to_path_buf(),
    })
    .expect("storage");
    storage.migrate().expect("migrate");
    storage.seed_defaults().expect("seed");
    let parent = storage
        .create_session(CreateSession {
            workspace: temp.path().to_path_buf(),
            title: Some("Parent".to_string()),
        })
        .expect("parent");
    let child = storage
        .create_session(CreateSession {
            workspace: temp.path().to_path_buf(),
            title: Some("Child".to_string()),
        })
        .expect("child");
    drop(storage);

    let supervisor = RuntimeSupervisor::open(RuntimeConfig {
        data_dir: temp.path().to_path_buf(),
    })
    .expect("runtime");
    let actor =
        RuntimeActor::spawn(RuntimeSupervisorBackend::new(supervisor)).expect("spawn actor");
    let handle = actor.handle();

    let set_parent = call(
        &handle,
        ActorRequest::SessionSetParent(SessionSetParentRequest {
            session_id: child.id.clone(),
            parent_session_id: parent.id.clone(),
        }),
    );
    assert_eq!(
        set_parent,
        RuntimeReply::Response(RuntimeResponse::Ack(homie_proto::transport::AckResult {
            ok: true
        }))
    );

    let children = call(
        &handle,
        ActorRequest::SessionListChildren(SessionChildrenRequest {
            parent_session_id: parent.id.clone(),
        }),
    );
    let RuntimeReply::Response(RuntimeResponse::Sessions(children)) = children else {
        panic!("unexpected children reply");
    };
    assert_eq!(children[0].id, child.id);

    let parent_reply = call(
        &handle,
        ActorRequest::SessionParent(SessionParentRequest {
            session_id: child.id.clone(),
        }),
    );
    let RuntimeReply::Response(RuntimeResponse::SessionParent(parent_reply)) = parent_reply else {
        panic!("unexpected parent reply");
    };
    assert_eq!(
        parent_reply.parent_session_id.as_deref(),
        Some(parent.id.as_str())
    );

    actor.shutdown().expect("shutdown");
}

#[tokio::test]
async fn production_terminal_descriptor_uses_default_geometry_without_live_holder() {
    let temp = tempfile::tempdir().expect("tempdir");
    let data_dir = temp.path().join("data");
    let storage = open_or_create(StorageConfig {
        data_dir: data_dir.clone(),
    })
    .expect("storage");
    storage.migrate().expect("migrate");
    storage.seed_defaults().expect("seed");
    let session = storage
        .create_session(CreateSession {
            workspace: temp.path().to_path_buf(),
            title: Some("Terminal".to_string()),
        })
        .expect("session");
    drop(storage);
    let supervisor = RuntimeSupervisor::open(RuntimeConfig {
        data_dir: data_dir.clone(),
    })
    .expect("runtime");
    let actor =
        RuntimeActor::spawn(RuntimeSupervisorBackend::new(supervisor)).expect("spawn actor");
    let backend = RuntimeTerminalBackend::new(actor.handle());

    let descriptor = backend.describe(&session.id).await.expect("descriptor");

    assert_eq!(
        descriptor,
        TerminalSourceDescriptor {
            session_id: session.id.clone(),
            output_path: data_dir
                .join("runtime")
                .join("output")
                .join(format!("{}.log", session.id)),
            cols: 120,
            rows: 40,
            modes: Vec::new(),
        }
    );
    actor.shutdown_async().await.expect("shutdown");
}

#[tokio::test]
async fn production_terminal_resize_updates_live_holder_descriptor() {
    let temp = tempfile::tempdir().expect("tempdir");
    let supervisor = RuntimeSupervisor::open_with_holder(
        RuntimeConfig {
            data_dir: temp.path().join("data"),
        },
        Path::new(env!("CARGO_BIN_EXE_homie-runtime-holder")).to_path_buf(),
    )
    .expect("runtime");
    let session = supervisor
        .spawn_shell(temp.path(), Some("Terminal"))
        .expect("spawn");
    let actor =
        RuntimeActor::spawn(RuntimeSupervisorBackend::new(supervisor)).expect("spawn actor");
    let handle = actor.handle();
    let backend = RuntimeTerminalBackend::new(handle.clone());

    backend.resize(&session.id, 101, 31).await.expect("resize");
    let descriptor = backend.describe(&session.id).await.expect("descriptor");

    assert_eq!((descriptor.cols, descriptor.rows), (101, 31));
    handle
        .try_call(RuntimeCall::Invoke(ActorRequest::SessionKill(
            SessionKillRequest {
                session_id: session.id.into(),
            },
        )))
        .expect("submit cleanup")
        .await
        .expect("cleanup reply")
        .expect("cleanup");
    actor.shutdown_async().await.expect("shutdown");
}

#[tokio::test]
async fn production_terminal_descriptor_rejects_unknown_session() {
    let temp = tempfile::tempdir().expect("tempdir");
    let supervisor = RuntimeSupervisor::open(RuntimeConfig {
        data_dir: temp.path().to_path_buf(),
    })
    .expect("runtime");
    let actor =
        RuntimeActor::spawn(RuntimeSupervisorBackend::new(supervisor)).expect("spawn actor");
    let backend = RuntimeTerminalBackend::new(actor.handle());

    let error = backend
        .describe("missing-session")
        .await
        .expect_err("unknown session");

    assert!(matches!(error, TerminalStreamError::Backend));
    actor.shutdown_async().await.expect("shutdown");
}

#[tokio::test]
async fn production_history_runs_prepare_lane_commit_and_persists_atomically() {
    let temp = tempfile::tempdir().expect("tempdir");
    let cwd = temp.path().join("project");
    std::fs::create_dir_all(&cwd).expect("cwd");
    let codex_root = temp.path().join("codex/2026/08/08");
    std::fs::create_dir_all(&codex_root).expect("codex root");
    std::fs::write(
        codex_root.join("rollout-history.jsonl"),
        format!(
            "{{\"type\":\"session_meta\",\"payload\":{{\"id\":\"thread-history\",\"cwd\":{}}}}}\n",
            serde_json::to_string(&cwd).expect("cwd json")
        ),
    )
    .expect("history fixture");
    let data_dir = temp.path().join("data");
    let supervisor = RuntimeSupervisor::open(RuntimeConfig {
        data_dir: data_dir.clone(),
    })
    .expect("runtime");
    let actor =
        RuntimeActor::spawn(RuntimeSupervisorBackend::new(supervisor)).expect("spawn actor");
    let lane = LongRunningLane::spawn().expect("lane");
    let dispatcher = RuntimeDispatcher::new(
        actor.handle(),
        lane.handle(),
        Arc::new(RuntimeLongRunningExecutor),
        Arc::new(NoopWait),
    );

    let result = dispatcher
        .dispatch(
            Method::SESSION_HISTORY,
            json!({
                "claudeRoot": temp.path().join("claude"),
                "codexRoot": temp.path().join("codex")
            }),
        )
        .await
        .expect("history dispatch");

    assert_eq!(result.as_array().map(Vec::len), Some(1));
    actor.shutdown_async().await.expect("actor shutdown");
    lane.shutdown_async().await.expect("lane shutdown");
    let storage = open_or_create(StorageConfig { data_dir }).expect("reopen storage");
    let entries = storage.list_history_entries(10).expect("stored history");
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].external_id, "thread-history");
}

#[tokio::test]
async fn production_worktree_list_uses_the_long_running_lane() {
    let temp = tempfile::tempdir().expect("tempdir");
    let repo = temp.path().join("repo");
    std::fs::create_dir_all(&repo).expect("repo");
    run_git(&repo, ["init", "-q", "-b", "main"]);
    let supervisor = RuntimeSupervisor::open(RuntimeConfig {
        data_dir: temp.path().join("data"),
    })
    .expect("runtime");
    let actor =
        RuntimeActor::spawn(RuntimeSupervisorBackend::new(supervisor)).expect("spawn actor");
    let lane = LongRunningLane::spawn().expect("lane");
    let dispatcher = RuntimeDispatcher::new(
        actor.handle(),
        lane.handle(),
        Arc::new(RuntimeLongRunningExecutor),
        Arc::new(NoopWait),
    );

    let result = dispatcher
        .dispatch(Method::WORKTREE_LIST, json!({"repoPath": repo}))
        .await
        .expect("worktree list");

    let worktrees = result.as_array().expect("worktree array");
    assert_eq!(worktrees.len(), 1);
    assert_eq!(
        worktrees[0].get("branch").and_then(Value::as_str),
        Some("main")
    );
    actor.shutdown_async().await.expect("actor shutdown");
    lane.shutdown_async().await.expect("lane shutdown");
}

#[tokio::test]
async fn production_worktree_mutations_use_bounded_mutation_jobs() {
    let temp = tempfile::tempdir().expect("tempdir");
    let repo = temp.path().join("repo");
    std::fs::create_dir_all(&repo).expect("repo");
    run_git(&repo, ["init", "-q", "-b", "main"]);
    run_git(&repo, ["config", "user.email", "test@example.invalid"]);
    run_git(&repo, ["config", "user.name", "Homie Test"]);
    std::fs::write(repo.join("README.md"), "hello\n").expect("readme");
    run_git(&repo, ["add", "README.md"]);
    run_git(&repo, ["commit", "-q", "-m", "initial"]);
    let supervisor = RuntimeSupervisor::open(RuntimeConfig {
        data_dir: temp.path().join("data"),
    })
    .expect("runtime");
    let actor =
        RuntimeActor::spawn(RuntimeSupervisorBackend::new(supervisor)).expect("spawn actor");
    let lane = LongRunningLane::spawn().expect("lane");
    let dispatcher = RuntimeDispatcher::new(
        actor.handle(),
        lane.handle(),
        Arc::new(RuntimeLongRunningExecutor),
        Arc::new(NoopWait),
    );

    let created = dispatcher
        .dispatch(
            Method::WORKTREE_CREATE,
            json!({
                "repoPath": repo,
                "branch": "feature/runtime-lane",
                "base": "HEAD"
            }),
        )
        .await
        .expect("create worktree");
    let worktree_path = created
        .get("path")
        .and_then(Value::as_str)
        .expect("worktree path")
        .to_string();
    assert!(Path::new(&worktree_path).is_dir());

    let removed = dispatcher
        .dispatch(
            Method::WORKTREE_REMOVE,
            json!({
                "repoPath": repo,
                "worktreePath": worktree_path,
                "force": true
            }),
        )
        .await
        .expect("remove worktree");
    assert_eq!(removed, json!({"ok": true}));
    assert!(!Path::new(&worktree_path).exists());

    actor.shutdown_async().await.expect("actor shutdown");
    lane.shutdown_async().await.expect("lane shutdown");
}

#[tokio::test]
async fn production_locate_and_overview_use_bounded_git_candidates() {
    let temp = tempfile::tempdir().expect("tempdir");
    let repo = temp.path().join("repo");
    std::fs::create_dir_all(&repo).expect("repo");
    run_git(&repo, ["init", "-q", "-b", "main"]);
    run_git(&repo, ["config", "user.email", "test@example.invalid"]);
    run_git(&repo, ["config", "user.name", "Homie Test"]);
    run_git(
        &repo,
        [
            "remote",
            "add",
            "origin",
            "https://example.invalid/acme/repo.git",
        ],
    );
    std::fs::write(repo.join("README.md"), "hello\n").expect("readme");
    run_git(&repo, ["add", "README.md"]);
    run_git(&repo, ["commit", "-q", "-m", "initial"]);

    let data_dir = temp.path().join("data");
    let storage = open_or_create(StorageConfig {
        data_dir: data_dir.clone(),
    })
    .expect("storage");
    storage.migrate().expect("migrate");
    storage.seed_defaults().expect("seed");
    storage
        .upsert_project(ProjectUpsert {
            root_path: repo.clone(),
            display_name: Some("Repo".to_string()),
            remote_origin: Some("https://example.invalid/acme/repo.git".to_string()),
            pinned_order: None,
        })
        .expect("project");
    drop(storage);
    let supervisor = RuntimeSupervisor::open(RuntimeConfig { data_dir }).expect("runtime");
    let actor =
        RuntimeActor::spawn(RuntimeSupervisorBackend::new(supervisor)).expect("spawn actor");
    let lane = LongRunningLane::spawn().expect("lane");
    let dispatcher = RuntimeDispatcher::new(
        actor.handle(),
        lane.handle(),
        Arc::new(RuntimeLongRunningExecutor),
        Arc::new(NoopWait),
    );

    let located = dispatcher
        .dispatch(
            Method::HOST_LOCATE_REPO,
            json!({"originURL": "https://example.invalid/acme/repo.git"}),
        )
        .await
        .expect("locate repo");
    assert_eq!(
        located.get("path").and_then(Value::as_str),
        Some(
            repo.canonicalize()
                .expect("canonical repo")
                .to_string_lossy()
                .as_ref()
        )
    );

    let overview = dispatcher
        .dispatch(Method::WORKTREE_OVERVIEW, json!({}))
        .await
        .expect("worktree overview");
    let entries = overview
        .get("entries")
        .and_then(Value::as_array)
        .expect("overview entries");
    assert_eq!(entries.len(), 1);
    assert_eq!(
        entries[0].get("branch").and_then(Value::as_str),
        Some("main")
    );

    actor.shutdown_async().await.expect("actor shutdown");
    lane.shutdown_async().await.expect("lane shutdown");
}

#[tokio::test]
async fn production_session_diff_prepares_cwd_then_runs_bounded_git() {
    let temp = tempfile::tempdir().expect("tempdir");
    let repo = temp.path().join("repo");
    std::fs::create_dir_all(&repo).expect("repo");
    run_git(&repo, ["init", "-q", "-b", "main"]);
    run_git(&repo, ["config", "user.email", "test@example.invalid"]);
    run_git(&repo, ["config", "user.name", "Homie Test"]);
    std::fs::write(repo.join("tracked.txt"), "before\n").expect("tracked");
    run_git(&repo, ["add", "tracked.txt"]);
    run_git(&repo, ["commit", "-q", "-m", "initial"]);
    std::fs::write(repo.join("tracked.txt"), "after\n").expect("tracked");

    let data_dir = temp.path().join("data");
    let storage = open_or_create(StorageConfig {
        data_dir: data_dir.clone(),
    })
    .expect("storage");
    storage.migrate().expect("migrate");
    storage.seed_defaults().expect("seed");
    let session = storage
        .create_session(CreateSession {
            workspace: repo.clone(),
            title: Some("Diff".to_string()),
        })
        .expect("session");
    drop(storage);
    let supervisor = RuntimeSupervisor::open(RuntimeConfig { data_dir }).expect("runtime");
    let actor =
        RuntimeActor::spawn(RuntimeSupervisorBackend::new(supervisor)).expect("spawn actor");
    let lane = LongRunningLane::spawn().expect("lane");
    let dispatcher = RuntimeDispatcher::new(
        actor.handle(),
        lane.handle(),
        Arc::new(RuntimeLongRunningExecutor),
        Arc::new(NoopWait),
    );

    let result = dispatcher
        .dispatch(
            Method::SESSION_READ_DIFF,
            json!({"sessionID": session.id, "base": SessionDiffBase::Head}),
        )
        .await
        .expect("session diff");

    assert_eq!(
        result.get("repoRoot").and_then(Value::as_str),
        Some(
            repo.canonicalize()
                .expect("canonical repo")
                .to_string_lossy()
                .as_ref()
        )
    );
    assert!(
        result
            .get("patch")
            .and_then(Value::as_str)
            .is_some_and(|patch| !patch.is_empty())
    );
    actor.shutdown_async().await.expect("actor shutdown");
    lane.shutdown_async().await.expect("lane shutdown");
}

#[tokio::test]
async fn production_default_branch_diff_includes_committed_and_working_changes_from_merge_base() {
    let temp = tempfile::tempdir().expect("tempdir");
    let repo = temp.path().join("repo");
    std::fs::create_dir_all(&repo).expect("repo");
    run_git(&repo, ["init", "-q", "-b", "main"]);
    run_git(&repo, ["config", "user.email", "test@example.invalid"]);
    run_git(&repo, ["config", "user.name", "Homie Test"]);
    std::fs::write(repo.join("tracked.txt"), "before\n").expect("tracked");
    run_git(&repo, ["add", "tracked.txt"]);
    run_git(&repo, ["commit", "-q", "-m", "initial"]);
    run_git(&repo, ["checkout", "-q", "-b", "feature"]);
    std::fs::write(repo.join("committed.txt"), "committed branch change\n").expect("committed");
    run_git(&repo, ["add", "committed.txt"]);
    run_git(&repo, ["commit", "-q", "-m", "feature"]);
    std::fs::write(repo.join("tracked.txt"), "working change\n").expect("working");

    let result = production_session_diff(&repo, SessionDiffBase::DefaultBranch).await;
    let patch = String::from_utf8(result.patch).expect("utf-8 patch");

    assert_eq!(result.base_ref.as_deref(), Some("main"));
    assert!(patch.contains("committed.txt"), "{patch}");
    assert!(patch.contains("+committed branch change"), "{patch}");
    assert!(patch.contains("+working change"), "{patch}");
}

#[tokio::test]
async fn production_head_diff_excludes_committed_changes_and_includes_working_changes() {
    let temp = tempfile::tempdir().expect("tempdir");
    let repo = temp.path().join("repo");
    std::fs::create_dir_all(&repo).expect("repo");
    run_git(&repo, ["init", "-q", "-b", "main"]);
    run_git(&repo, ["config", "user.email", "test@example.invalid"]);
    run_git(&repo, ["config", "user.name", "Homie Test"]);
    std::fs::write(repo.join("tracked.txt"), "before\n").expect("tracked");
    run_git(&repo, ["add", "tracked.txt"]);
    run_git(&repo, ["commit", "-q", "-m", "initial"]);
    run_git(&repo, ["checkout", "-q", "-b", "feature"]);
    std::fs::write(repo.join("committed.txt"), "committed branch change\n").expect("committed");
    run_git(&repo, ["add", "committed.txt"]);
    run_git(&repo, ["commit", "-q", "-m", "feature"]);
    std::fs::write(repo.join("tracked.txt"), "working change\n").expect("working");

    let result = production_session_diff(&repo, SessionDiffBase::Head).await;
    let patch = String::from_utf8(result.patch).expect("utf-8 patch");

    assert_eq!(result.base_ref.as_deref(), Some("HEAD"));
    assert!(!patch.contains("committed.txt"), "{patch}");
    assert!(!patch.contains("committed branch change"), "{patch}");
    assert!(patch.contains("+working change"), "{patch}");
}

#[tokio::test]
async fn production_unborn_head_diff_includes_staged_unstaged_and_untracked_changes() {
    let temp = tempfile::tempdir().expect("tempdir");
    let repo = temp.path().join("repo");
    std::fs::create_dir_all(&repo).expect("repo");
    run_git(&repo, ["init", "-q", "-b", "main"]);
    std::fs::write(repo.join("staged.txt"), "staged change\n").expect("staged");
    run_git(&repo, ["add", "staged.txt"]);
    std::fs::write(repo.join("unstaged.txt"), "staged baseline\n").expect("unstaged baseline");
    run_git(&repo, ["add", "unstaged.txt"]);
    std::fs::write(repo.join("unstaged.txt"), "unstaged change\n").expect("unstaged");
    std::fs::write(repo.join("untracked.txt"), "untracked change\n").expect("untracked");

    let result = production_session_diff(&repo, SessionDiffBase::Head).await;
    let patch = String::from_utf8(result.patch).expect("utf-8 patch");

    assert!(patch.contains("staged.txt"), "{patch}");
    assert!(patch.contains("+staged change"), "{patch}");
    assert!(patch.contains("unstaged.txt"), "{patch}");
    assert!(patch.contains("+unstaged change"), "{patch}");
    assert!(patch.contains("untracked.txt"), "{patch}");
    assert!(patch.contains("+untracked change"), "{patch}");
}

#[tokio::test]
async fn production_output_handlers_scan_only_on_the_long_running_lane() {
    let temp = tempfile::tempdir().expect("tempdir");
    let data_dir = temp.path().join("data");
    let storage = open_or_create(StorageConfig {
        data_dir: data_dir.clone(),
    })
    .expect("storage");
    storage.migrate().expect("migrate");
    storage.seed_defaults().expect("seed");
    let session = storage
        .create_session(CreateSession {
            workspace: temp.path().to_path_buf(),
            title: Some("Output".to_string()),
        })
        .expect("session");
    let output_dir = data_dir.join("runtime/output");
    std::fs::create_dir_all(&output_dir).expect("output dir");
    std::fs::write(
        output_dir.join(format!("{}.log", session.id)),
        "preview http://localhost:4312\nhomie-status:idle\n",
    )
    .expect("output");
    drop(storage);

    let supervisor = RuntimeSupervisor::open(RuntimeConfig { data_dir }).expect("runtime");
    let actor =
        RuntimeActor::spawn(RuntimeSupervisorBackend::new(supervisor)).expect("spawn actor");
    let lane = LongRunningLane::spawn().expect("lane");
    let dispatcher = RuntimeDispatcher::new(
        actor.handle(),
        lane.handle(),
        Arc::new(RuntimeLongRunningExecutor),
        Arc::new(NoopWait),
    );

    let snapshot = dispatcher
        .dispatch(
            Method::SESSION_SNAPSHOT,
            json!({"sessionId": session.id, "maxBytes": 4096}),
        )
        .await
        .expect("snapshot");
    assert!(
        snapshot
            .get("outputText")
            .and_then(Value::as_str)
            .is_some_and(|output| output.contains("localhost:4312"))
    );

    let artifacts = dispatcher
        .dispatch(Method::SESSION_ARTIFACTS, json!({"sessionId": session.id}))
        .await
        .expect("artifacts");
    assert_eq!(
        artifacts
            .get("ports")
            .and_then(Value::as_array)
            .map(Vec::len),
        Some(1)
    );

    let ports = dispatcher
        .dispatch(Method::SESSION_PORTS, json!({}))
        .await
        .expect("ports");
    assert_eq!(ports.as_array().map(Vec::len), Some(1));

    let status = dispatcher
        .dispatch(Method::SESSION_STATUS, json!({"sessionId": session.id}))
        .await
        .expect("status");
    assert!(
        status
            .get("screenLines")
            .and_then(Value::as_array)
            .is_some_and(|lines| lines.iter().any(|line| {
                line.as_str()
                    .is_some_and(|line| line.contains("homie-status:idle"))
            }))
    );

    actor.shutdown_async().await.expect("actor shutdown");
    lane.shutdown_async().await.expect("lane shutdown");
}

#[tokio::test]
async fn production_actor_handlers_execute_runtime_lifecycle_and_shutdown() {
    let temp = tempfile::tempdir().expect("tempdir");
    let data_dir = temp.path().join("data");
    let supervisor = RuntimeSupervisor::open_with_holder(
        RuntimeConfig {
            data_dir: data_dir.clone(),
        },
        Path::new(env!("CARGO_BIN_EXE_homie-runtime-holder")).to_path_buf(),
    )
    .expect("runtime");
    let actor =
        RuntimeActor::spawn(RuntimeSupervisorBackend::new(supervisor)).expect("spawn actor");
    let lane = LongRunningLane::spawn().expect("lane");
    let dispatcher = RuntimeDispatcher::new(
        actor.handle(),
        lane.handle(),
        Arc::new(RuntimeLongRunningExecutor),
        Arc::new(NoopWait),
    );

    let state = dispatcher
        .dispatch(Method::STATE_SNAPSHOT, json!({}))
        .await
        .expect("state snapshot");
    assert_eq!(
        state
            .get("sessions")
            .and_then(Value::as_array)
            .map(Vec::len),
        Some(0)
    );

    let spawned = dispatcher
        .dispatch(
            Method::SESSION_SPAWN,
            json!({"cwd": temp.path(), "title": "Lifecycle"}),
        )
        .await
        .expect("spawn");
    let session_id = spawned
        .get("id")
        .and_then(Value::as_str)
        .expect("session id")
        .to_string();

    let sent = dispatcher
        .dispatch(
            Method::SESSION_SEND_TEXT,
            json!({
                "sessionId": session_id,
                "text": "printf 'actor-handler\\n'",
                "submit": true
            }),
        )
        .await
        .expect("send text");
    assert_eq!(sent, json!({"ok": true}));

    let resized = dispatcher
        .dispatch(
            Method::SESSION_RESIZE,
            json!({"sessionId": session_id, "cols": 100, "rows": 30}),
        )
        .await
        .expect("resize");
    assert_eq!(resized, json!({"ok": true}));

    let reported = dispatcher
        .dispatch(
            Method::HOOK_REPORT,
            json!({
                "sessionId": session_id,
                "event": "turn_completed",
                "turnCompleted": true
            }),
        )
        .await
        .expect("hook report");
    assert_eq!(reported, json!({"ok": true}));

    let killed = dispatcher
        .dispatch(Method::SESSION_KILL, json!({"sessionId": session_id}))
        .await
        .expect("kill");
    assert_eq!(killed, json!({"ok": true}));

    let preparing = dispatcher
        .dispatch(Method::DAEMON_PREPARE_SHUTDOWN, json!({}))
        .await
        .expect("prepare shutdown");
    assert_eq!(preparing, json!({"ok": true}));
    assert_eq!(
        dispatcher.dispatch(Method::SESSION_LIST, json!({})).await,
        Err(homie_runtime::runtime_actor::ServiceError::Unavailable)
    );
    actor.shutdown_async().await.expect("actor shutdown");
    lane.shutdown_async().await.expect("lane shutdown");

    let supervisor = RuntimeSupervisor::open(RuntimeConfig { data_dir }).expect("reopen runtime");
    let actor =
        RuntimeActor::spawn(RuntimeSupervisorBackend::new(supervisor)).expect("spawn actor");
    let lane = LongRunningLane::spawn().expect("lane");
    let dispatcher = RuntimeDispatcher::new(
        actor.handle(),
        lane.handle(),
        Arc::new(RuntimeLongRunningExecutor),
        Arc::new(NoopWait),
    );
    let shutdown = dispatcher
        .dispatch(Method::DAEMON_SHUTDOWN, json!({}))
        .await
        .expect("shutdown request");
    assert_eq!(shutdown, json!({"acknowledged": true}));
    actor.shutdown_async().await.expect("actor shutdown");
    lane.shutdown_async().await.expect("lane shutdown");
}

#[tokio::test]
async fn production_resume_from_history_uses_actor_owned_runtime_and_storage() {
    let temp = tempfile::tempdir().expect("tempdir");
    let cwd = temp.path().join("project");
    std::fs::create_dir_all(&cwd).expect("cwd");
    let codex_root = temp.path().join("codex/2026/08/08");
    std::fs::create_dir_all(&codex_root).expect("codex root");
    std::fs::write(
        codex_root.join("rollout-resume.jsonl"),
        format!(
            "{{\"type\":\"session_meta\",\"payload\":{{\"id\":\"thread-resume\",\"cwd\":{}}}}}\n",
            serde_json::to_string(&cwd).expect("cwd json")
        ),
    )
    .expect("history fixture");
    let supervisor = RuntimeSupervisor::open_with_holder(
        RuntimeConfig {
            data_dir: temp.path().join("data"),
        },
        Path::new(env!("CARGO_BIN_EXE_homie-runtime-holder")).to_path_buf(),
    )
    .expect("runtime");
    let actor =
        RuntimeActor::spawn(RuntimeSupervisorBackend::new(supervisor)).expect("spawn actor");
    let lane = LongRunningLane::spawn().expect("lane");
    let dispatcher = RuntimeDispatcher::new(
        actor.handle(),
        lane.handle(),
        Arc::new(RuntimeLongRunningExecutor),
        Arc::new(NoopWait),
    );
    dispatcher
        .dispatch(
            Method::SESSION_HISTORY,
            json!({
                "claudeRoot": temp.path().join("claude"),
                "codexRoot": temp.path().join("codex")
            }),
        )
        .await
        .expect("history");

    let resumed = dispatcher
        .dispatch(
            Method::SESSION_RESUME_FROM_HISTORY,
            json!({
                "agentKind": "codex",
                "externalId": "thread-resume",
                "cwd": cwd,
                "title": "Resumed"
            }),
        )
        .await
        .expect("resume");
    let session_id = resumed
        .get("id")
        .and_then(Value::as_str)
        .expect("session id")
        .to_string();
    assert_eq!(
        resumed.get("status").and_then(Value::as_str),
        Some("running")
    );

    dispatcher
        .dispatch(Method::SESSION_KILL, json!({"sessionId": session_id}))
        .await
        .expect("cleanup resumed session");
    actor.shutdown_async().await.expect("actor shutdown");
    lane.shutdown_async().await.expect("lane shutdown");
}

struct NoopWait;

impl AsyncWaitHandler for NoopWait {
    fn wait(
        &self,
        _request: EventsWaitRequest,
    ) -> Pin<Box<dyn Future<Output = ServiceResult<Value>> + Send + '_>> {
        Box::pin(async { Ok(json!([])) })
    }
}

fn call(
    handle: &homie_runtime::runtime_actor::RuntimeActorHandle,
    request: ActorRequest,
) -> RuntimeReply {
    handle
        .try_call(RuntimeCall::Invoke(request))
        .expect("submit")
        .blocking_recv()
        .expect("reply")
        .expect("runtime call")
}

async fn production_session_diff(
    repo: &Path,
    comparison: SessionDiffBase,
) -> SessionReadDiffResult {
    let data_dir = repo.parent().expect("repo parent").join("data");
    let storage = open_or_create(StorageConfig {
        data_dir: data_dir.clone(),
    })
    .expect("storage");
    storage.migrate().expect("migrate");
    storage.seed_defaults().expect("seed");
    let session = storage
        .create_session(CreateSession {
            workspace: repo.to_path_buf(),
            title: Some("Diff".to_string()),
        })
        .expect("session");
    drop(storage);
    let supervisor = RuntimeSupervisor::open(RuntimeConfig { data_dir }).expect("runtime");
    let actor =
        RuntimeActor::spawn(RuntimeSupervisorBackend::new(supervisor)).expect("spawn actor");
    let lane = LongRunningLane::spawn().expect("lane");
    let dispatcher = RuntimeDispatcher::new(
        actor.handle(),
        lane.handle(),
        Arc::new(RuntimeLongRunningExecutor),
        Arc::new(NoopWait),
    );

    let result = dispatcher
        .dispatch(
            Method::SESSION_READ_DIFF,
            json!({"sessionID": session.id, "base": comparison}),
        )
        .await
        .expect("session diff");

    actor.shutdown_async().await.expect("actor shutdown");
    lane.shutdown_async().await.expect("lane shutdown");
    serde_json::from_value(result).expect("diff result")
}

fn run_git<const N: usize>(cwd: &Path, args: [&str; N]) {
    let output = Command::new("/usr/bin/git")
        .args(args)
        .current_dir(cwd)
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .output()
        .expect("run git");
    assert!(
        output.status.success(),
        "git failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
