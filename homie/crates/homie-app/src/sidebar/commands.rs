use super::*;
impl Sidebar {
    pub(crate) fn surface_fill(colors: SemanticColors) -> Rgba {
        colors.sidebar_surface()
    }
    /// Top/bottom gradient masks over the session list, each fading in over the
    /// first few pixels of travel so a list that fits shows neither.
    pub(crate) fn scroll_fades(&self, colors: SemanticColors) -> Vec<AnyElement> {
        const HEIGHT: f32 = 28.0;
        /// Scroll distance over which a mask reaches full strength.
        const RAMP: f32 = 14.0;
        let scrolled = f32::from(self.list_scroll.offset().y).min(0.0).abs();
        let remaining = (f32::from(self.list_scroll.max_offset().y) - scrolled).max(0.0);
        // Opaque at the edge: the sidebar's own fill is translucent, and
        // fading to it would leave a legible ghost of the clipped row.
        let fill = Hsla {
            a: 1.0,
            ..Self::surface_fill(colors).into()
        };
        let mut fades = Vec::new();
        for (strength, angle, edge) in [
            ((scrolled / RAMP).min(1.0), 180.0, true),
            ((remaining / RAMP).min(1.0), 0.0, false),
        ] {
            if strength <= 0.01 {
                continue;
            }
            let mask = div()
                .absolute()
                .left_0()
                .right_0()
                .h(px(HEIGHT))
                .opacity(strength)
                .bg(linear_gradient(
                    angle,
                    linear_color_stop(fill, 0.0),
                    linear_color_stop(fill.opacity(0.0), 1.0),
                ));
            fades.push(if edge {
                mask.top_0().into_any_element()
            } else {
                mask.bottom_0().into_any_element()
            });
        }
        fades
    }
    pub(crate) fn hover_card(&self, colors: SemanticColors) -> Option<AnyElement> {
        let (id, pointer_y) = self.ui.hover_card.as_ref()?;
        let (session, project) = {
            let store = self.store.read().expect("session store lock poisoned");
            let session = store.sessions().get(id)?.clone();
            let project = store.projects().get(&session.project_id).cloned();
            (session, project)
        };
        let mut details = div()
            .flex()
            .flex_col()
            .gap(px(7.0))
            .px(px(12.0))
            .py(px(9.0));
        if let Some(project) = &project {
            details = details.child(hover_detail("folder.fill", &project.name, false, colors));
        }
        if let Some(branch) = &session.git_branch {
            details = details.child(hover_detail("arrow.branch", branch, true, colors));
        }
        details = details.child(hover_detail(
            if session.worktree_path.is_some() {
                "point.3.filled.connected.trianglepath.dotted"
            } else {
                "internaldrive"
            },
            &clamp_path(session.worktree_path.as_deref().unwrap_or(&session.cwd)),
            true,
            colors,
        ));
        if let Some(ports) = &session.listening_ports
            && !ports.is_empty()
        {
            details = details.child(hover_detail(
                "network",
                &ports
                    .iter()
                    .map(|port| format!(":{}", port.port))
                    .collect::<Vec<_>>()
                    .join(", "),
                false,
                colors,
            ));
        }
        let card = div()
            .w(px(260.0))
            .rounded(px(Radius::CARD))
            .bg(colors.background.alpha(0.98))
            .border_1()
            .border_color(colors.primary.alpha(0.08))
            .shadow_lg()
            .overflow_hidden()
            .child(
                div()
                    .px(px(12.0))
                    .pt(px(10.0))
                    .pb(px(8.0))
                    .text_size(px(Typo::ROW_EMPHASIZED.size))
                    .font_weight(Typo::ROW_EMPHASIZED.weight)
                    .text_color(colors.primary)
                    .child(display_title(&session)),
            )
            .child(HairlineDivider::horizontal(colors))
            .child(details);
        // Deferred + anchored so the card floats over the terminal instead of
        // being clipped at the sidebar edge. No mouse listeners: like the
        // Swift click-through panel, it never eats the first click on a row.
        Some(
            deferred(
                anchored()
                    .position(point(
                        px((self.ui.width - 4.0).max(0.0)),
                        px(pointer_y - 14.0),
                    ))
                    .snap_to_window_with_margin(px(8.0))
                    .child(card),
            )
            .into_any_element(),
        )
    }
    pub(crate) fn status_glyph(
        &mut self,
        session: &SessionRecord,
        migrating: bool,
        colors: SemanticColors,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Entity<StatusGlyph> {
        let kind = ui_agent_kind(session.effective_kind());
        let state = status_state(session, migrating);
        let entity = self
            .glyphs
            .entry(session.id.clone())
            .or_insert_with(|| cx.new(|_| StatusGlyph::new(kind, state, 16.0, colors)))
            .clone();
        entity.update(cx, |glyph, cx| {
            glyph.set_kind(kind, cx);
            glyph.set_state(state, window, cx);
            glyph.set_colors(colors, cx);
        });
        entity
    }
    /// ⌘1–⌘8 address the first eight rows; ⌘9 always jumps to the last one,
    /// so the hint follows the same rule rather than labelling row nine.
    pub(crate) fn shortcut_for(&mut self, id: &SessionId) -> Option<usize> {
        self.shortcut_ranks.get(id).copied()
    }
    pub(crate) fn reorder_project(&mut self, moved: &ProjectId, target: &ProjectId) {
        let mut store = self.store.write().expect("session store lock poisoned");
        let mut order = store.sidebar_project_order();
        move_before(&mut order, moved, target);
        self.ui.order_dirty |= store.stage_project_order(order);
    }
    pub(crate) fn reorder_session(&mut self, moved: &SessionId, target: &SessionId) {
        let mut store = self.store.write().expect("session store lock poisoned");
        let mut order = store.sidebar_session_order();
        move_before(&mut order, moved, target);
        self.ui.order_dirty |= store.stage_session_order(order);
    }
    /// Ends a drag gesture: clears the visual state and writes any staged
    /// reorder to disk exactly once.
    pub(crate) fn finish_drag(&mut self) {
        self.ui.drag = None;
        self.ui.drag_target = None;
        if self.ui.order_dirty {
            self.ui.order_dirty = false;
            let _ = self
                .store
                .read()
                .expect("session store lock poisoned")
                .persist_preferences();
        }
    }
    /// Drops the moved session at the end of the manual order. The projection
    /// groups by project before it sorts, so "last overall" reads as "last in
    /// its own group" — which is what ⌃⌘↓ on the bottom-but-one row means.
    pub(crate) fn reorder_session_to_end(&mut self, moved: &SessionId) {
        let mut store = self.store.write().expect("session store lock poisoned");
        let mut order = store.sidebar_session_order();
        move_to_end(&mut order, moved);
        let _ = store.set_session_order(order);
    }
    pub(crate) fn archive_sessions(&mut self, ids: Vec<SessionId>) {
        self.store
            .write()
            .expect("session store lock poisoned")
            .archive_sessions(ids);
    }
    pub(crate) fn close_sessions(&mut self, ids: Vec<SessionId>, cx: &mut Context<Self>) {
        let mut store = self.store.write().expect("session store lock poisoned");
        store.request_close(ids.clone());
        let raised = store.pending_close().is_some();
        if self.preview && !raised {
            for id in ids {
                store.remove_session_record(&id);
            }
        }
        drop(store);
        if raised {
            // Wake RootView so the confirmation shows on this click, not the
            // next time something else happens to redraw the window.
            cx.emit(SidebarEvent::ConfirmationChanged);
        }
    }
    /// Selects the nth session (⌘1–⌘9 order, matching the row hints) and
    /// reports whether a session existed at that index.
    pub fn select_shortcut(&mut self, index: usize, cx: &mut Context<Self>) -> bool {
        self.commit_rename();
        let id = {
            let mut store = self.store.write().expect("session store lock poisoned");
            let id = store
                .ordered_sessions()
                .get(index)
                .map(|session| session.id.clone());
            if let Some(id) = &id {
                store.select(id.clone());
            }
            id
        };
        if id.is_none() {
            return false;
        }
        cx.emit(SidebarEvent::SessionActivated);
        cx.notify();
        true
    }
    /// Selects the last session in sidebar order (⌘9, matching the browser
    /// convention where the last digit jumps to the final tab).
    pub fn select_last(&mut self, cx: &mut Context<Self>) -> bool {
        let count = self
            .store
            .write()
            .expect("session store lock poisoned")
            .ordered_sessions()
            .len();
        if count == 0 {
            return false;
        }
        self.select_shortcut(count - 1, cx)
    }
    /// Moves the selection `delta` rows through the sidebar order (⌘↑/⌘↓ and
    /// ⌘←/⌘→), wrapping at both ends. Returns false when there are no
    /// sessions to move between.
    pub fn select_relative(&mut self, delta: isize, cx: &mut Context<Self>) -> bool {
        self.commit_rename();
        {
            let mut store = self.store.write().expect("session store lock poisoned");
            let sessions = store.ordered_sessions();
            if sessions.is_empty() {
                return false;
            }
            let len = sessions.len() as isize;
            let current = store
                .selected_session_id()
                .and_then(|id| sessions.iter().position(|session| &session.id == id));
            let index = match current {
                Some(current) => (current as isize + delta).rem_euclid(len),
                // Nothing selected yet: ⌘↓ enters at the top, ⌘↑ at the bottom.
                None if delta >= 0 => 0,
                None => len - 1,
            } as usize;
            store.select(sessions[index].id.clone());
        }
        cx.emit(SidebarEvent::SessionActivated);
        cx.notify();
        true
    }
    /// ⌘J: select the next session waiting on a human, in sidebar order and
    /// wrapping past the current row. Returns false when nothing is waiting.
    pub fn select_next_needing_input(&mut self, cx: &mut Context<Self>) -> bool {
        self.commit_rename();
        {
            let mut store = self.store.write().expect("session store lock poisoned");
            let sessions = store.ordered_sessions();
            if sessions.is_empty() {
                return false;
            }
            let current = store
                .selected_session_id()
                .and_then(|id| sessions.iter().position(|session| &session.id == id));
            // Start one past the selection so repeated ⌘J walks the queue
            // instead of landing on the same row.
            let start = current.map_or(0, |index| index + 1);
            let Some(next) = (0..sessions.len())
                .map(|offset| &sessions[(start + offset) % sessions.len()])
                .find(|session| session.attention() == ProtoAttentionLevel::NeedsInput)
            else {
                return false;
            };
            store.select(next.id.clone());
        }
        cx.emit(SidebarEvent::SessionActivated);
        cx.notify();
        true
    }
    /// ⌃⌘↑/⌃⌘↓: move the selected session one place among its own siblings —
    /// the rows sharing its parent inside its project. Clamps at the ends of
    /// that run: a reorder that wrapped would teleport the row past every
    /// other project, and one that crossed levels would silently re-parent a
    /// session, which is the daemon's call to make, not a keystroke's.
    pub fn reorder_selected(&mut self, delta: isize, cx: &mut Context<Self>) -> bool {
        self.commit_rename();
        let (moved, target) = {
            let mut store = self.store.write().expect("session store lock poisoned");
            let Some(selected) = store.selected_session_id().cloned() else {
                return false;
            };
            let projection = store.sidebar_projection();
            let Some(group) = projection
                .projects
                .iter()
                .find(|group| group.sessions.iter().any(|row| row.id() == &selected))
            else {
                return false;
            };
            let parent = group
                .sessions
                .iter()
                .find(|row| row.id() == &selected)
                .and_then(|row| row.session.parent.clone());
            let siblings: Vec<&SessionId> = group
                .sessions
                .iter()
                .filter(|row| row.session.parent == parent)
                .map(|row| row.id())
                .collect();
            let index = siblings
                .iter()
                .position(|id| *id == &selected)
                .expect("the group was found by this id");
            let destination = index as isize + delta;
            if destination < 0 || destination >= siblings.len() as isize {
                return false;
            }
            // Moving up lands before the sibling above; moving down lands
            // before the one two below, i.e. just after the row it swaps with.
            // Off the end there is no anchor, so the move goes to the tail.
            let target = if delta < 0 {
                siblings.get(destination as usize)
            } else {
                siblings.get(destination as usize + 1)
            }
            .map(|id| (*id).clone());
            (selected, target)
        };
        match target {
            Some(target) => self.reorder_session(&moved, &target),
            None => self.reorder_session_to_end(&moved),
        }
        cx.notify();
        true
    }
    /// ⌘R: start renaming the selected row inline, the same edit the context
    /// menu's "Rename…" opens. Returns false when nothing is selected.
    pub fn rename_selected(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        let selected = self
            .store
            .read()
            .expect("session store lock poisoned")
            .selected_session()
            .cloned();
        let Some(session) = selected else {
            return false;
        };
        self.begin_rename(&session, window, cx);
        true
    }
    /// ⌘⇧W: archive the selected session, where ⌘W removes it from the
    /// sidebar. Returns false when nothing is selected.
    pub fn archive_selected(&mut self, cx: &mut Context<Self>) -> bool {
        let selected = self
            .store
            .read()
            .expect("session store lock poisoned")
            .selected_session_id()
            .cloned();
        let Some(id) = selected else {
            return false;
        };
        self.archive_sessions(vec![id]);
        cx.notify();
        true
    }
    /// ⌘W: close the selected session, honoring the
    /// confirm-before-closing preference (a running session raises the
    /// confirmation dialog; an already-exited one closes at once). Returns
    /// false when nothing is selected so ⌘W falls through to closing the
    /// window.
    pub fn close_selected_now(&mut self, cx: &mut Context<Self>) -> bool {
        let selected = self
            .store
            .read()
            .expect("session store lock poisoned")
            .selected_session_id()
            .cloned();
        let Some(id) = selected else {
            return false;
        };
        if self.preview {
            self.close_sessions_immediately(vec![id]);
        } else {
            self.close_sessions(vec![id], cx);
        }
        cx.notify();
        true
    }
    /// Close that bypasses the confirm-before-closing preference entirely.
    pub(crate) fn close_sessions_immediately(&mut self, ids: Vec<SessionId>) {
        let mut store = self.store.write().expect("session store lock poisoned");
        store.remove_sessions(ids.clone());
        if self.preview {
            for id in ids {
                store.remove_session_record(&id);
            }
        }
    }
}
