use gpui::{
    App, Bounds, Context, FocusHandle, FontWeight, KeyDownEvent, MouseButton, MouseDownEvent,
    Render, Window, WindowBackgroundAppearance, WindowBounds, WindowOptions, actions, div,
    prelude::*, px, rgb, rgba, size,
};
use gpui_platform::application;
use homie_app::runtime_bridge::{
    BridgeConnectionState, BridgeDispatchError, BridgeEvent, BridgeProjection, RuntimeBridge,
    RuntimeBridgeConfig, RuntimeCommand,
};
use homie_app::user_paths;
use homie_proto::model::SessionSummary;
use homie_storage::{SettingsPreferences, StorageConfig, open_or_create};
use homie_term::buffer::GridBuffer;
use homie_term::element::{SharedGridBuffer, TerminalElement};
use homie_term::find::TerminalFindModel;
use homie_term::theme::TermTheme;
use homie_ui::{
    NotificationSession, SidebarSessionModel, SidebarSessionRow, notification_rollup, rank_items,
};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use homie_app::palette::{PaletteCommand, PaletteView};

const MIN_WINDOW_WIDTH: f32 = 980.0;
const MIN_WINDOW_HEIGHT: f32 = 620.0;

actions!(
    homie_app,
    [
        Quit,
        HideApp,
        CloseWindow,
        ToggleCommandPalette,
        ToggleSidebar,
        ToggleInspector,
        ZoomIn,
        ZoomOut,
        ResetZoom,
        CopySelection,
        Paste,
    ]
);

#[derive(Clone)]
struct AppState {
    data_dir: PathBuf,
    default_profile: String,
    session_count: usize,
    runtime_available: bool,
    inspector_visible: bool,
    terminal_notice: String,
    sessions: Vec<SessionRow>,
    session_id: Option<String>,
    session_title: String,
    runtime_status: String,
    terminal_geometry: Option<TerminalGeometry>,
    settings_visible: bool,
    settings_tab: SettingsTab,
    settings_preferences: SettingsPreferences,
    artifact_summary: ArtifactSummary,
    quick_open_visible: bool,
    quick_open_query: String,
    worktrees_visible: bool,
    worktrees: Vec<WorktreeRow>,
    needs_input_summary: Option<String>,
    find_visible: bool,
    find_model: TerminalFindModel,
    sidebar_model: SidebarSessionModel,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SessionRow {
    id: String,
    title: String,
    status: String,
    workspace: String,
}

impl From<&SessionSummary> for SessionRow {
    fn from(value: &SessionSummary) -> Self {
        Self {
            id: value.id.clone(),
            title: value.title.clone(),
            status: value.status.clone(),
            workspace: value.workspace.clone(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct TerminalGeometry {
    cols: u16,
    rows: u16,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct ArtifactSummary {
    ports: usize,
    pull_requests: usize,
    previews: usize,
    links: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum QuickOpenTarget {
    Session(String),
    Settings,
    NewTerminal,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct QuickOpenItem {
    label: String,
    detail: String,
    target: QuickOpenTarget,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct WorktreeRow {
    path: String,
    status: String,
    cleanup: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SettingsTab {
    General,
    Terminal,
    Resources,
    Remote,
}

impl SettingsTab {
    fn label(self) -> &'static str {
        match self {
            Self::General => "General",
            Self::Terminal => "Terminal",
            Self::Resources => "Resources",
            Self::Remote => "Remote",
        }
    }
}

struct HomieWorkbench {
    state: AppState,
    terminal: TerminalElement,
    terminal_buffer: SharedGridBuffer,
    bridge: Option<RuntimeBridge>,
    terminal_attachment: TerminalAttachmentState,
    palette: Option<PaletteView>,
    focus_handle: FocusHandle,
}

#[derive(Default)]
struct TerminalAttachmentState {
    pending_session_id: Option<String>,
    attached_session_id: Option<String>,
    retry_session_id: Option<String>,
    retry_offset: u64,
}

impl TerminalAttachmentState {
    fn begin_attach(&mut self, selected_session_id: Option<&str>) -> Option<String> {
        let Some(selected_session_id) = selected_session_id else {
            self.pending_session_id = None;
            self.attached_session_id = None;
            self.retry_session_id = None;
            self.retry_offset = 0;
            return None;
        };
        if self.pending_session_id.as_deref() == Some(selected_session_id)
            || self.attached_session_id.as_deref() == Some(selected_session_id)
        {
            return None;
        }
        self.pending_session_id = Some(selected_session_id.to_string());
        Some(selected_session_id.to_string())
    }

    fn apply_event(&mut self, selected_session_id: Option<&str>, event: &BridgeEvent) {
        match event {
            BridgeEvent::TerminalAttached { session_id }
                if selected_session_id == Some(session_id.as_str())
                    && self.pending_session_id.as_deref() == Some(session_id.as_str()) =>
            {
                self.attached_session_id = Some(session_id.clone());
                self.pending_session_id = None;
            }
            BridgeEvent::TerminalUnavailable {
                last_confirmed_offset,
            } => {
                self.retry_session_id = selected_session_id.map(str::to_string);
                self.retry_offset = *last_confirmed_offset;
                self.pending_session_id = None;
                self.attached_session_id = None;
            }
            BridgeEvent::CommandFailed {
                command: "terminal.open",
                ..
            } => {
                self.pending_session_id = None;
                self.attached_session_id = None;
            }
            _ => {}
        }
    }

    fn retry_offset(&self, session_id: &str) -> u64 {
        if self.retry_session_id.as_deref() == Some(session_id) {
            self.retry_offset
        } else {
            0
        }
    }

    fn dispatch_failed(&mut self) {
        self.pending_session_id = None;
        self.attached_session_id = None;
    }

    fn attached_session_id(&self) -> Option<&str> {
        self.attached_session_id.as_deref()
    }
}

fn select_session_summary(state: &mut AppState, session: &SessionSummary) {
    state.session_id = Some(session.id.clone());
    state.session_title = session.title.clone();
    state.runtime_status = session.status.clone();
}

impl HomieWorkbench {
    fn load(cx: &mut Context<Self>) -> Self {
        let data_dir = default_data_dir();
        let workspace = default_workspace();
        let terminal_buffer: SharedGridBuffer = Arc::new(RwLock::new(GridBuffer::new(120, 40)));
        let terminal = TerminalElement::new(terminal_buffer.clone())
            .theme(TermTheme::default())
            .font_size(px(13.0));
        let mut state = AppState {
            data_dir: data_dir.clone(),
            default_profile: "unavailable".to_string(),
            session_count: 0,
            runtime_available: false,
            inspector_visible: true,
            terminal_notice: "runtime connecting".to_string(),
            sessions: Vec::new(),
            session_id: None,
            session_title: "No live session".to_string(),
            runtime_status: "connecting".to_string(),
            terminal_geometry: None,
            settings_visible: false,
            settings_tab: SettingsTab::General,
            settings_preferences: SettingsPreferences::default(),
            artifact_summary: ArtifactSummary::default(),
            quick_open_visible: false,
            quick_open_query: String::new(),
            worktrees_visible: false,
            worktrees: Vec::new(),
            needs_input_summary: None,
            find_visible: false,
            find_model: TerminalFindModel::default(),
            sidebar_model: SidebarSessionModel::default(),
        };
        let bridge = std::env::current_exe()
            .map_err(|_| ())
            .and_then(|current_executable| {
                RuntimeBridge::start(RuntimeBridgeConfig {
                    data_dir: data_dir.clone(),
                    current_executable,
                    workspace,
                    startup_probe_timeout: Duration::from_secs(1),
                    connect_timeout: Duration::from_secs(10),
                    request_timeout: Duration::from_secs(10),
                })
                .map_err(|_| ())
            })
            .ok();
        if bridge.is_none() {
            state.runtime_status = "unavailable".to_string();
            state.terminal_notice = "runtime bridge unavailable".to_string();
        }
        let mut workbench = Self {
            state,
            terminal,
            terminal_buffer,
            bridge,
            terminal_attachment: TerminalAttachmentState::default(),
            palette: None,
            focus_handle: cx.focus_handle(),
        };
        workbench.start_bridge_poll(cx);
        workbench
    }

    fn start_bridge_poll(&mut self, cx: &mut Context<Self>) {
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(50))
                    .await;
                if this
                    .update(cx, |this, cx| {
                        if this.drain_bridge_updates() {
                            cx.notify();
                        }
                    })
                    .is_err()
                {
                    break;
                }
            }
        })
        .detach();
    }

    fn drain_bridge_updates(&mut self) -> bool {
        let Some(bridge) = self.bridge.as_mut() else {
            return false;
        };
        let events = bridge.drain_events();
        if events.is_empty() {
            return false;
        }
        let projection = bridge.projection().clone();
        for event in events {
            self.apply_bridge_event(event);
        }
        self.sync_runtime_projection(&projection);
        true
    }

    fn apply_bridge_event(&mut self, event: BridgeEvent) {
        self.terminal_attachment
            .apply_event(self.state.session_id.as_deref(), &event);
        match event {
            BridgeEvent::Connection(connection) => {
                self.state.runtime_status = connection_label(&connection).to_string();
                self.state.runtime_available = connection == BridgeConnectionState::Connected;
                self.state.terminal_notice = format!("runtime {}", self.state.runtime_status);
                if connection == BridgeConnectionState::Connected {
                    self.dispatch_runtime(RuntimeCommand::RefreshSessions);
                }
            }
            BridgeEvent::TerminalGrid(grid) => {
                self.terminal_buffer
                    .write()
                    .expect("terminal buffer")
                    .apply(grid);
                self.state.terminal_notice = "live terminal stream".to_string();
            }
            BridgeEvent::TerminalAttached { .. } => {
                self.state.terminal_notice = "live terminal stream".to_string();
            }
            BridgeEvent::TerminalUnavailable {
                last_confirmed_offset,
            } => {
                self.state.terminal_notice =
                    format!("terminal reconnecting from offset {last_confirmed_offset}");
            }
            BridgeEvent::CommandFailed { command, code } => {
                self.state.terminal_notice = format!("{command} failed: {code}");
            }
            BridgeEvent::SessionSpawned(session) => {
                select_session_summary(&mut self.state, &session);
                self.attach_selected_session();
            }
            BridgeEvent::RuntimeEvent(_) => {
                self.dispatch_runtime(RuntimeCommand::RefreshSessions);
            }
            BridgeEvent::DaemonIdentity { .. }
            | BridgeEvent::Snapshot(_)
            | BridgeEvent::Sessions(_)
            | BridgeEvent::Artifacts(_)
            | BridgeEvent::Worktrees(_)
            | BridgeEvent::TerminalOutput(_) => {}
        }
    }

    fn sync_runtime_projection(&mut self, projection: &BridgeProjection) {
        self.state.runtime_available = projection.runtime_available;
        self.state.runtime_status = connection_label(&projection.connection).to_string();
        self.state.sessions = projection.sessions.iter().map(SessionRow::from).collect();
        self.state.session_count = self.state.sessions.len();
        if let Some(selected) = self.state.session_id.as_ref()
            && !self
                .state
                .sessions
                .iter()
                .any(|session| &session.id == selected)
        {
            self.state.session_id = None;
        }
        if self.state.session_id.is_none()
            && let Some(selected) = projection
                .selected_session_id
                .as_ref()
                .and_then(|selected| {
                    self.state
                        .sessions
                        .iter()
                        .find(|session| &session.id == selected)
                })
                .cloned()
        {
            self.state.session_id = Some(selected.id.clone());
            self.state.session_title = selected.title;
            self.state.runtime_status = selected.status;
        }
        if let Some(selected) = self.state.session_id.as_ref()
            && let Some(session) = self
                .state
                .sessions
                .iter()
                .find(|session| &session.id == selected)
        {
            self.state.default_profile = projection
                .sessions
                .iter()
                .find(|summary| &summary.id == selected)
                .map_or_else(
                    || "unavailable".to_string(),
                    |summary| summary.agent_profile_id.clone(),
                );
            self.state.runtime_status = session.status.clone();
        }
        self.state.artifact_summary = ArtifactSummary {
            ports: projection.artifacts.ports.len(),
            pull_requests: projection
                .artifacts
                .artifacts
                .iter()
                .filter(|artifact| artifact.kind == homie_proto::model::ArtifactKind::PullRequest)
                .count(),
            previews: projection
                .artifacts
                .artifacts
                .iter()
                .filter(|artifact| artifact.kind == homie_proto::model::ArtifactKind::Preview)
                .count(),
            links: projection
                .artifacts
                .artifacts
                .iter()
                .filter(|artifact| artifact.kind == homie_proto::model::ArtifactKind::Link)
                .count(),
        };
        self.state.worktrees = projection
            .worktrees
            .entries
            .iter()
            .map(|entry| WorktreeRow {
                path: entry.path.clone(),
                status: entry
                    .session_status
                    .clone()
                    .unwrap_or_else(|| "unbound".to_string()),
                cleanup: entry.stale_suggestion && !entry.dirty && entry.merged,
            })
            .collect();
        self.sync_sidebar_model();
        if self.terminal_attachment.attached_session_id() != self.state.session_id.as_deref() {
            self.attach_selected_session();
        }
    }

    fn refresh_artifact_summary(&mut self) {
        let Some(session_id) = self.state.session_id.clone() else {
            return;
        };
        self.dispatch_runtime(RuntimeCommand::RefreshArtifacts { session_id });
    }

    fn attach_selected_session(&mut self) {
        let Some(session_id) = self
            .terminal_attachment
            .begin_attach(self.state.session_id.as_deref())
        else {
            return;
        };
        let output_offset = self.terminal_attachment.retry_offset(&session_id);
        if !self.dispatch_runtime(RuntimeCommand::SelectSession {
            session_id: session_id.clone(),
            output_offset,
        }) {
            self.terminal_attachment.dispatch_failed();
            return;
        }
        self.dispatch_runtime(RuntimeCommand::RefreshArtifacts {
            session_id: session_id.clone(),
        });
        self.state.terminal_geometry = None;
        self.sync_terminal_geometry(120, 40);
    }

    fn dispatch_runtime(&mut self, command: RuntimeCommand) -> bool {
        let result = self
            .bridge
            .as_ref()
            .ok_or(BridgeDispatchError::Unavailable)
            .and_then(|bridge| bridge.dispatch(command));
        match result {
            Ok(()) => true,
            Err(error) => {
                self.state.terminal_notice = error.to_string();
                false
            }
        }
    }

    fn spawn_runtime_shell(&mut self) {
        if self.dispatch_runtime(RuntimeCommand::SpawnSession {
            cwd: default_workspace(),
            title: Some("Homie live shell".to_string()),
        }) {
            self.state.terminal_notice = "spawn queued".to_string();
        }
    }

    fn select_session(&mut self, session_id: &str) {
        if let Some(session) = self
            .state
            .sessions
            .iter()
            .find(|session| session.id == session_id)
            .cloned()
        {
            self.state.session_id = Some(session.id);
            self.state.session_title = session.title;
            self.state.runtime_status = session.status;
            self.state.sidebar_model.select(session_id);
            self.attach_selected_session();
            self.sync_terminal_geometry(120, 40);
            self.refresh_artifact_summary();
        }
    }

    fn sync_sidebar_model(&mut self) {
        let previous = self.state.sidebar_model.clone();
        let mut rows = self
            .state
            .sessions
            .iter()
            .map(|session| {
                let prior = previous.rows.iter().find(|row| row.id == session.id);
                SidebarSessionRow {
                    id: session.id.clone(),
                    title: session.title.clone(),
                    status: session.status.clone(),
                    pinned: prior.is_some_and(|row| row.pinned),
                    archived: prior.is_some_and(|row| row.archived),
                }
            })
            .collect::<Vec<_>>();
        rows.sort_by(|left, right| {
            right
                .pinned
                .cmp(&left.pinned)
                .then_with(|| left.title.cmp(&right.title))
        });
        let mut model = SidebarSessionModel::new(rows);
        model.selected = self.state.session_id.clone().filter(|selected| {
            model
                .rows
                .iter()
                .any(|row| &row.id == selected && !row.archived)
        });
        model.multi_selected = previous
            .multi_selected
            .into_iter()
            .filter(|selected| {
                model
                    .rows
                    .iter()
                    .any(|row| &row.id == selected && !row.archived)
            })
            .collect();
        model.multi_selected.sort();
        self.state.sidebar_model = model;
    }

    fn pin_sidebar_session(&mut self, session_id: &str) {
        self.state.sidebar_model.toggle_pin(session_id);
        self.state.terminal_notice = "sidebar pin updated".into();
    }

    fn archive_sidebar_session(&mut self, session_id: &str) {
        self.state.sidebar_model.archive(session_id);
        if self.state.session_id.as_deref() == Some(session_id) {
            let next_session_id = self
                .state
                .sidebar_model
                .rows
                .iter()
                .find(|row| !row.archived)
                .map(|row| row.id.clone());
            if let Some(next_session_id) = next_session_id {
                self.select_session(&next_session_id);
            } else {
                self.state.session_id = None;
                self.state.session_title = "No live session".into();
                self.state.runtime_status = "archived".into();
            }
        }
        self.state.terminal_notice = "sidebar session archived".into();
    }

    fn toggle_sidebar_multi_select(&mut self, session_id: &str) {
        self.state.sidebar_model.toggle_multi_select(session_id);
        self.state.terminal_notice =
            format!("{} selected", self.state.sidebar_model.multi_selected.len());
    }

    fn sync_terminal_geometry(&mut self, cols: u16, rows: u16) {
        let geometry = TerminalGeometry { cols, rows };
        if self.state.terminal_geometry == Some(geometry) {
            return;
        }
        let Some(session_id) = self.state.session_id.clone() else {
            return;
        };
        if self.dispatch_runtime(RuntimeCommand::Resize {
            session_id,
            cols,
            rows,
        }) {
            self.state.terminal_geometry = Some(geometry);
        }
    }

    fn open_settings(&mut self) {
        self.state.settings_visible = true;
        self.state.settings_tab = SettingsTab::General;
        self.reload_settings_preferences();
        self.state.terminal_notice = "settings".into();
    }

    fn set_settings_tab(&mut self, tab: SettingsTab) {
        self.state.settings_tab = tab;
    }

    fn reload_settings_preferences(&mut self) {
        match open_ready_storage(self.state.data_dir.clone())
            .and_then(|storage| storage.load_settings_preferences())
        {
            Ok(preferences) => {
                self.state.settings_preferences = preferences;
            }
            Err(error) => {
                self.state.terminal_notice = format!("settings load error: {error}");
            }
        }
    }

    fn save_settings_preferences(&mut self) {
        match open_ready_storage(self.state.data_dir.clone())
            .and_then(|storage| storage.save_settings_preferences(&self.state.settings_preferences))
        {
            Ok(()) => {
                self.state.terminal_notice = "settings saved".into();
            }
            Err(error) => {
                self.state.terminal_notice = format!("settings save error: {error}");
            }
        }
    }

    fn bump_terminal_font_size(&mut self, delta: i8) {
        let current = i16::from(self.state.settings_preferences.terminal_font_size);
        self.state.settings_preferences.terminal_font_size = (current + i16::from(delta))
            .clamp(10, 24)
            .try_into()
            .expect("font size range");
        self.save_settings_preferences();
    }

    fn toggle_remote_companion_access(&mut self) {
        self.state.settings_preferences.remote_companion_access =
            !self.state.settings_preferences.remote_companion_access;
        self.save_settings_preferences();
    }

    fn open_quick_open(&mut self) {
        self.state.quick_open_visible = true;
        self.state.quick_open_query.clear();
        self.state.terminal_notice = "quick open".into();
    }

    fn dismiss_quick_open(&mut self) {
        self.state.quick_open_visible = false;
        self.state.quick_open_query.clear();
    }

    fn quick_open_items(&self) -> Vec<QuickOpenItem> {
        let mut items = self
            .state
            .sessions
            .iter()
            .map(|session| QuickOpenItem {
                label: session.title.clone(),
                detail: session.workspace.clone(),
                target: QuickOpenTarget::Session(session.id.clone()),
            })
            .collect::<Vec<_>>();
        items.push(QuickOpenItem {
            label: "Settings".to_string(),
            detail: "General, Terminal, Resources, Remote".to_string(),
            target: QuickOpenTarget::Settings,
        });
        items.push(QuickOpenItem {
            label: "New Terminal".to_string(),
            detail: "Spawn a local runtime session".to_string(),
            target: QuickOpenTarget::NewTerminal,
        });
        let labels = items
            .iter()
            .map(|item| item.label.as_str())
            .collect::<Vec<_>>();
        rank_items(&self.state.quick_open_query, labels)
            .into_iter()
            .filter_map(|ranked| {
                items
                    .iter()
                    .find(|item| item.label == ranked.label)
                    .cloned()
            })
            .collect()
    }

    fn activate_quick_open(&mut self, target: QuickOpenTarget) {
        match target {
            QuickOpenTarget::Session(session_id) => self.select_session(&session_id),
            QuickOpenTarget::Settings => self.open_settings(),
            QuickOpenTarget::NewTerminal => self.spawn_runtime_shell(),
        }
        self.dismiss_quick_open();
    }

    fn open_worktrees(&mut self) {
        self.state.worktrees_visible = true;
        self.refresh_worktrees();
        self.state.terminal_notice = "worktrees".into();
    }

    fn approve_needs_input(&mut self) {
        self.state.terminal_notice = "approval action queued".into();
        self.state.needs_input_summary = None;
    }

    fn deny_needs_input(&mut self) {
        self.state.terminal_notice = "deny action queued".into();
        self.state.needs_input_summary = None;
    }

    fn open_find(&mut self) {
        self.state.find_visible = true;
        self.state.terminal_notice = "find".into();
        self.apply_find_now();
    }

    fn dismiss_find(&mut self) {
        self.state.find_visible = false;
        self.state.find_model.set_query("", Duration::ZERO);
    }

    fn apply_find_now(&mut self) {
        let query = self.state.find_model.query().to_string();
        let generation = self
            .state
            .find_model
            .take_due_search(Duration::from_secs(1))
            .map(|(_, _, generation)| generation)
            .unwrap_or(0);
        let buffer = self
            .terminal_buffer
            .read()
            .expect("terminal buffer")
            .clone();
        let _ = self
            .state
            .find_model
            .apply_snapshot(&query, generation, &buffer);
        self.terminal.sync_find_highlights(&self.state.find_model);
    }

    fn refresh_worktrees(&mut self) {
        self.dispatch_runtime(RuntimeCommand::RefreshWorktrees);
    }

    fn toggle_palette(&mut self, cx: &mut Context<Self>) {
        self.palette = if self.palette.is_some() {
            None
        } else {
            Some(PaletteView::new())
        };
        cx.notify();
    }
    fn exec_palette(&mut self, cmd: &PaletteCommand, cx: &mut Context<Self>) {
        match cmd {
            PaletteCommand::SpawnShell => self.spawn_runtime_shell(),
            PaletteCommand::OpenQuickOpen => self.open_quick_open(),
            PaletteCommand::ToggleSidebar => self.open_worktrees(),
            PaletteCommand::OpenSettings => self.open_settings(),
            PaletteCommand::OpenFind => self.open_find(),
            PaletteCommand::CheckForUpdates => self.state.terminal_notice = "updates".into(),
        }
        self.palette = None;
        cx.notify();
    }
    fn zoom_in(&mut self, _: &ZoomIn, _w: &mut Window, cx: &mut Context<Self>) {
        self.state.terminal_notice = "zoom in".into();
        cx.notify();
    }
    fn zoom_out(&mut self, _: &ZoomOut, _w: &mut Window, cx: &mut Context<Self>) {
        self.state.terminal_notice = "zoom out".into();
        cx.notify();
    }
    fn reset_zoom(&mut self, _: &ResetZoom, _w: &mut Window, cx: &mut Context<Self>) {
        self.state.terminal_notice = "zoom reset".into();
        cx.notify();
    }
    fn copy_sel(&mut self, _: &CopySelection, _w: &mut Window, cx: &mut Context<Self>) {
        let t = self.terminal.selected_text();
        if !t.is_empty() {
            cx.write_to_clipboard(gpui::ClipboardItem::new_string(t));
            self.state.terminal_notice = "copied".into();
        }
        cx.notify();
    }
    fn do_paste(&mut self, _: &Paste, _w: &mut Window, cx: &mut Context<Self>) {
        if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
            if let Some(session_id) = self.state.session_id.clone() {
                if self.dispatch_runtime(RuntimeCommand::SendText {
                    session_id,
                    text,
                    submit: false,
                }) {
                    self.state.terminal_notice = "paste queued".into();
                }
            } else {
                self.state.terminal_notice = "paste unavailable".into();
            }
        }
        cx.notify();
    }
    fn on_key(&mut self, e: &KeyDownEvent, cx: &mut Context<Self>) {
        if self.state.find_visible {
            let keystroke = &e.keystroke;
            match keystroke.key.as_str() {
                "escape" => {
                    self.dismiss_find();
                    cx.notify();
                    return;
                }
                "backspace" => {
                    let mut query = self.state.find_model.query().to_string();
                    query.pop();
                    self.state.find_model.set_query(query, Duration::ZERO);
                    self.apply_find_now();
                    cx.notify();
                    return;
                }
                _ => {
                    if let Some(ch) = keystroke.key.chars().next()
                        && (ch.is_ascii_graphic() || ch == ' ')
                    {
                        let mut query = self.state.find_model.query().to_string();
                        query.push(ch);
                        self.state.find_model.set_query(query, Duration::ZERO);
                        self.apply_find_now();
                        cx.notify();
                        return;
                    }
                }
            }
        }
        if self.state.quick_open_visible {
            let keystroke = &e.keystroke;
            match keystroke.key.as_str() {
                "escape" => {
                    self.dismiss_quick_open();
                    cx.notify();
                    return;
                }
                "enter" | "return" => {
                    if let Some(item) = self.quick_open_items().into_iter().next() {
                        self.activate_quick_open(item.target);
                    }
                    cx.notify();
                    return;
                }
                "backspace" => {
                    self.state.quick_open_query.pop();
                    cx.notify();
                    return;
                }
                _ => {
                    if let Some(ch) = keystroke.key.chars().next()
                        && (ch.is_ascii_graphic() || ch == ' ')
                    {
                        self.state.quick_open_query.push(ch);
                        cx.notify();
                        return;
                    }
                }
            }
        }
        if let Some(ref mut p) = self.palette {
            // Handle palette keyboard input
            let keystroke = &e.keystroke;
            match keystroke.key.as_str() {
                "escape" => {
                    self.palette = None;
                    cx.notify();
                    return;
                }
                "enter" | "return" => {
                    if let Some(cmd) = p.selected_command() {
                        self.exec_palette(&cmd, cx);
                    }
                    return;
                }
                "down" => {
                    p.move_down();
                    cx.notify();
                    return;
                }
                "up" => {
                    p.move_up();
                    cx.notify();
                    return;
                }
                "backspace" => {
                    p.pop_char();
                    cx.notify();
                    return;
                }
                _ => {
                    if let Some(ch) = keystroke.key.chars().next()
                        && (ch.is_ascii_graphic() || ch == ' ')
                    {
                        p.push_char(ch);
                        cx.notify();
                        return;
                    }
                }
            }
        }
        cx.notify();
    }
    fn status_color(&self) -> gpui::Rgba {
        if self.state.runtime_available {
            rgb(0x34c759)
        } else {
            rgb(0xff453a)
        }
    }

    fn sec_hdr(s: &str, n: usize) -> gpui::Div {
        div()
            .flex()
            .items_center()
            .justify_between()
            .pt_2()
            .px_1()
            .child(
                div()
                    .text_size(px(11.0))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(rgba(0x0000008c))
                    .child(s.to_string()),
            )
            .child(
                div()
                    .text_size(px(11.0))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(rgba(0x00000066))
                    .child(n.to_string()),
            )
    }
    fn dot(c: gpui::Rgba) -> gpui::Div {
        div().size(px(9.0)).rounded_full().bg(c)
    }
    fn srow(t: &str, s: &str, m: &str, c: gpui::Rgba, sel: bool) -> gpui::Div {
        let bg = if sel {
            rgba(0x00000010)
        } else {
            rgba(0x00000000)
        };
        div()
            .h(px(36.0))
            .rounded(px(7.0))
            .px_2()
            .bg(bg)
            .flex()
            .items_center()
            .gap_2()
            .child(Self::dot(c))
            .child(
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .gap_0p5()
                    .child(
                        div()
                            .text_size(px(13.0))
                            .font_weight(if sel {
                                FontWeight::MEDIUM
                            } else {
                                FontWeight::NORMAL
                            })
                            .text_color(if sel {
                                rgba(0x000000e0)
                            } else {
                                rgba(0x000000bf)
                            })
                            .child(t.to_string()),
                    )
                    .child(
                        div()
                            .text_size(px(10.0))
                            .text_color(rgba(0x00000066))
                            .child(s.to_string()),
                    ),
            )
            .child(
                div()
                    .text_size(px(10.0))
                    .text_color(rgba(0x00000066))
                    .child(m.to_string()),
            )
    }
    fn sidebar_action_chip(label: &str, active: bool) -> gpui::Div {
        div()
            .h(px(20.0))
            .rounded(px(5.0))
            .px_1p5()
            .border_1()
            .border_color(if active {
                rgba(0x0000002a)
            } else {
                rgba(0x00000012)
            })
            .bg(if active {
                rgba(0x00000016)
            } else {
                rgba(0x00000006)
            })
            .flex()
            .items_center()
            .text_size(px(9.0))
            .font_weight(FontWeight::MEDIUM)
            .text_color(if active {
                rgba(0x000000d9)
            } else {
                rgba(0x00000073)
            })
            .child(label.to_string())
    }
    fn chip(s: &str, c: gpui::Rgba) -> gpui::Div {
        div()
            .h(px(24.0))
            .rounded(px(6.0))
            .px_2()
            .border_1()
            .border_color(rgba(0xffffff12))
            .bg(c)
            .flex()
            .items_center()
            .text_size(px(11.0))
            .font_weight(FontWeight::MEDIUM)
            .text_color(rgb(0xffffff))
            .child(s.to_string())
    }
    fn itab_light(s: &str, sel: bool) -> gpui::Div {
        div()
            .h(px(24.0))
            .rounded(px(6.0))
            .px_2()
            .bg(if sel {
                rgba(0x00000012)
            } else {
                rgba(0x00000005)
            })
            .border_1()
            .border_color(if sel {
                rgba(0x00000018)
            } else {
                rgba(0x0000000a)
            })
            .flex()
            .items_center()
            .text_size(px(11.0))
            .font_weight(FontWeight::MEDIUM)
            .text_color(if sel {
                rgba(0x000000d9)
            } else {
                rgba(0x00000099)
            })
            .child(s.to_string())
    }
    fn settings_tab_button(
        tab: SettingsTab,
        selected: bool,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        div()
            .h(px(30.0))
            .rounded(px(7.0))
            .px_3()
            .bg(if selected {
                rgba(0x4f8cff33)
            } else {
                rgba(0xffffff08)
            })
            .border_1()
            .border_color(if selected {
                rgba(0x4f8cff66)
            } else {
                rgba(0xffffff10)
            })
            .flex()
            .items_center()
            .text_size(px(12.0))
            .font_weight(FontWeight::MEDIUM)
            .text_color(rgb(0xffffff))
            .child(tab.label())
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, _: &MouseDownEvent, _, cx| {
                    this.set_settings_tab(tab);
                    cx.notify();
                }),
            )
    }
    fn settings_value(label: &'static str, value: impl Into<String>) -> gpui::Div {
        div()
            .rounded(px(8.0))
            .border_1()
            .border_color(rgba(0xffffff10))
            .bg(rgba(0xffffff08))
            .px_3()
            .py_2()
            .flex()
            .items_center()
            .justify_between()
            .child(
                div()
                    .text_size(px(12.0))
                    .text_color(rgba(0xffffff99))
                    .child(label),
            )
            .child(
                div()
                    .text_size(px(12.0))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(rgb(0xffffff))
                    .child(value.into()),
            )
    }
    fn sidebar_row(label: &'static str, value: impl Into<String>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_1()
            .rounded(px(8.0))
            .px_3()
            .py_2()
            .bg(rgba(0x00000008))
            .child(
                div()
                    .text_size(px(11.0))
                    .text_color(rgba(0x00000088))
                    .child(label),
            )
            .child(
                div()
                    .text_size(px(13.0))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(rgba(0x000000d9))
                    .child(value.into()),
            )
    }

    fn inspector_section_light(
        title: &'static str,
        rows: Vec<(&'static str, String)>,
    ) -> gpui::Div {
        let mut section = div().flex().flex_col().gap_2().pb_3().child(
            div()
                .text_size(px(11.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(rgba(0x0000008f))
                .child(title),
        );
        for (label, value) in rows {
            section = section.child(
                div()
                    .rounded(px(8.0))
                    .border_1()
                    .border_color(rgba(0x0000000d))
                    .bg(rgb(0xffffff))
                    .px_3()
                    .py_2()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .text_size(px(11.0))
                            .text_color(rgba(0x00000080))
                            .child(label),
                    )
                    .child(
                        div()
                            .text_size(px(12.0))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(rgba(0x000000d9))
                            .child(value),
                    ),
            );
        }
        section
    }
}

impl Render for HomieWorkbench {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let status_label = if self.state.runtime_available {
            "Connected"
        } else {
            "Unavailable"
        };

        let mut session_rows = div().flex().flex_col().gap_1();
        for row in self
            .state
            .sidebar_model
            .rows
            .iter()
            .filter(|row| !row.archived)
            .take(8)
        {
            let session_id = row.id.clone();
            let pin_session_id = row.id.clone();
            let multi_session_id = row.id.clone();
            let archive_session_id = row.id.clone();
            let selected = self.state.session_id.as_deref() == Some(row.id.as_str());
            let multi_selected = self
                .state
                .sidebar_model
                .multi_selected
                .iter()
                .any(|selected| selected == &row.id);
            session_rows = session_rows.child(
                Self::srow(
                    &row.title,
                    &row.status,
                    if row.pinned { "pinned" } else { "local" },
                    if selected {
                        self.status_color()
                    } else {
                        rgba(0xffffff55)
                    },
                    selected,
                )
                .child(
                    Self::sidebar_action_chip(
                        if row.pinned { "pinned" } else { "pin" },
                        row.pinned,
                    )
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _: &MouseDownEvent, _, cx| {
                            this.pin_sidebar_session(&pin_session_id);
                            cx.notify();
                        }),
                    ),
                )
                .child(
                    Self::sidebar_action_chip(
                        if multi_selected { "selected" } else { "select" },
                        multi_selected,
                    )
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _: &MouseDownEvent, _, cx| {
                            this.toggle_sidebar_multi_select(&multi_session_id);
                            cx.notify();
                        }),
                    ),
                )
                .child(Self::sidebar_action_chip("archive", false).on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, _: &MouseDownEvent, _, cx| {
                        this.archive_sidebar_session(&archive_session_id);
                        cx.notify();
                    }),
                ))
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, _: &MouseDownEvent, _, cx| {
                        this.select_session(&session_id);
                        cx.notify();
                    }),
                ),
            );
        }
        let notification_sessions = self
            .state
            .sessions
            .iter()
            .map(|session| NotificationSession {
                id: session.id.clone(),
                title: session.title.clone(),
                status: session.status.clone(),
                needs_input: session.status == "needs_input",
                destructive: false,
                agent_has_approve_deny: true,
            })
            .collect::<Vec<_>>();
        let notifications = notification_rollup(&notification_sessions);

        let mut root = div()
            .size_full()
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, _, cx| {
                this.on_key(event, cx);
            }))
            .on_action(cx.listener(
                |this, _: &ToggleCommandPalette, _, cx| this.toggle_palette(cx),
            ))
            .on_action(cx.listener(|this, _: &ToggleSidebar, _, cx| {
                this.state.worktrees_visible = !this.state.worktrees_visible;
                cx.notify();
            }))
            .on_action(cx.listener(|this, _: &ToggleInspector, _, cx| {
                this.state.inspector_visible = !this.state.inspector_visible;
                cx.notify();
            }))
            .on_action(cx.listener(Self::zoom_in))
            .on_action(cx.listener(Self::zoom_out))
            .on_action(cx.listener(Self::reset_zoom))
            .on_action(cx.listener(Self::copy_sel))
            .on_action(cx.listener(Self::do_paste))
            .bg(rgb(0x12131a))
            .text_color(rgb(0xffffff))
            .flex()
            .child(
                div()
                    .w(px(280.0))
                    .h_full()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .p_4()
                    .text_color(rgba(0x000000d9))
                    .bg(rgb(0xf4f4f2))
                    .border_r_1()
                    .border_color(rgba(0x00000014))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .pb_2()
                            .child(
                                div()
                                    .size(px(12.0))
                                    .rounded_full()
                                    .bg(self.status_color()),
                            )
                            .child(
                                div()
                                    .text_size(px(18.0))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(rgba(0x000000e0))
                                    .child("Homie"),
                            ),
                    )
                    .child(Self::sec_hdr("LIVE SESSIONS", self.state.session_count.max(1)))
                    .child(session_rows)
                    .child(Self::sec_hdr("SURFACES", 3))
                    .child(Self::sidebar_row("Default profile", self.state.default_profile.clone()))
                    .child(Self::sidebar_row("Storage", "settings on demand"))
                    .child(div().flex_1())
                    .child(
                        div()
                            .flex()
                            .gap_2()
                            .child(Self::chip("Cmd+P", rgba(0x4f8cff33)))
                            .child(Self::chip("Cmd+B", rgba(0x00000022)))
                            .child(Self::chip("Cmd+I", rgba(0x00000022))),
                    )
                    .child(
                        div()
                            .text_size(px(11.0))
                            .line_height(px(16.0))
                            .text_color(rgba(0x00000078))
                            .child(self.state.data_dir.display().to_string()),
                    ),
            )
            .child(
                div()
                    .flex_1()
                    .h_full()
                    .flex()
                    .flex_col()
                    .p_6()
                    .gap_5()
                    .child(
                        div()
                            .flex()
                            .items_start()
                            .justify_between()
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_2()
                                    .child(
                                        div()
                                            .text_size(px(26.0))
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .child("Agent workbench"),
                                    )
                                    .child(
                                        div()
                                            .text_size(px(13.0))
                                            .text_color(rgba(0xffffffaa))
                                            .child("Local sessions, terminal output, artifacts, and inspector surfaces share one Diri-aligned shell."),
                                    ),
                            )
                            .child(
                                div().flex().gap_2()
                                    .child(Self::chip("terminal", rgba(0xffffff10)))
                                    .child(Self::chip("inspector", rgba(0xffffff10)))
                                    .child(Self::chip("usage", rgba(0x34c75922))),
                            ),
                    )
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .gap_4()
                            .child(
                                div()
                                    .flex_1()
                                    .rounded(px(12.0))
                                    .border_1()
                                    .border_color(rgba(0xffffff12))
                                    .bg(rgb(0x0d0f14))
                                    .overflow_hidden()
                                    .child(self.terminal.clone()),
                            )
                            .when(self.state.inspector_visible, |element| {
                                element.child(
                                    div()
                                        .w(px(280.0))
                                        .h_full()
                                        .rounded(px(12.0))
                                        .border_1()
                                        .border_color(rgba(0x00000014))
                                        .bg(rgb(0xf4f4f2))
                                        .p_4()
                                        .flex()
                                        .flex_col()
                                        .gap_3()
                                        .child(
                                            div()
                                                .flex()
                                                .gap_2()
                                                .child(Self::itab_light("Info", true))
                                                .child(Self::itab_light("Changes", false))
                                                .child(Self::itab_light("Artifacts", false)),
                                        )
                                        .child(Self::inspector_section_light(
                                            "SESSION",
                                            vec![
                                                ("Runtime", self.state.runtime_status.clone()),
                                                ("Profile", self.state.default_profile.clone()),
                                                ("Session", self.state.session_id.clone().unwrap_or_else(|| "none".to_string())),
                                                ("Storage", "settings on demand".to_string()),
                                            ],
                                        ))
                                        .child(Self::inspector_section_light(
                                            "ARTIFACTS",
                                            vec![
                                                (
                                                    "Ports",
                                                    self.state.artifact_summary.ports.to_string(),
                                                ),
                                                (
                                                    "PRs",
                                                    self.state
                                                        .artifact_summary
                                                        .pull_requests
                                                        .to_string(),
                                                ),
                                                (
                                                    "Previews",
                                                    self.state
                                                        .artifact_summary
                                                        .previews
                                                        .to_string(),
                                                ),
                                                (
                                                    "Links",
                                                    self.state.artifact_summary.links.to_string(),
                                                ),
                                            ],
                                        ))
                                        .child(Self::inspector_section_light(
                                            "NOTIFICATIONS",
                                            vec![
                                                ("Rollup", notifications.badge()),
                                                ("Needs input", notifications.needs_input.to_string()),
                                                ("Actions", notifications.items.iter().map(|item| item.actions.len()).sum::<usize>().to_string()),
                                            ],
                                        ))
                                        .child(div().flex_1())
                                        .child(
                                            div()
                                                .text_size(px(11.0))
                                                .line_height(px(16.0))
                                                    .text_color(rgba(0x00000073))
                                                .child("This shell is backed by a real local PTY session owned by Homie runtime."),
                                        ),
                                )
                            }),
                    )
                    .child(
                        div()
                            .h(px(36.0))
                            .flex_1()
                            .flex()
                            .items_center()
                            .justify_between()
                            .text_size(px(11.0))
                            .text_color(rgba(0xffffff80))
                            .border_t_1()
                            .border_color(rgba(0xffffff10))
                            .child(
                                div().child(format!(
                                    "{} session records · {}",
                                    self.state.session_count, status_label
                                )),
                            )
                            .child(div().child(self.state.terminal_notice.clone())),
                    ),
            );

        if let Some(palette) = &self.palette {
            let mut entries = div().flex().flex_col().gap_1();
            for (index, item) in palette.matches().iter().take(6).enumerate() {
                entries = entries.child(
                    div()
                        .h(px(28.0))
                        .rounded(px(7.0))
                        .px_2()
                        .flex()
                        .items_center()
                        .justify_between()
                        .bg(if index == palette.selected {
                            rgba(0xffffff18)
                        } else {
                            rgba(0xffffff00)
                        })
                        .child(
                            div()
                                .text_size(px(13.0))
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(rgb(0xffffff))
                                .child(item.title.clone()),
                        )
                        .child(
                            div()
                                .text_size(px(11.0))
                                .text_color(rgba(0xffffff80))
                                .child(item.shortcut.unwrap_or("").to_string()),
                        ),
                );
            }
            root = root.child(
                div()
                    .absolute()
                    .top(px(72.0))
                    .left(px(360.0))
                    .w(px(420.0))
                    .rounded(px(12.0))
                    .border_1()
                    .border_color(rgba(0xffffff18))
                    .bg(rgb(0x20232d))
                    .p_3()
                    .shadow_lg()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        div()
                            .text_size(px(12.0))
                            .text_color(rgba(0xffffff80))
                            .child(format!("Command palette · {}", palette.query())),
                    )
                    .child(entries),
            );
        }

        if self.state.quick_open_visible {
            let mut entries = div().flex().flex_col().gap_1();
            for item in self.quick_open_items().into_iter().take(8) {
                let target = item.target.clone();
                entries = entries.child(
                    div()
                        .h(px(36.0))
                        .rounded(px(7.0))
                        .px_2()
                        .flex()
                        .items_center()
                        .justify_between()
                        .bg(rgba(0xffffff08))
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .child(
                                    div()
                                        .text_size(px(13.0))
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(rgb(0xffffff))
                                        .child(item.label),
                                )
                                .child(
                                    div()
                                        .text_size(px(10.0))
                                        .text_color(rgba(0xffffff80))
                                        .child(item.detail),
                                ),
                        )
                        .child(
                            div()
                                .text_size(px(10.0))
                                .text_color(rgba(0xffffff66))
                                .child("open"),
                        )
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(move |this, _: &MouseDownEvent, _, cx| {
                                this.activate_quick_open(target.clone());
                                cx.notify();
                            }),
                        ),
                );
            }
            root = root.child(
                div()
                    .absolute()
                    .top(px(104.0))
                    .left(px(360.0))
                    .w(px(460.0))
                    .rounded(px(12.0))
                    .border_1()
                    .border_color(rgba(0xffffff18))
                    .bg(rgb(0x20232d))
                    .p_3()
                    .shadow_lg()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .text_size(px(12.0))
                                    .text_color(rgba(0xffffff80))
                                    .child("Quick Open"),
                            )
                            .child(Self::chip("Esc", rgba(0xffffff10)).on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|this, _: &MouseDownEvent, _, cx| {
                                    this.dismiss_quick_open();
                                    cx.notify();
                                }),
                            )),
                    )
                    .child(entries),
            );
        }

        if self.state.find_visible {
            root = root.child(
                div()
                    .absolute()
                    .top(px(108.0))
                    .left(px(440.0))
                    .w(px(420.0))
                    .rounded(px(10.0))
                    .border_1()
                    .border_color(rgba(0xffffff18))
                    .bg(rgb(0x20232d))
                    .shadow_lg()
                    .p_3()
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(
                        div()
                            .text_size(px(12.0))
                            .text_color(rgba(0xffffff80))
                            .child("Find"),
                    )
                    .child(
                        div()
                            .flex_1()
                            .rounded(px(7.0))
                            .bg(rgba(0xffffff10))
                            .px_2()
                            .py_1()
                            .text_size(px(12.0))
                            .text_color(rgb(0xffffff))
                            .child(if self.state.find_model.query().is_empty() {
                                "type to search terminal output".to_string()
                            } else {
                                self.state.find_model.query().to_string()
                            }),
                    )
                    .child(
                        div()
                            .text_size(px(11.0))
                            .text_color(rgba(0xffffff80))
                            .child(format!("{} matches", self.state.find_model.matches().len())),
                    )
                    .child(Self::chip("Esc", rgba(0xffffff10)).on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _: &MouseDownEvent, _, cx| {
                            this.dismiss_find();
                            cx.notify();
                        }),
                    )),
            );
        }

        if self.state.worktrees_visible {
            let mut rows = div().flex().flex_col().gap_1();
            for worktree in self.state.worktrees.iter().take(8) {
                rows = rows.child(
                    div()
                        .rounded(px(7.0))
                        .px_2()
                        .py_2()
                        .bg(rgba(0xffffff08))
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .child(
                                    div()
                                        .text_size(px(12.0))
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(rgb(0xffffff))
                                        .child(worktree.path.clone()),
                                )
                                .child(
                                    div()
                                        .text_size(px(10.0))
                                        .text_color(rgba(0xffffff80))
                                        .child(worktree.status.clone()),
                                ),
                        )
                        .child(Self::chip(
                            if worktree.cleanup { "cleanup" } else { "keep" },
                            if worktree.cleanup {
                                rgba(0xffcc0033)
                            } else {
                                rgba(0xffffff10)
                            },
                        )),
                );
            }
            root = root.child(
                div()
                    .absolute()
                    .top(px(120.0))
                    .left(px(320.0))
                    .w(px(560.0))
                    .rounded(px(12.0))
                    .border_1()
                    .border_color(rgba(0xffffff18))
                    .bg(rgb(0x20232d))
                    .shadow_lg()
                    .p_4()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .text_size(px(16.0))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child("Worktrees"),
                            )
                            .child(Self::chip("Close", rgba(0xffffff10)).on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|this, _: &MouseDownEvent, _, cx| {
                                    this.state.worktrees_visible = false;
                                    cx.notify();
                                }),
                            )),
                    )
                    .child(rows),
            );
        }

        if let Some(summary) = self.state.needs_input_summary.clone() {
            root = root.child(
                div()
                    .absolute()
                    .top(px(148.0))
                    .left(px(520.0))
                    .w(px(360.0))
                    .rounded(px(12.0))
                    .border_1()
                    .border_color(rgba(0x00000020))
                    .bg(rgb(0xf4f4f2))
                    .shadow_lg()
                    .p_4()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(
                        div()
                            .text_size(px(15.0))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(rgba(0x000000d9))
                            .child("Permission required"),
                    )
                    .child(
                        div()
                            .text_size(px(12.0))
                            .line_height(px(18.0))
                            .text_color(rgba(0x00000099))
                            .child(summary),
                    )
                    .child(
                        div()
                            .flex()
                            .justify_end()
                            .gap_2()
                            .child(Self::chip("Deny", rgba(0x00000022)).on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|this, _: &MouseDownEvent, _, cx| {
                                    this.deny_needs_input();
                                    cx.notify();
                                }),
                            ))
                            .child(Self::chip("Approve", rgba(0x34c75966)).on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|this, _: &MouseDownEvent, _, cx| {
                                    this.approve_needs_input();
                                    cx.notify();
                                }),
                            )),
                    ),
            );
        }

        if self.state.settings_visible {
            let prefs = self.state.settings_preferences.clone();
            let tab = self.state.settings_tab;
            let mut body = div().flex().flex_col().gap_2();
            body = match tab {
                SettingsTab::General => body
                    .child(Self::settings_value(
                        "Startup",
                        prefs.startup_behavior.clone(),
                    ))
                    .child(Self::settings_value(
                        "Default profile",
                        self.state.default_profile.clone(),
                    ))
                    .child(Self::settings_value("Storage", "settings on demand")),
                SettingsTab::Terminal => body
                    .child(Self::settings_value(
                        "Font size",
                        format!("{} px", prefs.terminal_font_size),
                    ))
                    .child(
                        div()
                            .flex()
                            .gap_2()
                            .child(Self::chip("A-", rgba(0xffffff10)).on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|this, _: &MouseDownEvent, _, cx| {
                                    this.bump_terminal_font_size(-1);
                                    cx.notify();
                                }),
                            ))
                            .child(Self::chip("A+", rgba(0x4f8cff33)).on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|this, _: &MouseDownEvent, _, cx| {
                                    this.bump_terminal_font_size(1);
                                    cx.notify();
                                }),
                            )),
                    ),
                SettingsTab::Resources => body
                    .child(Self::settings_value(
                        "Hibernate idle",
                        format!("{} minutes", prefs.hibernate_idle_minutes),
                    ))
                    .child(Self::settings_value("Resource policy", "local holder")),
                SettingsTab::Remote => body
                    .child(Self::settings_value(
                        "Companion access",
                        if prefs.remote_companion_access {
                            "enabled"
                        } else {
                            "disabled"
                        },
                    ))
                    .child(
                        Self::chip(
                            if prefs.remote_companion_access {
                                "Disable companion"
                            } else {
                                "Enable companion"
                            },
                            rgba(0x4f8cff33),
                        )
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(|this, _: &MouseDownEvent, _, cx| {
                                this.toggle_remote_companion_access();
                                cx.notify();
                            }),
                        ),
                    ),
            };
            root = root.child(
                div()
                    .absolute()
                    .top(px(96.0))
                    .right(px(32.0))
                    .w(px(420.0))
                    .rounded(px(12.0))
                    .border_1()
                    .border_color(rgba(0xffffff18))
                    .bg(rgb(0x20232d))
                    .shadow_lg()
                    .p_4()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .text_size(px(16.0))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child("Settings"),
                            )
                            .child(Self::chip("Close", rgba(0xffffff10)).on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|this, _: &MouseDownEvent, _, cx| {
                                    this.state.settings_visible = false;
                                    cx.notify();
                                }),
                            )),
                    )
                    .child(
                        div()
                            .flex()
                            .gap_2()
                            .child(Self::settings_tab_button(
                                SettingsTab::General,
                                tab == SettingsTab::General,
                                cx,
                            ))
                            .child(Self::settings_tab_button(
                                SettingsTab::Terminal,
                                tab == SettingsTab::Terminal,
                                cx,
                            ))
                            .child(Self::settings_tab_button(
                                SettingsTab::Resources,
                                tab == SettingsTab::Resources,
                                cx,
                            ))
                            .child(Self::settings_tab_button(
                                SettingsTab::Remote,
                                tab == SettingsTab::Remote,
                                cx,
                            )),
                    )
                    .child(body),
            );
        }

        root
    }
}

fn main() {
    application().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(1100.0), px(700.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                window_min_size: Some(size(px(MIN_WINDOW_WIDTH), px(MIN_WINDOW_HEIGHT))),
                window_background: WindowBackgroundAppearance::Opaque,
                app_id: Some("com.superops.homie".to_string()),
                ..Default::default()
            },
            |_, cx| cx.new(HomieWorkbench::load),
        )
        .expect("failed to open Homie window");
        cx.activate(true);
    });
}

fn default_data_dir() -> PathBuf {
    user_paths::default_data_dir().unwrap_or_else(|_| default_workspace().join(".homie"))
}

fn default_workspace() -> PathBuf {
    std::env::current_dir()
        .ok()
        .filter(|path| path.is_dir())
        .unwrap_or_else(|| PathBuf::from("/"))
}

fn open_ready_storage(
    data_dir: PathBuf,
) -> Result<homie_storage::Storage, homie_storage::StorageError> {
    let storage = open_or_create(StorageConfig { data_dir })?;
    storage.migrate()?;
    storage.seed_defaults()?;
    Ok(storage)
}

fn connection_label(connection: &BridgeConnectionState) -> &'static str {
    match connection {
        BridgeConnectionState::Connecting => "connecting",
        BridgeConnectionState::Connected => "connected",
        BridgeConnectionState::Degraded => "degraded",
        BridgeConnectionState::Reconnecting => "reconnecting",
        BridgeConnectionState::Unavailable => "unavailable",
        BridgeConnectionState::Shutdown => "shutdown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terminal_attachment_is_confirmed_only_after_success_event() {
        let mut attachment = TerminalAttachmentState::default();

        assert_eq!(
            attachment.begin_attach(Some("session-1")),
            Some("session-1".to_string())
        );
        assert_eq!(attachment.attached_session_id(), None);
        assert_eq!(attachment.begin_attach(Some("session-1")), None);

        attachment.apply_event(
            Some("session-1"),
            &BridgeEvent::TerminalAttached {
                session_id: "session-1".to_string(),
            },
        );

        assert_eq!(attachment.attached_session_id(), Some("session-1"));
    }

    #[test]
    fn terminal_unavailable_clears_attachment_for_retry() {
        let mut attachment = confirmed_attachment("session-1");

        attachment.apply_event(
            Some("session-1"),
            &BridgeEvent::TerminalUnavailable {
                last_confirmed_offset: 12,
            },
        );

        assert_eq!(attachment.attached_session_id(), None);
        assert_eq!(attachment.retry_offset("session-1"), 12);
        assert_eq!(
            attachment.begin_attach(Some("session-1")),
            Some("session-1".to_string())
        );
    }

    #[test]
    fn terminal_open_failure_clears_pending_attachment_for_retry() {
        let mut attachment = TerminalAttachmentState::default();
        assert_eq!(
            attachment.begin_attach(Some("session-1")),
            Some("session-1".to_string())
        );

        attachment.apply_event(
            Some("session-1"),
            &BridgeEvent::CommandFailed {
                command: "terminal.open",
                code: "unavailable".to_string(),
            },
        );

        assert_eq!(
            attachment.begin_attach(Some("session-1")),
            Some("session-1".to_string())
        );
    }

    #[test]
    fn dispatch_failure_clears_pending_attachment_for_retry() {
        let mut attachment = TerminalAttachmentState::default();
        assert_eq!(
            attachment.begin_attach(Some("session-1")),
            Some("session-1".to_string())
        );

        attachment.dispatch_failed();

        assert_eq!(
            attachment.begin_attach(Some("session-1")),
            Some("session-1".to_string())
        );
    }

    #[test]
    fn selection_change_rebinds_after_new_session_confirmation() {
        let mut attachment = confirmed_attachment("session-1");

        assert_eq!(
            attachment.begin_attach(Some("session-2")),
            Some("session-2".to_string())
        );
        assert_eq!(attachment.attached_session_id(), Some("session-1"));

        attachment.apply_event(
            Some("session-2"),
            &BridgeEvent::TerminalAttached {
                session_id: "session-2".to_string(),
            },
        );

        assert_eq!(attachment.attached_session_id(), Some("session-2"));
    }

    fn confirmed_attachment(session_id: &str) -> TerminalAttachmentState {
        let mut attachment = TerminalAttachmentState::default();
        assert_eq!(
            attachment.begin_attach(Some(session_id)),
            Some(session_id.to_string())
        );
        attachment.apply_event(
            Some(session_id),
            &BridgeEvent::TerminalAttached {
                session_id: session_id.to_string(),
            },
        );
        attachment
    }
}
