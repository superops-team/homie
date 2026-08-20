use std::cell::Cell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::macos::sf_symbols::{SymbolWeight, sf_symbol, sf_symbol_weighted};
use crate::navigation::query_label;
use crate::query_editor::{self, ClipboardEdit, Edit, QueryEditor};
use crate::settings::{HostDraft, SettingsTab, default_agent_label, theme};
use crate::store::{DefaultAgent, Prefs, SessionStore, StoreRuntime};
use crate::updates::{UpdateCommand, UpdateHandle, UpdatePhase};
use crate::worktrees::WorktreesSheet;
use gpui::{
    AnyElement, App, Bounds, ClickEvent, Context, CursorStyle, FocusHandle, Focusable, FontWeight,
    IntoElement, KeyDownEvent, MouseButton, Pixels, Render, Rgba, SharedString, Task, TextRun,
    Window, actions, canvas, deferred, div, font, prelude::*, px, rgba,
};
use homie_proto::{AgentKind as ProtoAgentKind, HistoryEntry, HostEntry};
use homie_term::theme::{TermTheme, ThemeAppearance};
use homie_ui::{
    AgentLogo, Button, ButtonSize, ButtonVariant, Fill, FloatingSurface, HairlineDivider, Ink,
    LoadingIndicator, Metrics, Palette, Radius, SemanticColors, Space, Typo,
};
use tokio::runtime::Runtime;

const SETTINGS_WIDTH: f32 = 600.0;
const SETTINGS_HEIGHT: f32 = 420.0;
const SETTINGS_NAV_WIDTH: f32 = 150.0;
const SETTINGS_SECTION_GAP: f32 = 16.0;
const SETTINGS_ROW_HEIGHT: f32 = 50.0;
const RESULT_LIMIT: usize = 200;
/// Reinstall success is confirmation, not persistent host state. Errors stay
/// actionable and first-time setup keeps its "Use by default" action.
const HOST_REINSTALL_SUCCESS_VISIBILITY: Duration = Duration::from_secs(3);

actions!(
    homie,
    [
        ToggleHistory,
        OpenWorktrees,
        OpenSettings,
        CloseSurface,
        MoveUp,
        MoveDown,
        Activate
    ]
);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Surface {
    #[default]
    None,
    History,
    Worktrees,
    Settings,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)] // Remaining dropdowns are introduced incrementally.
enum SettingsMenu {
    DefaultAgent,
    TerminalTheme,
    HibernateAfter,
    MemoryLimit,
}

mod history;
mod host_editor;
mod host_init;
mod hosts;
mod hosts_view;
mod projection;
mod settings_view;
mod view;
mod widgets;
mod worktrees;

use host_editor::{HostEditor, HostFormField};
use host_init::{HostInitialization, HostInitializationCardModel, HostPreparationKind};
use projection::{
    folder_name, relative_parent, relative_time, ui_agent, ui_default_agent, update_detail,
};

pub struct UtilitySurfaces {
    focus: FocusHandle,
    surface: Surface,
    history: Vec<HistoryEntry>,
    history_query: QueryEditor,
    history_highlight: usize,
    history_loading: bool,
    history_error: Option<String>,
    history_generation: u64,
    history_task: Option<Task<()>>,
    worktrees: WorktreesSheet,
    worktrees_generation: u64,
    worktrees_task: Option<Task<()>>,
    settings_tab: SettingsTab,
    settings_menu: Option<SettingsMenu>,
    hosts_path: PathBuf,
    hosts: Vec<HostEntry>,
    host_editor: Option<HostEditor>,
    host_initialization: Option<HostInitialization>,
    host_initialization_generation: u64,
    host_field_bounds: [Rc<Cell<Option<Bounds<Pixels>>>>; 6],
    prefs: Prefs,
    store: Arc<RwLock<SessionStore>>,
    store_runtime: Arc<StoreRuntime>,
    runtime: Arc<Runtime>,
    updates: UpdateHandle,
    activity: String,
    _update_changes: Task<()>,
    _store_changes: Task<()>,
}

impl UtilitySurfaces {
    pub fn new(
        store_runtime: Arc<StoreRuntime>,
        runtime: Arc<Runtime>,
        updates: UpdateHandle,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let focus = cx.focus_handle();
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/nonexistent"));
        let hosts_path = homie_proto::paths::HomiePaths::hosts_config_file(&home);
        let (prefs, hosts) = {
            let store = store_runtime
                .store
                .read()
                .expect("session store lock poisoned");
            (store.preferences().clone(), store.hosts().to_vec())
        };
        let settings_preview = std::env::var("HOMIE_SETTINGS_PREVIEW")
            .ok()
            .map(|value| value.to_ascii_lowercase());
        let settings_tab = match settings_preview.as_deref() {
            Some("terminal") => SettingsTab::Terminal,
            Some("resources") => SettingsTab::Resources,
            Some("remote") => SettingsTab::Remote,
            _ => SettingsTab::General,
        };
        // The Settings pane renders update state it does not own, so it has to
        // be woken when that state moves.
        let update_changes = {
            let mut states = updates.subscribe();
            cx.spawn(async move |this, cx| {
                while states.changed().await.is_ok() {
                    if this.update(cx, |_, cx| cx.notify()).is_err() {
                        return;
                    }
                }
            })
        };
        // This view is `.cached()` in RootView, so ambient window redraws no
        // longer reach it: store changes must notify it directly.
        let store_changes = {
            let mut changes = store_runtime.changes();
            cx.spawn(async move |this, cx| {
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
            })
        };
        Self {
            focus,
            surface: if settings_preview.is_some() {
                Surface::Settings
            } else {
                Surface::None
            },
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
            settings_tab,
            settings_menu: None,
            hosts_path,
            hosts,
            host_editor: None,
            host_initialization: None,
            host_initialization_generation: 0,
            host_field_bounds: std::array::from_fn(|_| Rc::new(Cell::new(None))),
            prefs,
            store: Arc::clone(&store_runtime.store),
            store_runtime,
            runtime,
            updates,
            activity: "Connected client · shared daemon remains untouched".to_owned(),
            _update_changes: update_changes,
            _store_changes: store_changes,
        }
    }

    fn colors(&self) -> SemanticColors {
        crate::app_theme::colors(&self.prefs.terminal_theme)
    }

    fn settings_colors(&self) -> SemanticColors {
        crate::app_theme::sidebar_colors(&self.prefs.terminal_theme)
    }

    fn next_history_generation(&mut self) -> u64 {
        self.history_generation = self.history_generation.wrapping_add(1);
        self.history_generation
    }

    fn next_worktrees_generation(&mut self) -> u64 {
        self.worktrees_generation = self.worktrees_generation.wrapping_add(1);
        self.worktrees_generation
    }

    fn persist_prefs(&mut self) {
        self.prefs.normalize();
        let prefs = self.prefs.clone();
        if let Err(error) = self
            .store
            .write()
            .expect("session store lock poisoned")
            .update_preferences(|shared| *shared = prefs)
        {
            self.activity = format!("Could not save settings: {error}");
        } else {
            self.activity = "Settings saved for homie".to_owned();
        }
    }

    fn close_surface(&mut self, cx: &mut Context<Self>) {
        if self.worktrees.pending_cleanup.is_some() {
            self.worktrees.cancel_cleanup();
        } else {
            match self.surface {
                Surface::History => {
                    self.history_task = None;
                    self.next_history_generation();
                }
                Surface::Worktrees => {
                    self.worktrees_task = None;
                    self.next_worktrees_generation();
                }
                Surface::None | Surface::Settings => {}
            }
            self.surface = Surface::None;
            self.settings_menu = None;
            self.host_editor = None;
        }
        cx.notify();
    }

    pub(crate) fn is_open(&self) -> bool {
        self.surface != Surface::None
    }

    pub(crate) fn open_settings(&mut self, cx: &mut Context<Self>) {
        self.prefs = self
            .store
            .read()
            .expect("session store lock poisoned")
            .preferences()
            .clone();
        self.surface = Surface::Settings;
        self.settings_menu = None;
        self.host_editor = None;
        cx.notify();
    }

    pub(crate) fn open_add_remote_host(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.open_settings(cx);
        self.settings_tab = SettingsTab::Remote;
        self.reload_hosts();
        self.begin_adding_host(window, cx);
    }

    pub(crate) fn toggle_history(&mut self, cx: &mut Context<Self>) {
        if self.surface == Surface::History {
            self.close_surface(cx);
        } else {
            self.open_history(cx);
        }
    }

    pub(crate) fn key_down(
        &mut self,
        event: &KeyDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.surface == Surface::None {
            return;
        }
        if self.handle_host_editor_key(event, cx) {
            return;
        }
        let key = &event.keystroke;
        if key.key == "escape" && self.surface == Surface::Settings && self.settings_menu.is_some()
        {
            self.settings_menu = None;
            cx.notify();
        } else if key.key == "escape" {
            self.close_surface(cx);
        } else if key.key == "up" {
            self.move_history(-1, cx);
        } else if key.key == "down" {
            self.move_history(1, cx);
        } else if key.key == "enter" {
            self.activate_history(cx);
        } else if self.surface == Surface::History {
            let Some(edit) = query_editor::edit_for(key) else {
                return;
            };
            let changed = match edit {
                Edit::Local(local) => self.history_query.apply(local),
                Edit::Clipboard(ClipboardEdit::Copy) => {
                    query_editor::copy_selection(&self.history_query, cx);
                    false
                }
                Edit::Clipboard(ClipboardEdit::Cut) => {
                    query_editor::cut_selection(&mut self.history_query, cx)
                }
                Edit::Clipboard(ClipboardEdit::Paste) => cx
                    .read_from_clipboard()
                    .and_then(|item| item.text())
                    .is_some_and(|text| self.history_query.insert(&text)),
            };
            if changed {
                self.history_highlight = 0;
            }
            cx.notify();
        }
    }
}

impl Focusable for UtilitySurfaces {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus.clone()
    }
}

#[cfg(test)]
mod tests;
