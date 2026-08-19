use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};

use gpui::{ScrollDelta, ScrollWheelEvent, StyleRefinement, TestAppContext, size};
use homie_proto::{
    AgentKind as ProtoAgentKind, DateMillis, Project, ProjectId, Resumability, SessionListResult,
    TitleSource,
};

struct OverviewHarness {
    surfaces: Entity<SessionSurfaces>,
    background_scrolls: Arc<AtomicUsize>,
}

impl Render for OverviewHarness {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let background_scrolls = Arc::clone(&self.background_scrolls);
        div()
            .size_full()
            .child(div().absolute().inset_0().on_scroll_wheel(move |_, _, _| {
                background_scrolls.fetch_add(1, Ordering::Relaxed);
            }))
            .child(
                self.surfaces
                    .clone()
                    .cached(StyleRefinement::default().absolute().inset_0()),
            )
    }
}

fn session(index: usize) -> SessionRecord {
    SessionRecord {
        id: SessionId::new(format!("running-{index:02}")),
        kind: ProtoAgentKind::CODEX,
        cwd: "/work/overview".into(),
        project_id: ProjectId::new("overview"),
        worktree_path: None,
        git_branch: Some(format!("feature/session-{index:02}")),
        title: format!("Overflowing session {index:02}"),
        title_source: TitleSource::AgentProvided,
        agent_session_id: None,
        transcript_path: None,
        status: SessionStatus::Working,
        needs_input: None,
        resumability: Resumability::Live,
        parent: None,
        created_at: DateMillis(index as f64),
        updated_at: DateMillis(index as f64),
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

#[gpui::test]
fn overflowing_overview_lane_scrolls_without_reaching_the_background(cx: &mut TestAppContext) {
    let runtime = Arc::new(StoreRuntime::inert());
    runtime
        .store
        .write()
        .expect("session store lock poisoned")
        .hydrate(SessionListResult {
            sessions: (0..18).map(session).collect(),
            projects: vec![Project {
                id: ProjectId::new("overview"),
                root: "/work/overview".into(),
                name: "Overview".into(),
                pinned_order: None,
            }],
        });
    runtime
        .store
        .write()
        .expect("session store lock poisoned")
        .toggle_overview();

    let background_scrolls = Arc::new(AtomicUsize::new(0));
    let background_probe = Arc::clone(&background_scrolls);
    let (view, cx) = cx.add_window_view(move |_window, cx| OverviewHarness {
        surfaces: cx.new(|cx| SessionSurfaces::new(runtime, cx)),
        background_scrolls: background_probe,
    });
    cx.simulate_resize(size(px(1100.0), px(700.0)));

    let surfaces = view.read_with(cx, |harness, _| harness.surfaces.clone());
    let lane_bounds = cx
        .debug_bounds("OVERVIEW_LANE_RUNNING")
        .expect("running lane should render");
    let board_bounds = cx
        .debug_bounds("OVERVIEW_BOARD")
        .expect("overview board should render");
    let content_bounds = cx
        .debug_bounds("OVERVIEW_CONTENT")
        .expect("overview content should render");
    assert_eq!(
        content_bounds.size,
        size(px(1100.0), px(700.0)),
        "the opaque overview surface must cover the full cached viewport"
    );
    assert!(
        lane_bounds.size.height > px(300.0),
        "the lane viewport must receive the available window height"
    );
    let max_offset = surfaces.read_with(cx, |surfaces, _| {
        surfaces
            .overview_lane_scrolls
            .get(&OverviewLane::Running)
            .expect("running scroll handle")
            .max_offset()
    });
    assert!(
        max_offset.y > px(0.0),
        "overflowing lane must have a bounded, scrollable viewport"
    );
    assert!(
        surfaces
            .read_with(cx, |surfaces, _| surfaces
                .overview_board_scroll
                .max_offset())
            .x
            > px(0.0),
        "the five-lane board should expose horizontal overflow"
    );

    cx.simulate_event(ScrollWheelEvent {
        position: lane_bounds.center(),
        delta: ScrollDelta::Pixels(point(px(0.0), px(-80.0))),
        ..ScrollWheelEvent::default()
    });

    let offset = surfaces.read_with(cx, |surfaces, _| {
        surfaces
            .overview_lane_scrolls
            .get(&OverviewLane::Running)
            .expect("running scroll handle")
            .offset()
    });
    assert!(
        offset.y < px(0.0),
        "wheel input should move the overview lane (content: {content_bounds:?}, board: {board_bounds:?}, lane: {lane_bounds:?}, offset: {offset:?}, max: {max_offset:?}, background events: {})",
        background_scrolls.load(Ordering::Relaxed),
    );
    assert_eq!(
        surfaces
            .read_with(cx, |surfaces, _| surfaces.overview_board_scroll.offset())
            .x,
        px(0.0),
        "vertical lane scrolling must not shift the board sideways"
    );

    cx.simulate_event(ScrollWheelEvent {
        position: lane_bounds.center(),
        delta: ScrollDelta::Pixels(point(px(-80.0), px(0.0))),
        ..ScrollWheelEvent::default()
    });
    assert!(
        surfaces
            .read_with(cx, |surfaces, _| surfaces.overview_board_scroll.offset())
            .x
            < px(0.0),
        "horizontal trackpad input should move the lane board"
    );
    assert_eq!(
        background_scrolls.load(Ordering::Relaxed),
        0,
        "overview wheel input must not leak to the terminal behind it"
    );
}
