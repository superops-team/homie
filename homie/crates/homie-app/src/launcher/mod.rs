//! Compact new-session destination opened in the main pane by Command-N.

use std::path::Path;
use std::sync::Arc;

use gpui::{
    AnyElement, App, Context, EventEmitter, FocusHandle, Focusable, FontWeight, HighlightStyle,
    KeyDownEvent, MouseButton, PathPromptOptions, Render, Task, Window, div, prelude::*, px, rgba,
};
use homie_proto::{AgentKind, Project};
use homie_ui::{
    AgentKind as UiAgentKind, AgentLogo, Fill, FloatingSurface, Palette, Radius, SemanticColors,
};

use crate::AppServices;
use crate::composer::PromptComposer;
use crate::macos::sf_symbols::{SymbolWeight, sf_symbol, sf_symbol_weighted};
use crate::navigation::CARET;
use crate::query_editor::{self, ClipboardEdit, Edit};
use crate::store::SpawnOptions;

const PANEL_WIDTH: f32 = 540.0;
const TITLE_HEIGHT: f32 = 36.0;
const TITLE_GAP: f32 = 22.0;
const CONTROL_SIZE: f32 = 32.0;
const CONTROL_RADIUS: f32 = 9.0;
const SHELF_HEIGHT: f32 = 40.0;
const PICKER_HEIGHT: f32 = 200.0;

/// Composer metrics. The text area is sized from the wrapped line count
/// rather than pinned at one height: a one-line prompt should not sit in a
/// half-empty box, and a twenty-line one should not vanish out of the bottom
/// of a fixed one — it grows to [`COMPOSER_MAX_LINES`] and then scrolls,
/// following the caret.
const COMPOSER_FONT_SIZE: f32 = 13.0;
const COMPOSER_LINE_HEIGHT: f32 = 19.0;
const COMPOSER_MIN_LINES: usize = 3;
const COMPOSER_MAX_LINES: usize = 9;
const COMPOSER_INSET: f32 = 8.0;
const COMPOSER_PADDING: f32 = 16.0;
const COMPOSER_PAD_TOP: f32 = 12.0;
const COMPOSER_PAD_BOTTOM: f32 = 6.0;
const COMPOSER_CONTROLS_HEIGHT: f32 = 44.0;

/// The width text actually wraps at, derived from the panel so the two cannot
/// drift apart: the panel, less the composer's margin, padding and border.
const COMPOSER_TEXT_WIDTH: f32 = PANEL_WIDTH - 2.0 * COMPOSER_INSET - 2.0 * COMPOSER_PADDING - 2.0;

const fn composer_text_height(lines: usize) -> f32 {
    let visible = if lines < COMPOSER_MIN_LINES {
        COMPOSER_MIN_LINES
    } else if lines > COMPOSER_MAX_LINES {
        COMPOSER_MAX_LINES
    } else {
        lines
    };
    visible as f32 * COMPOSER_LINE_HEIGHT + COMPOSER_PAD_TOP + COMPOSER_PAD_BOTTOM
}

#[derive(Clone)]
struct HarnessChoice {
    kind: AgentKind,
    label: String,
    available: bool,
}

pub(crate) enum LauncherEvent {
    Closed,
}

pub(crate) struct LauncherOverlay {
    services: Arc<AppServices>,
    focus: FocusHandle,
    prompt: PromptComposer,
    selected_harness: AgentKind,
    selected_root: String,
    /// Which picker, if any, is open — and where its keyboard highlight sits,
    /// so both are reachable without the mouse.
    picker: Option<Picker>,
    highlight: usize,
    open: bool,
    preview: bool,
    _store_changes: Task<()>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Picker {
    Harness,
    Project,
}

impl EventEmitter<LauncherEvent> for LauncherOverlay {}

impl LauncherOverlay {
    pub(crate) fn new(services: Arc<AppServices>, preview: bool, cx: &mut Context<Self>) -> Self {
        let focus = cx.focus_handle();
        let (selected_harness, selected_root) = initial_target(&services);
        let mut changes = services.store.changes();
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
            services,
            focus,
            prompt: PromptComposer::default(),
            selected_harness,
            selected_root,
            picker: None,
            highlight: 0,
            open: false,
            preview,
            _store_changes: store_changes,
        }
    }

    pub(crate) const fn is_open(&self) -> bool {
        self.open
    }

    pub(crate) fn open(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        // A half-written prompt survives Escape. This used to clear on every
        // open, so closing the launcher by reflex — or bouncing off it to
        // check something — threw the prompt away with no way back. It is
        // cleared on submit, and only there.
        if self.prompt.is_empty() {
            let (harness, root) = initial_target(&self.services);
            self.selected_harness = harness;
            self.selected_root = root;
        }
        self.picker = None;
        self.open = true;
        window.focus(&self.focus, cx);
        cx.notify();
    }

    pub(crate) fn focus(&self, window: &mut Window, cx: &mut Context<Self>) {
        window.focus(&self.focus, cx);
    }

    fn close(&mut self, cx: &mut Context<Self>) {
        if !self.open {
            return;
        }
        self.open = false;
        self.picker = None;
        cx.emit(LauncherEvent::Closed);
        cx.notify();
    }

    fn harness_choices(&self) -> Vec<HarnessChoice> {
        let store = self
            .services
            .store
            .store
            .read()
            .expect("session store lock poisoned");
        let catalog = &store.agent_catalog().agents;
        if catalog.is_empty() {
            return [
                (AgentKind::CLAUDE_CODE, "Claude Code"),
                (AgentKind::CODEX, "Codex"),
                (AgentKind::CURSOR, "Cursor"),
                (AgentKind::GEMINI, "Gemini"),
            ]
            .into_iter()
            .map(|(kind, label)| HarnessChoice {
                kind,
                label: label.to_owned(),
                available: true,
            })
            .collect();
        }

        catalog
            .iter()
            .filter(|item| !item.kind.is_terminal())
            .map(|item| HarnessChoice {
                kind: item.kind.clone(),
                label: item
                    .descriptor
                    .as_ref()
                    .map(|descriptor| descriptor.display_name.clone())
                    .filter(|label| !label.is_empty())
                    .unwrap_or_else(|| title_case_id(item.kind.id())),
                available: item.available(),
            })
            .collect()
    }

    fn projects(&self) -> Vec<Project> {
        let store = self
            .services
            .store
            .store
            .read()
            .expect("session store lock poisoned");
        let mut projects: Vec<_> = store.projects().values().cloned().collect();
        projects.sort_by(|left, right| {
            left.pinned_order
                .unwrap_or(i64::MAX)
                .cmp(&right.pinned_order.unwrap_or(i64::MAX))
                .then_with(|| left.name.to_lowercase().cmp(&right.name.to_lowercase()))
        });
        projects
    }

    fn selected_harness_label(&self) -> String {
        self.harness_choices()
            .into_iter()
            .find(|choice| choice.kind == self.selected_harness)
            .map(|choice| choice.label)
            .unwrap_or_else(|| title_case_id(self.selected_harness.id()))
    }

    fn selected_project_label(&self) -> String {
        self.projects()
            .into_iter()
            .find(|project| project.root == self.selected_root)
            .map(|project| project.name)
            .or_else(|| {
                Path::new(&self.selected_root)
                    .file_name()
                    .and_then(|name| name.to_str())
                    .map(str::to_owned)
            })
            .filter(|name| !name.is_empty())
            .unwrap_or_else(|| "Choose project".to_owned())
    }

    /// Why the prompt cannot be sent yet, as something to show the user.
    /// `None` means it can. The submit button used to just sit there dimmed
    /// with no explanation, which reads as "broken" rather than "not yet".
    fn blocker(&self) -> Option<String> {
        if self.selected_root.is_empty() {
            return Some("Choose a project to start in".to_owned());
        }
        match self
            .harness_choices()
            .into_iter()
            .find(|choice| choice.kind == self.selected_harness)
        {
            Some(choice) if choice.available => None,
            Some(choice) => Some(format!("{} is not installed", choice.label)),
            None => Some(format!(
                "{} is not available on this machine",
                self.selected_harness_label()
            )),
        }
    }

    fn can_submit(&self) -> bool {
        !self.preview && !self.prompt.text().trim().is_empty() && self.blocker().is_none()
    }

    fn submit(&mut self, cx: &mut Context<Self>) -> bool {
        if !self.can_submit() {
            return false;
        }
        let prompt = self.prompt.text().trim().to_owned();
        self.services
            .store
            .store
            .write()
            .expect("session store lock poisoned")
            .spawn_kind(
                self.selected_harness.clone(),
                SpawnOptions {
                    cwd: Some(self.selected_root.clone()),
                    initial_prompt: Some(prompt),
                    ..SpawnOptions::default()
                },
            );
        self.prompt.clear();
        self.close(cx);
        true
    }

    pub(crate) fn handle_key_down(
        &mut self,
        event: &KeyDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if self.picker.is_some() && self.handle_picker_key(event, cx) {
            return true;
        }
        let shift = event.keystroke.modifiers.shift;
        match event.keystroke.key.as_str() {
            "escape" => {
                self.close(cx);
                true
            }
            "enter" if shift => {
                self.prompt.insert_multiline("\n");
                cx.notify();
                true
            }
            "enter" => self.submit(cx),
            // Cycling the agent from the keyboard: the picker was mouse-only,
            // which is a strange thing to require of a surface you reached
            // with ⌘N and are about to leave with ↵.
            "tab" => {
                self.cycle_harness(if shift { -1 } else { 1 });
                cx.notify();
                true
            }
            "up" => {
                self.prompt.move_up(shift);
                cx.notify();
                true
            }
            "down" => {
                self.prompt.move_down(shift);
                cx.notify();
                true
            }
            _ => self.edit_prompt(event, cx),
        }
    }

    /// Arrow keys drive the open picker instead of the prompt behind it.
    fn handle_picker_key(&mut self, event: &KeyDownEvent, cx: &mut Context<Self>) -> bool {
        let count = match self.picker {
            Some(Picker::Harness) => self.harness_choices().len(),
            Some(Picker::Project) => self.projects().len(),
            None => return false,
        };
        match event.keystroke.key.as_str() {
            "escape" => {
                self.picker = None;
                cx.notify();
                true
            }
            "up" | "down" if count > 0 => {
                self.highlight = if event.keystroke.key == "up" {
                    self.highlight.saturating_sub(1)
                } else {
                    (self.highlight + 1).min(count - 1)
                };
                cx.notify();
                true
            }
            "enter" => {
                self.commit_highlight();
                cx.notify();
                true
            }
            _ => false,
        }
    }

    fn commit_highlight(&mut self) {
        match self.picker {
            Some(Picker::Harness) => {
                if let Some(choice) = self
                    .harness_choices()
                    .get(self.highlight)
                    .filter(|choice| choice.available)
                {
                    self.selected_harness = choice.kind.clone();
                }
            }
            Some(Picker::Project) => {
                if let Some(project) = self.projects().get(self.highlight) {
                    self.selected_root.clone_from(&project.root);
                }
            }
            None => return,
        }
        self.picker = None;
    }

    fn toggle_picker(&mut self, picker: Picker) {
        if self.picker == Some(picker) {
            self.picker = None;
            return;
        }
        self.highlight = match picker {
            Picker::Harness => self
                .harness_choices()
                .iter()
                .position(|choice| choice.kind == self.selected_harness),
            Picker::Project => self
                .projects()
                .iter()
                .position(|project| project.root == self.selected_root),
        }
        .unwrap_or(0);
        self.picker = Some(picker);
    }

    /// Steps to the next installed agent, skipping any that cannot run.
    fn cycle_harness(&mut self, delta: isize) {
        let choices: Vec<_> = self
            .harness_choices()
            .into_iter()
            .filter(|choice| choice.available)
            .collect();
        if choices.is_empty() {
            return;
        }
        let current = choices
            .iter()
            .position(|choice| choice.kind == self.selected_harness)
            .unwrap_or(0);
        let count = choices.len() as isize;
        let next = (current as isize + delta).rem_euclid(count) as usize;
        self.selected_harness = choices[next].kind.clone();
    }

    fn edit_prompt(&mut self, event: &KeyDownEvent, cx: &mut Context<Self>) -> bool {
        let Some(edit) = query_editor::edit_for(&event.keystroke) else {
            return false;
        };
        match edit {
            Edit::Local(local) => {
                self.prompt.apply(local);
            }
            Edit::Clipboard(ClipboardEdit::Copy) => {
                query_editor::copy_selection(self.prompt.editor(), cx);
            }
            Edit::Clipboard(ClipboardEdit::Cut) => {
                query_editor::cut_selection(self.prompt.editor_mut(), cx);
            }
            Edit::Clipboard(ClipboardEdit::Paste) => {
                if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
                    self.prompt.insert_multiline(&text);
                }
            }
        }
        cx.notify();
        true
    }

    fn choose_folder(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.picker = None;
        let paths = cx.prompt_for_paths(PathPromptOptions {
            files: false,
            directories: true,
            multiple: false,
            prompt: Some("Start Here".into()),
        });
        cx.spawn_in(window, async move |this, cx| {
            let Ok(Ok(Some(mut paths))) = paths.await else {
                return;
            };
            let Some(path) = paths.pop() else {
                return;
            };
            let _ = this.update_in(cx, |this, _window, cx| {
                this.selected_root = path.to_string_lossy().into_owned();
                cx.notify();
            });
        })
        .detach();
    }
}

impl Focusable for LauncherOverlay {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

fn initial_target(services: &AppServices) -> (AgentKind, String) {
    let store = services
        .store
        .store
        .read()
        .expect("session store lock poisoned");
    let selected_root = store
        .selected_session()
        .and_then(|session| store.projects().get(&session.project_id))
        .map(|project| project.root.clone())
        .or_else(|| {
            store
                .projects()
                .values()
                .min_by(|left, right| left.name.cmp(&right.name))
                .map(|project| project.root.clone())
        })
        .unwrap_or_default();
    (store.preferences().default_agent.kind(), selected_root)
}

fn ui_agent_kind(kind: &AgentKind) -> UiAgentKind {
    match kind.id() {
        AgentKind::CLAUDE_CODE_ID => UiAgentKind::ClaudeCode,
        AgentKind::CODEX_ID => UiAgentKind::Codex,
        AgentKind::CURSOR_ID => UiAgentKind::Cursor,
        AgentKind::GEMINI_ID => UiAgentKind::Gemini,
        AgentKind::SHELL_ID => UiAgentKind::Shell,
        _ => UiAgentKind::Generic,
    }
}

fn title_case_id(id: &str) -> String {
    id.split(['-', '_'])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            chars.next().map_or_else(String::new, |first| {
                first.to_uppercase().collect::<String>() + chars.as_str()
            })
        })
        .collect::<Vec<_>>()
        .join(" ")
}

mod render;
#[cfg(test)]
mod tests;
