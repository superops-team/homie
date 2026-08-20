use super::*;

impl super::SessionStore {
    pub fn reload_hosts(&mut self) {
        self.hosts = std::env::var_os("HOME")
            .map(|home| HostsConfig::load(HomiePaths::hosts_config_file(home)).hosts)
            .unwrap_or_default();
        self.repair_default_spawn_host();
    }

    pub fn set_agent_catalog(&mut self, agents: AgentReadinessResult) {
        self.agents = agents;
    }

    pub fn refresh_agent_catalog(&self) {
        self.emit(StoreEffect::RefreshAgentCatalog);
    }

    pub fn agent_catalog(&self) -> &AgentReadinessResult {
        &self.agents
    }

    pub fn agent_descriptor(&self, kind: &AgentKind) -> Option<&AgentDescriptor> {
        self.agents.descriptor(kind)
    }

    pub fn set_hosts(&mut self, hosts: Vec<HostEntry>) {
        self.hosts = hosts;
        self.repair_default_spawn_host();
    }

    pub fn hosts(&self) -> &[HostEntry] {
        &self.hosts
    }

    pub fn host(&self, id: &str) -> Option<&HostEntry> {
        self.hosts.iter().find(|host| host.id == id)
    }

    pub fn host_display_name(&self, id: &str) -> String {
        self.host(id)
            .map_or_else(|| id.to_owned(), |host| host.display_name().to_owned())
    }

    pub fn default_spawn_host(&self) -> Option<String> {
        self.prefs
            .default_spawn_host
            .as_deref()
            .filter(|id| self.host(id).is_some())
            .map(str::to_owned)
    }

    pub fn set_default_spawn_host(&mut self, host: Option<String>) {
        let host = host.filter(|id| self.host(id).is_some());
        if self.prefs.default_spawn_host == host {
            return;
        }
        self.prefs.default_spawn_host = host;
        if let Err(error) = self.persist_preferences() {
            eprintln!("homie: could not save the default spawn host: {error}");
        }
    }

    pub(super) fn repair_default_spawn_host(&mut self) {
        if self
            .prefs
            .default_spawn_host
            .as_deref()
            .is_some_and(|id| self.host(id).is_none())
        {
            self.prefs.default_spawn_host = None;
            if let Err(error) = self.persist_preferences() {
                eprintln!("homie: could not clear a removed default host: {error}");
            }
        }
    }

    pub fn migrate_session(&mut self, id: SessionId, target_host: Option<String>) {
        let Some(session) = self.sessions.get(&id) else {
            return;
        };
        if session.kind != AgentKind::CLAUDE_CODE
            || session.host == target_host
            || self.migrating.contains(&id)
        {
            return;
        }
        self.migrating.insert(id.clone());
        self.emit(StoreEffect::Migrate { id, target_host });
    }

    pub fn finish_migration(&mut self, id: &SessionId) {
        self.migrating.remove(id);
    }

    pub fn sync_prefs(&mut self, host: String) {
        if self.syncing_prefs.contains(&host) {
            return;
        }
        let host_name = self.host_display_name(&host);
        self.syncing_prefs.insert(host.clone());
        self.emit(StoreEffect::SyncPrefs { host, host_name });
    }

    pub fn finish_prefs_sync(&mut self, host: &str) {
        self.syncing_prefs.remove(host);
    }

    pub fn begin_repo_targeting(&mut self) -> Option<String> {
        self.repo_targets.clear();
        self.repo_target_session = self.selected_session_id.clone();
        self.prefs
            .default_spawn_host
            .clone()
            .filter(|host| self.host(host).is_some())
    }

    pub fn request_repo_target(&mut self, host: Option<String>) {
        let Some(session_id) = self.repo_target_session.clone() else {
            return;
        };
        let key = repo_target_key(host.as_deref());
        if self.repo_targets.contains_key(&key) {
            return;
        }
        self.repo_targets.insert(key.clone(), RepoTarget::Pending);
        self.emit(StoreEffect::LocateRepo {
            key,
            host,
            session_id,
        });
    }

    pub fn repo_target(&self, host: Option<&str>) -> Option<&RepoTarget> {
        self.repo_targets.get(&repo_target_key(host))
    }

    pub fn request_directory_listing(&mut self, host: Option<String>, path: String) {
        self.directory_request_seq = self.directory_request_seq.wrapping_add(1);
        let request_id = self.directory_request_seq;
        self.directory_listing = Some(DirectoryListing {
            request_id,
            host: host.clone(),
            requested_path: path.clone(),
            state: DirectoryListingState::Loading,
        });
        self.emit(StoreEffect::ListDirectories {
            request_id,
            host,
            path,
        });
    }

    pub fn directory_listing(
        &self,
        host: Option<&str>,
        requested_path: &str,
    ) -> Option<&DirectoryListingState> {
        self.directory_listing
            .as_ref()
            .filter(|listing| {
                listing.host.as_deref() == host && listing.requested_path == requested_path
            })
            .map(|listing| &listing.state)
    }

    pub(super) fn finish_directory_listing(
        &mut self,
        request_id: u64,
        result: Result<DirectoryListResult, String>,
    ) {
        let Some(listing) = self
            .directory_listing
            .as_mut()
            .filter(|listing| listing.request_id == request_id)
        else {
            return;
        };
        listing.state = match result {
            Ok(result) => DirectoryListingState::Ready(result),
            Err(error) => DirectoryListingState::Error(error),
        };
    }

    pub fn set_repo_target(&mut self, key: String, target: RepoTarget) {
        self.repo_targets.insert(key, target);
    }

    pub fn preferences(&self) -> &Prefs {
        &self.prefs
    }

    pub fn terminal_residency(&self) -> &TerminalResidency {
        &self.terminal_residency
    }

    pub fn app_is_active(&self) -> bool {
        self.app_is_active
    }

    pub fn last_action_error(&self) -> Option<&str> {
        self.last_action_error.as_deref()
    }

    pub fn switcher_state(&self) -> &SessionSwitcherState {
        &self.switcher
    }

    pub fn overview_state(&self) -> &SessionOverviewState {
        &self.overview
    }

    pub fn request_snapshot_publish(&mut self) {
        self.emit(StoreEffect::PublishSnapshot);
    }

    pub fn update_preferences(&mut self, update: impl FnOnce(&mut Prefs)) -> io::Result<()> {
        update(&mut self.prefs);
        self.prefs.normalize();
        self.invalidate_projection();
        self.persist_preferences()?;
        if matches!(self.daemon_state, DaemonState::Connected) {
            self.emit(StoreEffect::ConfigureGovernor(self.governor_settings()));
        }
        Ok(())
    }

    pub fn zoom_terminal(&mut self, delta: f32) -> io::Result<()> {
        self.prefs.zoom_terminal(delta);
        self.persist_preferences()
    }

    pub fn reset_terminal_zoom(&mut self) -> io::Result<()> {
        self.prefs.reset_terminal_zoom();
        self.persist_preferences()
    }
}
