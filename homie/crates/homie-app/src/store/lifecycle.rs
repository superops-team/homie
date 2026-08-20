use super::*;

impl super::SessionStore {
    pub fn headless(prefs: Prefs) -> (Self, mpsc::UnboundedReceiver<StoreEffect>) {
        Self::with_path(prefs, None)
    }

    pub fn load(
        path: impl Into<PathBuf>,
    ) -> io::Result<(Self, mpsc::UnboundedReceiver<StoreEffect>)> {
        let path = path.into();
        let prefs = Prefs::load(&path)?;
        let (mut store, receiver) = Self::with_path(prefs, Some(path));
        store.reload_hosts();
        Ok((store, receiver))
    }

    pub(super) fn with_path(
        prefs: Prefs,
        prefs_path: Option<PathBuf>,
    ) -> (Self, mpsc::UnboundedReceiver<StoreEffect>) {
        let (effects, receiver) = mpsc::unbounded_channel();
        let selected_session_id = prefs.last_selected_session.clone();
        (
            Self {
                daemon_state: DaemonState::Connecting,
                sessions: HashMap::new(),
                projects: HashMap::new(),
                selected_session_id: selected_session_id.clone(),
                sidebar_selection: HashSet::new(),
                pending_close: None,
                closing: HashSet::new(),
                auto_resuming: HashSet::new(),
                migrating: HashSet::new(),
                syncing_prefs: HashSet::new(),
                repo_targets: HashMap::new(),
                repo_target_session: None,
                directory_request_seq: 0,
                directory_listing: None,
                prefs,
                terminal_residency: TerminalResidency::default(),
                app_is_active: true,
                last_action_error: None,
                sidebar_selection_anchor: None,
                mru_order: selected_session_id.into_iter().collect(),
                switcher: SessionSwitcherState::default(),
                overview: SessionOverviewState::default(),
                auto_resume_attempted: HashSet::new(),
                revision: 0,
                cached_projection: None,
                prefs_path,
                hosts: Vec::new(),
                agents: AgentReadinessResult::default(),
                effects,
            },
            receiver,
        )
    }

    pub fn load_default() -> io::Result<(Self, mpsc::UnboundedReceiver<StoreEffect>)> {
        Self::load(Prefs::path())
    }

    pub fn persist_preferences(&self) -> io::Result<()> {
        self.prefs_path
            .as_deref()
            .map_or(Ok(()), |path| self.prefs.save(path))
    }

    pub fn remember_window_placement(&mut self, placement: WindowPlacement) {
        self.prefs.window_placement = Some(placement);
    }

    pub fn daemon_state(&self) -> &DaemonState {
        &self.daemon_state
    }

    pub fn sessions(&self) -> &HashMap<SessionId, Arc<SessionRecord>> {
        &self.sessions
    }

    pub fn auxiliary_terminal_for(&self, parent: &SessionId) -> Option<Arc<SessionRecord>> {
        self.sessions
            .values()
            .filter(|session| {
                session.parent.as_ref() == Some(parent)
                    && is_auxiliary_terminal(session)
                    && !session.is_archived()
                    && !self.closing.contains(&session.id)
            })
            .max_by(|left, right| {
                left.created_at
                    .partial_cmp(&right.created_at)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .cloned()
    }

    pub fn projects(&self) -> &HashMap<ProjectId, Project> {
        &self.projects
    }

    pub fn selected_session_id(&self) -> Option<&SessionId> {
        self.selected_session_id.as_ref()
    }

    pub fn sidebar_selection(&self) -> &HashSet<SessionId> {
        &self.sidebar_selection
    }

    pub fn pending_close(&self) -> Option<&PendingClose> {
        self.pending_close.as_ref()
    }

    pub fn auto_resuming(&self) -> &HashSet<SessionId> {
        &self.auto_resuming
    }

    pub fn migrating(&self) -> &HashSet<SessionId> {
        &self.migrating
    }

    pub fn syncing_prefs(&self) -> &HashSet<String> {
        &self.syncing_prefs
    }
}
