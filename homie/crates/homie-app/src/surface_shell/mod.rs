use std::cell::Cell;
use std::collections::HashSet;
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
use homie_proto::{AgentKind as ProtoAgentKind, HistoryEntry, HostEntry, HostsConfig};
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

mod host_editor;
mod host_init;
mod hosts_view;
mod projection;
mod settings_view;
mod view;
mod widgets;

use host_editor::{HostEditor, HostFormField};
use host_init::{
    HostInitialization, HostInitializationCardModel, HostPreparationKind,
    expire_completed_reinstall,
};
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

    fn finish_history_load(
        &mut self,
        generation: u64,
        result: Result<Vec<HistoryEntry>, String>,
    ) -> bool {
        if self.surface != Surface::History || self.history_generation != generation {
            return false;
        }
        self.history_loading = false;
        match result {
            Ok(entries) => {
                self.activity = format!("{} past conversations found", entries.len());
                self.history = entries;
                self.history_error = None;
            }
            Err(error) => self.history_error = Some(error),
        }
        true
    }

    fn finish_history_resume(
        &mut self,
        generation: u64,
        result: Result<homie_proto::SessionId, String>,
    ) -> bool {
        if self.surface != Surface::History || self.history_generation != generation {
            return false;
        }
        self.history_loading = false;
        match result {
            Ok(id) => {
                self.surface = Surface::None;
                self.activity = format!("Resumed conversation in session {}", id.0);
                self.history_error = None;
            }
            Err(error) => self.history_error = Some(error),
        }
        true
    }

    fn finish_worktrees_refresh(
        &mut self,
        generation: u64,
        result: Result<Vec<homie_proto::WorktreeOverviewEntry>, String>,
    ) -> bool {
        if self.surface != Surface::Worktrees || self.worktrees_generation != generation {
            return false;
        }
        self.worktrees.finish_refresh(result);
        true
    }

    pub(crate) fn open_history(&mut self, cx: &mut Context<Self>) {
        self.surface = Surface::History;
        self.history_query.clear();
        self.history_highlight = 0;
        self.history_loading = true;
        self.history_error = None;
        let generation = self.next_history_generation();
        cx.notify();

        let roots = crate::history::HistoryRoots::current_user();
        let client = Arc::clone(self.store_runtime.client());
        let runtime = Arc::clone(&self.runtime);
        self.history_task = Some(cx.spawn(async move |this, cx| {
            let task = runtime.spawn(async move {
                let tracked = if client
                    .wait_until_connected(Duration::from_secs(5))
                    .await
                    .is_ok()
                {
                    client
                        .sessions()
                        .await
                        .map(|result| {
                            result
                                .sessions
                                .into_iter()
                                .filter_map(|session| session.agent_session_id)
                                .collect()
                        })
                        .unwrap_or_default()
                } else {
                    HashSet::new()
                };
                tokio::task::spawn_blocking(move || crate::history::scan(&roots, &tracked))
                    .await
                    .map_err(|error| error.to_string())
            });
            let result = task
                .await
                .map_err(|error| error.to_string())
                .and_then(|r| r);
            let _ = this.update(cx, |this, cx| {
                this.history_task = None;
                if this.finish_history_load(generation, result) {
                    cx.notify();
                }
            });
        }));
    }

    pub(crate) fn open_worktrees(&mut self, cx: &mut Context<Self>) {
        self.surface = Surface::Worktrees;
        self.refresh_worktrees(cx);
    }

    fn refresh_worktrees(&mut self, cx: &mut Context<Self>) {
        self.worktrees.begin_refresh();
        let generation = self.next_worktrees_generation();
        cx.notify();
        let client = Arc::clone(self.store_runtime.client());
        let runtime = Arc::clone(&self.runtime);
        self.worktrees_task = Some(cx.spawn(async move |this, cx| {
            let task = runtime.spawn(async move {
                client.wait_until_connected(Duration::from_secs(5)).await?;
                client.worktree_overview().await
            });
            let result = match task.await {
                Ok(Ok(entries)) => Ok(entries),
                Ok(Err(error)) => Err(error.to_string()),
                Err(error) => Err(error.to_string()),
            };
            let _ = this.update(cx, |this, cx| {
                this.worktrees_task = None;
                if this.finish_worktrees_refresh(generation, result) {
                    cx.notify();
                }
            });
        }));
    }

    fn resume_history(&mut self, entry: HistoryEntry, cx: &mut Context<Self>) {
        let Some(params) = crate::history::resume_spawn(&entry) else {
            self.history_error = Some("The conversation folder is no longer available".to_owned());
            cx.notify();
            return;
        };
        self.history_loading = true;
        self.history_error = None;
        let generation = self.next_history_generation();
        let client = Arc::clone(self.store_runtime.client());
        let runtime = Arc::clone(&self.runtime);
        self.history_task = Some(cx.spawn(async move |this, cx| {
            let task = runtime.spawn(async move {
                client.wait_until_connected(Duration::from_secs(5)).await?;
                client.spawn(params).await
            });
            let result = match task.await {
                Ok(result) => result.map_err(|error| error.to_string()),
                Err(error) => Err(error.to_string()),
            };
            let _ = this.update(cx, |this, cx| {
                this.history_task = None;
                if this.finish_history_resume(generation, result) {
                    cx.notify();
                }
            });
        }));
    }

    fn confirm_cleanup(&mut self, cx: &mut Context<Self>) {
        let Some(params) = self.worktrees.confirm_cleanup() else {
            return;
        };
        self.worktrees.begin_refresh();
        let generation = self.next_worktrees_generation();
        let client = Arc::clone(self.store_runtime.client());
        let runtime = Arc::clone(&self.runtime);
        self.worktrees_task = Some(cx.spawn(async move |this, cx| {
            let task = runtime.spawn(async move {
                client.wait_until_connected(Duration::from_secs(5)).await?;
                client.worktree_remove(params).await?;
                client.worktree_overview().await
            });
            let result = match task.await {
                Ok(Ok(entries)) => Ok(entries),
                Ok(Err(error)) => Err(error.to_string()),
                Err(error) => Err(error.to_string()),
            };
            let _ = this.update(cx, |this, cx| {
                this.worktrees_task = None;
                if this.finish_worktrees_refresh(generation, result) {
                    cx.notify();
                }
            });
        }));
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

    fn reload_hosts(&mut self) {
        self.hosts = HostsConfig::load(&self.hosts_path).hosts;
        self.store
            .write()
            .expect("session store lock poisoned")
            .set_hosts(self.hosts.clone());
    }

    fn begin_adding_host(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.settings_menu = None;
        self.host_editor = Some(HostEditor::adding());
        self.focus.focus(window, cx);
        cx.notify();
    }

    fn begin_editing_host(&mut self, id: &str, window: &mut Window, cx: &mut Context<Self>) {
        let Some(host) = self.hosts.iter().find(|host| host.id == id) else {
            return;
        };
        self.settings_menu = None;
        self.host_editor = Some(HostEditor::editing(host));
        self.focus.focus(window, cx);
        cx.notify();
    }

    fn select_host_field(
        &mut self,
        field: HostFormField,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(editor) = &mut self.host_editor {
            editor.active_field = field;
            editor.error = None;
            editor.confirm_remove = false;
            self.focus.focus(window, cx);
            cx.notify();
        }
    }

    fn save_host(&mut self, cx: &mut Context<Self>) {
        let Some(editor) = &self.host_editor else {
            return;
        };
        let entry = match editor.draft().entry(&self.hosts) {
            Ok(entry) => entry,
            Err(error) => {
                if let Some(editor) = &mut self.host_editor {
                    editor.error = Some(error);
                    editor.confirm_remove = false;
                }
                cx.notify();
                return;
            }
        };
        let is_new = editor.original_id.is_none();
        let mut hosts = self.hosts.clone();
        if let Some(index) = hosts.iter().position(|host| host.id == entry.id) {
            hosts[index] = entry.clone();
        } else {
            hosts.push(entry.clone());
        }
        self.persist_hosts(hosts, format!("{} is ready", entry.display_name()), cx);
        if is_new && self.host_editor.is_none() {
            self.initialize_host(entry, cx);
        }
    }

    fn initialize_host(&mut self, host: HostEntry, cx: &mut Context<Self>) {
        self.prepare_host(host, HostPreparationKind::Initialize, cx);
    }

    fn reinstall_host(&mut self, id: &str, cx: &mut Context<Self>) {
        let Some(host) = self.hosts.iter().find(|host| host.id == id).cloned() else {
            return;
        };
        // Reinstallation always targets the saved HostEntry. Discarding the
        // editor avoids implying that unsaved SSH credentials are in use and
        // exposes the progress card immediately.
        self.host_editor = None;
        self.prepare_host(host, HostPreparationKind::Reinstall, cx);
    }

    fn prepare_host(&mut self, host: HostEntry, kind: HostPreparationKind, cx: &mut Context<Self>) {
        let id = host.id.clone();
        let name = host.display_name().to_owned();
        self.host_initialization_generation = self.host_initialization_generation.wrapping_add(1);
        let operation = self.host_initialization_generation;
        self.host_initialization = Some(HostInitialization::Running {
            id: id.clone(),
            name: name.clone(),
            kind,
            operation,
        });
        self.activity = match kind {
            HostPreparationKind::Initialize => format!("Setting up {name} over SSH…"),
            HostPreparationKind::Reinstall => {
                format!("Reinstalling the remote environment on {name}…")
            }
        };
        cx.notify();

        let client = Arc::clone(self.store_runtime.client());
        let runtime = Arc::clone(&self.runtime);
        cx.spawn(async move |this, cx| {
            let task = runtime.spawn(async move {
                match kind {
                    HostPreparationKind::Initialize => client.initialize_host(&id).await,
                    HostPreparationKind::Reinstall => client.reinstall_host(&id).await,
                }
            });
            let outcome = match task.await {
                Ok(result) => result.map_err(|error| error.to_string()),
                Err(error) => Err(error.to_string()),
            };
            let expire_success = outcome.is_ok() && kind == HostPreparationKind::Reinstall;
            let expiration_id = host.id.clone();
            let _ = this.update(cx, |this, cx| {
                if this
                    .host_initialization
                    .as_ref()
                    .is_none_or(|state| state.id() != host.id || state.operation() != operation)
                {
                    return;
                }
                match outcome {
                    Ok(result) => {
                        this.activity = match kind {
                            HostPreparationKind::Initialize => {
                                format!("{} is ready", host.display_name())
                            }
                            HostPreparationKind::Reinstall => {
                                format!("Remote environment reinstalled on {}", host.display_name())
                            }
                        };
                        this.host_initialization = Some(HostInitialization::Ready {
                            id: host.id.clone(),
                            name: host.display_name().to_owned(),
                            kind,
                            operation,
                            result,
                        });
                    }
                    Err(message) => {
                        this.activity = match kind {
                            HostPreparationKind::Initialize => {
                                format!("Could not initialize {}", host.display_name())
                            }
                            HostPreparationKind::Reinstall => format!(
                                "Could not reinstall the remote environment on {}",
                                host.display_name()
                            ),
                        };
                        this.host_initialization = Some(HostInitialization::Failed {
                            id: host.id.clone(),
                            name: host.display_name().to_owned(),
                            kind,
                            operation,
                            message,
                        });
                    }
                }
                cx.notify();
            });
            if expire_success {
                cx.background_executor()
                    .timer(HOST_REINSTALL_SUCCESS_VISIBILITY)
                    .await;
                let _ = this.update(cx, |this, cx| {
                    if expire_completed_reinstall(
                        &mut this.host_initialization,
                        &expiration_id,
                        operation,
                    ) {
                        cx.notify();
                    }
                });
            }
        })
        .detach();
    }

    fn retry_host_initialization(
        &mut self,
        id: &str,
        kind: HostPreparationKind,
        cx: &mut Context<Self>,
    ) {
        if let Some(host) = self.hosts.iter().find(|host| host.id == id).cloned() {
            self.prepare_host(host, kind, cx);
        }
    }

    fn request_remove_host(&mut self, cx: &mut Context<Self>) {
        let Some(editor) = &mut self.host_editor else {
            return;
        };
        if !editor.confirm_remove {
            editor.confirm_remove = true;
            editor.error = None;
            cx.notify();
            return;
        }
        let Some(id) = editor.original_id.clone() else {
            return;
        };
        let in_use = self
            .store
            .read()
            .expect("session store lock poisoned")
            .sessions()
            .values()
            .any(|session| session.host.as_deref() == Some(id.as_str()));
        if in_use {
            editor.confirm_remove = false;
            editor.error =
                Some("Move or close every session on this host before removing it.".to_owned());
            cx.notify();
            return;
        }
        let name = self
            .hosts
            .iter()
            .find(|host| host.id == id)
            .map_or_else(|| id.clone(), |host| host.display_name().to_owned());
        let hosts = self
            .hosts
            .iter()
            .filter(|host| host.id != id)
            .cloned()
            .collect();
        self.persist_hosts(hosts, format!("Removed {name}"), cx);
    }

    fn persist_hosts(&mut self, hosts: Vec<HostEntry>, activity: String, cx: &mut Context<Self>) {
        match (HostsConfig {
            hosts: hosts.clone(),
        })
        .save(&self.hosts_path)
        {
            Ok(()) => {
                self.hosts = hosts;
                if self
                    .host_initialization
                    .as_ref()
                    .is_some_and(|state| !self.hosts.iter().any(|host| host.id == state.id()))
                {
                    self.host_initialization = None;
                }
                self.store
                    .write()
                    .expect("session store lock poisoned")
                    .set_hosts(self.hosts.clone());
                self.host_editor = None;
                self.activity = activity;
            }
            Err(error) => {
                if let Some(editor) = &mut self.host_editor {
                    editor.error = Some(format!("Could not save hosts: {error}"));
                    editor.confirm_remove = false;
                }
            }
        }
        cx.notify();
    }

    fn edit_host_field(&mut self, edit: Edit, cx: &mut Context<Self>) {
        let Some(host_editor) = &mut self.host_editor else {
            return;
        };
        match edit {
            Edit::Local(local) => {
                host_editor.field_mut().apply(local);
            }
            Edit::Clipboard(ClipboardEdit::Copy) => {
                query_editor::copy_selection(host_editor.field_mut(), cx);
            }
            Edit::Clipboard(ClipboardEdit::Cut) => {
                query_editor::cut_selection(host_editor.field_mut(), cx);
            }
            Edit::Clipboard(ClipboardEdit::Paste) => {
                if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
                    host_editor.field_mut().insert(&text);
                }
            }
        }
        host_editor.error = None;
        host_editor.confirm_remove = false;
        cx.notify();
    }

    fn handle_host_editor_key(&mut self, event: &KeyDownEvent, cx: &mut Context<Self>) -> bool {
        if self.surface != Surface::Settings
            || self.settings_tab != SettingsTab::Remote
            || self.host_editor.is_none()
        {
            return false;
        }
        let key = &event.keystroke;
        match key.key.as_str() {
            "escape" => {
                if let Some(editor) = &mut self.host_editor
                    && editor.confirm_remove
                {
                    editor.confirm_remove = false;
                } else {
                    self.host_editor = None;
                }
                cx.notify();
            }
            "tab" => {
                if let Some(editor) = &mut self.host_editor {
                    editor.active_field = editor.active_field.adjacent(key.modifiers.shift);
                    editor.error = None;
                }
                cx.notify();
            }
            "enter" => self.save_host(cx),
            _ => {
                let Some(edit) = query_editor::edit_for(key) else {
                    return false;
                };
                self.edit_host_field(edit, cx);
            }
        }
        true
    }

    fn visible_history(&self) -> Vec<HistoryEntry> {
        self.history
            .iter()
            .filter(|entry| crate::history::matches_query(entry, self.history_query.text()))
            .take(RESULT_LIMIT)
            .cloned()
            .collect()
    }

    fn move_history(&mut self, delta: isize, cx: &mut Context<Self>) {
        if self.surface != Surface::History {
            return;
        }
        let count = self.visible_history().len();
        if count == 0 {
            return;
        }
        self.history_highlight =
            (self.history_highlight as isize + delta).rem_euclid(count as isize) as usize;
        cx.notify();
    }

    fn activate_history(&mut self, cx: &mut Context<Self>) {
        if self.surface != Surface::History {
            return;
        }
        if let Some(entry) = self.visible_history().get(self.history_highlight).cloned() {
            self.resume_history(entry, cx);
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
