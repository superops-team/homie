use super::view::setting_row;
use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};

use gpui::{
    Entity, Modifiers, MouseDownEvent, ScrollDelta, ScrollWheelEvent, StyleRefinement,
    TestAppContext, point, size,
};
use homie_proto::{AgentKind, DateMillis, WorktreeOverviewEntry};

/// RootView paints the utility surfaces through a cached wrapper. A cached
/// entity root is laid out independently of its content, so mount the
/// surfaces the way the app does -- not bare -- and a root that cannot size
/// itself fails here instead of on screen.
struct CachedOverlayHarness {
    surfaces: Entity<UtilitySurfaces>,
}

impl Render for CachedOverlayHarness {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().size_full().child(
            self.surfaces
                .clone()
                .cached(StyleRefinement::default().absolute().inset_0()),
        )
    }
}

struct SettingsModalHarness {
    surfaces: Entity<UtilitySurfaces>,
    background_events: Arc<AtomicUsize>,
}

impl Render for SettingsModalHarness {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let mouse_events = Arc::clone(&self.background_events);
        let scroll_events = Arc::clone(&self.background_events);
        div()
            .size_full()
            .child(
                div()
                    .absolute()
                    .inset_0()
                    .on_mouse_down(MouseButton::Left, move |_, _, _| {
                        mouse_events.fetch_add(1, Ordering::Relaxed);
                    })
                    .on_scroll_wheel(move |_, _, _| {
                        scroll_events.fetch_add(1, Ordering::Relaxed);
                    }),
            )
            .child(self.surfaces.clone())
    }
}

#[test]
fn completed_reinstall_expires_without_clearing_a_newer_operation() {
    let result = homie_proto::HostInitializeResult {
        helper_build_id: "test-build".into(),
        protocol: homie_proto::remote_pty::ProtocolVersion::CURRENT,
        persistence: homie_proto::remote_pty::PersistenceCapability::NativeDetach,
        cwd: "/Users/remote".into(),
        shell: "/bin/zsh".into(),
    };
    let mut state = Some(HostInitialization::Ready {
        id: "forge".into(),
        name: "Forge".into(),
        kind: HostPreparationKind::Reinstall,
        operation: 7,
        result: result.clone(),
    });

    expire_completed_reinstall(&mut state, "forge", 7);
    assert!(state.is_none());

    state = Some(HostInitialization::Ready {
        id: "forge".into(),
        name: "Forge".into(),
        kind: HostPreparationKind::Reinstall,
        operation: 8,
        result,
    });
    expire_completed_reinstall(&mut state, "forge", 7);
    assert!(state.is_some(), "a stale timer must preserve newer work");
}

fn utility_surfaces_for_unit_tests(cx: &mut TestAppContext) -> UtilitySurfaces {
    let runtime = Arc::new(StoreRuntime::inert());
    let tokio = Arc::new(
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("test runtime"),
    );
    let focus = cx.update(|cx| cx.focus_handle());
    UtilitySurfaces {
        focus,
        surface: Surface::None,
        history: Vec::new(),
        history_query: QueryEditor::default(),
        history_highlight: 0,
        history_loading: false,
        history_error: None,
        history_generation: 0,
        history_task: None,
        worktrees: WorktreesSheet::default(),
        worktrees_generation: 0,
        worktrees_task: None,
        settings_tab: SettingsTab::General,
        settings_menu: None,
        hosts_path: PathBuf::from("/tmp/homie-hosts.json"),
        hosts: Vec::new(),
        host_editor: None,
        host_initialization: None,
        host_initialization_generation: 0,
        host_field_bounds: std::array::from_fn(|_| Rc::new(Cell::new(None))),
        prefs: Prefs::default(),
        store: Arc::clone(&runtime.store),
        store_runtime: runtime,
        runtime: tokio,
        updates: crate::updates::inert(),
        activity: String::new(),
        _update_changes: Task::ready(()),
        _store_changes: Task::ready(()),
    }
}

fn history_entry(id: &str) -> HistoryEntry {
    HistoryEntry {
        id: id.to_owned(),
        kind: AgentKind::generic("test-agent"),
        cwd: "/tmp".to_owned(),
        title: Some(id.to_owned()),
        transcript_path: format!("/tmp/{id}.jsonl"),
        last_active_at: DateMillis(0.0),
        created_at: Some(DateMillis(0.0)),
        cwd_exists: true,
    }
}

fn worktree_entry(path: &str) -> WorktreeOverviewEntry {
    WorktreeOverviewEntry {
        path: path.to_owned(),
        branch: Some("feature".to_owned()),
        project_root: "/repo".to_owned(),
        session_id: None,
        session_status: None,
        dirty: false,
        merged: true,
        age_days: 7,
        stale_suggestion: true,
    }
}

#[gpui::test]
fn stale_history_result_does_not_overwrite_newer_state(cx: &mut TestAppContext) {
    let mut surfaces = utility_surfaces_for_unit_tests(cx);
    surfaces.surface = Surface::History;
    surfaces.history_generation = 2;
    surfaces.history_loading = true;
    surfaces.history_error = None;
    surfaces.history = vec![history_entry("newer")];

    assert!(!surfaces.finish_history_load(1, Ok(vec![history_entry("stale")])));

    assert!(surfaces.history_loading);
    assert_eq!(surfaces.history.len(), 1);
    assert_eq!(surfaces.history[0].id, "newer");
}

#[gpui::test]
fn stale_worktrees_result_does_not_overwrite_newer_state(cx: &mut TestAppContext) {
    let mut surfaces = utility_surfaces_for_unit_tests(cx);
    surfaces.surface = Surface::Worktrees;
    surfaces.worktrees_generation = 2;
    surfaces.worktrees.begin_refresh();
    surfaces.worktrees.entries = vec![worktree_entry("/repo/newer")];

    assert!(!surfaces.finish_worktrees_refresh(1, Ok(vec![worktree_entry("/repo/stale")])));

    assert!(surfaces.worktrees.loading);
    assert_eq!(surfaces.worktrees.entries.len(), 1);
    assert_eq!(surfaces.worktrees.entries[0].path, "/repo/newer");
}

#[gpui::test]
fn closed_utility_surface_ignores_late_results(cx: &mut TestAppContext) {
    let mut surfaces = utility_surfaces_for_unit_tests(cx);
    surfaces.surface = Surface::None;
    surfaces.history_generation = 1;
    surfaces.worktrees_generation = 1;

    assert!(!surfaces.finish_history_load(1, Ok(vec![history_entry("late")])));
    assert!(!surfaces.finish_worktrees_refresh(1, Ok(vec![worktree_entry("/repo/late")])));

    assert!(surfaces.history.is_empty());
    assert!(surfaces.worktrees.entries.is_empty());
}

struct CachedSettingsModalHarness {
    surfaces: Entity<UtilitySurfaces>,
}

struct SettingRowHarness;

impl Render for SettingRowHarness {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .debug_selector(|| "settings-row-root".into())
            .w(px(260.0))
            .child(setting_row(
                "Long setting",
                "averylongremotehostdestinationwithoutnaturalwhitespaceorlinebreaks.example.internal",
                div().w(px(92.0)).child("Control"),
                SemanticColors::dark(),
            ))
    }
}

#[gpui::test]
fn setting_copy_wraps_long_tokens_inside_the_component(cx: &mut TestAppContext) {
    let (_view, cx) = cx.add_window_view(|_, _| SettingRowHarness);
    let root = cx.debug_bounds("settings-row-root").expect("row root");
    let row = cx.debug_bounds("settings-row").expect("setting row");
    let detail = cx
        .debug_bounds("settings-row-detail")
        .expect("setting detail");

    assert!(detail.right() <= root.right());
    assert!(
        row.size.height > px(SETTINGS_ROW_HEIGHT),
        "a long token should wrap and grow the shared row"
    );
}

#[gpui::test]
fn focused_empty_host_field_does_not_render_the_placeholder_as_input(cx: &mut TestAppContext) {
    let runtime = Arc::new(StoreRuntime::inert());
    let tokio = Arc::new(
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("test runtime"),
    );
    let updates = crate::updates::inert();
    let (_view, cx) = cx.add_window_view(move |window, cx| {
        let surfaces = cx.new(|cx| {
            let mut surfaces = UtilitySurfaces::new(runtime, tokio, updates, window, cx);
            surfaces.open_settings(cx);
            surfaces.settings_tab = SettingsTab::Remote;
            surfaces.begin_adding_host(window, cx);
            surfaces
        });
        SettingsModalHarness {
            surfaces,
            background_events: Arc::new(AtomicUsize::new(0)),
        }
    });

    assert!(cx.debug_bounds("host-field-caret").is_some());
    assert!(
        cx.debug_bounds("HOST_FIELD_PLACEHOLDER_NAME").is_none(),
        "placeholder must not become editable-looking text beside the caret"
    );
}

#[gpui::test]
fn remote_setting_copy_stays_inside_the_scrollable_pane(cx: &mut TestAppContext) {
    let runtime = Arc::new(StoreRuntime::inert());
    let tokio = Arc::new(
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("test runtime"),
    );
    let updates = crate::updates::inert();
    let (_view, cx) = cx.add_window_view(move |window, cx| {
        let surfaces = cx.new(|cx| {
            let mut surfaces = UtilitySurfaces::new(runtime, tokio, updates, window, cx);
            surfaces.open_settings(cx);
            surfaces.settings_tab = SettingsTab::Remote;
            surfaces
        });
        SettingsModalHarness {
            surfaces,
            background_events: Arc::new(AtomicUsize::new(0)),
        }
    });

    let pane = cx.debug_bounds("settings-pane").expect("settings pane");
    let copy = cx
        .debug_bounds("SETTINGS_NOTE_COPY")
        .expect("remote privacy copy");
    assert!(
        copy.right() <= pane.right(),
        "remote copy escaped its pane: {copy:?} vs {pane:?}"
    );
}

#[gpui::test]
fn remote_initialization_is_visible_and_can_become_the_default_host(cx: &mut TestAppContext) {
    let runtime = Arc::new(StoreRuntime::inert());
    let store = Arc::clone(&runtime.store);
    let tokio = Arc::new(
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("test runtime"),
    );
    let updates = crate::updates::inert();
    let (_view, cx) = cx.add_window_view(move |window, cx| {
        let surfaces = cx.new(|cx| {
            let mut surfaces = UtilitySurfaces::new(runtime, tokio, updates, window, cx);
            let host = HostEntry {
                id: "forge".into(),
                name: Some("Forge".into()),
                ssh: "you@forge".into(),
                default_cwd: Some("~".into()),
                node: None,
            };
            surfaces.hosts = vec![host.clone()];
            surfaces
                .store
                .write()
                .expect("session store lock poisoned")
                .set_hosts(vec![host]);
            surfaces.host_initialization = Some(HostInitialization::Ready {
                id: "forge".into(),
                name: "Forge".into(),
                kind: HostPreparationKind::Initialize,
                operation: 1,
                result: homie_proto::HostInitializeResult {
                    helper_build_id: "test-build".into(),
                    protocol: homie_proto::remote_pty::ProtocolVersion::CURRENT,
                    persistence: homie_proto::remote_pty::PersistenceCapability::NativeDetach,
                    cwd: "/Users/remote".into(),
                    shell: "/bin/zsh".into(),
                },
            });
            surfaces.open_settings(cx);
            surfaces.settings_tab = SettingsTab::Remote;
            surfaces
        });
        SettingsModalHarness {
            surfaces,
            background_events: Arc::new(AtomicUsize::new(0)),
        }
    });

    assert!(cx.debug_bounds("HOST_INITIALIZATION").is_some());
    let action = cx
        .debug_bounds("HOST_INITIALIZATION_ACTION")
        .expect("default host action")
        .center();
    cx.simulate_click(action, Modifiers::default());
    assert_eq!(
        store
            .read()
            .expect("session store lock poisoned")
            .default_spawn_host()
            .as_deref(),
        Some("forge")
    );
}

#[gpui::test]
fn editing_a_saved_host_offers_remote_environment_reinstallation(cx: &mut TestAppContext) {
    let runtime = Arc::new(StoreRuntime::inert());
    let tokio = Arc::new(
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("test runtime"),
    );
    let updates = crate::updates::inert();
    let (_view, cx) = cx.add_window_view(move |window, cx| {
        let surfaces = cx.new(|cx| {
            let mut surfaces = UtilitySurfaces::new(runtime, tokio, updates, window, cx);
            surfaces.hosts = vec![HostEntry {
                id: "forge".into(),
                name: Some("Forge".into()),
                ssh: "you@forge".into(),
                default_cwd: Some("~".into()),
                node: None,
            }];
            surfaces.open_settings(cx);
            surfaces.settings_tab = SettingsTab::Remote;
            surfaces.begin_editing_host("forge", window, cx);
            surfaces
        });
        SettingsModalHarness {
            surfaces,
            background_events: Arc::new(AtomicUsize::new(0)),
        }
    });

    assert!(
        cx.debug_bounds("REINSTALL_REMOTE_ENVIRONMENT").is_some(),
        "the saved SSH host editor must expose the reinstall action"
    );
}

#[gpui::test]
fn clicking_a_host_field_places_the_caret_at_the_pointer(cx: &mut TestAppContext) {
    let runtime = Arc::new(StoreRuntime::inert());
    let tokio = Arc::new(
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("test runtime"),
    );
    let updates = crate::updates::inert();
    let (view, cx) = cx.add_window_view(move |window, cx| {
        let surfaces = cx.new(|cx| {
            let mut surfaces = UtilitySurfaces::new(runtime, tokio, updates, window, cx);
            surfaces.open_add_remote_host(window, cx);
            surfaces
                .host_editor
                .as_mut()
                .expect("host editor")
                .name
                .insert("abcdef");
            surfaces
        });
        SettingsModalHarness {
            surfaces,
            background_events: Arc::new(AtomicUsize::new(0)),
        }
    });
    let surfaces = view.read_with(cx, |harness, _| harness.surfaces.clone());
    let field = cx.debug_bounds("HOST_FIELD_NAME").expect("name field");

    cx.simulate_click(
        point(field.left() + px(11.0), field.center().y),
        Modifiers::default(),
    );
    assert_eq!(
        surfaces.read_with(cx, |surfaces, _| surfaces
            .host_editor
            .as_ref()
            .expect("host editor")
            .name
            .cursor()),
        0
    );

    cx.simulate_click(
        point(field.right() - px(11.0), field.center().y),
        Modifiers::default(),
    );
    assert_eq!(
        surfaces.read_with(cx, |surfaces, _| surfaces
            .host_editor
            .as_ref()
            .expect("host editor")
            .name
            .cursor()),
        "abcdef".len()
    );
}

#[gpui::test]
fn remote_host_shortcut_opens_the_add_form_directly(cx: &mut TestAppContext) {
    let runtime = Arc::new(StoreRuntime::inert());
    let tokio = Arc::new(
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("test runtime"),
    );
    let updates = crate::updates::inert();
    let (view, cx) = cx.add_window_view(move |window, cx| {
        let surfaces = cx.new(|cx| UtilitySurfaces::new(runtime, tokio, updates, window, cx));
        SettingsModalHarness {
            surfaces,
            background_events: Arc::new(AtomicUsize::new(0)),
        }
    });
    let surfaces = view.read_with(cx, |harness, _| harness.surfaces.clone());

    surfaces.update_in(cx, |surfaces, window, cx| {
        surfaces.open_add_remote_host(window, cx);
    });

    surfaces.read_with(cx, |surfaces, _| {
        assert_eq!(surfaces.surface, Surface::Settings);
        assert_eq!(surfaces.settings_tab, SettingsTab::Remote);
        assert!(surfaces.host_editor.is_some());
    });
}

impl Render for CachedSettingsModalHarness {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .debug_selector(|| "cached-settings-root".into())
            .size_full()
            .child(crate::root::cached_window_overlay(self.surfaces.clone()))
    }
}

#[gpui::test]
fn cached_settings_modal_stays_inside_window(cx: &mut TestAppContext) {
    let runtime = Arc::new(StoreRuntime::inert());
    let tokio = Arc::new(
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("test runtime"),
    );
    let updates = crate::updates::inert();
    let (_view, cx) = cx.add_window_view(move |window, cx| {
        let surfaces = cx.new(|cx| {
            let mut surfaces = UtilitySurfaces::new(runtime, tokio, updates, window, cx);
            surfaces.open_settings(cx);
            surfaces
        });
        CachedSettingsModalHarness { surfaces }
    });

    let root = cx
        .debug_bounds("cached-settings-root")
        .expect("settings root should render");
    let dialog = cx
        .debug_bounds("settings-dialog")
        .expect("settings dialog should render");

    assert_eq!(dialog.center(), root.center());
    assert!(dialog.top() >= root.top());
    assert!(dialog.bottom() <= root.bottom());
}

#[gpui::test]
fn settings_modal_blocks_background_selection_and_scroll(cx: &mut TestAppContext) {
    let runtime = Arc::new(StoreRuntime::inert());
    let tokio = Arc::new(
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("test runtime"),
    );
    let updates = crate::updates::inert();
    let background_events = Arc::new(AtomicUsize::new(0));
    let event_probe = Arc::clone(&background_events);
    let (view, cx) = cx.add_window_view(move |window, cx| {
        let surfaces = cx.new(|cx| {
            let mut surfaces = UtilitySurfaces::new(runtime, tokio, updates, window, cx);
            surfaces.open_settings(cx);
            surfaces
        });
        SettingsModalHarness {
            surfaces,
            background_events: event_probe,
        }
    });

    let surfaces = view.read_with(cx, |harness, _| harness.surfaces.clone());
    let outside_panel = point(px(8.0), px(320.0));
    cx.simulate_event(MouseDownEvent {
        position: outside_panel,
        modifiers: Modifiers::default(),
        button: MouseButton::Left,
        click_count: 1,
        first_mouse: false,
    });

    assert_eq!(background_events.load(Ordering::Relaxed), 0);
    assert_eq!(
        surfaces.read_with(cx, |surfaces, _| surfaces.surface),
        Surface::None
    );

    surfaces.update(cx, |surfaces, cx| surfaces.open_settings(cx));
    cx.simulate_event(ScrollWheelEvent {
        position: outside_panel,
        delta: ScrollDelta::Pixels(point(px(0.0), px(-40.0))),
        ..ScrollWheelEvent::default()
    });

    assert_eq!(background_events.load(Ordering::Relaxed), 0);
    assert_eq!(
        surfaces.read_with(cx, |surfaces, _| surfaces.surface),
        Surface::Settings
    );
}

#[gpui::test]
fn clicking_inside_settings_but_outside_a_dropdown_closes_only_the_dropdown(
    cx: &mut TestAppContext,
) {
    let runtime = Arc::new(StoreRuntime::inert());
    let tokio = Arc::new(
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("test runtime"),
    );
    let updates = crate::updates::inert();
    let background_events = Arc::new(AtomicUsize::new(0));
    let event_probe = Arc::clone(&background_events);
    let (view, cx) = cx.add_window_view(move |window, cx| {
        let surfaces = cx.new(|cx| {
            let mut surfaces = UtilitySurfaces::new(runtime, tokio, updates, window, cx);
            surfaces.open_settings(cx);
            surfaces.settings_menu = Some(SettingsMenu::DefaultAgent);
            surfaces
        });
        SettingsModalHarness {
            surfaces,
            background_events: event_probe,
        }
    });

    let dialog = cx
        .debug_bounds("settings-dialog")
        .expect("settings dialog should render");
    cx.simulate_click(
        point(dialog.center().x, dialog.top() + px(29.0)),
        Modifiers::default(),
    );

    let surfaces = view.read_with(cx, |harness, _| harness.surfaces.clone());
    assert_eq!(
        surfaces.read_with(cx, |surfaces, _| surfaces.settings_menu),
        None
    );
    assert_eq!(
        surfaces.read_with(cx, |surfaces, _| surfaces.surface),
        Surface::Settings
    );
    assert_eq!(background_events.load(Ordering::Relaxed), 0);
}

#[gpui::test]
fn settings_dialog_centers_in_the_window_through_the_cached_wrapper(cx: &mut TestAppContext) {
    let runtime = Arc::new(StoreRuntime::inert());
    let tokio = Arc::new(
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("test runtime"),
    );
    let updates = crate::updates::inert();
    let (_, cx) = cx.add_window_view(move |window, cx| {
        let surfaces = cx.new(|cx| {
            let mut surfaces = UtilitySurfaces::new(runtime, tokio, updates, window, cx);
            surfaces.open_settings(cx);
            surfaces
        });
        CachedOverlayHarness { surfaces }
    });

    let viewport = size(px(1200.0), px(800.0));
    cx.simulate_resize(viewport);

    // The backdrop must cover the window. A collapsed root leaves it
    // 1200x0, which dims nothing and blocks nothing.
    let backdrop = cx
        .debug_bounds("surface-backdrop")
        .expect("modal backdrop should render");
    assert_eq!(backdrop.size, viewport);

    let dialog = cx
        .debug_bounds("settings-dialog")
        .expect("settings dialog should render");
    assert_eq!(dialog.size.width, px(SETTINGS_WIDTH));
    assert_eq!(dialog.size.height, px(SETTINGS_HEIGHT));

    // The dialog is taller than a collapsed root, so a zero-height root
    // parks it half above the window instead of in the middle.
    let center = dialog.center();
    assert_eq!(center.x, viewport.width / 2.0);
    assert_eq!(center.y, viewport.height / 2.0);
    assert!(dialog.top() > px(0.0), "dialog hangs above the window top");
}

#[gpui::test]
fn settings_content_close_control_dismisses_the_surface(cx: &mut TestAppContext) {
    let runtime = Arc::new(StoreRuntime::inert());
    let tokio = Arc::new(
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("test runtime"),
    );
    let updates = crate::updates::inert();
    let (view, cx) = cx.add_window_view(move |window, cx| {
        let surfaces = cx.new(|cx| {
            let mut surfaces = UtilitySurfaces::new(runtime, tokio, updates, window, cx);
            surfaces.open_settings(cx);
            surfaces
        });
        SettingsModalHarness {
            surfaces,
            background_events: Arc::new(AtomicUsize::new(0)),
        }
    });

    let surfaces = view.read_with(cx, |harness, _| harness.surfaces.clone());
    let close = cx
        .debug_bounds("close-settings")
        .expect("settings close control should render");
    cx.simulate_click(close.center(), Modifiers::default());

    assert_eq!(
        surfaces.read_with(cx, |surfaces, _| surfaces.surface),
        Surface::None
    );
}
