use super::*;

impl super::SessionStore {
    pub fn request_close(&mut self, ids: Vec<SessionId>) {
        // Rows already on their way out ignore further clicks: without this a
        // second ✕ re-arms the confirmation for a session that is gone.
        let ids: Vec<_> = ids
            .into_iter()
            .filter(|id| !self.closing.contains(id))
            .collect();
        if ids.is_empty() {
            return;
        }
        let has_running = ids.iter().any(|id| {
            self.sessions
                .get(id)
                .is_some_and(|session| !matches!(session.status, SessionStatus::Exited(_)))
        });
        if self.prefs.confirm_before_closing_session && has_running {
            self.pending_close = Some(PendingClose { ids });
        } else {
            self.remove_sessions(ids);
        }
    }

    pub fn confirm_pending_close(&mut self) {
        if let Some(pending) = self.pending_close.take() {
            self.remove_sessions(pending.ids);
        }
    }

    pub fn cancel_pending_close(&mut self) {
        self.pending_close = None;
    }

    pub fn remove_sessions(&mut self, ids: Vec<SessionId>) {
        let mut ids = ids;
        let parents: HashSet<_> = ids.iter().cloned().collect();
        ids.extend(
            self.sessions
                .values()
                .filter(|session| {
                    session
                        .parent
                        .as_ref()
                        .is_some_and(|parent| parents.contains(parent))
                        && is_auxiliary_terminal(session)
                })
                .map(|session| session.id.clone()),
        );
        let mut unique = HashSet::new();
        ids.retain(|id| unique.insert(id.clone()));
        let excluded: HashSet<_> = ids.iter().cloned().collect();
        if self
            .selected_session_id
            .as_ref()
            .is_some_and(|id| excluded.contains(id))
        {
            self.focus_neighbor(&excluded);
        }
        for id in ids {
            self.sidebar_selection.remove(&id);
            if self.terminal_residency.remove(&id) {
                self.emit(StoreEffect::DetachAttachment(id.clone()));
            }
            // Hide now; `session.removed` (or a resync) settles the record.
            self.closing.insert(id.clone());
            self.emit(StoreEffect::Remove(id));
        }
        self.invalidate_projection();
        self.reconcile_navigation();
    }

    pub fn archive_sessions(&mut self, ids: Vec<SessionId>) {
        let targets: Vec<_> = ids
            .into_iter()
            .filter(|id| {
                self.sessions
                    .get(id)
                    .is_some_and(|session| !session.is_archived())
            })
            .collect();
        let excluded: HashSet<_> = targets.iter().cloned().collect();
        if self
            .selected_session_id
            .as_ref()
            .is_some_and(|id| excluded.contains(id))
        {
            self.focus_neighbor(&excluded);
        }
        for id in targets {
            self.sidebar_selection.remove(&id);
            if let Some(session) = self.sessions.get_mut(&id) {
                Arc::make_mut(session).archived_at = Some(now_millis());
            }
            if self.terminal_residency.remove(&id) {
                self.emit(StoreEffect::DetachAttachment(id.clone()));
            }
            self.emit(StoreEffect::Archive(id));
        }
        self.invalidate_projection();
    }

    pub fn revive_sessions(&mut self, ids: Vec<SessionId>) {
        let mut revived = Vec::new();
        for id in ids {
            let Some(session) = self.sessions.get_mut(&id) else {
                continue;
            };
            let session = Arc::make_mut(session);
            if !session.is_archived() {
                continue;
            }
            session.archived_at = None;
            let resumable = session.resumability == Resumability::Resumable;
            revived.push(id.clone());
            self.emit(if resumable {
                StoreEffect::Resume {
                    id,
                    automatic: false,
                }
            } else {
                StoreEffect::Unarchive(id)
            });
        }
        self.invalidate_projection();
        if let Some(first) = revived.first().cloned() {
            self.select(first);
        }
    }

    pub fn auto_resume_if_needed(&mut self, id: &SessionId) -> bool {
        let eligible = self.selected_session_id.as_ref() == Some(id)
            && self.sessions.get(id).is_some_and(|session| {
                !session.is_archived()
                    && session.resumability == Resumability::Resumable
                    && matches!(
                        &session.status,
                        SessionStatus::Exited(info) if info.reason == ExitReason::DaemonRestart
                    )
            });
        if !eligible || !self.auto_resume_attempted.insert(id.clone()) {
            return false;
        }
        self.auto_resuming.insert(id.clone());
        self.emit(StoreEffect::Resume {
            id: id.clone(),
            automatic: true,
        });
        true
    }

    pub fn finish_auto_resume(&mut self, id: &SessionId) {
        self.auto_resuming.remove(id);
    }

    pub fn resume(&self, id: SessionId) {
        self.emit(StoreEffect::Resume {
            id,
            automatic: false,
        });
    }

    pub fn rename(&mut self, id: SessionId, title: impl Into<String>) {
        let title = title.into();
        if let Some(session) = self.sessions.get_mut(&id) {
            let session = Arc::make_mut(session);
            session.title.clone_from(&title);
            session.title_source = homie_proto::TitleSource::UserRename;
            self.invalidate_projection();
        }
        self.emit(StoreEffect::Rename { id, title });
    }

    pub fn reopen_last(&self) {
        self.emit(StoreEffect::ReopenLast);
    }

    pub fn spawn_default(&mut self, mut options: SpawnOptions) {
        if options.host.is_none() && options.cwd.is_none() && options.same_repo_as.is_none() {
            options.host = self.default_spawn_host();
        }
        self.spawn_kind(self.prefs.default_agent.kind(), options);
    }

    pub fn spawn_shell(&mut self, mut options: SpawnOptions) {
        if options.host.is_none() && options.cwd.is_none() && options.same_repo_as.is_none() {
            options.host = self.default_spawn_host();
        }
        self.spawn_kind(AgentKind::SHELL, options);
    }

    pub fn spawn_auxiliary_terminal(&mut self, parent: SessionId) -> bool {
        let Some(session) = self.sessions.get(&parent) else {
            return false;
        };
        if self.auxiliary_terminal_for(&parent).is_some() {
            return false;
        }
        self.last_action_error = None;
        self.emit(StoreEffect::SpawnAuxiliary(SessionSpawnParams {
            kind: AgentKind::SHELL,
            cwd: session.cwd.clone(),
            new_worktree: None,
            worktree_branch: None,
            title: Some(AUXILIARY_TERMINAL_TITLE.to_owned()),
            initial_prompt: None,
            parent: Some(parent),
            initial_cols: None,
            initial_rows: None,
            host: session.host.clone(),
            same_repo_as: None,
        }));
        true
    }

    pub fn spawn_kind(&mut self, kind: AgentKind, options: SpawnOptions) {
        let host = options.host;
        let cwd = if let Some(host_id) = &host {
            // Remote spawn: local directories are meaningless — use the
            // explicit remote override or the host's default cwd.
            options
                .cwd
                .or_else(|| self.host(host_id).and_then(|host| host.default_cwd.clone()))
                .unwrap_or_else(|| "~".to_owned())
        } else {
            options.cwd.unwrap_or_else(|| self.active_directory())
        };
        // Worktrees are a local-git feature; drop them for remote spawns (the
        // daemon rejects the combination outright).
        let worktree = if host.is_some() {
            None
        } else {
            options.worktree
        };
        let (new_worktree, worktree_branch) = worktree.map_or((None, None), |worktree| {
            (Some(worktree.create), worktree.branch)
        });
        self.emit(StoreEffect::Spawn(SessionSpawnParams {
            kind,
            cwd,
            new_worktree,
            worktree_branch,
            title: options.title,
            initial_prompt: options.initial_prompt,
            parent: options.parent,
            initial_cols: options.initial_cols,
            initial_rows: options.initial_rows,
            host,
            same_repo_as: options.same_repo_as,
        }));
    }

    pub fn local_fallback_directory(&self) -> String {
        if self
            .selected_session()
            .is_none_or(|session| session.host.is_none())
        {
            return self.default_new_agent_directory();
        }
        let mut roots: Vec<_> = self
            .projects
            .values()
            .map(|project| project.root.clone())
            .filter(|root| Path::new(root).is_dir())
            .collect();
        roots.sort();
        roots
            .into_iter()
            .next()
            .or_else(|| std::env::var("HOME").ok())
            .unwrap_or_else(|| "/".to_owned())
    }

    pub fn default_new_agent_directory(&self) -> String {
        if let Some(session) = self.selected_session()
            && let Some(project) = self.projects.get(&session.project_id)
        {
            return project.root.clone();
        }
        self.active_directory()
    }

    pub fn active_directory(&self) -> String {
        if let Some(session) = self.selected_session() {
            return session.cwd.clone();
        }
        let projection = projection::build_projection(
            &self.sessions,
            &self.projects,
            &self.prefs,
            self.selected_session_id.as_ref(),
            &self.closing,
        );
        projection
            .first_active()
            .map(|session| session.cwd.clone())
            .or_else(|| {
                projection
                    .projects
                    .first()
                    .map(|group| group.project.root.clone())
            })
            .or_else(|| std::env::var("HOME").ok())
            .unwrap_or_else(|| "/".to_owned())
    }

    pub fn set_active(&mut self, active: bool) {
        if self.app_is_active == active {
            return;
        }
        self.app_is_active = active;
        self.emit(StoreEffect::SetActive(active));
    }
}
