use super::*;

impl super::SessionStore {
    pub(super) fn focus_session(&mut self, id: SessionId) {
        let selection_changed = self.selected_session_id.as_ref() != Some(&id);
        self.selected_session_id = Some(id.clone());
        let revealed = self.reveal(&id);
        if selection_changed || revealed || self.prefs.last_selected_session.as_ref() != Some(&id) {
            self.prefs.last_selected_session = Some(id.clone());
            if let Err(error) = self.persist_preferences() {
                eprintln!("homie: could not remember the selected session: {error}");
            }
        }
        self.mru_order.retain(|candidate| candidate != &id);
        self.mru_order.insert(0, id.clone());
        self.invalidate_projection();
        if self
            .sessions
            .get(&id)
            .is_some_and(|session| !session.is_archived())
        {
            let update = self.terminal_residency.touch(id.clone());
            if let Some(evicted) = update.evicted {
                self.emit(StoreEffect::DetachAttachment(evicted));
            }
        }
        if self
            .sessions
            .get(&id)
            .is_some_and(|session| !session.is_archived())
        {
            // Selection is also the PR/artifact visibility signal. The
            // daemon uses mark_seen to wake a fresh foreground refresh, even
            // when there was no unseen completion to acknowledge.
            self.emit(StoreEffect::MarkSeen(id.clone()));
        }
        self.auto_resume_if_needed(&id);
        self.emit(StoreEffect::UiChanged);
    }

    pub(super) fn set_selected_survivor(&mut self, survivor: Option<SessionId>) {
        if let Some(id) = survivor {
            self.focus_session(id);
        } else {
            self.selected_session_id = None;
            if self.prefs.last_selected_session.take().is_some()
                && let Err(error) = self.persist_preferences()
            {
                eprintln!("homie: could not clear the selected session: {error}");
            }
            self.invalidate_projection();
        }
    }

    pub(super) fn auto_select_if_needed(&mut self) {
        if self.selected_session_id.is_some() {
            return;
        }
        let first = self
            .sidebar_projection()
            .first_active()
            .map(|session| session.id.clone());
        self.set_selected_survivor(first);
    }

    pub(super) fn auto_resume_selected_if_needed(&mut self) {
        let Some(id) = self.selected_session_id.clone() else {
            return;
        };
        self.auto_resume_if_needed(&id);
    }

    pub(super) fn apply_switcher_outcome(&mut self, outcome: SwitcherOutcome) {
        if let SwitcherOutcome::Committed(id) = outcome {
            self.select(id);
        }
    }

    pub(super) fn apply_overview_outcome(&mut self, outcome: OverviewOutcome) {
        match outcome {
            OverviewOutcome::Activate(id) => self.select(id),
            OverviewOutcome::RequestClose(ids) => self.request_close(ids),
            OverviewOutcome::Ignored | OverviewOutcome::Changed | OverviewOutcome::Dismissed => {}
        }
    }

    pub(super) fn reconcile_navigation(&mut self) {
        let live_ids: HashSet<_> = self.sessions.keys().cloned().collect();
        self.switcher.reconcile(&live_ids);
        let sessions = self.ordered_sessions();
        self.overview.reconcile(&sessions);
    }

    pub(super) fn reveal(&mut self, id: &SessionId) -> bool {
        let Some(session) = self.sessions.get(id).cloned() else {
            return false;
        };
        let mut changed = false;
        let project = session.project_id.clone();
        if let Some(index) = self
            .prefs
            .sidebar_collapsed_projects
            .iter()
            .position(|candidate| candidate == &project)
        {
            self.prefs.sidebar_collapsed_projects.remove(index);
            changed = true;
        }
        if session.is_archived() && !self.prefs.sidebar_expanded_archives.contains(&project) {
            self.prefs.sidebar_expanded_archives.push(project);
            changed = true;
        }
        // Walk up the spawn chain. `seen` guards against a cycle the daemon
        // should never produce but which would spin here if it did.
        let mut seen = HashSet::from([id.clone()]);
        let mut cursor = session.parent.clone();
        while let Some(ancestor) = cursor {
            if !seen.insert(ancestor.clone()) {
                break;
            }
            if let Some(index) = self
                .prefs
                .sidebar_collapsed_sessions
                .iter()
                .position(|candidate| candidate == &ancestor)
            {
                self.prefs.sidebar_collapsed_sessions.remove(index);
                changed = true;
            }
            cursor = self
                .sessions
                .get(&ancestor)
                .and_then(|record| record.parent.clone());
        }
        if changed {
            self.invalidate_projection();
        }
        changed
    }

    pub(super) fn sidebar_visible_order(&mut self) -> Vec<SessionId> {
        let expanded: HashSet<_> = self
            .prefs
            .sidebar_expanded_archives
            .iter()
            .cloned()
            .collect();
        self.sidebar_projection()
            .projects
            .iter()
            .flat_map(|group| {
                group.sessions.iter().map(|row| row.id().clone()).chain(
                    group
                        .archived
                        .iter()
                        .filter(|_| expanded.contains(&group.project.id))
                        .map(|session| session.id.clone()),
                )
            })
            .collect()
    }

    pub(super) fn invalidate_projection(&mut self) {
        self.revision = self.revision.wrapping_add(1);
        self.cached_projection = None;
    }

    pub(super) fn emit(&self, effect: StoreEffect) {
        let _ = self.effects.send(effect);
    }
}
