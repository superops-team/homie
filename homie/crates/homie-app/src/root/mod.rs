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
mod input;
mod inspector;
mod layout;
mod new;
mod seams;
mod sessions;
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
