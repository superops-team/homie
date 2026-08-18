use std::collections::HashSet;
use std::sync::Arc;
use std::time::{Duration, Instant};

use gpui::{
    AnyElement, App, Context, CursorStyle, DragMoveEvent, Entity, FocusHandle, Focusable,
    FontWeight, KeyDownEvent, KeyUpEvent, ModifiersChangedEvent, MouseButton, Render,
    StyleRefinement, Subscription, Task, Window, actions, deferred, div, prelude::*, px, rgba,
};
use homie_proto::SessionId;
use homie_ui::{FloatingSurface, Radius, SemanticColors, Typo};

use crate::AppServices;
use crate::inspector::{InspectorEvent, WorkbenchInspector};
use crate::launcher::{LauncherEvent, LauncherOverlay};
use crate::macos::sf_symbols::{SymbolWeight, sf_symbol, sf_symbol_weighted};
use crate::navigation::{
    NavigationEvent, NavigationOverlay, ToggleCommandPalette, ToggleQuickOpen,
};
use crate::notifications::{InAppBanner, NotificationSound};
use crate::seam::{SeamSlide, toggle_has_settled};
use crate::session_surfaces::SessionSurfaces;
use crate::sidebar::{PreviewScenario, Sidebar, SidebarEvent};
use crate::sounds::{self, AfplayPlayer, SoundGate, StatusSound};
use crate::store::{DefaultAgent, SpawnOptions};
use crate::surface_shell::UtilitySurfaces;
use crate::terminal_pane::{TerminalPane, TerminalPaneEvent, TerminalViewport};
use crate::updates::UpdatePhase;
use crate::workbench::WorkbenchLayout;

const WINDOW_BOUNDS_SAVE_DELAY: Duration = Duration::from_millis(150);

pub(crate) fn cached_window_overlay<T: Render>(view: Entity<T>) -> impl IntoElement {
    view.cached(StyleRefinement::default().absolute().inset_0())
}

#[cfg(target_os = "macos")]
use crate::macos::{menu_bar::NativeMenuBar, notifier::NativeNotifier};

mod auxiliary;
mod seams;
mod shortcuts;
mod view;

use seams::{DraggedInspectorEdge, DraggedSidebarEdge, DraggedTerminalEdge, advance_seam};
use shortcuts::{NewSessionShortcut, new_session_shortcut, session_navigation_delta};

actions!(homie, [CloseSession, ReopenSession, OpenLauncher]);

pub struct RootView {
    pub(crate) sidebar: Entity<Sidebar>,
    pub(crate) terminal: Option<Entity<TerminalPane>>,
    pub(crate) navigation: Option<Entity<NavigationOverlay>>,
    pub(crate) session_surfaces: Option<Entity<SessionSurfaces>>,
    pub(crate) utility_surfaces: Option<Entity<UtilitySurfaces>>,
    pub(crate) launcher: Entity<LauncherOverlay>,
    pub(crate) inspector: Option<Entity<WorkbenchInspector>>,
    pub(crate) services: Arc<AppServices>,
    pub(crate) focus: FocusHandle,
    pub(crate) resize_origin: Option<(f32, f32)>,
    /// The sidebar open/close currently being painted, if any.
    pub(crate) sidebar_slide: Option<SeamSlide>,
    /// The sidebar seam width painted on the last frame. A new slide starts
    /// from this rather than from the settled width so it picks up wherever the
    /// previous frame left the panel.
    pub(crate) sidebar_seam: f32,
    pub(crate) auxiliary_terminal: Option<Entity<TerminalPane>>,
    pub(crate) auxiliary_id: Option<SessionId>,
    pub(crate) auxiliary_parent: Option<SessionId>,
    pub(crate) auxiliary_spawn_parent: Option<SessionId>,
    pub(crate) collapsed_auxiliary_parents: HashSet<SessionId>,
    pub(crate) workbench_layout: WorkbenchLayout,
    pub(crate) terminal_resize_origin: Option<(f32, f32)>,
    pub(crate) terminal_available_height: f32,
    pub(crate) inspector_open: bool,
    pub(crate) inspector_width: f32,
    pub(crate) inspector_max_width: f32,
    pub(crate) inspector_resize_origin: Option<(f32, f32)>,
    /// The inspector's mirror of `sidebar_slide` / `sidebar_seam`.
    pub(crate) inspector_slide: Option<SeamSlide>,
    pub(crate) inspector_seam: f32,
    /// When the inspector last opened or closed, so a held ⌘⇧D cannot outrun
    /// its slide. The sidebar's equivalent lives on the sidebar itself, which
    /// owns its own visibility; the inspector's lives here because RootView is
    /// what owns that flag.
    pub(crate) inspector_toggled_at: Option<Instant>,
    /// Debounces move/resize persistence while retaining the newest placement
    /// in memory immediately (the quit hook flushes that value synchronously).
    pub(crate) window_bounds_save: Option<Task<()>>,
    pub(crate) status_banner: Option<InAppBanner>,
    pub(crate) status_banner_generation: u64,
    pub(crate) sound_gate: SoundGate,
    pub(crate) preview: bool,
    pub(crate) preview_scenario: PreviewScenario,
    #[cfg(target_os = "macos")]
    pub(crate) menu_bar: Option<NativeMenuBar>,
    #[cfg(target_os = "macos")]
    pub(crate) notifier: NativeNotifier,
    pub(crate) _subscriptions: Vec<Subscription>,
    pub(crate) _service_events: Task<()>,
    pub(crate) _surface_sync: Option<Task<()>>,
    pub(crate) _workbench_sync: Task<()>,
}

impl Focusable for RootView {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl RootView {
    pub(crate) fn new(
        services: Arc<AppServices>,
        preview: bool,
        preview_scenario: PreviewScenario,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let sidebar_runtime = (!preview).then(|| Arc::clone(&services.store));
        let sidebar = cx.new(|cx| Sidebar::new(sidebar_runtime, preview, preview_scenario, cx));
        let terminal = (!preview).then(|| {
            let runtime = Arc::clone(&services.store);
            let tokio = Arc::clone(&services.tokio);
            cx.new(|cx| TerminalPane::new(runtime, tokio, window, cx))
        });
        let navigation = (!preview).then(|| {
            let runtime = Arc::clone(&services.store);
            cx.new(|cx| NavigationOverlay::new(runtime, window, cx))
        });
        let session_surfaces = (!preview).then(|| {
            let runtime = Arc::clone(&services.store);
            cx.new(|cx| SessionSurfaces::new(runtime, cx))
        });
        let utility_surfaces = (!preview).then(|| {
            let runtime = Arc::clone(&services.store);
            let tokio = Arc::clone(&services.tokio);
            let updates = services.updates.clone();
            cx.new(|cx| UtilitySurfaces::new(runtime, tokio, updates, window, cx))
        });
        let launcher = cx.new(|cx| LauncherOverlay::new(Arc::clone(&services), preview, cx));
        let inspector = (!preview || preview_scenario == PreviewScenario::Artifacts).then(|| {
            let runtime = Arc::clone(&services.store);
            let tokio = Arc::clone(&services.tokio);
            cx.new(|cx| WorkbenchInspector::new(runtime, tokio, cx))
        });
        if let (Some(terminal), Some(navigation), Some(utility_surfaces)) =
            (&terminal, &navigation, &utility_surfaces)
        {
            let navigation = navigation.clone();
            let utility_surfaces = utility_surfaces.clone();
            terminal.update(cx, |terminal, _| {
                terminal.set_shell_entities(navigation, utility_surfaces);
            });
        }
        if let Some(terminal) = &terminal {
            let terminal = terminal.clone();
            cx.defer_in(window, move |_, window, cx| {
                terminal.update(cx, |terminal, cx| terminal.focus(window, cx));
            });
        }
        let mut child_subscriptions = Vec::new();
        if let Some(terminal) = &terminal {
            child_subscriptions.push(cx.subscribe(terminal, |this, _, event, cx| match event {
                TerminalPaneEvent::ToggleSidebar => {
                    this.sidebar.update(cx, |sidebar, cx| sidebar.toggle(cx));
                }
                TerminalPaneEvent::ToggleInspector => this.toggle_inspector(cx),
                TerminalPaneEvent::OpenFileReference { reference, cwd, .. } => {
                    let inspector = this.inspector.clone();
                    this.reveal_inspector(cx);
                    if let Some(inspector) = inspector {
                        inspector.update(cx, |inspector, cx| {
                            inspector.open_file_reference(cwd.clone(), reference.clone(), cx);
                        });
                    }
                }
            }));
        }
        child_subscriptions.push(cx.subscribe_in(
            &sidebar,
            window,
            |this, _, event, window, cx| {
                if matches!(event, SidebarEvent::SessionActivated)
                    && let Some(terminal) = &this.terminal
                {
                    terminal.update(cx, |terminal, cx| terminal.focus(window, cx));
                    this.sync_auxiliary_terminal(window, cx);
                }
                if let SidebarEvent::Update(command) = event {
                    this.services.updates.send(command.clone());
                }
                if matches!(event, SidebarEvent::OpenSettings)
                    && let Some(surfaces) = &this.utility_surfaces
                {
                    surfaces.update(cx, |surfaces, cx| surfaces.open_settings(cx));
                }
                if matches!(event, SidebarEvent::AddRemoteHost)
                    && let Some(surfaces) = &this.utility_surfaces
                {
                    surfaces.update(cx, |surfaces, cx| {
                        surfaces.open_add_remote_host(window, cx);
                    });
                }
                if matches!(event, SidebarEvent::VisibilityChanged) {
                    this.begin_sidebar_slide(cx);
                }
                cx.notify();
            },
        ));
        child_subscriptions.push(cx.subscribe_in(
            &launcher,
            window,
            |this, _, _: &LauncherEvent, window, cx| {
                if let Some(terminal) = &this.terminal {
                    terminal.update(cx, |terminal, cx| terminal.focus(window, cx));
                } else {
                    window.focus(&this.focus, cx);
                }
                // The launcher is a main-pane destination, so closing it must
                // make RootView swap the terminal branch back into the row.
                cx.notify();
            },
        ));
        if let Some(navigation) = &navigation {
            child_subscriptions.push(cx.subscribe(navigation, |this, _, event, cx| match event {
                NavigationEvent::ToggleSidebar => {
                    this.sidebar.update(cx, |sidebar, cx| sidebar.toggle(cx));
                }
                NavigationEvent::OpenOverview => {
                    if let Some(surfaces) = &this.session_surfaces {
                        surfaces.update(cx, |surfaces, cx| surfaces.open_overview(cx));
                    }
                }
                NavigationEvent::OpenWorktrees => {
                    if let Some(surfaces) = &this.utility_surfaces {
                        surfaces.update(cx, |surfaces, cx| surfaces.open_worktrees(cx));
                    }
                }
                NavigationEvent::OpenSettings => {
                    if let Some(surfaces) = &this.utility_surfaces {
                        surfaces.update(cx, |surfaces, cx| surfaces.open_settings(cx));
                    }
                }
                NavigationEvent::CheckForUpdates => {
                    this.services.updates.check(true);
                }
            }));
        }
        if let Some(inspector) = &inspector {
            child_subscriptions.push(cx.subscribe(inspector, |this, _, event, cx| {
                if matches!(event, InspectorEvent::Close) {
                    this.set_inspector_open(false, cx);
                }
            }));
        }

        let mut status_events = services.store.status_events();
        let mut snapshots = services.store.snapshots();
        let mut usage = services.usage_tx.subscribe();
        let mut updates = services.updates.subscribe();
        sidebar.update(cx, |sidebar, cx| sidebar.set_usage(*usage.borrow(), cx));
        // Seed the current state: `watch` only wakes on changes, and an
        // unsupported build settles before this view exists.
        let initial_update = services.updates.state();
        sidebar.update(cx, |sidebar, cx| sidebar.set_update(initial_update, cx));

        #[cfg(target_os = "macos")]
        let mut menu_bar = objc2_foundation::MainThreadMarker::new()
            .and_then(|mtm| NativeMenuBar::new(mtm, Arc::clone(&services.store.store)));
        #[cfg(target_os = "macos")]
        if let Some(menu_bar) = &mut menu_bar {
            menu_bar.update(&snapshots.borrow());
        }
        #[cfg(target_os = "macos")]
        let notifier = NativeNotifier::new(services.store.notification_action_sender());

        let activation_services = Arc::clone(&services);
        let activation = cx.observe_window_activation(window, move |_this, window, _cx| {
            activation_services
                .store
                .store
                .write()
                .expect("session store lock poisoned")
                .set_active(window.is_window_active());
        });
        let bounds_observer = (!preview).then(|| {
            cx.observe_window_bounds(window, |this, window, cx| {
                this.window_bounds_changed(window, cx);
            })
        });

        let service_sidebar = sidebar.clone();
        let service_events = cx.spawn(async move |this, cx| {
            loop {
                tokio::select! {
                    status = status_events.recv() => {
                        let Ok(status) = status else { break };
                        let _ = this.update(cx, |this, cx| {
                            #[cfg(target_os = "macos")]
                            let app_is_active = this
                                .services
                                .store
                                .store
                                .read()
                                .expect("session store lock poisoned")
                                .app_is_active();
                            if let Some(sound) = status.sound {
                                let sound = match sound {
                                    NotificationSound::NeedsInput => StatusSound::NeedsInput,
                                    NotificationSound::Done => StatusSound::Done,
                                    NotificationSound::Frozen => StatusSound::Frozen,
                                };
                                if this.sound_gate.should_play(sound, Instant::now()) {
                                    let _ = sounds::play(&AfplayPlayer, sound);
                                }
                            }
                            #[cfg(target_os = "macos")]
                            if let Some(notification) = &status.notification
                                && (!app_is_active || status.in_app_banner.is_none())
                            {
                                this.notifier.post(notification);
                            }
                            if let Some(banner) = status.in_app_banner {
                                this.status_banner_generation =
                                    this.status_banner_generation.wrapping_add(1);
                                let generation = this.status_banner_generation;
                                this.status_banner = Some(banner);
                                cx.notify();
                                cx.spawn(async move |this, cx| {
                                    cx.background_executor()
                                        .timer(Duration::from_secs(7))
                                        .await;
                                    let _ = this.update(cx, |this, cx| {
                                        if this.status_banner_generation == generation {
                                            this.status_banner = None;
                                            cx.notify();
                                        }
                                    });
                                })
                                .detach();
                            }
                        });
                    }
                    changed = snapshots.changed() => {
                        if changed.is_err() { break; }
                        let snapshot = snapshots.borrow_and_update().clone();
                        let _ = this.update(cx, |this, _cx| {
                            #[cfg(target_os = "macos")]
                            if let Some(menu_bar) = &mut this.menu_bar {
                                menu_bar.update(&snapshot);
                            }
                        });
                    }
                    changed = usage.changed() => {
                        if changed.is_err() { break; }
                        let snapshot = *usage.borrow_and_update();
                        service_sidebar.update(cx, |sidebar, cx| {
                            sidebar.set_usage(snapshot, cx);
                        });
                    }
                    changed = updates.changed() => {
                        if changed.is_err() { break; }
                        let state = updates.borrow_and_update().clone();
                        let installing = state.phase == UpdatePhase::Installing;
                        service_sidebar.update(cx, |sidebar, cx| {
                            sidebar.set_update(state, cx);
                        });
                        // The swap helper is already polling for this process
                        // to exit; quitting is what lets the install proceed.
                        if installing {
                            cx.update(|cx| cx.quit());
                        }
                    }
                }
            }
        });
        let surface_sync =
            terminal
                .as_ref()
                .zip(session_surfaces.as_ref())
                .map(|(terminal, surfaces)| {
                    let terminal = terminal.clone();
                    let surfaces = surfaces.clone();
                    let mut changes = services.store.changes();
                    cx.spawn(async move |_this, cx| {
                        loop {
                            match changes.recv().await {
                                Ok(())
                                | Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                                    let buffers = terminal
                                        .update(cx, |terminal, _| terminal.resident_buffers());
                                    surfaces.update(cx, |surfaces, _| {
                                        surfaces.sync_resident_buffers(buffers);
                                    });
                                }
                                Err(tokio::sync::broadcast::error::RecvError::Closed) => return,
                            }
                        }
                    })
                });
        let mut workbench_changes = services.store.changes();
        let workbench_sync = cx.spawn_in(window, async move |this, cx| {
            loop {
                match workbench_changes.recv().await {
                    Ok(()) | Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                        if this
                            .update_in(cx, |this, window, cx| {
                                this.sync_auxiliary_terminal(window, cx);
                            })
                            .is_err()
                        {
                            return;
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => return,
                }
            }
        });
        let (workbench_layout, inspector_open, inspector_width) = {
            let store = services
                .store
                .store
                .read()
                .expect("session store lock poisoned");
            let prefs = store.preferences();
            (
                WorkbenchLayout::from_fraction(prefs.workbench_primary_fraction),
                prefs.inspector_open,
                prefs.inspector_width,
            )
        };
        if inspector_open && let Some(inspector) = &inspector {
            inspector.update(cx, |inspector, cx| inspector.set_visible(true, cx));
        }
        // Seed both seams from the restored layout so the first frame paints
        // the settled panels instead of sliding them open at launch.
        let sidebar_seam = if sidebar.read(cx).is_visible() {
            sidebar.read(cx).width()
        } else {
            0.0
        };
        let inspector_seam = if inspector_open { inspector_width } else { 0.0 };
        let mut root = Self {
            sidebar,
            terminal,
            navigation,
            session_surfaces,
            utility_surfaces,
            launcher,
            inspector,
            services,
            focus: cx.focus_handle(),
            resize_origin: None,
            sidebar_slide: None,
            sidebar_seam,
            auxiliary_terminal: None,
            auxiliary_id: None,
            auxiliary_parent: None,
            auxiliary_spawn_parent: None,
            collapsed_auxiliary_parents: HashSet::new(),
            workbench_layout,
            terminal_resize_origin: None,
            terminal_available_height: 0.0,
            inspector_open,
            inspector_width,
            inspector_max_width: 720.0,
            inspector_slide: None,
            inspector_seam,
            inspector_toggled_at: None,
            inspector_resize_origin: None,
            window_bounds_save: None,
            status_banner: None,
            status_banner_generation: 0,
            sound_gate: SoundGate::default(),
            preview,
            preview_scenario,
            #[cfg(target_os = "macos")]
            menu_bar,
            #[cfg(target_os = "macos")]
            notifier,
            _subscriptions: std::iter::once(activation)
                .chain(bounds_observer)
                .chain(child_subscriptions)
                .collect(),
            _service_events: service_events,
            _surface_sync: surface_sync,
            _workbench_sync: workbench_sync,
        };
        root.sync_auxiliary_terminal(window, cx);
        if !preview {
            // Do not rely on AppKit emitting a move/resize after the observer
            // is installed: even an untouched first launch should become the
            // placement restored by the next launch.
            root.window_bounds_changed(window, cx);
        }
        root
    }

    fn window_bounds_changed(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let placement = crate::current_window_placement(window, cx);
        self.services
            .store
            .store
            .write()
            .expect("session store lock poisoned")
            .remember_window_placement(placement);

        if self.window_bounds_save.is_some() {
            return;
        }
        self.window_bounds_save = Some(cx.spawn_in(window, async move |this, cx| {
            cx.background_executor()
                .timer(WINDOW_BOUNDS_SAVE_DELAY)
                .await;
            let _ = this.update_in(cx, |this, _window, _cx| {
                this.window_bounds_save.take();
                if let Err(error) = this
                    .services
                    .store
                    .store
                    .write()
                    .expect("session store lock poisoned")
                    .persist_preferences()
                {
                    eprintln!("homie: could not remember window placement: {error}");
                }
            });
        }));
    }

    fn colors(&self) -> SemanticColors {
        let store = self
            .services
            .store
            .store
            .read()
            .expect("session store lock poisoned");
        crate::app_theme::colors(&store.preferences().terminal_theme)
    }

    fn on_key_down(&mut self, event: &KeyDownEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if self.launcher.read(cx).is_open() {
            let reopen = event.keystroke.modifiers.platform
                && event.keystroke.key == "n"
                && !event.keystroke.modifiers.shift;
            self.launcher.update(cx, |launcher, cx| {
                launcher.handle_key_down(event, _window, cx);
            });
            if !reopen {
                cx.stop_propagation();
            }
            return;
        }
        if let Some(surfaces) = &self.utility_surfaces
            && surfaces.read(cx).is_open()
        {
            let global_overlay_shortcut = event.keystroke.modifiers.platform
                && matches!(event.keystroke.key.as_str(), "h" | "," | "k" | "p");
            if !global_overlay_shortcut {
                surfaces.update(cx, |surfaces, cx| {
                    surfaces.key_down(event, _window, cx);
                });
                cx.stop_propagation();
                return;
            }
        }
        if let Some(surfaces) = &self.session_surfaces {
            surfaces.update(cx, |surfaces, cx| {
                surfaces.handle_key_down(event, _window, cx);
            });
        }
        if !event.keystroke.modifiers.platform {
            return;
        }
        match event.keystroke.key.as_str() {
            "k" => {
                if let Some(navigation) = &self.navigation {
                    navigation.update(cx, |navigation, cx| {
                        navigation.toggle_command_palette(&ToggleCommandPalette, _window, cx);
                    });
                }
            }
            "p" => {
                if let Some(navigation) = &self.navigation {
                    navigation.update(cx, |navigation, cx| {
                        navigation.toggle_quick_open(&ToggleQuickOpen, _window, cx);
                    });
                }
            }
            "h" if event.keystroke.modifiers.shift => {
                if let Some(surfaces) = &self.utility_surfaces {
                    surfaces.update(cx, |surfaces, cx| surfaces.toggle_history(cx));
                }
            }
            "," => {
                if let Some(surfaces) = &self.utility_surfaces {
                    surfaces.update(cx, |surfaces, cx| surfaces.open_settings(cx));
                }
            }
            "b" => self.sidebar.update(cx, |sidebar, cx| sidebar.toggle(cx)),
            "d" if event.keystroke.modifiers.shift => self.toggle_inspector(cx),
            key @ ("t" | "n") => match new_session_shortcut(key, event.keystroke.modifiers) {
                Some(NewSessionShortcut::Default) => {
                    if !self.spawn_default() {
                        return;
                    }
                }
                Some(NewSessionShortcut::Shell) => {
                    if !self.spawn(None) {
                        return;
                    }
                }
                Some(NewSessionShortcut::Codex) => {
                    if !self.spawn(Some(DefaultAgent::Codex)) {
                        return;
                    }
                }
                None => return,
            },
            // ⌥⌘W: worktrees overview. ⌘⇧W archives the selected session;
            // plain ⌘W is bound globally to CloseSession.
            "w" if event.keystroke.modifiers.alt => {
                if let Some(surfaces) = &self.utility_surfaces {
                    surfaces.update(cx, |surfaces, cx| surfaces.open_worktrees(cx));
                } else {
                    return;
                }
            }
            "w" if event.keystroke.modifiers.shift => {
                let archived = self
                    .sidebar
                    .update(cx, |sidebar, cx| sidebar.archive_selected(cx));
                if !archived {
                    return;
                }
            }
            "r" if !event.keystroke.modifiers.shift => {
                let renaming = self
                    .sidebar
                    .update(cx, |sidebar, cx| sidebar.rename_selected(_window, cx));
                if !renaming {
                    return;
                }
            }
            // ⇧⌘J retains the attention-navigation command that previously
            // occupied plain ⌘J.
            "j" if event.keystroke.modifiers.shift => {
                let selected = self
                    .sidebar
                    .update(cx, |sidebar, cx| sidebar.select_next_needing_input(cx));
                if !selected {
                    return;
                }
            }
            // ⌘J opens (or focuses) a terminal owned by the selected agent's
            // workbench, below the primary pane.
            "j" => {
                if !self.open_auxiliary_terminal(_window, cx) {
                    return;
                }
            }
            digit @ ("1" | "2" | "3" | "4" | "5" | "6" | "7" | "8") => {
                // ⌘1–⌘8 select the nth session, matching the sidebar's row
                // hints; selection also focuses the terminal via
                // SessionActivated.
                let index = (digit.as_bytes()[0] - b'1') as usize;
                let selected = self
                    .sidebar
                    .update(cx, |sidebar, cx| sidebar.select_shortcut(index, cx));
                if !selected {
                    return;
                }
            }
            // ⌘9 jumps to the last session, the browser convention.
            "9" => {
                let selected = self
                    .sidebar
                    .update(cx, |sidebar, cx| sidebar.select_last(cx));
                if !selected {
                    return;
                }
            }
            // ⌃⌘↑/⌃⌘↓ move the selected row within its project group.
            "up" | "down" if event.keystroke.modifiers.control && !self.arrow_surface_visible() => {
                let delta = if event.keystroke.key == "up" { -1 } else { 1 };
                let moved = self
                    .sidebar
                    .update(cx, |sidebar, cx| sidebar.reorder_selected(delta, cx));
                if !moved {
                    return;
                }
            }
            // The explicit session-navigation shortcut steps through sidebar
            // order, wrapping. The switcher and overview own arrows while open.
            key if session_navigation_delta(
                key,
                event.keystroke.modifiers,
                self.arrow_surface_visible(),
            )
            .is_some() =>
            {
                let delta = session_navigation_delta(
                    key,
                    event.keystroke.modifiers,
                    self.arrow_surface_visible(),
                )
                .expect("guard checked navigation shortcut");
                let selected = self
                    .sidebar
                    .update(cx, |sidebar, cx| sidebar.select_relative(delta, cx));
                if !selected {
                    return;
                }
            }
            _ => return,
        }
        cx.stop_propagation();
    }

    /// Spawns a shell (`None`) or a specific agent straight from a shortcut,
    /// bypassing the sidebar's picker. No-ops in preview, which has no daemon
    /// to spawn into. Reports whether the spawn was dispatched.
    fn spawn(&self, agent: Option<DefaultAgent>) -> bool {
        if self.preview {
            return false;
        }
        let mut store = self
            .services
            .store
            .store
            .write()
            .expect("session store lock poisoned");
        match agent {
            Some(agent) => {
                let host = store.default_spawn_host();
                store.spawn_kind(
                    agent.kind(),
                    SpawnOptions {
                        host,
                        ..SpawnOptions::default()
                    },
                );
            }
            None => store.spawn_shell(SpawnOptions::default()),
        }
        true
    }

    fn spawn_default(&self) -> bool {
        if self.preview {
            return false;
        }
        self.services
            .store
            .store
            .write()
            .expect("session store lock poisoned")
            .spawn_default(SpawnOptions::default());
        true
    }

    /// True while the ⌃Tab switcher or the overview is up: both drive their
    /// own arrow-key navigation, so ⌘↑/⌘↓ stays out of their way.
    fn arrow_surface_visible(&self) -> bool {
        let store = self
            .services
            .store
            .store
            .read()
            .expect("session store lock poisoned");
        store.switcher_state().is_visible() || store.overview_state().is_visible()
    }

    /// Cmd+W: close the selected session with the sidebar ✕ semantics.
    /// With no session selected the action propagates to the global
    /// handler in main.rs, which closes the window instead.
    fn close_selected_session(
        &mut self,
        _: &CloseSession,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self
            .auxiliary_terminal
            .as_ref()
            .is_some_and(|terminal| terminal.read(cx).is_focused(window))
            && let Some(id) = self.auxiliary_id.clone()
        {
            self.services
                .store
                .store
                .write()
                .expect("session store lock poisoned")
                .remove_sessions(vec![id]);
            if let Some(terminal) = &self.terminal {
                terminal.update(cx, |terminal, cx| terminal.focus(window, cx));
            }
            return;
        }
        let closed = self
            .sidebar
            .update(cx, |sidebar, cx| sidebar.close_selected_now(cx));
        if !closed {
            cx.propagate();
        }
    }

    /// Cmd+Shift+T: reopen the most recently closed session (daemon-backed,
    /// survives restarts).
    fn reopen_last_session(
        &mut self,
        _: &ReopenSession,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.sidebar
            .update(cx, |sidebar, cx| sidebar.reopen_last(cx));
    }

    fn open_launcher(&mut self, _: &OpenLauncher, window: &mut Window, cx: &mut Context<Self>) {
        self.launcher
            .update(cx, |launcher, cx| launcher.open(window, cx));
        // Opening changes which main-pane branch RootView renders.
        cx.notify();
        // The launcher was not mounted while the terminal branch was active.
        // Focus it on the next frame, after GPUI has installed its focus node.
        let launcher = self.launcher.clone();
        cx.defer_in(window, move |_, window, cx| {
            launcher.update(cx, |launcher, cx| launcher.focus(window, cx));
        });
    }

    fn on_key_up(&mut self, event: &KeyUpEvent, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(surfaces) = &self.session_surfaces {
            surfaces.update(cx, |surfaces, cx| {
                surfaces.handle_key_up(event, window, cx);
            });
        }
    }

    fn on_modifiers_changed(
        &mut self,
        event: &ModifiersChangedEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(surfaces) = &self.session_surfaces {
            surfaces.update(cx, |surfaces, cx| {
                surfaces.handle_modifiers_changed(event, window, cx);
            });
        }
    }

    /// The settled seam width: what the sidebar wrapper is worth once nothing
    /// is animating. This -- not the painted seam -- is what the terminal is
    /// told about, so the PTY hears one resize per toggle rather than one per
    /// animation frame.
    fn settled_sidebar_seam(&self, cx: &App) -> f32 {
        let sidebar = self.sidebar.read(cx);
        if sidebar.is_visible() {
            sidebar.width()
        } else {
            0.0
        }
    }

    /// Starts sliding the seam toward the visibility the sidebar just adopted.
    /// Reduced-motion users get the settled width immediately.
    fn begin_sidebar_slide(&mut self, cx: &mut Context<Self>) {
        let to = self.settled_sidebar_seam(cx);
        self.sidebar_slide = (!cx.reduce_motion())
            .then(|| SeamSlide::begin(self.sidebar_seam, to))
            .flatten();
        if self.sidebar_slide.is_none() {
            self.sidebar_seam = to;
        }
    }

    /// The inspector's settled seam. Like the sidebar's, this is what the
    /// terminal is told about, so a slide costs no PTY resizes.
    fn settled_inspector_seam(&self) -> f32 {
        if self.inspector_open {
            self.inspector_width.min(self.inspector_max_width)
        } else {
            0.0
        }
    }

    fn begin_inspector_slide(&mut self, cx: &mut Context<Self>) {
        let to = self.settled_inspector_seam();
        self.inspector_slide = (!cx.reduce_motion())
            .then(|| SeamSlide::begin(self.inspector_seam, to))
            .flatten();
        if self.inspector_slide.is_none() {
            self.inspector_seam = to;
        }
    }

    fn drag_resize(&mut self, pointer_x: f32, cx: &mut Context<Self>) {
        let Some((origin_x, base_width)) = self.resize_origin else {
            return;
        };
        let width = base_width + pointer_x - origin_x;
        self.sidebar
            .update(cx, |sidebar, cx| sidebar.set_width(width, cx));
    }

    fn drag_terminal_resize(&mut self, pointer_y: f32, cx: &mut Context<Self>) {
        let Some((origin_y, base_height)) = self.terminal_resize_origin else {
            return;
        };
        self.workbench_layout.resize_primary(
            base_height + pointer_y - origin_y,
            self.terminal_available_height,
        );
        cx.notify();
    }

    fn finish_terminal_resize(&mut self, cx: &mut Context<Self>) {
        if self.terminal_resize_origin.take().is_none() {
            return;
        }
        let fraction = self.workbench_layout.primary_fraction();
        if let Err(error) = self
            .services
            .store
            .store
            .write()
            .expect("session store lock poisoned")
            .update_preferences(|prefs| prefs.workbench_primary_fraction = fraction)
        {
            eprintln!("homie: could not remember workbench split: {error}");
        }
        cx.notify();
    }

    /// End of a resize drag: the live width only lived in the sidebar's UI
    /// state, so write it through to preferences now.
    fn finish_resize(&mut self, cx: &mut Context<Self>) {
        if self.resize_origin.take().is_some() {
            self.sidebar
                .update(cx, |sidebar, cx| sidebar.commit_width(cx));
        }
    }

    /// The single gate every inspector open and close passes through -- ⌘⇧D,
    /// the terminal chrome button, and the panel's own ✕ -- so the debounce
    /// only has to hold here.
    fn set_inspector_open(&mut self, open: bool, cx: &mut Context<Self>) {
        if self.preview || self.inspector_open == open {
            return;
        }
        let now = Instant::now();
        if !toggle_has_settled(self.inspector_toggled_at.map(|at| now.duration_since(at))) {
            return;
        }
        self.inspector_toggled_at = Some(now);
        self.inspector_open = open;
        if let Some(inspector) = &self.inspector {
            inspector.update(cx, |inspector, cx| inspector.set_visible(open, cx));
        }
        if let Err(error) = self
            .services
            .store
            .store
            .write()
            .expect("session store lock poisoned")
            .update_preferences(|prefs| prefs.inspector_open = open)
        {
            eprintln!("homie: could not remember inspector visibility: {error}");
        }
        self.begin_inspector_slide(cx);
        cx.notify();
    }

    fn toggle_inspector(&mut self, cx: &mut Context<Self>) {
        self.set_inspector_open(!self.inspector_open, cx);
    }

    /// Source navigation is an explicit destination, so it must not be lost
    /// behind the short debounce that protects repeated panel toggles.
    fn reveal_inspector(&mut self, cx: &mut Context<Self>) {
        self.inspector_toggled_at = None;
        self.set_inspector_open(true, cx);
    }

    fn drag_inspector_resize(&mut self, pointer_x: f32, cx: &mut Context<Self>) {
        let Some((origin_x, base_width)) = self.inspector_resize_origin else {
            return;
        };
        self.inspector_width = (base_width - pointer_x + origin_x).clamp(
            300.0_f32.min(self.inspector_max_width),
            self.inspector_max_width,
        );
        cx.notify();
    }

    fn finish_inspector_resize(&mut self, cx: &mut Context<Self>) {
        if self.inspector_resize_origin.take().is_none() {
            return;
        }
        let width = self.inspector_width;
        if let Err(error) = self
            .services
            .store
            .store
            .write()
            .expect("session store lock poisoned")
            .update_preferences(|prefs| prefs.inspector_width = width)
        {
            eprintln!("homie: could not remember inspector width: {error}");
        }
        cx.notify();
    }
}

impl Render for RootView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = self.colors();
        let launcher_open = self.launcher.read(cx).is_open();
        let sidebar_visible = self.sidebar.read(cx).is_visible();
        let sidebar_width = self.sidebar.read(cx).width();
        let window_width = f32::from(window.inner_window_bounds().get_bounds().size.width);
        let occupied_sidebar_width = if sidebar_visible { sidebar_width } else { 0.0 };
        self.inspector_max_width =
            (window_width - occupied_sidebar_width - 320.0).clamp(0.0, 720.0);
        // The inspector's own width, whether or not it is currently shown --
        // the panel keeps painting at full width while it slides away.
        let inspector_panel_width = self.inspector_width.min(self.inspector_max_width);
        let inspector_width = if self.inspector_open && !launcher_open {
            inspector_panel_width
        } else {
            0.0
        };
        let now = Instant::now();
        self.sidebar_seam =
            advance_seam(&mut self.sidebar_slide, occupied_sidebar_width, now, window);
        self.inspector_seam = advance_seam(&mut self.inspector_slide, inspector_width, now, window);
        let seam = self.sidebar_seam;
        let inspector_seam = self.inspector_seam;
        // Each panel keeps its full width and is pinned to the wrapper edge it
        // lives against -- the sidebar's right, the inspector's left -- so
        // narrowing a wrapper slides its panel out under the clip instead of
        // squeezing every row's contents down with it.
        let sidebar_wrapper = div()
            .relative()
            .flex_none()
            .h_full()
            .overflow_hidden()
            .w(px(seam))
            .when(seam > 0.0, |wrapper| {
                wrapper.child(
                    div()
                        .absolute()
                        .top(px(0.0))
                        .right(px(0.0))
                        .h_full()
                        .w(px(sidebar_width))
                        // A reactive boundary: the sidebar re-renders on its
                        // own notifies, not on the terminal's 60fps repaints.
                        .child(
                            self.sidebar
                                .clone()
                                .cached(StyleRefinement::default().size_full()),
                        ),
                )
            });

        let mut root = div()
            .id("root")
            .size_full()
            // Real SF Pro (registered from SFNS.ttf at startup) for every UI
            // surface; the terminal grid sets its own mono font.
            .font_family(crate::fonts::ui_family())
            .flex()
            // Match the opaque platform window so content behind homie never
            // participates in compositing. The sidebar keeps its own surface
            // treatment above this base.
            .bg(colors.background)
            .track_focus(&self.focus)
            .capture_key_down(cx.listener(Self::on_key_down))
            .capture_key_up(cx.listener(Self::on_key_up))
            .on_action(cx.listener(Self::close_selected_session))
            .on_action(cx.listener(Self::reopen_last_session))
            .on_action(cx.listener(Self::open_launcher))
            .on_modifiers_changed(cx.listener(Self::on_modifiers_changed))
            // Fires for every move once the seam drag starts, wherever the
            // pointer wanders -- unlike hover-gated move listeners.
            .on_drag_move(
                cx.listener(|this, event: &DragMoveEvent<DraggedSidebarEdge>, _, cx| {
                    this.drag_resize(f32::from(event.event.position.x), cx);
                }),
            )
            .on_drag_move(
                cx.listener(|this, event: &DragMoveEvent<DraggedTerminalEdge>, _, cx| {
                    this.drag_terminal_resize(f32::from(event.event.position.y), cx);
                }),
            )
            .on_drag_move(cx.listener(
                |this, event: &DragMoveEvent<DraggedInspectorEdge>, _, cx| {
                    this.drag_inspector_resize(f32::from(event.event.position.x), cx);
                },
            ))
            .child(sidebar_wrapper)
            .when(seam > 0.0, |root| root.child(self.resize_handle(cx)));
        if launcher_open {
            // Command-N behaves like an unsaved new tab: preserve the app
            // shell, but replace the live session pane instead of floating a
            // dialog above it or manufacturing another session/tab up front.
            root = root.child(
                div()
                    .relative()
                    .flex_1()
                    .h_full()
                    .min_w(px(0.0))
                    .overflow_hidden()
                    .child(
                        self.launcher
                            .clone()
                            .cached(StyleRefinement::default().size_full()),
                    ),
            );
        } else {
            root = root.child(self.terminal_card(
                sidebar_visible,
                seam,
                inspector_width,
                inspector_seam,
                window,
                cx,
            ));
        }
        if inspector_seam > 0.0 {
            root = root.child(self.inspector_resize_handle(cx));
            if let Some(inspector) = &self.inspector {
                root = root.child(
                    div()
                        .relative()
                        .flex_none()
                        .h_full()
                        .w(px(inspector_seam))
                        .overflow_hidden()
                        .border_l_1()
                        .border_color(colors.primary.alpha(0.08))
                        .child(
                            div()
                                .absolute()
                                .top(px(0.0))
                                .left(px(0.0))
                                .h_full()
                                .w(px(inspector_panel_width))
                                .child(
                                    inspector
                                        .clone()
                                        .cached(StyleRefinement::default().size_full()),
                                ),
                        ),
                );
            }
        }
        if self.resize_origin.is_some()
            || self.terminal_resize_origin.is_some()
            || self.inspector_resize_origin.is_some()
        {
            root = root.child(self.resize_shield(cx));
        }
        if let Some(confirmation) = self.close_confirmation(colors, cx) {
            root = root.child(confirmation);
        }
        // Overlay views are cached reactive boundaries too: each subscribes to
        // store changes itself, so the only thing these wrappers must do is
        // stay out of the root flex row (absolute, zero-size at rest).
        if let Some(surfaces) = &self.session_surfaces {
            root = root.child(cached_window_overlay(surfaces.clone()));
        }
        if let Some(surfaces) = &self.utility_surfaces {
            root = root.child(cached_window_overlay(surfaces.clone()));
        }
        if let Some(navigation) = &self.navigation {
            root = root.child(cached_window_overlay(navigation.clone()));
        }
        if let Some(status) = self.status_banner(colors, cx) {
            root = root.child(status);
        }
        root
    }
}

#[cfg(test)]
mod tests;
