use super::*;

impl ControlServer {
    /// `host.sync_prefs`: push the local agent preferences to a host so
    /// agents there behave like local ones. Additive rsync, fixed include
    /// list, per-tool reporting.
    pub(crate) fn host_sync_prefs(
        &self,
        params: Option<JsonValue>,
    ) -> Result<JsonValue, ControlError> {
        let p: homie_proto::HostSyncPrefsParams = decode(params)?;
        let entry = self.resolve_host(&p.host)?;
        let home = std::env::var("HOME")
            .map(PathBuf::from)
            .map_err(|_| ControlError::internal("HOME is not set"))?;
        encode(&crate::hosts::sync_prefs(&entry, &home))
    }

    /// `host.initialize`: run the complete idempotent SSH bootstrap before a
    /// user creates the first session. No environment values cross back into
    /// the app; only facts suitable for a visible readiness summary do.
    pub(crate) fn host_initialize(
        &self,
        params: Option<JsonValue>,
    ) -> Result<JsonValue, ControlError> {
        let p: homie_proto::HostInitializeParams = decode(params)?;
        let manager = self
            .remote
            .as_ref()
            .ok_or_else(crate::remote::transport_unavailable)?;
        let host = self.resolve_host(&p.host)?;
        let helper = if p.force_reinstall {
            manager.reinstall_helper(&host)
        } else {
            manager.ensure_helper(&host)
        }
        .map_err(io_control_error)?;
        let persistence = manager
            .probe_persistence(&host, &helper)
            .map_err(io_control_error)?;
        let captured = manager
            .capture_environment(
                &helper,
                &homie_proto::remote_pty::EnvironmentCaptureRequest {
                    cwd: Some(host.default_cwd.clone().unwrap_or_else(|| "~".into())),
                    timeout_millis: 10_000,
                },
            )
            .map_err(io_control_error)?;
        encode(&homie_proto::HostInitializeResult {
            helper_build_id: helper.build_id,
            protocol: helper.protocol,
            persistence,
            cwd: captured.cwd,
            shell: captured.shell,
        })
    }

    /// `host.list_directories`: one shallow, bounded filesystem read on the
    /// requested execution machine. Remote work stays behind the Engine and
    /// uses the verified Helper over `ssh -T`; the app never executes SSH.
    pub(crate) fn host_list_directories(
        &self,
        params: Option<JsonValue>,
    ) -> Result<JsonValue, ControlError> {
        let p: homie_proto::HostListDirectoriesParams = decode(params)?;
        let request = homie_proto::remote_pty::DirectoryListRequest { path: p.path };
        let result = if let Some(host_id) = p.host {
            let manager = self
                .remote
                .as_ref()
                .ok_or_else(crate::remote::transport_unavailable)?;
            let host = self.resolve_host(&host_id)?;
            manager
                .list_directories(&host, &request)
                .map_err(io_control_error)?
        } else {
            crate::directories::list(&request).map_err(io_control_error)?
        };
        encode(&result)
    }

    /// `host.locate_repo`: find a checkout by origin URL (given directly, or
    /// derived from a session's cwd + host).
    pub(crate) fn host_locate_repo(
        &self,
        params: Option<JsonValue>,
    ) -> Result<JsonValue, ControlError> {
        let p: homie_proto::HostLocateRepoParams = decode(params)?;
        let target = p
            .host
            .as_deref()
            .map(|id| self.resolve_host(id))
            .transpose()?;

        let mut origin = p.origin_url.clone();
        if origin.is_none()
            && let Some(session_id) = &p.session_id
        {
            let (cwd, source_host) = {
                let registry = self.registry.lock().map_err(poisoned)?;
                let record = registry
                    .records()
                    .into_iter()
                    .find(|record| record.id.0 == session_id.0)
                    .ok_or_else(|| ControlError::not_found(session_id.0.clone()))?;
                (record.cwd, record.host)
            };
            let source = source_host
                .as_deref()
                .map(|id| self.resolve_host(id))
                .transpose()?;
            origin = crate::hosts::origin_of_cwd(&cwd, source.as_ref());
        }
        let Some(origin) = origin else {
            return encode(&homie_proto::HostLocateRepoResult {
                path: None,
                origin_url: None,
            });
        };

        let local_roots: Vec<String> = {
            let registry = self.registry.lock().map_err(poisoned)?;
            registry
                .projects_raw()
                .iter()
                .filter_map(|project| project.get("root").and_then(|value| value.as_str()))
                .map(str::to_string)
                .collect()
        };
        let path = crate::hosts::locate(&origin, target.as_ref(), &local_roots);
        encode(&homie_proto::HostLocateRepoResult {
            path,
            origin_url: Some(origin),
        })
    }

    /// Resolves a host id against `hosts.json`, read fresh each call so
    /// Settings edits apply without a daemon restart.
    pub(crate) fn resolve_host(
        &self,
        host_id: &str,
    ) -> Result<homie_proto::HostEntry, ControlError> {
        crate::session::resolve_host(&self.socket_path, host_id)
    }

    /// Applies the current application build's remote environment gate before
    /// a stateless SSH action. Live Holder operations deliberately use their
    /// session binding's creation-time Helper instead.
    pub(crate) fn hosts_file(&self) -> PathBuf {
        self.socket_path
            .parent()
            .map(|parent| parent.join("hosts.json"))
            .unwrap_or_else(|| PathBuf::from("hosts.json"))
    }
}
