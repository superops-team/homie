//! Native, read-only code viewer for the trailing workbench.
//!
//! `code_intelligence` owns filesystem discovery, containment and loading.
//! This module owns only the presentation state: asynchronous opens, source
//! history, line targeting, virtualization, and lightweight lexical color.

use std::ops::Range;
use std::path::PathBuf;
use std::sync::Arc;

use gpui::{
    AnyElement, App, Context, FocusHandle, Focusable, FontWeight, HighlightStyle, KeyDownEvent,
    ListHorizontalSizingBehavior, MouseButton, Render, ScrollStrategy, SharedString, StyledText,
    Task, UniformListScrollHandle, Window, div, prelude::*, px, rgba, uniform_list,
};

use crate::code_intelligence::{
    CodeIntelligence, CodeIntelligenceError, SearchHit, SearchHitKind, SourceSnapshot,
};
use crate::macos::sf_symbols::{SymbolWeight, sf_symbol, sf_symbol_weighted};
use crate::query_editor::{self, ClipboardEdit, Edit, QueryEditor};
use homie_ui::{FloatingSurface, Radius, SemanticColors, Typo};

#[cfg(test)]
use crate::code_intelligence::SourceTarget;
const SOURCE_ROW_HEIGHT: f32 = 20.0;
const SOURCE_GUTTER_WIDTH: f32 = 52.0;
#[derive(Clone)]
enum ViewerState {
    Empty,
    Loading { reference: String },
    Ready(Arc<SourceSnapshot>),
    Error { reference: String, message: String },
}

pub struct CodeViewer {
    tokio: tokio::runtime::Handle,
    focus: FocusHandle,
    workspace_cwd: Option<PathBuf>,
    intelligence: Option<Arc<CodeIntelligence>>,
    state: ViewerState,
    scroll: UniformListScrollHandle,
    generation: u64,
    _load_task: Option<Task<()>>,
    _search_task: Option<Task<()>>,
    search_generation: u64,
    picker_open: bool,
    query: QueryEditor,
    results: Vec<SearchHit>,
    highlighted_result: usize,
    history: Vec<(PathBuf, String)>,
    history_index: usize,
}
impl CodeViewer {
    pub fn new(tokio: tokio::runtime::Handle, cx: &mut Context<Self>) -> Self {
        Self {
            tokio,
            focus: cx.focus_handle(),
            workspace_cwd: None,
            intelligence: None,
            state: ViewerState::Empty,
            scroll: UniformListScrollHandle::new(),
            generation: 0,
            _load_task: None,
            _search_task: None,
            search_generation: 0,
            picker_open: false,
            query: QueryEditor::default(),
            results: Vec::new(),
            highlighted_result: 0,
            history: Vec::new(),
            history_index: 0,
        }
    }

    fn toggle_picker(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.picker_open = !self.picker_open;
        if self.picker_open {
            window.focus(&self.focus, cx);
            self.schedule_search(cx);
        } else {
            self.query.clear();
            self.results.clear();
            self.highlighted_result = 0;
        }
        cx.notify();
    }

    fn schedule_search(&mut self, cx: &mut Context<Self>) {
        let intelligence = self.intelligence.clone();
        let workspace_cwd = self.workspace_cwd.clone();
        if intelligence.is_none() && workspace_cwd.is_none() {
            self.results.clear();
            return;
        }
        self.search_generation = self.search_generation.wrapping_add(1);
        let generation = self.search_generation;
        let query = self.query.text().to_owned();
        let tokio = self.tokio.clone();
        self._search_task = Some(cx.spawn(async move |this, cx| {
            let result = tokio
                .spawn_blocking(move || {
                    let intelligence = match intelligence {
                        Some(intelligence) => intelligence,
                        None => Arc::new(CodeIntelligence::for_session(workspace_cwd?).ok()?),
                    };
                    let results = intelligence.search(&query, 40);
                    Some((intelligence, results))
                })
                .await
                .ok()
                .flatten();
            let _ = this.update(cx, |this, cx| {
                if this.search_generation != generation || !this.picker_open {
                    return;
                }
                if let Some((intelligence, results)) = result {
                    this.intelligence = Some(intelligence);
                    this.results = results;
                } else {
                    this.results.clear();
                }
                this.highlighted_result = 0;
                cx.notify();
            });
        }));
    }

    /// Selects the local workspace represented by the active agent. The file
    /// index remains lazy, but the picker can now be used before a source file
    /// has been opened. Switching agents clears stale source and history.
    pub fn set_workspace(&mut self, cwd: Option<PathBuf>, cx: &mut Context<Self>) {
        if self.workspace_cwd == cwd {
            return;
        }
        self.workspace_cwd = cwd;
        self.intelligence = None;
        self.state = ViewerState::Empty;
        self.scroll = UniformListScrollHandle::new();
        self.generation = self.generation.wrapping_add(1);
        self.search_generation = self.search_generation.wrapping_add(1);
        self.picker_open = false;
        self.query.clear();
        self.results.clear();
        self.highlighted_result = 0;
        self.history.clear();
        self.history_index = 0;
        cx.notify();
    }

    fn open_highlighted(&mut self, cx: &mut Context<Self>) {
        let Some(hit) = self.results.get(self.highlighted_result).cloned() else {
            return;
        };
        let Some(intelligence) = &self.intelligence else {
            return;
        };
        let mut reference = hit.relative_path.to_string_lossy().into_owned();
        if let Some(line) = hit.line {
            reference.push(':');
            reference.push_str(&line.to_string());
        }
        let cwd = intelligence.workspace_root().to_path_buf();
        self.picker_open = false;
        self.query.clear();
        self.results.clear();
        self.open_reference_inner(cwd, reference, true, cx);
    }

    fn handle_key_down(
        &mut self,
        event: &KeyDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.picker_open {
            return;
        }
        match event.keystroke.key.as_str() {
            "escape" => {
                self.picker_open = false;
                self.query.clear();
                self.results.clear();
                cx.notify();
            }
            "up" => {
                self.highlighted_result = self.highlighted_result.saturating_sub(1);
                cx.notify();
            }
            "down" => {
                self.highlighted_result =
                    (self.highlighted_result + 1).min(self.results.len().saturating_sub(1));
                cx.notify();
            }
            "enter" => self.open_highlighted(cx),
            _ => {
                let Some(edit) = query_editor::edit_for(&event.keystroke) else {
                    return;
                };
                let changed = match edit {
                    Edit::Local(local) => self.query.apply(local),
                    Edit::Clipboard(ClipboardEdit::Copy) => {
                        query_editor::copy_selection(&self.query, cx);
                        false
                    }
                    Edit::Clipboard(ClipboardEdit::Cut) => {
                        query_editor::cut_selection(&mut self.query, cx)
                    }
                    Edit::Clipboard(ClipboardEdit::Paste) => cx
                        .read_from_clipboard()
                        .and_then(|item| item.text())
                        .is_some_and(|text| self.query.insert(&text)),
                };
                if changed {
                    self.schedule_search(cx);
                } else {
                    cx.notify();
                }
            }
        }
        cx.stop_propagation();
    }

    /// Opens a terminal-shaped reference relative to a session cwd. All path
    /// safety and parsing stay behind `CodeIntelligence`'s interface.
    pub fn open_reference(
        &mut self,
        cwd: impl Into<PathBuf>,
        reference: impl Into<String>,
        cx: &mut Context<Self>,
    ) {
        let cwd = cwd.into();
        let reference = reference.into();
        if self.workspace_cwd.as_ref() != Some(&cwd) {
            self.set_workspace(Some(cwd.clone()), cx);
        }
        self.open_reference_inner(cwd, reference, true, cx);
    }

    fn open_reference_inner(
        &mut self,
        cwd: PathBuf,
        reference: String,
        record_history: bool,
        cx: &mut Context<Self>,
    ) {
        self.generation = self.generation.wrapping_add(1);
        let generation = self.generation;
        self.scroll = UniformListScrollHandle::new();
        self.state = ViewerState::Loading {
            reference: reference.clone(),
        };
        cx.notify();

        let tokio = self.tokio.clone();
        let history_cwd = cwd.clone();
        self._load_task = Some(cx.spawn(async move |this, cx| {
            let task_reference = reference.clone();
            let result = tokio
                .spawn_blocking(move || -> Result<_, CodeIntelligenceError> {
                    let intelligence = CodeIntelligence::for_session(&cwd)?;
                    let snapshot = intelligence.open_reference(&task_reference)?;
                    Ok((intelligence, snapshot))
                })
                .await
                .map_err(|error| format!("Code viewer stopped: {error}"))
                .and_then(|result| result.map_err(|error| error.to_string()));
            let _ = this.update(cx, |this, cx| {
                if this.generation != generation {
                    return;
                }
                match result {
                    Ok((intelligence, snapshot)) => {
                        this.workspace_cwd = Some(history_cwd.clone());
                        this.intelligence = Some(Arc::new(intelligence));
                        let target_line = snapshot.target.map(|target| target.line);
                        if record_history {
                            if this.history_index + 1 < this.history.len() {
                                this.history.truncate(this.history_index + 1);
                            }
                            let should_push =
                                this.history.last().is_none_or(|(current, current_ref)| {
                                    current != &history_cwd || current_ref != &reference
                                });
                            if should_push {
                                this.history.push((history_cwd, reference));
                                this.history_index = this.history.len().saturating_sub(1);
                            }
                        }
                        this.state = ViewerState::Ready(Arc::new(snapshot));
                        if let Some(line) = target_line {
                            this.scroll
                                .scroll_to_item(line.saturating_sub(1), ScrollStrategy::Center);
                        }
                    }
                    Err(message) => {
                        this.state = ViewerState::Error { reference, message };
                    }
                }
                cx.notify();
            });
        }));
    }

    fn navigate(&mut self, delta: isize, cx: &mut Context<Self>) {
        if self.history.is_empty() {
            return;
        }
        let next = self
            .history_index
            .saturating_add_signed(delta)
            .min(self.history.len() - 1);
        if next == self.history_index {
            return;
        }
        self.history_index = next;
        let (cwd, reference) = self.history[next].clone();
        self.open_reference_inner(cwd, reference, false, cx);
    }
}
impl Focusable for CodeViewer {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

mod highlight;
mod render;
#[cfg(test)]
mod tests;
