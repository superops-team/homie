use gpui::{Modifiers, TestAppContext};

use super::*;

struct SidebarPopoverHarness {
    sidebar: Entity<Sidebar>,
}

impl Render for SidebarPopoverHarness {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .child(div().h_full().w(px(248.0)).child(self.sidebar.clone()))
    }
}

#[test]
fn long_paths_keep_final_component() {
    let result = clamp_path("/Users/preview/Projects/a/very/long/path/settings-kit");
    assert!(result.ends_with("/settings-kit"));
    assert!(result.contains('…'));
}

#[test]
fn compact_duration_matches_usage_copy() {
    assert_eq!(compact_duration(8_040), "2h 14m");
    assert_eq!(compact_duration(540), "9m");
}

#[test]
fn title_overflow_threshold_accounts_for_sidebar_badges() {
    let plain =
        session_title_available_width(248.0, 0, false, false, false, None, false, false, false);
    let remote = session_title_available_width(
        248.0,
        0,
        false,
        false,
        false,
        Some("mini-b"),
        false,
        false,
        true,
    );
    assert!(plain > remote);
    // A nested row pays for every indent column it sits behind.
    let nested =
        session_title_available_width(248.0, 2, false, false, false, None, false, false, false);
    assert!(plain > nested);
    assert_eq!(
        session_title_available_width(
            200.0,
            1,
            true,
            true,
            true,
            Some("very-long-host"),
            true,
            true,
            true,
        ),
        36.0
    );
}

#[test]
fn agent_shortcuts_remain_visible_when_the_execution_host_changes() {
    assert_eq!(
        agent_picker_shortcut(
            &ProtoAgentKind::CLAUDE_CODE,
            &ProtoAgentKind::CLAUDE_CODE,
            ""
        ),
        "⌘T"
    );
    assert_eq!(
        agent_picker_shortcut(&ProtoAgentKind::CODEX, &ProtoAgentKind::CLAUDE_CODE, "⌘⇧N"),
        "⌘⇧N"
    );
    assert_eq!(
        agent_picker_shortcut(&ProtoAgentKind::SHELL, &ProtoAgentKind::CLAUDE_CODE, "⌥⌘T"),
        "⌥⌘T"
    );
}

#[test]
fn remote_directory_navigation_keeps_the_explicit_child_path() {
    assert_eq!(
        remote_picker_target(Some("/Users/remote/code/homie"), Some("~")),
        "/Users/remote/code/homie"
    );
}

#[test]
fn remote_default_directory_has_a_visible_final_component() {
    assert_eq!(remote_picker_target(None, Some("~/")), "~");
    assert_eq!(remote_picker_target(None, Some("/srv/app/")), "/srv/app");
    assert_eq!(remote_picker_target(None, Some("/")), "/");
}

#[test]
fn remote_new_agent_uses_the_selected_hosts_default_directory() {
    assert!(!should_resolve_active_repo(None, Some("forge"), None));
    assert!(!should_resolve_active_repo(
        None,
        Some("forge"),
        Some("studio")
    ));
    assert!(should_resolve_active_repo(None, None, Some("studio")));
    assert!(!should_resolve_active_repo(
        Some("/Users/me/code"),
        None,
        Some("studio")
    ));
}

#[test]
fn migrating_session_uses_an_immediate_working_status() {
    let fixture = SidebarPreviewFixture::make(PreviewScenario::Typical);
    let session = fixture.list.sessions.first().expect("preview session");

    assert_eq!(status_state(session, true), StatusState::Working);
}

/// A sidebar full of working Agents is homie's normal resting state, so a
/// repeating timer here is a permanent wake, not an occasional one. The
/// 10 Hz status ticker this replaces measured ~3% idle CPU and held
/// ~240 MB of GPU memory that an idle window returns within seconds of its
/// last frame. `homie-ui`'s `status_marks_never_sample_a_clock_while_rendering`
/// guards the other half: a glyph that needs repainting to look right.
#[test]
fn the_sidebar_owns_no_repeating_clock() {
    let source = include_str!("view.rs");
    let periodic_timer = ["background_executor()", ".timer("].concat();
    let frame_request = ["request_animation", "_frame("].concat();

    assert!(
        !source.contains(&periodic_timer),
        "the sidebar must stay event-driven; a status clock here never stops, because \
             sessions are usually working"
    );
    assert!(
        !source.contains(&frame_request),
        "the sidebar must not drive the compositor from a render pass"
    );
}

#[test]
fn status_glyph_lifecycle_follows_sidebar_projection() {
    let first = SessionId("first".into());
    let second = SessionId("second".into());
    let stale = SessionId("stale".into());
    let mut glyphs = HashMap::from([
        (first.clone(), ()),
        (second.clone(), ()),
        (stale.clone(), ()),
    ]);

    retain_live_glyphs(&mut glyphs, &[first.clone(), second.clone()]);

    assert_eq!(glyphs.len(), 2);
    assert!(glyphs.contains_key(&first));
    assert!(glyphs.contains_key(&second));
    assert!(!glyphs.contains_key(&stale));
}

fn shortcut_session(id: &str) -> Arc<SessionRecord> {
    Arc::new(SessionRecord {
        id: SessionId::new(id),
        kind: ProtoAgentKind::CLAUDE_CODE,
        cwd: "/tmp".into(),
        project_id: ProjectId::new("test"),
        worktree_path: None,
        git_branch: None,
        title: id.into(),
        title_source: homie_proto::TitleSource::Placeholder,
        agent_session_id: None,
        transcript_path: None,
        status: homie_proto::SessionStatus::Idle,
        needs_input: None,
        resumability: homie_proto::Resumability::Live,
        parent: None,
        created_at: homie_proto::DateMillis(0.0),
        updated_at: homie_proto::DateMillis(0.0),
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
    })
}

#[test]
fn shortcut_rank_maps_first_eight_and_last_session() {
    let sessions = (1..=10)
        .map(|index| shortcut_session(&format!("s{index}")))
        .collect::<Vec<_>>();

    let ranks = shortcut_ranks(&sessions);

    assert_eq!(ranks.get(&SessionId::new("s1")), Some(&1));
    assert_eq!(ranks.get(&SessionId::new("s8")), Some(&8));
    assert_eq!(ranks.get(&SessionId::new("s9")), None);
    assert_eq!(ranks.get(&SessionId::new("s10")), Some(&9));
}

#[gpui::test]
fn sidebar_popovers_dismiss_when_clicking_elsewhere_in_the_window(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(|_, cx| {
        let sidebar = cx.new(|cx| {
            let mut sidebar = Sidebar::new(None, true, PreviewScenario::Typical, cx);
            sidebar.ui.popover = Some(Popover::NewAgent {
                directory: None,
                host: None,
            });
            sidebar
        });
        SidebarPopoverHarness { sidebar }
    });

    cx.simulate_click(point(px(500.0), px(320.0)), Modifiers::default());

    let sidebar = view.read_with(cx, |harness, _| harness.sidebar.clone());
    assert_eq!(
        sidebar.read_with(cx, |sidebar, _| sidebar.ui.popover.clone()),
        None
    );
}

#[gpui::test]
fn account_popover_exposes_the_remote_host_shortcut(cx: &mut TestAppContext) {
    let (_view, cx) = cx.add_window_view(|_, cx| {
        let sidebar = cx.new(|cx| {
            let mut sidebar = Sidebar::new(None, true, PreviewScenario::Typical, cx);
            sidebar.ui.popover = Some(Popover::Account);
            sidebar
        });
        SidebarPopoverHarness { sidebar }
    });

    assert!(cx.debug_bounds("quick-add-remote-host").is_some());
}

#[gpui::test]
fn project_plus_opens_the_agent_kind_menu_in_that_project(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(|_, cx| {
        let sidebar = cx.new(|cx| Sidebar::new(None, true, PreviewScenario::Typical, cx));
        SidebarPopoverHarness { sidebar }
    });
    let project = cx
        .debug_bounds("PROJECT_preview-homie")
        .expect("project row");
    cx.simulate_mouse_move(project.center(), None, Modifiers::default());
    let plus = cx
        .debug_bounds("PROJECT_ADD_preview-homie")
        .expect("project add button");

    cx.simulate_click(plus.center(), Modifiers::default());

    let sidebar = view.read_with(cx, |harness, _| harness.sidebar.clone());
    assert_eq!(
        sidebar.read_with(cx, |sidebar, _| sidebar.ui.popover.clone()),
        Some(Popover::NewAgent {
            directory: Some("/Users/preview/Projects/homie".to_owned()),
            host: None,
        })
    );
    assert!(cx.debug_bounds("AGENT_OPTION_0").is_some());
    assert!(cx.debug_bounds("AGENT_OPTION_1").is_some());
}

#[gpui::test]
fn choosing_a_host_in_new_agent_makes_its_shortcuts_the_default(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(|_, cx| {
        let sidebar = cx.new(|cx| {
            let mut sidebar = Sidebar::new(None, true, PreviewScenario::Typical, cx);
            sidebar
                .store
                .write()
                .expect("session store lock poisoned")
                .set_hosts(vec![homie_proto::HostEntry {
                    id: "forge".into(),
                    name: Some("Forge".into()),
                    ssh: "you@forge".into(),
                    default_cwd: None,
                    node: None,
                }]);
            sidebar.ui.popover = Some(Popover::NewAgent {
                directory: None,
                host: None,
            });
            sidebar
        });
        SidebarPopoverHarness { sidebar }
    });
    let sidebar = view.read_with(cx, |harness, _| harness.sidebar.clone());
    let host = cx.debug_bounds("HOST_OPTION_1").expect("remote host row");

    cx.simulate_click(host.center(), Modifiers::default());

    assert_eq!(
        sidebar.read_with(cx, |sidebar, _| sidebar
            .store
            .read()
            .expect("session store lock poisoned")
            .default_spawn_host()),
        Some("forge".into())
    );
}

/// The picker persists the shortcut destination, so the same picker has to
/// be able to take it back: one click on the "This Mac" row must return
/// ⌘T / ⌥⌘T / the palette to local, with nothing else to undo.
#[gpui::test]
fn the_new_agent_picker_can_send_shortcuts_back_to_this_mac(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(|_, cx| {
        let sidebar = cx.new(|cx| {
            let mut sidebar = Sidebar::new(None, true, PreviewScenario::Typical, cx);
            {
                let mut store = sidebar.store.write().expect("session store lock poisoned");
                store.set_hosts(vec![homie_proto::HostEntry {
                    id: "forge".into(),
                    name: Some("Forge".into()),
                    ssh: "you@forge".into(),
                    default_cwd: None,
                    node: None,
                }]);
                // Start from the regressed state: shortcuts already point
                // at a remote host, as they would after an earlier click.
                store.set_default_spawn_host(Some("forge".into()));
            }
            sidebar.ui.popover = Some(Popover::NewAgent {
                directory: None,
                host: Some("forge".into()),
            });
            sidebar
        });
        SidebarPopoverHarness { sidebar }
    });
    let sidebar = view.read_with(cx, |harness, _| harness.sidebar.clone());
    assert_eq!(
        sidebar.read_with(cx, |sidebar, _| sidebar
            .store
            .read()
            .expect("session store lock poisoned")
            .default_spawn_host()),
        Some("forge".into())
    );

    let local = cx.debug_bounds("HOST_OPTION_0").expect("this-mac row");
    cx.simulate_click(local.center(), Modifiers::default());
    cx.run_until_parked();

    assert_eq!(
        sidebar.read_with(cx, |sidebar, _| sidebar
            .store
            .read()
            .expect("session store lock poisoned")
            .default_spawn_host()),
        None
    );
    assert_eq!(
        sidebar.read_with(cx, |sidebar, _| sidebar.ui.popover.clone()),
        Some(Popover::NewAgent {
            directory: None,
            host: None,
        })
    );
}
