use super::*;

impl ControlServer {
    /// Revives an exited session's conversation under the SAME record id.
    pub(crate) fn session_resume(
        &self,
        params: Option<JsonValue>,
    ) -> Result<JsonValue, ControlError> {
        let p: homie_proto::SessionIdParams = decode(params)?;
        let record = {
            let registry = self.registry.lock().map_err(poisoned)?;
            let record = registry
                .records()
                .into_iter()
                .find(|record| record.id.0 == p.session_id.0)
                .ok_or_else(|| ControlError::not_found(p.session_id.0.clone()))?;
            // Presence in the registry is not liveness: only an explicit kill
            // removes a session, so an agent that died on its own is still in
            // the map. Returning here on presence alone would hand back the
            // corpse this call was asked to revive; the exited case falls
            // through to the eviction path below.
            if registry.get(&p.session_id.0).is_some()
                && !matches!(record.status, homie_proto::SessionStatus::Exited(_))
            {
                // Genuinely live: resuming is a no-op, not an error.
                return serde_json::to_value(&record)
                    .map_err(|error| ControlError::internal(error.to_string()));
            }
            record
        };
        let spec = if record.host.is_some() {
            self.remote_resume_spec(&record)?
        } else {
            let registry = self.registry.lock().map_err(poisoned)?;
            self.resume_spec(
                &registry,
                &record.id.0,
                record.kind.id(),
                &record.cwd,
                record.agent_session_id.as_deref(),
            )?
        };
        let remote_persistence = spec.remote.as_ref().map(|remote| remote.launch.persistence);
        let mut registry = self.registry.lock().map_err(poisoned)?;
        let record = registry
            .records()
            .into_iter()
            .find(|record| record.id.0 == p.session_id.0)
            .ok_or_else(|| ControlError::not_found(p.session_id.0.clone()))?;
        let exited = matches!(record.status, homie_proto::SessionStatus::Exited(_));
        if registry.get(&p.session_id.0).is_some() {
            if !exited {
                // Already live: resuming is a no-op, not an error.
                return serde_json::to_value(&record)
                    .map_err(|error| ControlError::internal(error.to_string()));
            }
            // An agent that died on its own leaves its session behind: only an
            // explicit kill takes one out of the registry, so presence alone
            // does not mean alive. Evicting the corpse — which also releases
            // the holder still owning this id — is what keeps resume from
            // silently handing back the dead record it was asked to revive.
            let _ = registry.terminate(&p.session_id.0, std::time::Duration::from_millis(500));
        }
        registry
            .respawn(spec)
            .map_err(|error| ControlError::internal(error.to_string()))?;
        if let Some(persistence) = remote_persistence {
            registry.update_record(&p.session_id.0, |record| {
                record.remote_persistence = Some(persistence);
            });
        }
        let _ = registry.persist();
        self.publish_updated(&registry, &p.session_id.0);
        let record = registry
            .records()
            .into_iter()
            .find(|record| record.id.0 == p.session_id.0)
            .ok_or_else(|| ControlError::internal("the resumed session vanished"))?;
        serde_json::to_value(&record).map_err(|error| ControlError::internal(error.to_string()))
    }

    pub(crate) fn remote_resume_spec(
        &self,
        record: &homie_proto::SessionRecord,
    ) -> Result<crate::session::SessionSpec, ControlError> {
        let registry = self.registry.lock().map_err(poisoned)?;
        crate::session::remote_resume_spec(&self.launch_context(), &registry, record)
    }

    /// Revives a conversation found in an agent's own history: a NEW record
    /// whose agent-side id is the transcript's.
    pub(crate) fn session_resume_from_history(
        &self,
        params: Option<JsonValue>,
    ) -> Result<JsonValue, ControlError> {
        let p: homie_proto::ResumeFromHistoryParams = decode(params)?;
        let mut registry = self.registry.lock().map_err(poisoned)?;
        let id = next_session_id();
        let kind = p.entry.kind.id().to_string();
        let mut record = new_record(&id, &kind, &p.entry.cwd);
        record.agent_session_id = Some(p.entry.id.clone());
        record.transcript_path = Some(p.entry.transcript_path.clone());
        if let Some(title) = &p.entry.title {
            record.title = title.clone();
            record.title_source = homie_proto::TitleSource::FirstPrompt;
        }
        let spec = self.resume_spec(&registry, &id, &kind, &p.entry.cwd, Some(&p.entry.id))?;
        registry.ensure_session_project(&p.entry.cwd, None);
        registry
            .spawn(spec, record)
            .map_err(|error| ControlError::internal(error.to_string()))?;
        let _ = registry.persist();
        self.publish_updated(&registry, &id);
        let record = registry
            .records()
            .into_iter()
            .find(|record| record.id.0 == id)
            .ok_or_else(|| ControlError::internal("the resumed session vanished"))?;
        serde_json::to_value(&record).map_err(|error| ControlError::internal(error.to_string()))
    }

    /// The spawn spec that re-enters a conversation: the manifest's resume
    /// argv plus the same hook/MCP wiring a fresh spawn gets — a resumed
    /// Claude must not silently lose status detection or the homie tools.
    pub(crate) fn resume_spec(
        &self,
        registry: &Registry,
        id: &str,
        kind: &str,
        cwd: &str,
        agent_session_id: Option<&str>,
    ) -> Result<crate::session::SessionSpec, ControlError> {
        crate::session::resume_spec(
            &self.launch_context(),
            registry,
            id,
            kind,
            cwd,
            agent_session_id,
        )
    }

    /// Pops the most recently closed session whose folder still exists and
    /// re-lists it (exited), ready for the resume path.
    pub(crate) fn session_reopen_last(&self) -> Result<JsonValue, ControlError> {
        let mut registry = self.registry.lock().map_err(poisoned)?;
        let record = registry
            .reopen_last_closed()
            .ok_or_else(|| ControlError::bad_request("no recently closed session"))?;
        let _ = registry.persist();
        self.publish_updated(&registry, &record.id.0);
        serde_json::to_value(&record).map_err(|error| ControlError::internal(error.to_string()))
    }
}
