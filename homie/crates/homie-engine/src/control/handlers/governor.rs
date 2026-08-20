use super::*;

impl ControlServer {
    /// SIGSTOPs the session's whole tree and records it as hibernated. The
    /// PTY and holder stay alive; wake is one SIGCONT away.
    /// Updates the two governor tunables the app exposes; the rest keep the
    /// Swift defaults. Applies on the governor's next sweep.
    pub(crate) fn governor_configure(
        &self,
        params: Option<JsonValue>,
    ) -> Result<JsonValue, ControlError> {
        let p: homie_proto::GovernorSettingsParams = decode(params)?;
        let mut config = self.governor.lock().map_err(poisoned)?;
        config.idle_threshold_seconds = p.idle_threshold_seconds.max(0.0);
        config.hard_memory_bytes = p.hard_memory_bytes;
        Ok(json!({}))
    }

    pub(crate) fn session_hibernate(
        &self,
        params: Option<JsonValue>,
    ) -> Result<JsonValue, ControlError> {
        let p: homie_proto::SessionIdParams = decode(params)?;
        let mut registry = self.registry.lock().map_err(poisoned)?;
        registry
            .hibernate(&p.session_id.0, homie_proto::HibernationReason::Manual)
            .map_err(io_control_error)?;
        let _ = registry.persist();
        self.publish_updated(&registry, &p.session_id.0);
        Ok(json!({}))
    }

    pub(crate) fn session_wake(
        &self,
        params: Option<JsonValue>,
    ) -> Result<JsonValue, ControlError> {
        let p: homie_proto::SessionIdParams = decode(params)?;
        let mut registry = self.registry.lock().map_err(poisoned)?;
        registry
            .wake_session(&p.session_id.0)
            .map_err(|error| ControlError::internal(error.to_string()))?;
        let _ = registry.persist();
        self.publish_updated(&registry, &p.session_id.0);
        Ok(json!({}))
    }
}
