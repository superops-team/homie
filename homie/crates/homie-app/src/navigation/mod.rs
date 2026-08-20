use std::cmp::Ordering;
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::Instant;

use crate::fuzzy::{FuzzyMatcher, FuzzyQuery};
use crate::macos::sf_symbols::{SymbolWeight, sf_symbol, sf_symbol_weighted};
use crate::palette::{self, PaletteAction, PaletteCommand, Ranked};
use crate::query_editor::{self, ClipboardEdit, Edit, QueryEditor};
use crate::quick_open::{
    self, DirectoryIndex, QuickOpenItem, QuickOpenSnapshot, RANK_DEBOUNCE, RESULT_LIMIT,
    RankedFolder,
};
use crate::store::{SessionStore, SpawnOptions, StoreRuntime};
use gpui::{
    AnyElement, App, Context, EventEmitter, FocusHandle, Focusable, FontWeight, HighlightStyle,
    KeyDownEvent, MouseButton, Pixels, Render, ScrollHandle, SharedString,
    StatefulInteractiveElement, StyledText, Task, Window, actions, div, prelude::*, px, rgba,
};
use homie_proto::{AgentKind, AttentionLevel, SessionId, SessionRecord};
use homie_ui::{FloatingSurface, HairlineDivider, Palette, Radius, SemanticColors};

mod index;
mod render;
#[cfg(test)]
mod tests;

actions!(homie, [ToggleCommandPalette, ToggleQuickOpen]);

/// The search field above the results, and the gap the surface keeps from the
/// window edges. Everything else is measured against the live viewport so the
/// list grows into a tall window and never overflows a short one.
const SEARCH_HEIGHT: f32 = 46.0;
const ROW_HEIGHT: f32 = 32.0;
const QUICK_ROW_HEIGHT: f32 = 34.0;
/// Quick Open rows that show a parent path stack two lines.
const QUICK_ROW_HEIGHT_WITH_PATH: f32 = 44.0;
const SECTION_HEADER_HEIGHT: f32 = 24.0;
const LIST_PADDING_X: f32 = 8.0;
const LIST_PADDING_Y: f32 = 6.0;
const ROW_PADDING_X: f32 = 14.0;
const SURFACE_WIDTH: f32 = 580.0;
const MIN_LIST_HEIGHT: f32 = 96.0;
const MAX_LIST_HEIGHT: f32 = 640.0;
const MIN_TOP_INSET: f32 = 12.0;
const MAX_TOP_INSET: f32 = 96.0;
const BOTTOM_INSET: f32 = 24.0;

/// Where the overlay sits and how tall its list may grow in this window.
#[derive(Clone, Copy, Debug, PartialEq)]
struct OverlayLayout {
    top_inset: Pixels,
    width: Pixels,
    list_height: Pixels,
}

impl OverlayLayout {
    fn for_viewport(viewport: gpui::Size<Pixels>) -> Self {
        let height = viewport.height.as_f32();
        let chrome = SEARCH_HEIGHT + 1.0 + BOTTOM_INSET;
        // Float the surface a twelfth of the way down, but give the inset back
        // to the list before the list is allowed to fall below its minimum.
        let top = (height / 12.0)
            .clamp(MIN_TOP_INSET, MAX_TOP_INSET)
            .min((height - chrome - MIN_LIST_HEIGHT).max(MIN_TOP_INSET));
        let list = (height - top - chrome).clamp(MIN_LIST_HEIGHT, MAX_LIST_HEIGHT);
        Self {
            top_inset: px(top),
            width: px((viewport.width.as_f32() - 2.0 * BOTTOM_INSET).clamp(280.0, SURFACE_WIDTH)),
            list_height: px(list),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Overlay {
    CommandPalette,
    QuickOpen,
}

#[derive(Clone)]
enum CommandSelection {
    Action(PaletteCommand),
    Session(SessionId),
}

#[derive(Clone, Copy, Debug)]
pub enum NavigationEvent {
    ToggleSidebar,
    OpenOverview,
    OpenWorktrees,
    OpenSettings,
    CheckForUpdates,
}

pub struct NavigationOverlay {
    focus_handle: FocusHandle,
    store: Arc<RwLock<SessionStore>>,
    _runtime: Arc<StoreRuntime>,
    overlay: Option<Overlay>,
    query: QueryEditor,
    highlight: usize,
    /// Ranked once per keystroke, then read by hit-testing, keyboard
    /// navigation, and rendering alike — they must agree on what row 3 is.
    ranked_actions: Vec<Ranked<PaletteAction>>,
    ranked_sessions: Vec<Ranked<SessionRecord>>,
    matcher: FuzzyMatcher,
    directory_index: DirectoryIndex,
    quick_snapshot: QuickOpenSnapshot,
    ranked_items: Vec<RankedFolder>,
    scroll_handle: ScrollHandle,
    /// Separate slots: the disk-cache load and the filesystem scan both start
    /// at launch, and neither may cancel the other by sharing a `Task` slot.
    cache_task: Option<Task<()>>,
    scan_task: Option<Task<()>>,
    rank_task: Option<Task<()>>,
    /// This view is `.cached()` in RootView, so ambient window redraws no
    /// longer reach it: store changes must notify it directly, or an open
    /// palette's session rows go stale.
    _store_changes: Option<Task<()>>,
}

impl EventEmitter<NavigationEvent> for NavigationOverlay {}

impl NavigationOverlay {
    pub fn new(runtime: Arc<StoreRuntime>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        let _ = window;
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
        let mut overlay = Self {
            focus_handle,
            store: Arc::clone(&runtime.store),
            _runtime: runtime,
            overlay: None,
            query: QueryEditor::default(),
            highlight: 0,
            ranked_actions: Vec::new(),
            ranked_sessions: Vec::new(),
            matcher: FuzzyMatcher::text(),
            directory_index: DirectoryIndex::default(),
            quick_snapshot: QuickOpenSnapshot::default(),
            ranked_items: Vec::new(),
            scroll_handle: ScrollHandle::new(),
            cache_task: None,
            scan_task: None,
            rank_task: None,
            _store_changes: Some(store_changes),
        };
        // Warm at launch, the way Zed's worktree scan does: the cache makes the
        // index usable immediately and the scan refreshes it behind that, so the
        // first ⌘P of a session never waits on `read_dir`.
        overlay.load_cached_index(cx);
        overlay.refresh_directory_index(cx);
        overlay
    }

    #[cfg(test)]
    fn opened_for_test(runtime: Arc<StoreRuntime>, cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            store: Arc::clone(&runtime.store),
            _runtime: runtime,
            overlay: Some(Overlay::CommandPalette),
            query: QueryEditor::default(),
            highlight: 0,
            ranked_actions: Vec::new(),
            ranked_sessions: Vec::new(),
            matcher: FuzzyMatcher::text(),
            directory_index: DirectoryIndex::default(),
            quick_snapshot: QuickOpenSnapshot::default(),
            ranked_items: Vec::new(),
            scroll_handle: ScrollHandle::new(),
            cache_task: None,
            scan_task: None,
            rank_task: None,
            _store_changes: None,
        }
    }

    pub fn is_open(&self) -> bool {
        self.overlay.is_some()
    }

    pub(crate) fn toggle_command_palette(
        &mut self,
        _: &ToggleCommandPalette,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.overlay == Some(Overlay::CommandPalette) {
            self.close_overlay(cx);
        } else {
            self.open_overlay(Overlay::CommandPalette, window, cx);
        }
    }

    pub(crate) fn toggle_quick_open(
        &mut self,
        _: &ToggleQuickOpen,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.overlay == Some(Overlay::QuickOpen) {
            self.close_overlay(cx);
        } else {
            self.open_overlay(Overlay::QuickOpen, window, cx);
            self.refresh_directory_index(cx);
        }
    }

    fn open_overlay(&mut self, overlay: Overlay, window: &mut Window, cx: &mut Context<Self>) {
        self.overlay = Some(overlay);
        self.query.clear();
        self.reset_selection();
        self.ranked_items.clear();
        if overlay == Overlay::CommandPalette {
            self.refresh_command_items();
        }
        let _ = window;
        cx.notify();
    }

    fn close_overlay(&mut self, cx: &mut Context<Self>) {
        self.overlay = None;
        self.query.clear();
        self.highlight = 0;
        self.ranked_actions.clear();
        self.ranked_sessions.clear();
        self.rank_task = None;
        cx.notify();
    }

    /// Back to the first row, scrolled back to the top of the list.
    fn reset_selection(&mut self) {
        self.highlight = 0;
        self.scroll_handle.set_offset(gpui::point(px(0.0), px(0.0)));
    }

    fn schedule_rank(&mut self, cx: &mut Context<Self>) {
        self.rank_task = None;
        let query = self.query.text().trim().to_owned();
        if query.is_empty() {
            self.ranked_items.clear();
            cx.notify();
            return;
        }
        let pool = self.quick_snapshot.pool.clone();
        self.rank_task = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(RANK_DEBOUNCE).await;
            let ranked = cx
                .background_spawn(async move { quick_open::rank(&query, &pool, RESULT_LIMIT) })
                .await;
            this.update(cx, |this, cx| {
                this.ranked_items = ranked;
                this.reset_selection();
                cx.notify();
            })
            .ok();
        }));
    }

    pub(crate) fn on_key_down(
        &mut self,
        event: &KeyDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.overlay.is_none() {
            return;
        }
        let modifiers = event.keystroke.modifiers;
        match event.keystroke.key.as_str() {
            "escape" => self.close_overlay(cx),
            "up" => self.move_highlight(-1, cx),
            "down" => self.move_highlight(1, cx),
            "p" if modifiers.control => self.move_highlight(-1, cx),
            "n" if modifiers.control => self.move_highlight(1, cx),
            "enter" => self.run_highlighted(modifiers.platform, cx),
            _ => self.edit_query(event, cx),
        }
        cx.stop_propagation();
    }

    /// Everything the search field itself handles, through the key map shared
    /// with Quick Open and the terminal's find bar.
    fn edit_query(&mut self, event: &KeyDownEvent, cx: &mut Context<Self>) {
        let Some(edit) = query_editor::edit_for(&event.keystroke) else {
            return;
        };
        let changed = match edit {
            Edit::Local(local) => self.query.apply(local),
            Edit::Clipboard(ClipboardEdit::Copy) => {
                query_editor::copy_selection(&self.query, cx);
                false
            }
            Edit::Clipboard(ClipboardEdit::Cut) => query_editor::cut_selection(&mut self.query, cx),
            Edit::Clipboard(ClipboardEdit::Paste) => cx
                .read_from_clipboard()
                .and_then(|item| item.text())
                .is_some_and(|text| self.query.insert(&text)),
        };

        if changed {
            self.query_changed(cx);
        } else {
            // The caret or selection moved even when the text did not.
            cx.notify();
        }
    }

    fn query_changed(&mut self, cx: &mut Context<Self>) {
        self.reset_selection();
        if self.overlay == Some(Overlay::QuickOpen) {
            self.schedule_rank(cx);
        } else {
            self.refresh_command_items();
            cx.notify();
        }
    }

    fn move_highlight(&mut self, delta: isize, cx: &mut Context<Self>) {
        let count = self.visible_count();
        if count == 0 {
            return;
        }
        self.highlight = (self.highlight as isize + delta).rem_euclid(count as isize) as usize;
        self.scroll_to_highlight();
        cx.notify();
    }

    /// Keyboard navigation must drag the viewport along with it; the list is
    /// taller than the window on any real machine.
    fn scroll_to_highlight(&self) {
        self.scroll_handle.scroll_to_item(self.highlight_child());
    }

    /// Index of the highlighted row among the scroll container's children,
    /// which include the section headers.
    fn highlight_child(&self) -> usize {
        let first_section = match self.overlay {
            Some(Overlay::CommandPalette) => Some(self.ranked_actions.len()),
            Some(Overlay::QuickOpen) if self.query.text().trim().is_empty() => {
                Some(self.quick_snapshot.recent.len())
            }
            // A searched Quick Open list is one flat section, no headers.
            _ => None,
        };
        row_child_index(self.highlight, first_section)
    }

    fn visible_count(&self) -> usize {
        match self.overlay {
            Some(Overlay::CommandPalette) => self.ranked_actions.len() + self.ranked_sessions.len(),
            Some(Overlay::QuickOpen) if self.query.text().trim().is_empty() => {
                self.quick_snapshot.recent.len() + self.quick_snapshot.folders.len()
            }
            Some(Overlay::QuickOpen) => self.ranked_items.len(),
            None => 0,
        }
    }

    fn run_highlighted(&mut self, secondary: bool, cx: &mut Context<Self>) {
        match self.overlay {
            Some(Overlay::CommandPalette) => {
                let selection = if let Some(action) = self.ranked_actions.get(self.highlight) {
                    Some(CommandSelection::Action(action.item.command.clone()))
                } else {
                    self.ranked_sessions
                        .get(self.highlight.saturating_sub(self.ranked_actions.len()))
                        .map(|ranked| CommandSelection::Session(ranked.item.id.clone()))
                };
                if let Some(selection) = selection {
                    self.run_command_selection(selection, cx);
                }
            }
            Some(Overlay::QuickOpen) => {
                if let Some(item) = self.current_quick_item() {
                    let cwd = item.path.to_string_lossy().into_owned();
                    if secondary {
                        self.store
                            .write()
                            .expect("session store lock poisoned")
                            .spawn_shell(SpawnOptions {
                                cwd: Some(cwd.clone()),
                                ..SpawnOptions::default()
                            });
                    } else {
                        self.store
                            .write()
                            .expect("session store lock poisoned")
                            .spawn_default(SpawnOptions {
                                cwd: Some(cwd.clone()),
                                ..SpawnOptions::default()
                            });
                    }
                    self.close_overlay(cx);
                }
            }
            None => {}
        }
    }

    fn run_command_selection(&mut self, selection: CommandSelection, cx: &mut Context<Self>) {
        match selection {
            CommandSelection::Session(id) => {
                self.store
                    .write()
                    .expect("session store lock poisoned")
                    .select(id);
                self.close_overlay(cx);
            }
            CommandSelection::Action(command) => self.run_palette_command(command, cx),
        }
    }

    fn run_palette_command(&mut self, command: PaletteCommand, cx: &mut Context<Self>) {
        match command {
            PaletteCommand::SpawnAgent { agent, cwd, host } => {
                {
                    let mut store = self.store.write().expect("session store lock poisoned");
                    let mut options = SpawnOptions {
                        cwd: cwd.map(|path| path.to_string_lossy().into_owned()),
                        host: host.clone(),
                        ..SpawnOptions::default()
                    };
                    // Repo-preserving spawn: when no explicit directory was
                    // chosen and the spawn targets a remote host (or the
                    // active session lives on one), keep the active REPO —
                    // the daemon resolves its checkout on the target host.
                    let selected = store.selected_session();
                    let active_host = selected.and_then(|session| session.host.clone());
                    if options.cwd.is_none() && (host.is_some() || active_host.is_some()) {
                        options.same_repo_as = selected.map(|session| session.id.clone());
                        if host.is_none() && active_host.is_some() {
                            // Remote session spawning locally: its remote cwd
                            // is useless as a local path.
                            options.cwd = Some(store.local_fallback_directory());
                        }
                    }
                    store.spawn_kind(agent.kind(), options);
                }
                self.close_overlay(cx);
            }
            PaletteCommand::MigrateSelected { target_host } => {
                {
                    let mut store = self.store.write().expect("session store lock poisoned");
                    if let Some(id) = store.selected_session_id().cloned() {
                        store.migrate_session(id, target_host);
                    }
                }
                self.close_overlay(cx);
            }
            PaletteCommand::SyncPrefs { host } => {
                self.store
                    .write()
                    .expect("session store lock poisoned")
                    .sync_prefs(host);
                self.close_overlay(cx);
            }
            PaletteCommand::SpawnShell { host } => {
                self.store
                    .write()
                    .expect("session store lock poisoned")
                    .spawn_kind(
                        homie_proto::AgentKind::SHELL,
                        SpawnOptions {
                            host,
                            ..SpawnOptions::default()
                        },
                    );
                self.close_overlay(cx);
            }
            PaletteCommand::OpenQuickOpen => {
                self.overlay = Some(Overlay::QuickOpen);
                self.query.clear();
                self.reset_selection();
                self.ranked_items.clear();
                self.refresh_directory_index(cx);
                cx.notify();
            }
            PaletteCommand::ToggleSidebar => {
                cx.emit(NavigationEvent::ToggleSidebar);
                self.close_overlay(cx);
            }
            PaletteCommand::OpenSessionOverview => {
                cx.emit(NavigationEvent::OpenOverview);
                self.close_overlay(cx);
            }
            PaletteCommand::OpenWorktrees => {
                cx.emit(NavigationEvent::OpenWorktrees);
                self.close_overlay(cx);
            }
            PaletteCommand::OpenSettings => {
                cx.emit(NavigationEvent::OpenSettings);
                self.close_overlay(cx);
            }
            PaletteCommand::CheckForUpdates => {
                cx.emit(NavigationEvent::CheckForUpdates);
                self.close_overlay(cx);
            }
        }
    }

    /// Rebuild the palette's ranked rows for the current query. Cheap enough
    /// to run on every keystroke — a few hundred candidates against one
    /// matcher — and never run per frame.
    fn refresh_command_items(&mut self) {
        let (actions, sessions) = {
            let mut store = self.store.write().expect("session store lock poisoned");
            let projects: Vec<_> = store
                .sidebar_projection()
                .projects
                .iter()
                .map(|entry| entry.project.clone())
                .collect();
            let hosts = store.hosts().to_vec();
            let selected = store.selected_session().cloned();
            let default_host = store.default_spawn_host();
            let actions = palette::actions_for_default_host(
                store.preferences().default_agent,
                &projects,
                &hosts,
                selected.as_ref(),
                default_host.as_deref(),
            );
            (actions, store.ordered_sessions())
        };
        let query = FuzzyQuery::new(self.query.text());
        self.ranked_actions = palette::rank_actions(actions, &query, &mut self.matcher);
        self.ranked_sessions = palette::rank_sessions(sessions, &query, &mut self.matcher);
    }

    fn current_quick_item(&self) -> Option<QuickOpenItem> {
        if self.query.text().trim().is_empty() {
            self.quick_snapshot
                .recent
                .iter()
                .chain(&self.quick_snapshot.folders)
                .nth(self.highlight)
                .cloned()
        } else {
            self.ranked_items
                .get(self.highlight)
                .map(|folder| folder.item.clone())
        }
    }
}

impl Focusable for NavigationOverlay {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for NavigationOverlay {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let layout = OverlayLayout::for_viewport(window.viewport_size());
        let overlay = self.overlay.map(|_| self.render_overlay(layout, cx));
        let root = div()
            .id("navigation-overlay")
            .key_context("Homie")
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::toggle_command_palette))
            .on_action(cx.listener(Self::toggle_quick_open))
            .on_key_down(cx.listener(Self::on_key_down))
            .absolute()
            // Cached entity roots are laid out independently, so insets alone
            // leave this absolute root without a definite size and its height
            // collapses to its in-flow content, which is nothing.
            .size_full();
        if let Some(overlay) = overlay {
            root.inset_0().child(overlay)
        } else {
            root.size(px(0.0))
        }
    }
}

const fn row_child_index(row: usize, first_section: Option<usize>) -> usize {
    let Some(first) = first_section else {
        return row;
    };
    // Each non-empty section above the row contributes one header child.
    row + (first > 0) as usize + (row >= first) as usize
}

/// A static caret. Blinking would need an autonomous frame timer, which is
/// exactly what PERF.md's idle-CPU budget forbids; the terminal cursor is
/// static for the same reason.
pub(crate) const CARET: &str = "▏";

/// Draw a query field's contents: caret at the cursor, or the selection washed
/// in the brand accent. Shared by the palette, Quick Open, and the find bar so
/// all three fields look like the same control.
pub fn query_label(editor: &QueryEditor) -> AnyElement {
    let (text, selection) = editor.display(CARET);
    highlighted_label_styled(
        text,
        selection.as_slice(),
        HighlightStyle {
            background_color: Some(Palette::CLAY.alpha(0.35).into()),
            ..HighlightStyle::default()
        },
    )
}

/// Paint the characters the query actually matched in the brand accent, so a
/// glance at the list explains why each row is there and in that order.
fn highlighted_label(text: impl Into<SharedString>, matches: &[Range<usize>]) -> AnyElement {
    highlighted_label_styled(
        text,
        matches,
        HighlightStyle {
            color: Some(Palette::CLAY.into()),
            font_weight: Some(FontWeight::SEMIBOLD),
            ..HighlightStyle::default()
        },
    )
}

fn highlighted_label_styled(
    text: impl Into<SharedString>,
    matches: &[Range<usize>],
    style: HighlightStyle,
) -> AnyElement {
    let text = text.into();
    if matches.is_empty() {
        return div().child(text).into_any_element();
    }
    StyledText::new(text)
        .with_highlights(matches.iter().map(|range| (range.clone(), style)))
        .into_any_element()
}
