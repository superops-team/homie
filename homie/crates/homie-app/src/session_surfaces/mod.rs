//! GPUI rendering and event routing for T13 navigation surfaces.
//!
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::{Arc, RwLock};

use crate::macos::sf_symbols::{SymbolWeight, sf_symbol, sf_symbol_weighted};
use crate::store::{SessionStore, StoreRuntime};
use crate::switcher::{
    OverviewArrow, OverviewFilter, OverviewLane, OverviewMode, SwitcherKey, display_title,
};
use gpui::{
    AnyElement, BoxShadow, ClickEvent, Context, Entity, FocusHandle, FontWeight, KeyDownEvent,
    KeyUpEvent, ModifiersChangedEvent, MouseButton, Render, ScrollHandle, SharedString, Task,
    Window, div, point, prelude::*, px, rgba,
};
use homie_proto::{
    AgentKind as ProtoAgentKind, AttentionLevel, RiskHint, SessionId, SessionRecord, SessionStatus,
};
use homie_term::element::{SharedGridBuffer, TerminalElement};
use homie_ui::{
    AgentKind, AgentLogo, HairlineDivider, Ink, Palette, Radius, SemanticColors, StatusGlyph,
    StatusState,
};

mod overview;
mod overview_card;
mod projection;
mod switcher;
#[cfg(test)]
mod tests;

pub(crate) use projection::switcher_key;

use projection::{ui_agent_kind, ui_status_state};

pub struct SessionSurfaces {
    store: Arc<RwLock<SessionStore>>,
    focus_handle: FocusHandle,
    resident_previews: HashMap<SessionId, TerminalElement>,
    status_glyphs: HashMap<(SessionId, u16, homie_ui::AgentKind), Entity<StatusGlyph>>,
    overview_board_scroll: ScrollHandle,
    overview_lane_scrolls: HashMap<OverviewLane, ScrollHandle>,
    overview_list_scroll: ScrollHandle,
    /// This view is `.cached()` in RootView, so ambient window redraws no
    /// longer reach it: store changes must notify it directly.
    _store_changes: Task<()>,
}

impl SessionSurfaces {
    pub fn new(runtime: Arc<StoreRuntime>, cx: &mut Context<Self>) -> Self {
        let mut changes = runtime.changes();
        let store_changes = cx.spawn(async move |this, cx| {
            loop {
                match changes.recv().await {
                    Ok(()) | Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                        if this.update(cx, |_, cx| cx.notify()).is_err() {
                            return;
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => return,
                }
            }
        });
        Self {
            store: Arc::clone(&runtime.store),
            focus_handle: cx.focus_handle(),
            resident_previews: HashMap::new(),
            status_glyphs: HashMap::new(),
            overview_board_scroll: ScrollHandle::new(),
            overview_lane_scrolls: OverviewLane::ALL
                .into_iter()
                .map(|lane| (lane, ScrollHandle::new()))
                .collect(),
            overview_list_scroll: ScrollHandle::new(),
            _store_changes: store_changes,
        }
    }

    fn colors(&self) -> SemanticColors {
        let store = self.store.read().expect("session store lock poisoned");
        crate::app_theme::colors(&store.preferences().terminal_theme)
    }

    /// T11 supplies the same resident buffer used by the mounted terminal. A
    /// separate painter/cache renders it into switcher and overview thumbnails
    /// without reading back the onscreen Metal layer.
    pub(crate) fn set_resident_buffer(&mut self, id: SessionId, buffer: SharedGridBuffer) {
        self.resident_previews
            .insert(id, TerminalElement::new(buffer).focused(false));
    }

    pub(crate) fn remove_resident_buffer(&mut self, id: &SessionId) {
        self.resident_previews.remove(id);
    }

    pub(crate) fn sync_resident_buffers(&mut self, buffers: HashMap<SessionId, SharedGridBuffer>) {
        let stale: Vec<_> = self
            .resident_previews
            .keys()
            .filter(|id| !buffers.contains_key(*id))
            .cloned()
            .collect();
        for id in stale {
            self.remove_resident_buffer(&id);
        }
        for (id, buffer) in buffers {
            // Only rebuild when the underlying buffer actually changed: every
            // TerminalElement carries a fresh global element id, and GPUI
            // retains per-id render state, so unconditionally recreating
            // previews on each store event leaks textures without bound.
            let unchanged = self
                .resident_previews
                .get(&id)
                .is_some_and(|element| Arc::ptr_eq(&element.buffer(), &buffer));
            if !unchanged {
                self.set_resident_buffer(id, buffer);
            }
        }
    }

    pub(crate) fn open_overview(&mut self, cx: &mut Context<Self>) {
        let mut store = self.store.write().expect("session store lock poisoned");
        if !store.overview_state().is_visible() {
            store.toggle_overview();
        }
        cx.notify();
    }
}

impl Render for SessionSurfaces {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (overview_visible, switcher_visible) = {
            let store = self.store.read().expect("session store lock poisoned");
            (
                store.overview_state().is_visible(),
                store.switcher_state().is_visible(),
            )
        };
        if overview_visible || switcher_visible {
            let session_ids: HashSet<_> = {
                let store = self.store.read().expect("session store lock poisoned");
                store.sessions().keys().cloned().collect()
            };
            self.status_glyphs
                .retain(|(id, _, _), _| session_ids.contains(id));
        }
        let root = div()
            .id("session-surfaces")
            .absolute()
            // Cached entity roots are laid out independently. Insets alone do
            // not give this absolute root a definite size, which previously
            // collapsed the overview hitbox/background to its 42 pt top inset
            // while every child visibly overflowed into the window.
            .size_full()
            .track_focus(&self.focus_handle)
            .capture_key_down(cx.listener(Self::handle_key_down))
            .capture_key_up(cx.listener(Self::handle_key_up))
            .on_modifiers_changed(cx.listener(Self::handle_modifiers_changed));
        if overview_visible {
            root.inset_0().child(self.render_overview(window, cx))
        } else if switcher_visible {
            root.inset_0().child(self.render_switcher(window, cx))
        } else {
            root.size(px(0.0))
        }
    }
}

const SWITCHER_PREVIEW_WIDTH: f32 = 620.0;
const SWITCHER_PREVIEW_HEIGHT: f32 = 348.0;
const OVERVIEW_LANE_WIDTH: f32 = 272.0;

impl SessionSurfaces {
    pub(crate) fn handle_key_down(
        &mut self,
        event: &KeyDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let mut store = self.store.write().expect("session store lock poisoned");
        let key = switcher_key(event);
        let switcher_was_visible = store.switcher_state().is_visible();
        let switcher_handled =
            if switcher_was_visible || matches!(key, SwitcherKey::Tab { control: true, .. }) {
                store.handle_switcher_key(key)
            } else {
                false
            };
        if switcher_handled {
            if !switcher_was_visible && store.switcher_state().is_visible() {
                store.dismiss_overview();
            }
            cx.stop_propagation();
            cx.notify();
            return;
        }

        let modifiers = event.keystroke.modifiers;
        if event.keystroke.key == "o" && modifiers.platform && modifiers.shift {
            store.toggle_overview();
            cx.stop_propagation();
            cx.notify();
            return;
        }

        if !store.overview_state().is_visible() {
            if event.keystroke.key == "escape" && !store.sidebar_selection().is_empty() {
                // Match Swift: clear Finder-style sidebar gathering, but do not
                // swallow Esc because the focused terminal still needs it.
                store.clear_sidebar_selection();
                cx.notify();
            }
            return;
        }

        let handled = match event.keystroke.key.as_str() {
            "escape" => store.overview_escape(),
            "backspace" | "delete" => store.overview_backspace(),
            "left" => store.move_overview_focus(OverviewArrow::Left),
            "right" => store.move_overview_focus(OverviewArrow::Right),
            "up" => store.move_overview_focus(OverviewArrow::Up),
            "down" => store.move_overview_focus(OverviewArrow::Down),
            "enter" => store.activate_overview_focus(),
            "a" if modifiers.platform => {
                store.select_all_overview_sessions();
                true
            }
            "space" if !modifiers.platform && !modifiers.control => {
                store.append_overview_query(" ")
            }
            _ if !modifiers.platform && !modifiers.control => event
                .keystroke
                .key_char
                .as_deref()
                .is_some_and(|text| store.append_overview_query(text)),
            _ => false,
        };
        if handled {
            cx.stop_propagation();
            cx.notify();
        }
    }

    pub(crate) fn handle_key_up(
        &mut self,
        event: &KeyUpEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // macOS normally emits ModifiersChanged for this. KeyUp is a defensive
        // fallback for platforms/backends that report the released modifier as
        // a regular key.
        let mut store = self.store.write().expect("session store lock poisoned");
        if store.switcher_state().is_visible()
            && matches!(event.keystroke.key.as_str(), "control" | "ctrl")
        {
            store.handle_switcher_modifiers_changed(false);
            cx.notify();
        }
    }

    pub(crate) fn handle_modifiers_changed(
        &mut self,
        event: &ModifiersChangedEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let mut store = self.store.write().expect("session store lock poisoned");
        let was_visible = store.switcher_state().is_visible();
        store.handle_switcher_modifiers_changed(event.modifiers.control);
        if was_visible != store.switcher_state().is_visible() {
            cx.notify();
        }
    }

    fn render_grid_or_logo(
        &self,
        session: &SessionRecord,
        logo_size: f32,
        font_size: f32,
        colors: SemanticColors,
    ) -> AnyElement {
        if let Some(preview) = self.resident_previews.get(&session.id) {
            preview.clone().font_size(px(font_size)).into_any_element()
        } else {
            div()
                .flex()
                .size_full()
                .items_center()
                .justify_center()
                .bg(colors.background)
                .opacity(0.60)
                .child(
                    AgentLogo::new(ui_agent_kind(session.effective_kind()), logo_size, colors)
                        .badged(false),
                )
                .into_any_element()
        }
    }

    fn status_glyph(
        &mut self,
        session: &SessionRecord,
        size: f32,
        colors: SemanticColors,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Entity<StatusGlyph> {
        let state = ui_status_state(session);
        let kind = ui_agent_kind(session.effective_kind());
        let key = (session.id.clone(), (size * 10.0).round() as u16, kind);
        let glyph = self
            .status_glyphs
            .entry(key)
            .or_insert_with(|| StatusGlyph::entity(kind, state, size, colors, cx))
            .clone();
        glyph.update(cx, |glyph, cx| {
            glyph.set_state(state, window, cx);
            glyph.set_colors(colors, cx);
        });
        glyph
    }
}
