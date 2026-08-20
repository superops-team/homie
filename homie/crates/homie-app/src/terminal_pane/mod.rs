//! Terminal pane composition.
//!
//! The daemon remains authoritative: this module only composes
//! `homie-client::SessionAttachment`, `homie-term`, and the T9 session store.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};

use gpui::{
    AnyElement, App, ClipboardItem, Context, Entity, EventEmitter, FocusHandle, KeyBinding,
    KeyDownEvent, KeyUpEvent, ModifiersChangedEvent, MouseButton, Render, ScrollDelta,
    ScrollWheelEvent, SharedString, StatefulInteractiveElement, Task, Window, actions, div, font,
    prelude::*, px, rgba,
};
use homie_client::attachment::TerminalChunk;
use homie_proto::grid::GridUpdate;
use homie_proto::{Resumability, SessionId, SessionRecord, SessionStatus};
use homie_term::buffer::GridBuffer;
use homie_term::element::{SharedGridBuffer, TerminalElement, TerminalReference};
use homie_term::find::{FindSnapshot, SearchRequest, TerminalFindModel};
use homie_term::keys::{Modifiers as TermModifiers, TermInputModes, encode_key, paste};
use homie_term::metrics::CellMetrics;
use homie_term::repaint::{RepaintAction, RepaintPacer};
use homie_term::scrollback::{WheelDelta, WheelEvent, WheelRoute};
use homie_term::theme::TermTheme;
use homie_ui::{Fill, FloatingSurface, Metrics, Radius, SemanticColors, StatusGlyph, Typo};
use tokio::runtime::Handle;
use tokio::sync::mpsc;

use crate::clipboard_transfer::StagedClipboardImage;
use crate::macos::sf_symbols::{SymbolWeight, sf_symbol, sf_symbol_weighted};
use crate::navigation::{NavigationOverlay, ToggleCommandPalette, ToggleQuickOpen, query_label};
use crate::query_editor::{self, ClipboardEdit, Edit, QueryEditor};
use crate::session_surfaces::switcher_key;
use crate::store::StoreRuntime;
use crate::surface_shell::UtilitySurfaces;

const GRID_HORIZONTAL_PADDING: f32 = 24.0;
const GRID_VERTICAL_PADDING: f32 = 12.0;
// The outer terminal card has a one-pixel border on both sides and the pane
// adds its own left divider. These pixels are outside TerminalElement's actual
// paint bounds and therefore cannot be offered to the PTY as a text column.
const GRID_LAYOUT_HORIZONTAL_CHROME: f32 = 3.0;
const GRID_LAYOUT_VERTICAL_CHROME: f32 = 2.0;
const REATTACH_DELAY: Duration = Duration::from_millis(500);
/// Burst ceiling for repaints (~60fps). The pacer paints the first frame of a
/// burst and the next response after interactive input immediately; this only
/// caps sustained output, and background panes never invalidate the window, so
/// idle budgets are unaffected. Matched to the daemon's `gridFlushInterval`.
const ACTIVE_REPAINT_INTERVAL: Duration = Duration::from_millis(16);
/// How often a live drag is allowed to push a new PTY geometry. Matched to the
/// daemon's coalesced grid flush (also 16ms): resizing faster produces frames
/// the client can never see, resizing slower makes the drag look like it snaps
/// at the end instead of reflowing under the cursor.
const RESIZE_CADENCE: Duration = Duration::from_millis(16);
/// Two resizes further apart than this belong to different gestures. A drag
/// steps faster than this and must keep reflowing live; anything slower is a
/// discrete change -- a panel toggle, a window snap, a font-size change --
/// whose reflow is held still by [`REFLOW_HOLD`]. Matched to the window the
/// daemon uses to infer the same thing (`AgentSession.resizeDragWindow`).
const RESIZE_GESTURE_GAP: Duration = Duration::from_millis(200);
/// Ceiling on how long the grid is held still across a column change.
///
/// A cols-only resize comes back in two stages: the daemon re-wraps its
/// emulator and broadcasts that immediately, then the program answers SIGWINCH
/// and repaints. Painting the first stage is what made a sidebar toggle shove
/// the content up and drop it back a frame later -- re-wrapping at a fixed row
/// count spills the top into scrollback, and the grid is painted top-anchored
/// on row index, so every surviving line moves up until the program's repaint
/// puts it back. Holding both stages and applying them as one paint removes
/// the intermediate frame entirely. The hold ends as soon as the program's
/// repaint lands, so this bound only applies to one that is slow or absent.
const REFLOW_HOLD: Duration = Duration::from_millis(140);
/// Slack added to a bottom-anchored grid's height so layout rounding can never
/// shave its last row off. See `TerminalPane::grid_row_overflow`.
const ANCHOR_SLACK: f32 = 1.0;
/// How many evicted sessions keep their last-known grid parked for instant
/// re-selection. Cells only (~100KB each) — elements, channels, and shape
/// caches are rebuilt on promotion — so the ceiling is a memory bound, not a
/// residency one.
const PARKED_GRID_CAP: usize = 12;
actions!(
    homie_terminal,
    [
        OpenFind,
        FindNext,
        FindPrevious,
        CloseFind,
        ZoomIn,
        ZoomOut,
        ResetZoom,
        Paste,
        CopySelection,
    ]
);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TerminalPaneEvent {
    ToggleSidebar,
    ToggleInspector,
    OpenFileReference {
        reference: String,
        cwd: String,
        session_id: SessionId,
    },
}

pub fn bind_terminal_keys(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("cmd-f", OpenFind, None),
        KeyBinding::new("cmd-g", FindNext, None),
        KeyBinding::new("cmd-shift-g", FindPrevious, None),
        KeyBinding::new("cmd-=", ZoomIn, None),
        KeyBinding::new("cmd-+", ZoomIn, None),
        KeyBinding::new("cmd--", ZoomOut, None),
        KeyBinding::new("cmd-0", ResetZoom, None),
        KeyBinding::new("cmd-v", Paste, None),
        KeyBinding::new("cmd-c", CopySelection, None),
    ]);
}

mod attachment;
mod chip;
mod clipboard;
mod events;
mod find;
mod geometry;
mod input;
mod keys;
mod policy;
mod projection;
mod scroll;
mod view;

use attachment::{AttachmentControl, AttachmentState, spawn_attachment};
pub use chip::{ChipTint, PaneChip};
use keys::terminal_key_event;
use policy::{
    ResizePlan, clipboard_image, estimated_grid_size, plan_resize, should_hold_reflow,
    terminal_damage_should_repaint,
};
use projection::{exit_description, status_state, ui_agent_kind};

enum PaneEvent {
    InteractiveInput,
    AttachmentState(SessionId, AttachmentState),
    Chunk(SessionId, TerminalChunk),
    FindSnapshot(SessionId, SearchRequest, FindSnapshot),
    ScrollbackCells(SessionId, homie_proto::ReadScrollbackCellsResult, usize),
    ScrollbackFailed(SessionId),
    ClipboardUploadFinished(SessionId, Result<String, String>),
}

struct ResidentTerminal {
    element: TerminalElement,
    attachment: AttachmentControl,
    attachment_state: AttachmentState,
    find: Option<TerminalFindModel>,
    /// The editable text behind `find`'s query, so ⌘F gets the same caret,
    /// selection, and readline keys as the other query fields.
    find_query: QueryEditor,
    last_size: (u16, u16),
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum SessionSource {
    FollowSelection,
    Fixed(SessionId),
}

/// Grid frames parked while a column change round-trips through the daemon,
/// so the re-wrap and the program's repaint reach the screen as one paint
/// rather than as a jump and a correction. See [`REFLOW_HOLD`].
struct ReflowHold {
    parked: Vec<GridUpdate>,
    /// The daemon's re-wrapped snapshot has landed, so the next frame after it
    /// is the program answering SIGWINCH and completes the pair.
    saw_snapshot: bool,
    /// The ceiling timer. Dropped with the hold, which cancels it.
    _release: Task<()>,
}

impl ReflowHold {
    /// Parks a frame, reporting whether the pair is now complete and the hold
    /// should be released.
    fn park(&mut self, update: GridUpdate) -> bool {
        let snapshot = update.is_full_snapshot;
        self.parked.push(update);
        if snapshot {
            // A later snapshot supersedes the first (a re-seed after
            // backpressure, or the daemon's own settle pass) rather than
            // standing in for the repaint we are waiting on.
            self.saw_snapshot = true;
            return false;
        }
        self.saw_snapshot
    }
}

/// Window-space allocation supplied by the workbench. Terminal input needs
/// the origin while PTY sizing needs the local width and height.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TerminalViewport {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Drop for ResidentTerminal {
    fn drop(&mut self) {
        self.attachment.close();
    }
}

pub struct TerminalPane {
    runtime: Arc<StoreRuntime>,
    _tokio_owner: Arc<tokio::runtime::Runtime>,
    tokio: Handle,
    residents: HashMap<SessionId, ResidentTerminal>,
    /// Last-known grids of recently evicted sessions, most recent last.
    /// Selecting a session paints its parked grid on the very first frame
    /// while the fresh attachment round-trips; the attach's full snapshot
    /// then overwrites the same buffer in place. This is what makes session
    /// switching read as instant with a residency of one.
    parked_grids: Vec<(SessionId, SharedGridBuffer)>,
    pane_tx: mpsc::UnboundedSender<PaneEvent>,
    focus: FocusHandle,
    glyphs: HashMap<SessionId, Entity<StatusGlyph>>,
    /// Paced PTY resizes: window and sidebar drags relayout every frame, but
    /// grid frames only leave the daemon every 50ms, so intermediate sizes are
    /// coalesced onto that cadence rather than dropped (see [`RESIZE_CADENCE`]).
    pending_resizes: HashMap<SessionId, (u16, u16)>,
    resize_flush: Option<Task<()>>,
    /// A cadence tick is already armed; further changes fold into it instead of
    /// rescheduling (which is what used to starve the flush during a drag).
    resize_flush_armed: bool,
    last_resize_sent: Option<Instant>,
    /// Grids held still while a column change round-trips. Keyed by session id
    /// so a hold follows the session rather than the pane: selection can move
    /// on mid-hold, and the parked frames still belong to the session that was
    /// resized.
    reflow_holds: HashMap<SessionId, ReflowHold>,
    started_at: Instant,
    repaint_pacer: RepaintPacer,
    session_source: SessionSource,
    /// Last selection observed by the primary pane. Spawn responses select the
    /// daemon-created id asynchronously, so this transition is also the
    /// reliable point at which keyboard focus can leave the picker.
    observed_selected_id: Option<SessionId>,
    viewport: Option<TerminalViewport>,
    sidebar_visible: bool,
    inspector_open: bool,
    navigation: Option<Entity<NavigationOverlay>>,
    utility_surfaces: Option<Entity<UtilitySurfaces>>,
    local_clipboard_images: Vec<StagedClipboardImage>,
    _pane_events: Task<()>,
    _store_changes: Task<()>,
}

impl EventEmitter<TerminalPaneEvent> for TerminalPane {}
impl TerminalPane {
    pub fn new(
        runtime: Arc<StoreRuntime>,
        tokio_owner: Arc<tokio::runtime::Runtime>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        Self::new_with_source(
            runtime,
            tokio_owner,
            SessionSource::FollowSelection,
            window,
            cx,
        )
    }

    pub fn new_fixed(
        runtime: Arc<StoreRuntime>,
        tokio_owner: Arc<tokio::runtime::Runtime>,
        session_id: SessionId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        Self::new_with_source(
            runtime,
            tokio_owner,
            SessionSource::Fixed(session_id),
            window,
            cx,
        )
    }

    fn new_with_source(
        runtime: Arc<StoreRuntime>,
        tokio_owner: Arc<tokio::runtime::Runtime>,
        session_source: SessionSource,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let focus = cx.focus_handle();
        if matches!(session_source, SessionSource::FollowSelection) {
            window.focus(&focus, cx);
        }
        let (pane_tx, mut pane_rx) = mpsc::unbounded_channel();
        let pane_events = cx.spawn_in(window, async move |this, cx| {
            let mut batch = Vec::new();
            while let Some(event) = pane_rx.recv().await {
                // Drain whatever else has queued and cross to the main thread
                // once per burst, not once per frame: with several attached
                // sessions streaming, per-event hops made the UI thread wake
                // at frame-rate × session-count.
                batch.push(event);
                while let Ok(next) = pane_rx.try_recv() {
                    batch.push(next);
                }
                if this
                    .update_in(cx, |this, window, cx| {
                        for event in batch.drain(..) {
                            this.handle_pane_event(event, window, cx);
                        }
                    })
                    .is_err()
                {
                    return;
                }
            }
        });

        let mut changes = runtime.changes();
        let store_changes = cx.spawn_in(window, async move |this, cx| {
            loop {
                match changes.recv().await {
                    Ok(()) | Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                        if this
                            .update_in(cx, |this, window, cx| {
                                this.reconcile_store_change(window, cx);
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

        let tokio = tokio_owner.handle().clone();
        let observed_selected_id = matches!(session_source, SessionSource::FollowSelection)
            .then(|| {
                runtime
                    .store
                    .read()
                    .expect("session store lock poisoned")
                    .selected_session_id()
                    .cloned()
            })
            .flatten();
        let mut pane = Self {
            runtime,
            _tokio_owner: tokio_owner,
            tokio,
            residents: HashMap::new(),
            parked_grids: Vec::new(),
            pane_tx,
            focus,
            glyphs: HashMap::new(),
            pending_resizes: HashMap::new(),
            resize_flush: None,
            resize_flush_armed: false,
            last_resize_sent: None,
            reflow_holds: HashMap::new(),
            started_at: Instant::now(),
            repaint_pacer: RepaintPacer::new(ACTIVE_REPAINT_INTERVAL),
            session_source,
            observed_selected_id,
            viewport: None,
            sidebar_visible: true,
            inspector_open: false,
            navigation: None,
            utility_surfaces: None,
            local_clipboard_images: Vec::new(),
            _pane_events: pane_events,
            _store_changes: store_changes,
        };
        pane.reconcile_residency();
        pane.sync_status_glyphs(pane.current_colors(), window, cx);
        pane
    }

    fn reconcile_residency(&mut self) {
        let store = self
            .runtime
            .store
            .read()
            .expect("session store lock poisoned");
        let resident_ids: HashSet<_> = match &self.session_source {
            SessionSource::FollowSelection => {
                store.terminal_residency().resident().cloned().collect()
            }
            SessionSource::Fixed(id) if store.sessions().contains_key(id) => {
                HashSet::from([id.clone()])
            }
            SessionSource::Fixed(_) => HashSet::new(),
        };
        // A parked grid for a session the store no longer lists is dead
        // weight; one for a session that just became resident is superseded
        // below by promotion.
        self.parked_grids
            .retain(|(id, _)| store.sessions().contains_key(id));
        drop(store);
        // Park the last-known grid of every session about to be evicted, so
        // re-selecting it paints instantly instead of flashing blank while
        // the fresh attachment round-trips.
        for (id, resident) in &self.residents {
            if resident_ids.contains(id) {
                continue;
            }
            self.parked_grids.retain(|(parked, _)| parked != id);
            self.parked_grids
                .push((id.clone(), resident.element.buffer()));
        }
        if self.parked_grids.len() > PARKED_GRID_CAP {
            let excess = self.parked_grids.len() - PARKED_GRID_CAP;
            self.parked_grids.drain(..excess);
        }
        self.residents.retain(|id, _| resident_ids.contains(id));
        // A hold outliving its resident would park frames belonging to a
        // session id that has been re-attached since, and paint them into a
        // grid that never asked for them.
        let residents = &self.residents;
        self.reflow_holds.retain(|id, _| residents.contains_key(id));

        let socket = self.runtime.client().socket_path().to_path_buf();
        for id in resident_ids {
            if self.residents.contains_key(&id) {
                continue;
            }
            let mut mono = font(crate::fonts::mono_family());
            mono.fallbacks = Some(gpui::FontFallbacks::from_fonts(vec![
                ".SF NS Mono".to_owned(),
                "Menlo".to_owned(),
                "Apple Symbols".to_owned(),
                "STIX Two Math".to_owned(),
                "Apple Color Emoji".to_owned(),
            ]));
            let parked = self
                .parked_grids
                .iter()
                .position(|(parked, _)| parked == &id)
                .map(|index| self.parked_grids.remove(index).1);
            let attachment = spawn_attachment(
                &self.tokio,
                socket.clone(),
                id.clone(),
                self.pane_tx.clone(),
            );
            let ime_attachment = attachment.clone();
            let element = match parked {
                // The parked cells paint on the first frame; the attach's
                // full snapshot overwrites the same shared buffer moments
                // later, so stale content lives for one round-trip at most.
                Some(buffer) => TerminalElement::new(buffer),
                None => TerminalElement::with_buffer(GridBuffer::default()),
            }
            .font(mono)
            .focus_handle(self.focus.clone())
            .on_text_input(move |text| ime_attachment.input(text.as_bytes().to_vec()));
            self.residents.insert(
                id,
                ResidentTerminal {
                    element,
                    attachment,
                    attachment_state: AttachmentState::Attaching,
                    find: None,
                    find_query: QueryEditor::default(),
                    last_size: (0, 0),
                },
            );
        }
    }

    fn reconcile_store_change(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let selected_id = matches!(self.session_source, SessionSource::FollowSelection)
            .then(|| {
                self.runtime
                    .store
                    .read()
                    .expect("session store lock poisoned")
                    .selected_session_id()
                    .cloned()
            })
            .flatten();
        let selection_changed = selected_id != self.observed_selected_id;
        self.observed_selected_id = selected_id.clone();

        self.reconcile_residency();
        self.sync_status_glyphs(self.current_colors(), window, cx);

        // Explicit sidebar clicks already focus through SessionActivated, but
        // successful spawns select their daemon-assigned id on the async store
        // path. Following the selection here covers both RPC/event orderings
        // and avoids trying to focus a terminal before its id exists.
        if selection_changed && selected_id.is_some() {
            window.focus(&self.focus, cx);
        }
        cx.notify();
    }

    pub fn resident_buffers(&mut self) -> HashMap<SessionId, SharedGridBuffer> {
        self.reconcile_residency();
        self.residents
            .iter()
            .map(|(id, resident)| (id.clone(), resident.element.buffer()))
            .collect()
    }

    pub fn set_shell_entities(
        &mut self,
        navigation: Entity<NavigationOverlay>,
        utility_surfaces: Entity<UtilitySurfaces>,
    ) {
        self.navigation = Some(navigation);
        self.utility_surfaces = Some(utility_surfaces);
    }

    pub fn focus(&self, window: &mut Window, cx: &mut Context<Self>) {
        window.focus(&self.focus, cx);
    }

    pub fn set_viewport(&mut self, viewport: TerminalViewport, cx: &mut Context<Self>) {
        if self.viewport == Some(viewport) {
            return;
        }
        self.viewport = Some(viewport);
        cx.notify();
    }

    pub fn set_shell_chrome(
        &mut self,
        sidebar_visible: bool,
        inspector_open: bool,
        cx: &mut Context<Self>,
    ) {
        if self.sidebar_visible == sidebar_visible && self.inspector_open == inspector_open {
            return;
        }
        self.sidebar_visible = sidebar_visible;
        self.inspector_open = inspector_open;
        cx.notify();
    }

    pub fn is_focused(&self, window: &Window) -> bool {
        self.focus.is_focused(window)
    }

    fn sync_status_glyphs(
        &mut self,
        colors: SemanticColors,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let fixed_id = match &self.session_source {
            SessionSource::Fixed(id) => Some(id),
            SessionSource::FollowSelection => None,
        };
        let snapshots: Vec<_> = {
            let store = self
                .runtime
                .store
                .read()
                .expect("session store lock poisoned");
            store
                .sessions()
                .iter()
                .filter(|(id, _)| fixed_id.is_none_or(|fixed| fixed == *id))
                .map(|(id, session)| {
                    (
                        id.clone(),
                        ui_agent_kind(session.effective_kind()),
                        status_state(session),
                    )
                })
                .collect()
        };
        self.glyphs
            .retain(|id, _| snapshots.iter().any(|(live, _, _)| live == id));
        for (id, kind, state) in snapshots {
            if let Some(glyph) = self.glyphs.get(&id) {
                glyph.update(cx, |glyph, cx| {
                    glyph.set_kind(kind, cx);
                    glyph.set_state(state, window, cx);
                    glyph.set_colors(colors, cx);
                });
            } else {
                let glyph = StatusGlyph::entity(kind, state, 16.0, colors, cx);
                self.glyphs.insert(id, glyph);
            }
        }
    }

    fn current_colors(&self) -> SemanticColors {
        let store = self
            .runtime
            .store
            .read()
            .expect("session store lock poisoned");
        crate::app_theme::colors(&store.preferences().terminal_theme)
    }

    fn selected_id(&self) -> Option<SessionId> {
        match &self.session_source {
            SessionSource::FollowSelection => self
                .runtime
                .store
                .read()
                .expect("session store lock poisoned")
                .selected_session_id()
                .cloned(),
            SessionSource::Fixed(id) => Some(id.clone()),
        }
    }

    fn selected_session(&self) -> Option<Arc<SessionRecord>> {
        let id = self.selected_id()?;
        self.runtime
            .store
            .read()
            .expect("session store lock poisoned")
            .sessions()
            .get(&id)
            .map(Arc::clone)
    }
}

#[cfg(test)]
mod tests;
