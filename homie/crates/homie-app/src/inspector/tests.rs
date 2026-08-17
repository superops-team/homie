use super::*;

use homie_proto::{AgentKind as ProtoAgentKind, SessionId, SessionStatus};

fn inspector_fixture_session() -> SessionRecord {
    SessionRecord {
        id: SessionId::new("inspector-fixture"),
        kind: ProtoAgentKind::SHELL,
        cwd: "/tmp".to_owned(),
        project_id: homie_proto::ProjectId::new("p"),
        worktree_path: None,
        git_branch: None,
        title: "fixture".to_owned(),
        title_source: homie_proto::TitleSource::Placeholder,
        agent_session_id: None,
        transcript_path: None,
        status: SessionStatus::Idle,
        needs_input: None,
        resumability: homie_proto::Resumability::NotResumable,
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
use crate::sidebar::{PreviewScenario, SidebarPreviewFixture};
use gpui::{Entity, Modifiers, TestAppContext};
use homie_proto::DateMillis;

struct InspectorHarness {
    inspector: Entity<WorkbenchInspector>,
}

impl Render for InspectorHarness {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .w(px(300.0))
            .h_full()
            .overflow_hidden()
            .child(self.inspector.clone())
    }
}

#[test]
fn inspector_tabs_have_stable_spatial_order() {
    assert!(InspectorTab::Info.index() < InspectorTab::Changes.index());
    assert!(InspectorTab::Changes.index() < InspectorTab::Code.index());
    assert!(InspectorTab::Code.index() < InspectorTab::Artifacts.index());
}

#[test]
fn background_git_refresh_keeps_the_last_settled_surface() {
    assert!(!should_show_blocking_git_loading(
        false,
        &LoadState::Error("not a git repository".to_owned())
    ));
    assert!(!should_show_blocking_git_loading(
        false,
        &LoadState::Ready(Arc::new(DiffSnapshot::default()))
    ));
    assert!(should_show_blocking_git_loading(
        true,
        &LoadState::Error("old project".to_owned())
    ));
}

/// The Info tab renders the Git summary, so it must be refreshed when it
/// becomes visible and whenever the selected session changes — but it must
/// never install the periodic diff poll, which stays exclusive to Changes.
#[gpui::test]
fn info_refreshes_on_context_change_without_a_periodic_poll(cx: &mut TestAppContext) {
    let runtime = Arc::new(StoreRuntime::inert());
    let fixture = SidebarPreviewFixture::make(PreviewScenario::Typical);
    let ids: Vec<SessionId> = fixture
        .list
        .sessions
        .iter()
        .map(|session| session.id.clone())
        .collect();
    assert!(ids.len() >= 2, "fixture must offer two sessions to switch");
    {
        let mut store = runtime.store.write().expect("session store lock poisoned");
        store.hydrate(fixture.list);
        store.select(ids[0].clone());
    }
    let tokio = Arc::new(
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("test runtime"),
    );
    let inspector_runtime = Arc::clone(&runtime);
    let (harness, cx) = cx.add_window_view(move |_window, cx| {
        let inspector = cx.new(|cx| WorkbenchInspector::new(inspector_runtime, tokio, cx));
        InspectorHarness { inspector }
    });
    let inspector = harness.read_with(cx, |harness, _| harness.inspector.clone());

    // Shipping defaults: the inspector opens visible on Info.
    assert_eq!(
        inspector.read_with(cx, |inspector, _| inspector.selected_tab),
        InspectorTab::Info
    );
    inspector.update(cx, |inspector, cx| inspector.set_visible(true, cx));

    let (generation, context, polling) = inspector.read_with(cx, |inspector, _| {
        (
            inspector.generation,
            inspector.context.clone(),
            inspector.poll_task.is_some(),
        )
    });
    assert!(
        generation > 0,
        "becoming visible on Info must read Git once"
    );
    assert_eq!(context.map(|context| context.id), Some(ids[0].clone()));
    assert!(!polling, "Info must not install a periodic diff poll");

    {
        let mut store = runtime.store.write().expect("session store lock poisoned");
        store.select(ids[1].clone());
    }
    inspector.update(cx, |inspector, cx| inspector.refresh_if_context_changed(cx));

    let (next_generation, next_context, still_polling) = inspector.read_with(cx, |i, _| {
        (i.generation, i.context.clone(), i.poll_task.is_some())
    });
    assert!(
        next_generation > generation,
        "a session change on Info must refresh instead of stranding stale counts"
    );
    assert_eq!(next_context.map(|context| context.id), Some(ids[1].clone()));
    assert!(!still_polling, "Info must still hold no periodic poll");

    // Contrast: Changes owns the timer, and leaving it disposes of it.
    inspector.update(cx, |inspector, cx| {
        inspector.select_tab(InspectorTab::Changes, cx);
    });
    assert!(inspector.read_with(cx, |inspector, _| inspector.poll_task.is_some()));
    inspector.update(cx, |inspector, cx| {
        inspector.select_tab(InspectorTab::Info, cx);
    });
    assert!(inspector.read_with(cx, |inspector, _| inspector.poll_task.is_none()));
    inspector.update(cx, |inspector, cx| inspector.set_visible(false, cx));
    cx.run_until_parked();
}

#[test]
fn artifact_titles_extract_the_useful_destination() {
    let pull_request = SessionArtifact {
        kind: ArtifactKind::PullRequest,
        url: "https://github.com/acme/homie/pull/42".to_owned(),
        first_seen_at: DateMillis(0.0),
    };
    let issue = SessionArtifact {
        kind: ArtifactKind::LinearIssue,
        url: "https://linear.app/acme/issue/DIR-19/polish-inspector".to_owned(),
        first_seen_at: DateMillis(0.0),
    };
    let preview = SessionArtifact {
        kind: ArtifactKind::Preview,
        url: "https://feature-homie.vercel.app/build".to_owned(),
        first_seen_at: DateMillis(0.0),
    };

    assert_eq!(artifact_title(&pull_request), "PR #42");
    assert_eq!(artifact_title(&issue), "DIR-19");
    assert_eq!(artifact_title(&preview), "feature-homie.vercel.app");
}

#[test]
fn generic_link_artifacts_are_hidden_from_the_inspector_count() {
    let mut session = inspector_fixture_session();
    session.artifacts = Some(vec![
        SessionArtifact {
            kind: ArtifactKind::Link,
            url: "https://github.com".to_owned(),
            first_seen_at: DateMillis(0.0),
        },
        SessionArtifact {
            kind: ArtifactKind::Unknown,
            url: "https://chatgpt.com".to_owned(),
            first_seen_at: DateMillis(0.0),
        },
        SessionArtifact {
            kind: ArtifactKind::Preview,
            url: "https://preview.example.com".to_owned(),
            first_seen_at: DateMillis(0.0),
        },
    ]);

    assert_eq!(artifact_count(&session), 1);
}

#[test]
fn merge_gate_waits_for_checks_and_review_blockers() {
    let fixture = SidebarPreviewFixture::make(PreviewScenario::Artifacts);
    let pull_request = fixture.list.sessions[0].pull_requests.as_ref().unwrap()[0].clone();
    assert!(!pull_request_can_merge(&pull_request));
    assert_eq!(
        merge_blocker_label(&pull_request),
        "Checks are still running"
    );

    let mut ready = pull_request;
    ready.checks_pending = 0;
    ready.checks_passed = 3;
    for check in ready.checks.as_mut().unwrap() {
        check.result = "pass".to_owned();
    }
    assert!(pull_request_can_merge(&ready));
}

#[gpui::test]
fn tabs_fit_and_switch_at_the_minimum_inspector_width(cx: &mut TestAppContext) {
    let runtime = Arc::new(StoreRuntime::inert());
    let mut fixture = SidebarPreviewFixture::make(PreviewScenario::Artifacts);
    let selected = fixture.selected_session_id.clone();
    if let Some(session) = fixture
        .list
        .sessions
        .iter_mut()
        .find(|session| Some(&session.id) == selected.as_ref())
    {
        session.artifacts = Some(vec![SessionArtifact {
            kind: ArtifactKind::Preview,
            url: "https://preview.example.com".to_owned(),
            first_seen_at: DateMillis(0.0),
        }]);
        session.listening_ports = Some(vec![homie_proto::PortInfo {
            port: 3000,
            process_name: "node".to_owned(),
        }]);
    }
    {
        let mut store = runtime.store.write().expect("session store lock poisoned");
        store.hydrate(fixture.list);
        if let Some(selected) = selected {
            store.select(selected);
        }
    }
    let tokio = Arc::new(
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("test runtime"),
    );
    let inspector_runtime = Arc::clone(&runtime);
    let (harness, cx) = cx.add_window_view(move |_window, cx| {
        let inspector = cx.new(|cx| {
            let mut inspector = WorkbenchInspector::new(inspector_runtime, tokio, cx);
            inspector.state = LoadState::Ready(Arc::new(DiffSnapshot {
                files: 88,
                additions: 556,
                deletions: 19,
                ..DiffSnapshot::default()
            }));
            inspector
        });
        InspectorHarness { inspector }
    });
    cx.run_until_parked();

    let info = cx.debug_bounds("INSPECTOR_TAB_INFO").expect("Info tab");
    let changes = cx
        .debug_bounds("INSPECTOR_TAB_CHANGES")
        .expect("Changes tab");
    let code = cx.debug_bounds("INSPECTOR_TAB_CODE").expect("Code tab");
    let artifacts = cx
        .debug_bounds("INSPECTOR_TAB_ARTIFACTS")
        .expect("Artifacts tab");
    let close = cx.debug_bounds("INSPECTOR_CLOSE").expect("close button");

    assert!(info.right() <= changes.left());
    assert!(changes.right() <= code.left());
    assert!(code.right() <= artifacts.left());
    assert!(artifacts.right() <= close.left());
    assert!(close.right() <= px(300.0));

    cx.simulate_click(changes.center(), Modifiers::none());
    let inspector = harness.read_with(cx, |harness, _| harness.inspector.clone());
    assert_eq!(
        inspector.read_with(cx, |inspector, _| inspector.selected_tab),
        InspectorTab::Changes
    );
    cx.run_until_parked();

    let working = cx
        .debug_bounds("INSPECTOR_LAYER_WORKING")
        .expect("working-tree layer");
    assert_eq!(
        inspector.read_with(cx, |inspector, _| inspector.diff_layer),
        DiffLayer::Branch
    );
    cx.simulate_click(working.center(), Modifiers::none());
    cx.run_until_parked();
    assert_eq!(
        inspector.read_with(cx, |inspector, _| inspector.diff_layer),
        DiffLayer::Working
    );

    cx.simulate_click(artifacts.center(), Modifiers::none());
    assert_eq!(
        inspector.read_with(cx, |inspector, _| inspector.selected_tab),
        InspectorTab::Artifacts
    );
    assert_eq!(
        runtime
            .store
            .read()
            .expect("session store lock poisoned")
            .preferences()
            .inspector_tab,
        InspectorTab::Artifacts
    );

    cx.run_until_parked();
    assert!(cx.debug_bounds("INSPECTOR_PR_MERGE").is_some());
    assert!(cx.debug_bounds("INSPECTOR_PR_CHECK_0").is_some());
    assert!(cx.debug_bounds("INSPECTOR_PR_COMMENT_0").is_some());
    let ask = cx.debug_bounds("INSPECTOR_PR_ASK").expect("PR ask action");
    cx.simulate_click(ask.center(), Modifiers::none());
    cx.run_until_parked();
    assert!(cx.debug_bounds("INSPECTOR_ASK_COMPOSER").is_some());
    assert!(cx.debug_bounds("INSPECTOR_ASK_SEND").is_some());
}

#[test]
fn ordinary_remote_git_absence_is_rendered_as_compatibility_state() {
    assert!(git_is_not_a_repository(
        "internal: fatal: not a git repository (or any parent)"
    ));
    assert!(git_is_not_installed(
        "internal: git is not installed on this host"
    ));
    assert!(!git_is_not_a_repository("ssh connection timed out"));
}
