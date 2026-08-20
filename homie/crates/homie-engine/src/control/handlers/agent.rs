use super::*;

impl ControlServer {
    /// A hook or notify callback from inside an agent session: the signal
    /// that makes hook-authority agents' status precise. Parsed by the same
    /// rules the reference implementation used, metadata folded into the record, signal
    /// fed to the session's reducer.
    pub(crate) fn hook_report(&self, params: Option<JsonValue>) -> Result<JsonValue, ControlError> {
        let p: homie_proto::HookReportParams = decode(params)?;
        let Some(session_id) = p.homie_session_id else {
            return Ok(json!({}));
        };
        let parsed = match p.kind.as_str() {
            "claude-hook" => p.event.as_deref().and_then(|event| {
                crate::hooks::parse_claude_hook(event, &p.payload, std::time::SystemTime::now())
            }),
            "codex-notify" => crate::hooks::parse_codex_notify(&p.payload),
            _ => None,
        };
        let Some((signal, meta)) = parsed else {
            return Ok(json!({}));
        };
        let mut registry = self.registry.lock().map_err(poisoned)?;
        let changed = registry.apply_hook_metadata(&session_id.0, &meta);
        if let Some(session) = registry.get(&session_id.0) {
            session.feed_signal(signal);
        }
        if changed {
            let _ = registry.persist();
        }
        self.publish_updated(&registry, &session_id.0);
        Ok(json!({}))
    }

    /// Which agent binaries actually resolve, plus each manifest's descriptor
    /// — this doubles as the agent catalog the client's picker renders.
    pub(crate) fn agent_readiness(&self) -> Result<JsonValue, ControlError> {
        let registry = self.registry.lock().map_err(poisoned)?;
        let engine = registry.engine();
        let mut agents = Vec::new();
        for id in engine.ids() {
            let Some(manifest) = engine.manifest(id) else {
                continue;
            };
            let Some(descriptor) = &manifest.agent else {
                continue;
            };
            let Some(binary) = &descriptor.binary else {
                continue;
            };
            agents.push(json!({
                "kind": id,
                "binary": binary,
                "path": resolve_on_path(binary),
                "descriptor": engine.raw_agent(id),
            }));
        }
        Ok(json!({ "agents": agents }))
    }

    pub(crate) fn environment_refresh_path(&self) -> Result<JsonValue, ControlError> {
        let app_support = self
            .socket_path
            .parent()
            .ok_or_else(|| ControlError::internal("daemon socket has no parent directory"))?;
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".into());
        let outcome = crate::environment::refresh_path(app_support, &shell, Duration::from_secs(2))
            .map_err(|error| ControlError::internal(format!("path refresh failed: {error}")))?;
        Ok(json!({
            "path": outcome.path,
            "updated": matches!(outcome.status, crate::environment::RefreshStatus::Updated),
        }))
    }

    pub(crate) fn project_add(&self, params: Option<JsonValue>) -> Result<JsonValue, ControlError> {
        let p: homie_proto::ProjectAddParams = decode(params)?;
        let mut registry = self.registry.lock().map_err(poisoned)?;
        let project = registry.add_project(&p.root);
        let _ = registry.persist();
        Ok(project)
    }

    /// The working tree's diff against a base ref, for the app's diff pane.
    pub(crate) fn session_read_diff(
        &self,
        params: Option<JsonValue>,
    ) -> Result<JsonValue, ControlError> {
        let p: homie_proto::SessionReadDiffParams = decode(params)?;
        let (cwd, host_id) = {
            let registry = self.registry.lock().map_err(poisoned)?;
            registry
                .records()
                .into_iter()
                .find(|record| record.id.0 == p.session_id.0)
                .map(|record| (record.cwd, record.host))
                .ok_or_else(|| ControlError::not_found(p.session_id.0.clone()))?
        };
        let result = if let Some(host_id) = host_id {
            let manager = self
                .remote
                .as_ref()
                .ok_or_else(crate::remote::transport_unavailable)?;
            let host = self.resolve_host(&host_id)?;
            crate::git::working_diff_remote(manager, &host, &cwd, p.base.as_ref())
                .map_err(io_control_error)?
        } else {
            crate::git::working_diff(Path::new(&cwd), p.base.as_ref()).map_err(io_control_error)?
        };
        encode(&result)
    }
}
