//! Native trailing workbench inspector.
//!
//! The root knows only whether this view is mounted and how wide its dock is.
//! This module owns selection tracking, session/PR/artifact projections,
//! background Git refreshes, unified-diff snapshots, and diff virtualization.

mod artifacts;
mod ask;
mod changes;
mod diff;
mod policy;
mod pr;
mod projection;
mod review;
mod scrollbar;
mod state;
mod view;

use std::collections::HashMap;
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use gpui::{
    Animation, AnimationExt, AnyElement, App, Context, DragMoveEvent, Entity, EventEmitter,
    FocusHandle, Focusable, FontWeight, KeyDownEvent, ListHorizontalSizingBehavior, MouseButton,
    Render, ScrollStrategy, SharedString, StatefulInteractiveElement, Task,
    UniformListScrollHandle, Window, div, ease_out_quint, point, prelude::*, px, rgba,
    uniform_list,
};
use homie_proto::{
    ArtifactKind, PrCheck, PrDiscussionItem, PullRequestStatus, SessionArtifact, SessionDiffBase,
    SessionRecord,
};
use homie_ui::{
    AgentLogo, Fill, FloatingSurface, Ink, LoadingIndicator, Metrics, Radius, SemanticColors, Typo,
};

use crate::code_viewer::CodeViewer;
use crate::diff::{
    DiffFile, DiffHunk, DiffLayer, DiffRow, DiffRowKind, DiffSnapshot, load_local_diff,
    snapshot_from_read_diff,
};
use crate::git_review::{GitRepository, GitReviewError, PatchMutation};
use crate::macos::sf_symbols::{SymbolWeight, sf_symbol, sf_symbol_weighted};
use crate::markdown::MarkdownDocument;
use crate::markdown_view::render_markdown;
use crate::query_editor::{self, ClipboardEdit, Edit, QueryEditor};
use crate::review_prompt::{ReviewEvidence, ReviewPrompt};
use crate::store::{InspectorTab, StoreRuntime};

use state::{AskDraft, DiffContext, LoadState, ReviewAction, ReviewLoadState};

use policy::{git_is_not_a_repository, git_is_not_installed, should_show_blocking_git_loading};

use projection::{
    artifact_count, artifact_title, artifact_visible, checks_rollup, discussion_state, folder_name,
    format_bytes, humanize_github_state, merge_blocker_label, patch_creates_file, prompt_layer,
    pull_request_can_merge, pull_request_discussion, pull_request_state, relative_time,
    session_status, sorted_pr_checks, ui_agent_kind,
};

use artifacts::{artifact_icon, render_artifact_row};
use diff::render_rows;
use pr::render_pull_request;
use view::section_label;

const DIFF_ROW_HEIGHT: f32 = 20.0;
const GUTTER_WIDTH: f32 = 68.0;
const REFRESH_INTERVAL: Duration = Duration::from_millis(1400);
const SCROLLBAR_INSET: f32 = 4.0;
const SCROLLBAR_MIN_THUMB: f32 = 34.0;

#[derive(Clone, Copy)]
struct DraggedDiffScrollbar;

impl Render for DraggedDiffScrollbar {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        gpui::Empty
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct ScrollbarInteraction {
    dragging: bool,
    grab_offset: f32,
}

#[derive(Clone, Copy, Debug)]
struct ScrollbarMetrics {
    track_top: f32,
    track_height: f32,
    thumb_height: f32,
    thumb_top: f32,
}

#[derive(Clone, Copy, Debug)]
pub enum InspectorEvent {
    Close,
}

pub struct WorkbenchInspector {
    runtime: Arc<StoreRuntime>,
    _tokio_owner: Arc<tokio::runtime::Runtime>,
    tokio: tokio::runtime::Handle,
    code_viewer: Entity<CodeViewer>,
    markdown_cache: HashMap<String, Arc<MarkdownDocument>>,
    focus: FocusHandle,
    visible: bool,
    selected_tab: InspectorTab,
    tab_direction: f32,
    tab_transition_generation: u64,
    context: Option<DiffContext>,
    state: LoadState,
    review_state: ReviewLoadState,
    review_generation: u64,
    review_task: Option<Task<()>>,
    review_action_task: Option<Task<()>>,
    review_action_busy: bool,
    review_feedback: Option<(bool, String)>,
    ask_draft: Option<AskDraft>,
    ask_query: QueryEditor,
    ask_task: Option<Task<()>>,
    ask_busy: bool,
    ask_feedback: Option<(bool, String)>,
    commit_open: bool,
    commit_query: QueryEditor,
    discard_armed: bool,
    armed_hunk: Option<u64>,
    diff_layer: DiffLayer,
    files_open: bool,
    comparison: SessionDiffBase,
    comparison_menu_open: bool,
    loading: bool,
    generation: u64,
    scroll: UniformListScrollHandle,
    scrollbar_interaction: ScrollbarInteraction,
    scrollbar_layout_primed: bool,
    refresh_task: Option<Task<()>>,
    poll_task: Option<Task<()>>,
    _store_changes: Task<()>,
}

impl EventEmitter<InspectorEvent> for WorkbenchInspector {}

impl Focusable for WorkbenchInspector {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl WorkbenchInspector {
    pub fn new(
        runtime: Arc<StoreRuntime>,
        tokio_owner: Arc<tokio::runtime::Runtime>,
        cx: &mut Context<Self>,
    ) -> Self {
        let tokio = tokio_owner.handle().clone();
        let code_viewer = cx.new(|cx| CodeViewer::new(tokio.clone(), cx));
        let focus = cx.focus_handle();
        let mut changes = runtime.changes();
        let store_changes = cx.spawn(async move |this, cx| {
            loop {
                match changes.recv().await {
                    Ok(()) | Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                        if this
                            .update(cx, |this, cx| this.refresh_if_context_changed(cx))
                            .is_err()
                        {
                            return;
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => return,
                }
            }
        });
        let selected_tab = runtime
            .store
            .read()
            .expect("session store lock poisoned")
            .preferences()
            .inspector_tab;
        Self {
            runtime,
            _tokio_owner: tokio_owner,
            tokio,
            code_viewer,
            markdown_cache: HashMap::new(),
            focus,
            visible: false,
            selected_tab,
            tab_direction: 1.0,
            tab_transition_generation: 0,
            context: None,
            state: LoadState::NoSession,
            review_state: ReviewLoadState::NoSession,
            review_generation: 0,
            review_task: None,
            review_action_task: None,
            review_action_busy: false,
            review_feedback: None,
            ask_draft: None,
            ask_query: QueryEditor::default(),
            ask_task: None,
            ask_busy: false,
            ask_feedback: None,
            commit_open: false,
            commit_query: QueryEditor::default(),
            discard_armed: false,
            armed_hunk: None,
            diff_layer: DiffLayer::Branch,
            files_open: false,
            comparison: SessionDiffBase::DefaultBranch,
            comparison_menu_open: false,
            loading: false,
            generation: 0,
            scroll: UniformListScrollHandle::new(),
            scrollbar_interaction: ScrollbarInteraction::default(),
            scrollbar_layout_primed: false,
            refresh_task: None,
            poll_task: None,
            _store_changes: store_changes,
        }
    }

    pub fn set_visible(&mut self, visible: bool, cx: &mut Context<Self>) {
        if self.visible == visible {
            return;
        }
        self.visible = visible;
        if visible {
            // One-shot, every tab. Info renders the Git summary and the header
            // renders the Changes badge, so becoming visible always needs one
            // settled read of the working tree — what stays tab-gated is the
            // *periodic* poll below, not this edge-triggered refresh.
            self.refresh(true, cx);
        } else {
            self.refresh_task = None;
            self.review_task = None;
            self.comparison_menu_open = false;
            self.files_open = false;
            self.ask_draft = None;
            self.ask_feedback = None;
            self.ask_query.clear();
        }
        self.reconcile_diff_polling(cx);
        cx.notify();
    }

    fn reconcile_diff_polling(&mut self, cx: &mut Context<Self>) {
        let should_poll = self.visible && self.selected_tab == InspectorTab::Changes;
        if !should_poll {
            // Dropping a GPUI Task cancels its timer/future. Info and Artifacts
            // therefore perform no periodic Git work and have no idle wakeup.
            self.poll_task = None;
            return;
        }
        if self.poll_task.is_some() {
            return;
        }
        self.poll_task = Some(cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor().timer(REFRESH_INTERVAL).await;
                if this
                    .update(cx, |this, cx| {
                        if this.visible && this.selected_tab == InspectorTab::Changes {
                            this.refresh(false, cx);
                        }
                    })
                    .is_err()
                {
                    return;
                }
            }
        }));
    }

    /// Opens a terminal or diff-shaped file reference in the native code tab.
    /// The viewer owns resolution and loading; the inspector only preserves
    /// the workbench's spatial context and selects the destination tab.
    pub fn open_file_reference(
        &mut self,
        cwd: impl Into<PathBuf>,
        reference: impl Into<String>,
        cx: &mut Context<Self>,
    ) {
        let cwd = cwd.into();
        let reference = reference.into();
        self.code_viewer.update(cx, |viewer, cx| {
            viewer.open_reference(cwd, reference, cx);
        });
        self.select_tab(InspectorTab::Code, cx);
    }

    fn selected_context(&self) -> Option<DiffContext> {
        let store = self
            .runtime
            .store
            .read()
            .expect("session store lock poisoned");
        let session = store.selected_session()?;
        Some(DiffContext {
            id: session.id.clone(),
            cwd: PathBuf::from(&session.cwd),
            remote: session.host.is_some(),
        })
    }

    fn refresh_if_context_changed(&mut self, cx: &mut Context<Self>) {
        if !self.visible {
            return;
        }
        // Edge-triggered on a real context change, on every tab: a store change
        // that moves the selection must not leave Info showing the previous
        // session's counts. This is not periodic work — an idle Info tab makes
        // no Git calls because `reconcile_diff_polling` installs no timer.
        if self.selected_context() != self.context {
            self.refresh(true, cx);
        } else {
            // Info and Artifacts are projections of the live session record,
            // so same-session store changes still need to repaint the panel.
            cx.notify();
        }
    }

    pub(super) fn select_tab(&mut self, tab: InspectorTab, cx: &mut Context<Self>) {
        if self.selected_tab == tab {
            return;
        }
        self.tab_direction = if tab.index() > self.selected_tab.index() {
            1.0
        } else {
            -1.0
        };
        self.selected_tab = tab;
        if let Err(error) = self
            .runtime
            .store
            .write()
            .expect("session store lock poisoned")
            .update_preferences(|prefs| prefs.inspector_tab = tab)
        {
            eprintln!("homie: could not remember inspector tab: {error}");
        }
        self.comparison_menu_open = false;
        self.tab_transition_generation = self.tab_transition_generation.wrapping_add(1);
        if tab == InspectorTab::Changes {
            self.refresh(true, cx);
        }
        self.reconcile_diff_polling(cx);
        cx.notify();
    }

    pub(super) fn select_comparison(
        &mut self,
        comparison: SessionDiffBase,
        cx: &mut Context<Self>,
    ) {
        self.comparison_menu_open = false;
        if self.comparison == comparison {
            cx.notify();
            return;
        }
        self.comparison = comparison;
        self.scroll = UniformListScrollHandle::new();
        self.scrollbar_interaction = ScrollbarInteraction::default();
        self.scrollbar_layout_primed = false;
        self.refresh(true, cx);
    }

    pub(super) fn select_diff_layer(&mut self, layer: DiffLayer, cx: &mut Context<Self>) {
        self.files_open = false;
        self.armed_hunk = None;
        self.discard_armed = false;
        self.commit_open = false;
        if self.diff_layer == layer {
            cx.notify();
            return;
        }
        self.diff_layer = layer;
        self.scroll = UniformListScrollHandle::new();
        self.scrollbar_interaction = ScrollbarInteraction::default();
        self.scrollbar_layout_primed = false;
        self.refresh(true, cx);
    }

    pub(super) fn jump_to_diff_row(&mut self, row: usize, cx: &mut Context<Self>) {
        self.files_open = false;
        self.scroll.scroll_to_item(row, ScrollStrategy::Top);
        cx.notify();
    }

    fn refresh(&mut self, force: bool, cx: &mut Context<Self>) {
        if !self.visible || (self.loading && !force) {
            return;
        }
        let Some(context) = self.selected_context() else {
            self.context = None;
            self.state = LoadState::NoSession;
            self.review_state = ReviewLoadState::NoSession;
            self.code_viewer
                .update(cx, |viewer, cx| viewer.set_workspace(None, cx));
            cx.notify();
            return;
        };
        let context_changed = self.context.as_ref() != Some(&context);
        if context_changed {
            self.scroll = UniformListScrollHandle::new();
            self.scrollbar_interaction = ScrollbarInteraction::default();
            self.scrollbar_layout_primed = false;
            self.files_open = false;
            self.armed_hunk = None;
            self.ask_draft = None;
            self.ask_feedback = None;
            self.ask_query.clear();
            let workspace = (!context.remote).then(|| context.cwd.clone());
            self.code_viewer
                .update(cx, |viewer, cx| viewer.set_workspace(workspace, cx));
        }
        self.context = Some(context.clone());
        self.refresh_review(&context, force, cx);
        if !force && !context_changed && matches!(self.state, LoadState::NoSession) {
            return;
        }

        self.loading = true;
        self.generation = self.generation.wrapping_add(1);
        let generation = self.generation;
        if should_show_blocking_git_loading(context_changed, &self.state) {
            self.state = LoadState::Loading;
            cx.notify();
        }
        let cwd = context.cwd;
        let session_id = context.id;
        let remote = context.remote;
        let comparison = self.comparison;
        let layer = self.diff_layer;
        let client = Arc::clone(self.runtime.client());
        let tokio = self.tokio.clone();
        let background = cx.background_executor().clone();
        self.refresh_task = Some(cx.spawn(async move |this, cx| {
            let result = background
                .spawn(async move {
                    if remote {
                        tokio
                            .spawn(async move { client.read_diff(&session_id, comparison).await })
                            .await
                            .map_err(|error| format!("Diff request stopped: {error}"))
                            .and_then(|result| result.map_err(|error| error.to_string()))
                            .map(snapshot_from_read_diff)
                            .map(Arc::new)
                    } else {
                        tokio
                            .spawn_blocking(move || load_local_diff(&cwd, layer))
                            .await
                            .map_err(|error| format!("Diff worker stopped: {error}"))
                            .and_then(|result| result.map_err(|error| error.to_string()))
                            .map(Arc::new)
                    }
                })
                .await;
            let _ = this.update(cx, |this, cx| {
                if this.generation != generation {
                    return;
                }
                this.loading = false;
                let next = match result {
                    Ok(snapshot) => LoadState::Ready(snapshot),
                    Err(error) => LoadState::Error(error),
                };
                if this.state != next {
                    this.state = next;
                    this.scrollbar_layout_primed = false;
                    cx.notify();
                }
            });
        }));
    }

    fn refresh_review(&mut self, context: &DiffContext, force: bool, cx: &mut Context<Self>) {
        if context.remote {
            self.review_state = ReviewLoadState::Remote;
            return;
        }
        if self.review_action_busy && !force {
            return;
        }
        self.review_generation = self.review_generation.wrapping_add(1);
        let generation = self.review_generation;
        let cwd = context.cwd.clone();
        if !matches!(self.review_state, ReviewLoadState::Ready(_)) {
            self.review_state = ReviewLoadState::Loading;
        }
        let tokio = self.tokio.clone();
        let background = cx.background_executor().clone();
        self.review_task = Some(cx.spawn(async move |this, cx| {
            let result = background
                .spawn(async move {
                    tokio
                        .spawn_blocking(move || {
                            let repository = GitRepository::discover(&cwd)?;
                            repository.status()
                        })
                        .await
                        .map_err(|error| format!("Git status worker stopped: {error}"))
                        .and_then(|result| result.map_err(|error| error.to_string()))
                })
                .await;
            let _ = this.update(cx, |this, cx| {
                if this.review_generation != generation {
                    return;
                }
                this.review_state = match result {
                    Ok(status) => ReviewLoadState::Ready(Arc::new(status)),
                    Err(error) => ReviewLoadState::Error(error),
                };
                cx.notify();
            });
        }));
    }

    pub(super) fn run_review_action(&mut self, action: ReviewAction, cx: &mut Context<Self>) {
        if self.review_action_busy {
            return;
        }
        let Some(context) = self.context.clone().filter(|context| !context.remote) else {
            return;
        };
        self.review_action_busy = true;
        self.review_feedback = None;
        self.discard_armed = false;
        self.armed_hunk = None;
        cx.notify();
        let tokio = self.tokio.clone();
        let background = cx.background_executor().clone();
        self.review_action_task = Some(cx.spawn(async move |this, cx| {
            let result = background
                .spawn(async move {
                    tokio
                        .spawn_blocking(move || -> Result<String, GitReviewError> {
                            let repository = GitRepository::discover(&context.cwd)?;
                            match action {
                                ReviewAction::Stage(paths) => {
                                    repository.stage_paths(&paths)?;
                                    Ok("Changes staged".to_owned())
                                }
                                ReviewAction::Unstage(paths) => {
                                    repository.unstage_paths(&paths)?;
                                    Ok("Changes moved back to the working tree".to_owned())
                                }
                                ReviewAction::Discard(paths) => {
                                    repository.discard_unstaged(&paths)?;
                                    Ok("Unstaged edits discarded".to_owned())
                                }
                                ReviewAction::Patch { patch, mutation } => {
                                    repository.apply_patch(&patch, mutation)?;
                                    Ok(match mutation {
                                        PatchMutation::Stage => "Hunk staged",
                                        PatchMutation::Unstage => {
                                            "Hunk moved back to the working tree"
                                        }
                                        PatchMutation::Discard => "Hunk discarded",
                                    }
                                    .to_owned())
                                }
                                ReviewAction::Commit(message) => {
                                    let commit = repository.commit(&message)?;
                                    Ok(format!("Committed {} · {}", commit.oid, commit.summary))
                                }
                            }
                        })
                        .await
                        .map_err(|error| format!("Git action worker stopped: {error}"))
                        .and_then(|result| result.map_err(|error| error.to_string()))
                })
                .await;
            let _ = this.update(cx, |this, cx| {
                this.review_action_busy = false;
                match result {
                    Ok(message) => {
                        this.review_feedback = Some((true, message));
                        this.commit_open = false;
                        this.commit_query.clear();
                    }
                    Err(message) => this.review_feedback = Some((false, message)),
                }
                this.refresh(true, cx);
                cx.notify();
            });
        }));
    }

    pub(super) fn submit_commit(&mut self, cx: &mut Context<Self>) {
        let message = self.commit_query.text().trim().to_owned();
        if message.is_empty() {
            self.review_feedback = Some((false, "Write a commit message first".to_owned()));
            cx.notify();
            return;
        }
        self.run_review_action(ReviewAction::Commit(message), cx);
    }

    fn open_ask(
        &mut self,
        evidence: Vec<ReviewEvidence>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let label = if evidence.len() == 1 {
            evidence[0].label()
        } else {
            format!("{} review contexts", evidence.len())
        };
        self.ask_draft = Some(AskDraft { evidence, label });
        self.ask_feedback = None;
        self.ask_query.clear();
        self.ask_query
            .insert("Review this for correctness, regressions, and missing tests.");
        self.ask_query.select_all();
        self.commit_open = false;
        window.focus(&self.focus, cx);
        cx.notify();
    }

    pub(super) fn set_ask_question(&mut self, question: &str, cx: &mut Context<Self>) {
        self.ask_query.clear();
        self.ask_query.insert(question);
        self.ask_query.select_all();
        cx.notify();
    }

    pub(super) fn submit_ask(&mut self, cx: &mut Context<Self>) {
        if self.ask_busy {
            return;
        }
        let Some(draft) = self.ask_draft.clone() else {
            return;
        };
        let question = self.ask_query.text().trim().to_owned();
        let prompt = match ReviewPrompt::compose(&draft.evidence, &question) {
            Ok(prompt) => prompt,
            Err(error) => {
                self.ask_feedback = Some((false, error.to_string()));
                cx.notify();
                return;
            }
        };
        let Some(session) = self.selected_session() else {
            self.ask_feedback = Some((false, "Select an agent first".to_owned()));
            cx.notify();
            return;
        };

        self.ask_busy = true;
        self.ask_feedback = None;
        let subject = prompt.subject_label.clone();
        let session_id = session.id;
        let client = Arc::clone(self.runtime.client());
        let tokio = self.tokio.clone();
        let background = cx.background_executor().clone();
        self.ask_task = Some(cx.spawn(async move |this, cx| {
            let result = background
                .spawn(async move {
                    tokio
                        .spawn(
                            async move { client.send_text(&session_id, prompt.text, true).await },
                        )
                        .await
                        .map_err(|error| format!("Agent send stopped: {error}"))
                        .and_then(|result| result.map_err(|error| error.to_string()))
                })
                .await;
            let _ = this.update(cx, |this, cx| {
                this.ask_busy = false;
                match result {
                    Ok(()) => {
                        this.ask_feedback = Some((true, format!("Sent · {subject}")));
                        this.ask_query.clear();
                    }
                    Err(error) => this.ask_feedback = Some((false, error)),
                }
                cx.notify();
            });
        }));
    }

    pub(super) fn selected_session(&self) -> Option<SessionRecord> {
        self.runtime
            .store
            .read()
            .expect("session store lock poisoned")
            .selected_session()
            .cloned()
    }

    pub(super) fn markdown_document(&mut self, source: &str) -> Arc<MarkdownDocument> {
        if let Some(document) = self.markdown_cache.get(source) {
            return Arc::clone(document);
        }
        if self.markdown_cache.len() >= 24 {
            self.markdown_cache.clear();
        }
        let document = Arc::new(MarkdownDocument::parse(source));
        self.markdown_cache
            .insert(source.to_owned(), Arc::clone(&document));
        document
    }
}

#[cfg(test)]
mod tests;
