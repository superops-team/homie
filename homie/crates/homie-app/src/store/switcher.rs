use super::*;

impl super::SessionStore {
    pub fn handle_switcher_key(&mut self, key: SwitcherKey) -> bool {
        let order = if matches!(key, SwitcherKey::Tab { control: true, .. })
            && !self.switcher.is_visible()
        {
            self.mru_sessions()
        } else {
            Vec::new()
        };
        let outcome = self.switcher.key_down(key, &order);
        let consumed = outcome.consumed();
        self.apply_switcher_outcome(outcome);
        consumed
    }

    pub fn handle_switcher_modifiers_changed(&mut self, control_held: bool) -> bool {
        let outcome = self.switcher.modifiers_changed(control_held);
        self.apply_switcher_outcome(outcome);
        false
    }

    pub fn commit_switcher_index(&mut self, index: usize) {
        if let Some(id) = self.switcher.commit_index(index) {
            self.select(id);
        }
    }

    pub fn cancel_switcher(&mut self) {
        self.switcher.cancel();
    }

    pub fn toggle_overview(&mut self) {
        let sessions = self.ordered_sessions();
        self.overview.toggle(&sessions);
        if self.overview.is_visible() {
            self.switcher.cancel();
        }
    }

    pub fn dismiss_overview(&mut self) {
        self.overview.dismiss();
    }

    pub fn set_overview_mode(&mut self, mode: OverviewMode) {
        let sessions = self.ordered_sessions();
        self.overview.set_mode(mode, &sessions);
    }

    pub fn set_overview_filter(&mut self, filter: OverviewFilter) {
        let sessions = self.ordered_sessions();
        self.overview.set_filter(filter, &sessions);
    }

    pub fn append_overview_query(&mut self, text: &str) -> bool {
        let sessions = self.ordered_sessions();
        self.overview.append_query(text, &sessions)
    }

    pub fn overview_backspace(&mut self) -> bool {
        let sessions = self.ordered_sessions();
        let outcome = self.overview.backspace(&sessions);
        let handled = !matches!(outcome, OverviewOutcome::Ignored);
        self.apply_overview_outcome(outcome);
        handled
    }

    pub fn overview_escape(&mut self) -> bool {
        let sessions = self.ordered_sessions();
        let outcome = self.overview.escape(&sessions);
        let handled = !matches!(outcome, OverviewOutcome::Ignored);
        self.apply_overview_outcome(outcome);
        handled
    }

    pub fn move_overview_focus(&mut self, arrow: OverviewArrow) -> bool {
        let sessions = self.ordered_sessions();
        self.overview.move_focus(arrow, &sessions)
    }

    pub fn activate_overview_focus(&mut self) -> bool {
        let outcome = self.overview.activate_focused();
        let handled = !matches!(outcome, OverviewOutcome::Ignored);
        self.apply_overview_outcome(outcome);
        handled
    }

    pub fn activate_overview_session(&mut self, id: SessionId) {
        let outcome = self.overview.activate(id);
        self.apply_overview_outcome(outcome);
    }

    pub fn toggle_overview_selection(&mut self, id: SessionId) {
        if self.sessions.contains_key(&id) {
            self.overview.toggle_selection(id);
        }
    }

    pub fn clear_overview_selection(&mut self) {
        self.overview.clear_selection();
    }

    pub fn select_all_overview_sessions(&mut self) {
        let sessions = self.ordered_sessions();
        self.overview.select_all_visible(&sessions);
    }

    pub fn close_overview_selection(&mut self) -> bool {
        let outcome = self.overview.close_selected();
        let handled = !matches!(outcome, OverviewOutcome::Ignored);
        self.apply_overview_outcome(outcome);
        handled
    }

    pub fn close_overview_session(&mut self, id: SessionId) {
        let outcome = self.overview.close_one(id);
        self.apply_overview_outcome(outcome);
    }

    pub fn global_attention(&self) -> AttentionLevel {
        self.sessions
            .values()
            .fold(AttentionLevel::None, |rollup, session| {
                let attention = session.attention();
                if attention_rank(&attention) > attention_rank(&rollup) {
                    attention
                } else {
                    rollup
                }
            })
    }

    pub fn needs_input_sessions(&self) -> Vec<SessionRecord> {
        let mut sessions: Vec<_> = self
            .sessions
            .values()
            .filter(|session| session.attention() == AttentionLevel::NeedsInput)
            .map(|session| session.as_ref().clone())
            .collect();
        sessions.sort_by(|left, right| {
            right
                .updated_at
                .partial_cmp(&left.updated_at)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| left.id.0.cmp(&right.id.0))
        });
        sessions
    }

    pub fn snapshot(&self) -> StoreSnapshot {
        let mut sessions: Vec<_> = self.sessions.values().map(Arc::clone).collect();
        sessions.sort_by(|left, right| {
            right
                .updated_at
                .partial_cmp(&left.updated_at)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| left.id.0.cmp(&right.id.0))
        });
        let mut projects: Vec<_> = self.projects.values().cloned().collect();
        projects.sort_by(|left, right| left.name.cmp(&right.name));
        StoreSnapshot {
            sessions,
            projects,
            selected_session_id: self.selected_session_id.clone(),
            global_attention: self.global_attention(),
        }
    }
}
