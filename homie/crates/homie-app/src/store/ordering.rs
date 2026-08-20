use super::*;

impl super::SessionStore {
    pub(super) fn reconcile_sidebar_order(&mut self) -> bool {
        let live_sessions: HashSet<&SessionId> = self.sessions.keys().collect();
        let live_projects: HashSet<&ProjectId> = self.projects.keys().collect();
        let mut arrivals: Vec<(f64, &SessionId)> = self
            .sessions
            .values()
            .map(|session| (session.created_at.0, &session.id))
            .collect();
        arrivals.sort_by(|(left, left_id), (right, right_id)| {
            left.total_cmp(right)
                .then_with(|| left_id.0.cmp(&right_id.0))
        });

        // A project arrives with its oldest session, matching the projection.
        let mut project_arrivals: HashMap<&ProjectId, f64> = HashMap::new();
        for session in self.sessions.values() {
            let arrival = project_arrivals
                .entry(&session.project_id)
                .or_insert(f64::INFINITY);
            *arrival = arrival.min(session.created_at.0);
        }
        let mut projects: Vec<(f64, &ProjectId)> = self
            .projects
            .keys()
            .map(|id| {
                (
                    project_arrivals.get(id).copied().unwrap_or(f64::INFINITY),
                    id,
                )
            })
            .collect();
        projects.sort_by(|(left, left_id), (right, right_id)| {
            left.total_cmp(right)
                .then_with(|| left_id.0.cmp(&right_id.0))
        });

        let sessions = reconcile_order(
            &self.prefs.sidebar_session_order,
            &live_sessions,
            arrivals.into_iter().map(|(_, id)| id),
        );
        let ordered_projects = reconcile_order(
            &self.prefs.sidebar_project_order,
            &live_projects,
            projects.into_iter().map(|(_, id)| id),
        );
        // Collapse and pin state for rows that no longer exist would otherwise
        // accumulate in prefs.json forever and reattach to a recycled id.
        let collapsed_sessions =
            retain_live(&self.prefs.sidebar_collapsed_sessions, &live_sessions);
        let pinned_sessions = retain_live(&self.prefs.sidebar_pinned_sessions, &live_sessions);

        let mut changed = false;
        if let Some(order) = sessions {
            self.prefs.sidebar_session_order = order;
            changed = true;
        }
        if let Some(order) = ordered_projects {
            self.prefs.sidebar_project_order = order;
            changed = true;
        }
        if let Some(collapsed) = collapsed_sessions {
            self.prefs.sidebar_collapsed_sessions = collapsed;
            changed = true;
        }
        if let Some(pinned) = pinned_sessions {
            self.prefs.sidebar_pinned_sessions = pinned;
            changed = true;
        }
        if changed {
            self.invalidate_projection();
        }
        changed
    }

    pub(super) fn sync_sidebar_order(&mut self) {
        if self.reconcile_sidebar_order() {
            let _ = self.persist_preferences();
        }
    }

    pub fn sidebar_session_order(&mut self) -> Vec<SessionId> {
        self.reconcile_sidebar_order();
        self.prefs.sidebar_session_order.clone()
    }

    pub fn sidebar_project_order(&mut self) -> Vec<ProjectId> {
        self.reconcile_sidebar_order();
        self.prefs.sidebar_project_order.clone()
    }

    pub fn set_project_order(&mut self, order: Vec<ProjectId>) -> io::Result<()> {
        self.update_preferences(|prefs| prefs.sidebar_project_order = order)
    }

    pub fn set_session_order(&mut self, order: Vec<SessionId>) -> io::Result<()> {
        self.update_preferences(|prefs| prefs.sidebar_session_order = order)
    }

    pub fn stage_project_order(&mut self, order: Vec<ProjectId>) -> bool {
        if self.prefs.sidebar_project_order == order {
            return false;
        }
        self.prefs.sidebar_project_order = order;
        self.prefs.normalize();
        self.invalidate_projection();
        true
    }

    pub fn stage_session_order(&mut self, order: Vec<SessionId>) -> bool {
        if self.prefs.sidebar_session_order == order {
            return false;
        }
        self.prefs.sidebar_session_order = order;
        self.prefs.normalize();
        self.invalidate_projection();
        true
    }

    pub fn toggle_project_pin(&mut self, id: ProjectId) -> io::Result<()> {
        self.update_preferences(|prefs| toggle_vec_member(&mut prefs.sidebar_pinned_projects, id))
    }

    pub fn toggle_session_pin(&mut self, id: SessionId) -> io::Result<()> {
        self.update_preferences(|prefs| toggle_vec_member(&mut prefs.sidebar_pinned_sessions, id))
    }

    pub fn toggle_project_collapsed(&mut self, id: ProjectId) -> io::Result<()> {
        self.update_preferences(|prefs| {
            toggle_vec_member(&mut prefs.sidebar_collapsed_projects, id)
        })
    }

    pub fn toggle_session_collapsed(&mut self, id: SessionId) -> io::Result<()> {
        let collapsing = !self.prefs.sidebar_collapsed_sessions.contains(&id);
        if collapsing
            && let Some(selected) = self.selected_session_id.clone()
            && self.is_descendant_of(&selected, &id)
        {
            self.select(id.clone());
        }
        self.update_preferences(|prefs| {
            toggle_vec_member(&mut prefs.sidebar_collapsed_sessions, id)
        })
    }

    pub(super) fn is_descendant_of(&self, candidate: &SessionId, ancestor: &SessionId) -> bool {
        let mut seen = HashSet::from([candidate.clone()]);
        let mut cursor = self
            .sessions
            .get(candidate)
            .and_then(|session| session.parent.clone());
        while let Some(node) = cursor {
            if &node == ancestor {
                return true;
            }
            if !seen.insert(node.clone()) {
                return false;
            }
            cursor = self
                .sessions
                .get(&node)
                .and_then(|session| session.parent.clone());
        }
        false
    }

    pub fn toggle_archive_expanded(&mut self, id: ProjectId) -> io::Result<()> {
        let was_expanded = self.prefs.sidebar_expanded_archives.contains(&id);
        if was_expanded {
            self.sidebar_selection.retain(|session_id| {
                !self
                    .sessions
                    .get(session_id)
                    .is_some_and(|session| session.project_id == id && session.is_archived())
            });
        }
        self.update_preferences(|prefs| toggle_vec_member(&mut prefs.sidebar_expanded_archives, id))
    }

    pub fn governor_settings(&self) -> GovernorConfigureParams {
        GovernorConfigureParams::new(
            f64::from(self.prefs.hibernate_after_minutes) * 60.0,
            self.prefs
                .memory_hard_limit_gb
                .saturating_mul(1024 * 1024 * 1024),
        )
    }

    pub fn sidebar_projection(&mut self) -> Arc<SidebarProjection> {
        if let Some((revision, projection)) = &self.cached_projection
            && *revision == self.revision
        {
            return Arc::clone(projection);
        }
        let projection = Arc::new(projection::build_projection(
            &self.sessions,
            &self.projects,
            &self.prefs,
            self.selected_session_id.as_ref(),
            &self.closing,
        ));
        self.cached_projection = Some((self.revision, Arc::clone(&projection)));
        projection
    }

    pub fn ordered_sessions(&mut self) -> Vec<SessionRecord> {
        self.sidebar_projection()
            .ordered_sessions
            .iter()
            .map(|session| session.as_ref().clone())
            .collect()
    }

    pub fn selected_session(&self) -> Option<&SessionRecord> {
        self.selected_session_id
            .as_ref()
            .and_then(|id| self.sessions.get(id))
            .map(Arc::as_ref)
    }
}
