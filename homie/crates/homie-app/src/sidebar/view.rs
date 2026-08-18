use super::*;

pub enum SidebarEvent {
    VisibilityChanged,
    WidthChanged,
    /// The title-bar gear is a settings affordance. RootView owns the settings
    /// surface, so the sidebar requests it instead of opening its account menu.
    OpenSettings,
    /// One-click path from the footer menu into the Remote host editor.
    AddRemoteHost,
    /// A plain click (or shortcut) selected a session: hand keyboard focus
    /// to its terminal surface so the user can type immediately.
    SessionActivated,
    /// The user acted on the update pill. The sidebar holds no updater of its
    /// own; RootView owns the handle and forwards these.
    Update(UpdateCommand),
    /// The close confirmation was raised, confirmed, or cancelled. RootView
    /// paints that dialog but only re-renders on our events -- without this it
    /// keeps showing a stale frame until some unrelated update wakes it, which
    /// reads as "the ✕ did nothing".
    ConfirmationChanged,
}

#[derive(Clone)]
pub(crate) struct DraggedSidebarItem(pub(crate) DragItem);

pub(crate) struct DragPreview {
    pub(crate) label: SharedString,
    pub(crate) colors: SemanticColors,
}

impl Render for DragPreview {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .px(px(10.0))
            .h(px(28.0))
            .flex()
            .items_center()
            .rounded(px(Radius::ROW))
            .bg(self.colors.background.alpha(0.92))
            .border_1()
            .border_color(self.colors.primary.alpha(0.10))
            .text_size(px(Typo::META.size))
            .text_color(self.colors.primary)
            .child(self.label.clone())
    }
}

pub struct Sidebar {
    pub(crate) store: Arc<RwLock<SessionStore>>,
    // Preview stores have no daemon adapter, so retain their effect receiver.
    pub(crate) _preview_effects: Option<mpsc::UnboundedReceiver<StoreEffect>>,
    pub(crate) _store_changes: Option<Task<()>>,
    pub(crate) ui: SidebarUiState,
    /// Session list scroll position, read back each frame to size the top and
    /// bottom fades.
    pub(crate) list_scroll: ScrollHandle,
    pub(crate) directory_scroll: ScrollHandle,
    pub(crate) glyphs: HashMap<SessionId, Entity<StatusGlyph>>,
    /// Rebuilt once per projection render. Looking up ⌘1…⌘9 inside every row
    /// previously re-locked the store and scanned the full session list N times.
    pub(crate) shortcut_ranks: HashMap<SessionId, usize>,
    pub(crate) rename_focus: FocusHandle,
    pub(crate) hover_generation: u64,
    pub(crate) usage: Option<UsageSnapshot>,
    pub(crate) update: UpdateState,
    /// When visibility last flipped, so a held ⌘B cannot outrun the slide.
    pub(crate) last_toggle: Option<Instant>,
    pub(crate) preview: bool,
    /// The New Agent popover toggles between agent choices and a bounded
    /// one-level directory browser. The listing payload itself lives in the
    /// Store so the daemon adapter can complete it asynchronously.
    pub(crate) directory_picker_open: bool,
}

impl EventEmitter<SidebarEvent> for Sidebar {}

impl Focusable for Sidebar {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.rename_focus.clone()
    }
}

impl Sidebar {
    pub fn new(
        runtime: Option<Arc<StoreRuntime>>,
        preview: bool,
        scenario: PreviewScenario,
        cx: &mut Context<Self>,
    ) -> Self {
        let (store, preview_effects) = if preview {
            let fixture = SidebarPreviewFixture::make(scenario);
            let (mut store, effects) = SessionStore::headless(fixture.prefs);
            store.hydrate(fixture.list);
            if let Some(id) = fixture.selected_session_id {
                store.select(id);
            }
            (Arc::new(RwLock::new(store)), Some(effects))
        } else {
            (
                Arc::clone(
                    &runtime
                        .as_ref()
                        .expect("live sidebar requires StoreRuntime")
                        .store,
                ),
                None,
            )
        };
        let (width, visible) = {
            let store = store.read().expect("session store lock poisoned");
            let prefs = store.preferences();
            (prefs.sidebar_width, prefs.sidebar_visible)
        };
        let store_changes = runtime.map(|runtime| {
            let mut changes = runtime.changes();
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
        });
        let mut ui = SidebarUiState::new(width);
        ui.visible = visible;
        let mut sidebar = Self {
            store,
            _preview_effects: preview_effects,
            _store_changes: store_changes,
            ui,
            list_scroll: ScrollHandle::new(),
            directory_scroll: ScrollHandle::new(),
            glyphs: HashMap::new(),
            shortcut_ranks: HashMap::new(),
            rename_focus: cx.focus_handle(),
            hover_generation: 0,
            usage: None,
            update: UpdateState::default(),
            last_toggle: None,
            preview,
            directory_picker_open: false,
        };
        sidebar.ui.preview_account = preview;
        // Preview-only hook so headless screenshots can verify popover layout.
        if preview && std::env::var("HOMIE_SIDEBAR_POPOVER").is_ok_and(|value| value == "new-agent")
        {
            sidebar.ui.popover = Some(Popover::NewAgent {
                directory: None,
                host: None,
            });
        }
        sidebar
    }

    pub fn width(&self) -> f32 {
        self.ui.width
    }

    pub fn is_visible(&self) -> bool {
        self.ui.visible
    }

    pub fn selected_session(&self) -> Option<SessionRecord> {
        self.store
            .read()
            .expect("session store lock poisoned")
            .selected_session()
            .cloned()
    }

    pub fn session_count(&self) -> usize {
        self.store
            .read()
            .expect("session store lock poisoned")
            .sessions()
            .len()
    }

    pub fn set_update(&mut self, state: UpdateState, cx: &mut Context<Self>) {
        self.update = state;
        cx.notify();
    }

    pub fn set_usage(&mut self, snapshot: UsageSnapshot, cx: &mut Context<Self>) {
        self.usage = Some(snapshot);
        cx.notify();
    }

    pub fn pending_close_copy(&self) -> Option<(String, String)> {
        let store = self.store.read().expect("session store lock poisoned");
        let pending = store.pending_close()?;
        let title = if pending.ids.len() == 1 {
            store
                .sessions()
                .get(&pending.ids[0])
                .map(|session| format!("Close “{}”?", session.title))
                .unwrap_or_else(|| "Close session?".into())
        } else {
            format!("Close {} sessions?", pending.ids.len())
        };
        let running = pending
            .ids
            .iter()
            .filter(|id| {
                store.sessions().get(*id).is_some_and(|session| {
                    !matches!(session.status, homie_proto::SessionStatus::Exited(_))
                })
            })
            .count();
        Some((title, format!("{running} still running.")))
    }

    pub fn confirm_close(&mut self, cx: &mut Context<Self>) {
        let mut store = self.store.write().expect("session store lock poisoned");
        let ids = store
            .pending_close()
            .map(|pending| pending.ids.clone())
            .unwrap_or_default();
        store.confirm_pending_close();
        if self.preview {
            for id in ids {
                store.remove_session_record(&id);
            }
        }
        drop(store);
        cx.emit(SidebarEvent::ConfirmationChanged);
        cx.notify();
    }

    pub fn cancel_close(&mut self, cx: &mut Context<Self>) {
        self.store
            .write()
            .expect("session store lock poisoned")
            .cancel_pending_close();
        cx.emit(SidebarEvent::ConfirmationChanged);
        cx.notify();
    }

    /// Flips sidebar visibility, unless the last flip is still sliding. Every
    /// entry point -- ⌘B, the terminal chrome button, the menu bar, and the
    /// sidebar's own collapse button -- routes through here, so the gate is the
    /// single place the debounce has to hold.
    pub fn toggle(&mut self, cx: &mut Context<Self>) {
        let now = Instant::now();
        if !toggle_has_settled(self.last_toggle.map(|at| now.duration_since(at))) {
            return;
        }
        self.last_toggle = Some(now);
        self.ui.toggle();
        let visible = self.ui.visible;
        if let Err(error) = self
            .store
            .write()
            .expect("session store lock poisoned")
            .update_preferences(|prefs| prefs.sidebar_visible = visible)
        {
            eprintln!("homie: could not remember sidebar visibility: {error}");
        }
        cx.emit(SidebarEvent::VisibilityChanged);
        cx.notify();
    }

    pub fn show_new_agent(&mut self, cx: &mut Context<Self>) {
        self.open_new_agent_popover(None, cx);
    }

    /// Opens the new-agent picker, refreshing the host catalog first so
    /// hosts.json edits show up without an app relaunch. The picker remembers
    /// the last local/remote spawn target. A remote target always starts from
    /// its own default cwd; repo resolution is only needed when switching from
    /// an active remote session back to this Mac.
    pub(crate) fn open_new_agent_popover(
        &mut self,
        directory: Option<String>,
        cx: &mut Context<Self>,
    ) {
        self.open_new_agent_popover_at(directory, None, cx);
    }

    pub(crate) fn open_new_agent_popover_at(
        &mut self,
        directory: Option<String>,
        location_host: Option<String>,
        cx: &mut Context<Self>,
    ) {
        self.directory_picker_open = false;
        let host = {
            let mut store = self.store.write().expect("session store lock poisoned");
            store.reload_hosts();
            store.refresh_agent_catalog();
            let remembered_host = store
                .begin_repo_targeting()
                .filter(|id| store.host(id).is_some());
            if location_host.is_some() {
                location_host
            } else if directory.is_none() {
                let active_host = store
                    .selected_session()
                    .and_then(|session| session.host.as_deref());
                if should_resolve_active_repo(
                    directory.as_deref(),
                    remembered_host.as_deref(),
                    active_host,
                ) {
                    store.request_repo_target(remembered_host.clone());
                }
                remembered_host
            } else {
                None
            }
        };
        self.ui.popover = Some(Popover::NewAgent { directory, host });
        cx.notify();
    }

    /// Reopen the most recently closed session via the daemon's reopen stack.
    pub fn reopen_last(&mut self, cx: &mut Context<Self>) {
        self.store
            .read()
            .expect("session store lock poisoned")
            .reopen_last();
        cx.notify();
    }

    /// Live width during a resize drag. Deliberately does not touch the
    /// preferences store: `update_preferences` writes the prefs file and
    /// reconfigures the daemon governor, which is far too heavy to run on
    /// every mouse-move frame. `commit_width` persists once the drag ends.
    pub fn set_width(&mut self, width: f32, cx: &mut Context<Self>) {
        let previous = self.ui.width;
        self.ui.set_width(width);
        // Dragging past the clamp keeps producing the same width; don't
        // repaint the world for it.
        if (self.ui.width - previous).abs() < f32::EPSILON {
            return;
        }
        cx.emit(SidebarEvent::WidthChanged);
        cx.notify();
    }

    /// Persist whatever width the drag settled on.
    pub fn commit_width(&mut self, _cx: &mut Context<Self>) {
        let persisted_width = self.ui.width;
        let _ = self
            .store
            .write()
            .expect("session store lock poisoned")
            .update_preferences(|prefs| prefs.sidebar_width = persisted_width);
    }

    pub fn reset_width(&mut self, cx: &mut Context<Self>) {
        self.ui.reset_width();
        let persisted_width = self.ui.width;
        let _ = self
            .store
            .write()
            .expect("session store lock poisoned")
            .update_preferences(|prefs| prefs.sidebar_width = persisted_width);
        cx.emit(SidebarEvent::WidthChanged);
        cx.notify();
    }

    pub(crate) fn colors(&self) -> SemanticColors {
        let store = self.store.read().expect("session store lock poisoned");
        crate::app_theme::sidebar_colors(&store.preferences().terminal_theme)
    }

    pub(crate) fn begin_rename(
        &mut self,
        session: &SessionRecord,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.commit_rename();
        self.ui
            .begin_rename(session.id.clone(), session.title.clone());
        self.rename_focus.focus(window, cx);
        cx.notify();
    }

    pub(crate) fn commit_rename(&mut self) {
        if let Some((id, title)) = self.ui.take_rename() {
            self.store
                .write()
                .expect("session store lock poisoned")
                .rename(id, title);
        }
    }

    pub(crate) fn on_key_down(
        &mut self,
        event: &gpui::KeyDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.ui.renaming.is_none() {
            if self.ui.popover.is_some() && event.keystroke.key.as_str() == "escape" {
                self.ui.popover = None;
                cx.notify();
            }
            return;
        }
        match event.keystroke.key.as_str() {
            "enter" => self.commit_rename(),
            "escape" => self.ui.cancel_rename(),
            _ => {
                let Some(edit) = query_editor::edit_for(&event.keystroke) else {
                    return;
                };
                match edit {
                    Edit::Local(local) => {
                        self.ui.rename_draft.apply(local);
                    }
                    Edit::Clipboard(ClipboardEdit::Copy) => {
                        query_editor::copy_selection(&self.ui.rename_draft, cx);
                    }
                    Edit::Clipboard(ClipboardEdit::Cut) => {
                        query_editor::cut_selection(&mut self.ui.rename_draft, cx);
                    }
                    Edit::Clipboard(ClipboardEdit::Paste) => {
                        if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
                            self.ui.rename_draft.insert(&text);
                        }
                    }
                }
            }
        }
        cx.notify();
    }

    pub(crate) fn schedule_hover_card(
        &mut self,
        id: SessionId,
        hovering: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.hover_generation = self.hover_generation.wrapping_add(1);
        let generation = self.hover_generation;
        if !hovering {
            if self
                .ui
                .hover_card
                .as_ref()
                .is_some_and(|(card_id, _)| card_id == &id)
            {
                self.ui.hover_card = None;
            }
            cx.notify();
            return;
        }
        cx.spawn_in(window, async move |this, cx| {
            cx.background_executor()
                .timer(Duration::from_millis(700))
                .await;
            let _ = this.update_in(cx, |this, window, cx| {
                if this.hover_generation == generation
                    && this.ui.hovered_session.as_ref() == Some(&id)
                {
                    let pointer_y = f32::from(window.mouse_position().y);
                    this.ui.hover_card = Some((id, pointer_y));
                    cx.notify();
                }
            });
        })
        .detach();
    }
}

impl Render for Sidebar {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = self.colors();
        let projection = {
            let mut store = self.store.write().expect("session store lock poisoned");
            store.sidebar_projection()
        };
        self.shortcut_ranks = shortcut_ranks(&projection.ordered_sessions);
        retain_live_glyphs(&mut self.glyphs, &projection.display_order);
        let mut list = div()
            .id("sidebar-list")
            .track_scroll(&self.list_scroll)
            .flex_1()
            .min_h(px(0.0))
            .overflow_y_scroll()
            .px(px(Space::INSET))
            .pt(px(2.0))
            .pb(px(Metrics::ROW_HEIGHT + 17.0))
            .flex()
            .flex_col()
            .gap(px(2.0));
        for group in &projection.projects {
            list = list.child(self.project_section(group, colors, window, cx));
        }

        let mut root = div()
            .id("sidebar")
            .relative()
            .size_full()
            .flex()
            .flex_col()
            .text_color(colors.primary)
            .bg(Self::surface_fill(colors))
            .track_focus(&self.rename_focus)
            .on_key_down(cx.listener(Self::on_key_down))
            .child(self.top_bar(colors, cx))
            .child(self.new_agent_row(colors, cx));
        if projection.projects.is_empty() {
            root = root.child(self.empty_state(colors, cx));
        } else {
            // Rows dissolve into the chrome at both ends of the scroll instead
            // of being sliced off by the container edge.
            root = root.child(
                div()
                    .relative()
                    .flex_1()
                    .min_h(px(0.0))
                    .flex()
                    .flex_col()
                    .child(list)
                    .children(self.scroll_fades(colors)),
            );
        }
        root = root.child(self.account_footer(colors, cx));
        if let Some(popover) = self.popover(colors, window, cx) {
            root = root.child(popover);
        }
        if let Some(card) = self.hover_card(colors) {
            root = root.child(card);
        }
        root
    }
}
