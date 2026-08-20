use super::*;

impl super::SessionStore {
    pub fn hydrate(&mut self, result: SessionListResult) {
        // A full resync is the authority: anything still listed here outlived
        // its close (daemon restart, failed terminate) and belongs on screen.
        self.closing.clear();
        self.sessions = result
            .sessions
            .into_iter()
            .map(|session| (session.id.clone(), Arc::new(session)))
            .collect();
        self.projects = result
            .projects
            .into_iter()
            .map(|project| (project.id.clone(), project))
            .collect();
        if self
            .selected_session_id
            .as_ref()
            .is_some_and(|id| !self.sessions.contains_key(id))
        {
            self.selected_session_id = None;
        }
        self.invalidate_projection();
        self.sync_sidebar_order();
        self.auto_select_if_needed();
        // A restored selection did not travel through `focus_session`, so it
        // still needs terminal residency before the pane can attach.
        if let Some(id) = self.selected_session_id.clone()
            && self
                .sessions
                .get(&id)
                .is_some_and(|session| !session.is_archived())
            && !self.terminal_residency.contains(&id)
        {
            let update = self.terminal_residency.touch(id);
            if let Some(evicted) = update.evicted {
                self.emit(StoreEffect::DetachAttachment(evicted));
            }
        }
        self.auto_resume_selected_if_needed();
        self.reconcile_navigation();
    }

    pub fn handle_event(&mut self, event: EventEnvelope) -> bool {
        self.handle_event_change(event) != StoreEventChange::None
    }

    pub(super) fn handle_event_change(&mut self, event: EventEnvelope) -> StoreEventChange {
        match event.name.as_str() {
            EventName::SESSION_UPDATED => {
                if let Ok(session) = serde_json::from_value::<SessionRecord>(event.params) {
                    if self
                        .sessions
                        .get(&session.id)
                        .is_some_and(|existing| existing.as_ref() == &session)
                    {
                        return StoreEventChange::None;
                    }
                    self.upsert_session(session);
                    return StoreEventChange::Model;
                }
            }
            EventName::SESSION_RESOURCES => {
                if let Ok(resources) =
                    serde_json::from_value::<homie_proto::SessionResourcesEvent>(event.params)
                    && let Some(existing) = self.sessions.get_mut(&resources.id)
                {
                    let record = Arc::make_mut(existing);
                    let mut changed = false;
                    if let Some(memory) = resources.memory_bytes
                        && record.memory_bytes != Some(memory)
                    {
                        record.memory_bytes = Some(memory);
                        changed = true;
                    }
                    if let Some(ports) = resources.listening_ports
                        && record.listening_ports.as_deref() != Some(ports.as_slice())
                    {
                        record.listening_ports = Some(ports);
                        changed = true;
                    }
                    if let Some(artifacts) = resources.artifacts
                        && record.artifacts.as_deref() != Some(artifacts.as_slice())
                    {
                        record.artifacts = Some(artifacts);
                        changed = true;
                    }
                    if changed {
                        self.invalidate_projection();
                        return StoreEventChange::Resources;
                    }
                }
            }
            EventName::SESSION_REMOVED => {
                let id = event
                    .params
                    .get("id")
                    .or_else(|| event.params.get("sessionID"))
                    .and_then(|value| value.as_str())
                    .map(SessionId::new);
                if let Some(id) = id
                    && self.sessions.contains_key(&id)
                {
                    self.remove_session_record(&id);
                    return StoreEventChange::Model;
                }
            }
            EventName::PROJECT_UPDATED => {
                if let Ok(project) = serde_json::from_value::<Project>(event.params) {
                    if self.projects.get(&project.id) == Some(&project) {
                        return StoreEventChange::None;
                    }
                    self.projects.insert(project.id.clone(), project);
                    self.invalidate_projection();
                    self.sync_sidebar_order();
                    return StoreEventChange::Model;
                }
            }
            _ => {}
        }
        StoreEventChange::None
    }

    pub fn upsert_session(&mut self, session: SessionRecord) {
        let previous = self.sessions.get(&session.id).cloned();
        let is_new = previous.is_none();
        let id = session.id.clone();
        let transitions = transitions_for_update(
            previous.as_deref(),
            &session,
            self.selected_session_id.as_ref(),
            self.app_is_active,
            self.prefs.status_sounds,
            self.agents.descriptor(session.effective_kind()),
        );
        let arriving_archived = session.is_archived();
        // Closing the tab also drops the Engine record and deletes the
        // session's output log, so it may only happen where nothing is lost.
        // A clean `exit 0` from something with no conversation to return to —
        // a shell — is that case. A crash, a signal (macOS memory pressure
        // kills agents with SIGTERM), or anything resumable stays listed with
        // its exit pill and Resume button: that is the whole point of deriving
        // resumability for exited sessions, and the scrollback is the only
        // record of what went wrong.
        let should_auto_close = !self.closing.contains(&id)
            && matches!(
                &session.status,
                SessionStatus::Exited(info)
                    if info.reason == ExitReason::Exited
                        && info.code == Some(0)
                        && session.resumability != Resumability::Resumable
            )
            && previous
                .as_deref()
                .is_none_or(|record| !matches!(record.status, SessionStatus::Exited(_)));
        self.sessions.insert(id.clone(), Arc::new(session));
        // Spawn selects the id before the authoritative record arrives, and
        // only focus_session grants terminal residency -- without this, a
        // session created from the UI stays "Preparing terminal" forever.
        if is_new
            && self.selected_session_id.as_ref() == Some(&id)
            && !arriving_archived
            && !self.terminal_residency.contains(&id)
        {
            let update = self.terminal_residency.touch(id.clone());
            if let Some(evicted) = update.evicted {
                self.emit(StoreEffect::DetachAttachment(evicted));
            }
        }
        self.invalidate_projection();
        // Only membership moves the order, and a session record is republished
        // on every title, status, and resource tick.
        if is_new {
            self.sync_sidebar_order();
        }
        for transition in transitions {
            self.emit(StoreEffect::StatusTransition(transition));
        }
        if should_auto_close {
            // Process exit is terminal UI state, not a historical tab. Hide
            // the row and detach its terminal immediately; `session.remove`
            // then clears the authoritative Engine record.
            self.remove_sessions(vec![id]);
            return;
        }
        self.auto_resume_if_needed(&id);
        if is_new {
            if self.selected_session_id.as_ref() == Some(&id) {
                self.focus_session(id);
            } else {
                self.auto_select_if_needed();
            }
        }
        self.reconcile_navigation();
    }

    pub fn remove_session_record(&mut self, id: &SessionId) {
        if self.selected_session_id.as_ref() == Some(id) {
            self.focus_neighbor(&HashSet::from([id.clone()]));
        }
        self.sessions.remove(id);
        self.closing.remove(id);
        self.sidebar_selection.remove(id);
        self.mru_order.retain(|candidate| candidate != id);
        self.auto_resuming.remove(id);
        self.migrating.remove(id);
        if self.terminal_residency.remove(id) {
            self.emit(StoreEffect::DetachAttachment(id.clone()));
        }
        self.invalidate_projection();
        self.sync_sidebar_order();
        self.reconcile_navigation();
    }

    pub fn select(&mut self, id: SessionId) {
        if !self.sessions.contains_key(&id) {
            return;
        }
        self.sidebar_selection.clear();
        self.sidebar_selection_anchor = Some(id.clone());
        self.focus_session(id);
    }

    pub(super) fn apply_spawn_result(&mut self, id: SessionId) {
        // session.updated and the spawn response travel on separate channels,
        // so either can arrive first. focus_session handles both orderings:
        // if the record is already present it grants terminal residency now;
        // otherwise upsert_session will finish that work when the event lands.
        self.focus_session(id);
    }

    pub fn sidebar_click(&mut self, id: SessionId, modifiers: ClickModifiers) {
        if !self.sessions.contains_key(&id) {
            return;
        }
        if modifiers.shift {
            let order = self.sidebar_visible_order();
            let Some(clicked) = order.iter().position(|candidate| candidate == &id) else {
                return;
            };
            let anchor = self
                .sidebar_selection_anchor
                .as_ref()
                .and_then(|anchor| order.iter().position(|candidate| candidate == anchor))
                .or_else(|| {
                    self.selected_session_id.as_ref().and_then(|selected| {
                        order.iter().position(|candidate| candidate == selected)
                    })
                })
                .unwrap_or(clicked);
            let range = anchor.min(clicked)..=anchor.max(clicked);
            self.sidebar_selection = order[range].iter().cloned().collect();
        } else if modifiers.command {
            if self.sidebar_selection.is_empty()
                && let Some(focused) = self.selected_session_id.clone()
                && focused != id
                && self.sessions.contains_key(&focused)
            {
                self.sidebar_selection.insert(focused);
            }
            if self.sidebar_selection.remove(&id) {
                if self.selected_session_id.as_ref() == Some(&id)
                    && let Some(next) = self.sidebar_selection_ordered().first().cloned()
                {
                    self.focus_session(next);
                }
            } else {
                self.sidebar_selection.insert(id.clone());
                self.sidebar_selection_anchor = Some(id);
            }
        } else {
            self.select(id);
        }
    }

    pub fn clear_sidebar_selection(&mut self) {
        self.sidebar_selection.clear();
    }

    pub fn sidebar_selection_ordered(&mut self) -> Vec<SessionId> {
        self.sidebar_projection()
            .display_order
            .iter()
            .filter(|id| self.sidebar_selection.contains(*id))
            .cloned()
            .collect()
    }

    pub fn focus_neighbor(&mut self, excluded: &HashSet<SessionId>) {
        let order: Vec<_> = self
            .sidebar_projection()
            .ordered_sessions
            .iter()
            .map(|session| session.id.clone())
            .collect();
        let survivors: Vec<_> = order
            .iter()
            .filter(|id| !excluded.contains(*id))
            .cloned()
            .collect();
        let Some(current) = self.selected_session_id.clone() else {
            self.set_selected_survivor(survivors.first().cloned());
            return;
        };
        let Some(index) = order.iter().position(|id| id == &current) else {
            self.set_selected_survivor(survivors.first().cloned());
            return;
        };
        let project = self
            .sessions
            .get(&current)
            .map(|session| &session.project_id);
        let eligible = |id: &&SessionId| !excluded.contains(*id);
        let same_project = |id: &&SessionId| {
            eligible(id) && self.sessions.get(*id).map(|session| &session.project_id) == project
        };
        let next = order[index..]
            .iter()
            .find(same_project)
            .or_else(|| order[..index].iter().rev().find(same_project))
            .or_else(|| order[index..].iter().find(eligible))
            .or_else(|| order[..index].iter().rev().find(eligible))
            .cloned()
            .or_else(|| survivors.first().cloned());
        self.set_selected_survivor(next);
    }

    pub fn mru_sessions(&mut self) -> Vec<SessionId> {
        let mut seen = HashSet::new();
        let mut result = Vec::new();
        for id in &self.mru_order {
            if self.sessions.contains_key(id) && seen.insert(id.clone()) {
                result.push(id.clone());
            }
        }
        for session in &self.sidebar_projection().ordered_sessions {
            if seen.insert(session.id.clone()) {
                result.push(session.id.clone());
            }
        }
        result
    }
}
