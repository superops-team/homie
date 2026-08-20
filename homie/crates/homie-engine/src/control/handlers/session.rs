use super::*;

impl ControlServer {
    /// `session.list` and `state.snapshot` are the same view: every record
    /// plus the project list, exactly as the reference implementation answers them.
    pub(crate) fn session_list(&self) -> Result<JsonValue, ControlError> {
        let registry = self.registry.lock().map_err(poisoned)?;
        serde_json::to_value(json!({
            "sessions": registry.records(),
            "projects": registry.projects_raw(),
        }))
        .map_err(|error| ControlError::internal(error.to_string()))
    }

    pub(crate) fn session_send_text(
        &self,
        params: Option<JsonValue>,
    ) -> Result<JsonValue, ControlError> {
        let p: homie_proto::SendTextParams = decode(params)?;
        let mut registry = self.registry.lock().map_err(poisoned)?;
        // Typing into a hibernated session wakes it; the text is queued and
        // flushed after SIGCONT, so no keystroke is lost.
        let _ = registry.wake_session(&p.session_id.0);
        self.publish_updated(&registry, &p.session_id.0);
        let session = registry
            .get(&p.session_id.0)
            .ok_or_else(|| ControlError::not_found(p.session_id.0.clone()))?;
        session
            .send_text(&p.text, p.submit)
            .map_err(|error| ControlError::internal(error.to_string()))?;
        Ok(json!({}))
    }

    pub(crate) fn session_resize(
        &self,
        params: Option<JsonValue>,
    ) -> Result<JsonValue, ControlError> {
        let p: homie_proto::ResizeParams = decode(params)?;
        let cols = u16::try_from(p.cols.clamp(2, u16::MAX as i64)).expect("clamped");
        let rows = u16::try_from(p.rows.clamp(2, u16::MAX as i64)).expect("clamped");
        let registry = self.registry.lock().map_err(poisoned)?;
        let session = registry
            .get(&p.session_id.0)
            .ok_or_else(|| ControlError::not_found(p.session_id.0.clone()))?;
        session
            .resize(cols, rows)
            .map_err(|error| ControlError::internal(error.to_string()))?;
        Ok(json!({}))
    }

    pub(crate) fn session_read_screen(
        &self,
        params: Option<JsonValue>,
    ) -> Result<JsonValue, ControlError> {
        let p: homie_proto::SessionIdParams = decode(params)?;
        let registry = self.registry.lock().map_err(poisoned)?;
        let session = registry
            .get(&p.session_id.0)
            .ok_or_else(|| ControlError::not_found(p.session_id.0.clone()))?;
        let (cols, rows) = session.screen_size();
        encode(&homie_proto::ReadScreenResult {
            text: session.screen_lines().join("\n"),
            cols: cols as i64,
            rows: rows as i64,
        })
    }

    pub(crate) fn session_read_scrollback(
        &self,
        params: Option<JsonValue>,
    ) -> Result<JsonValue, ControlError> {
        let p: homie_proto::SessionIdParams = decode(params)?;
        let registry = self.registry.lock().map_err(poisoned)?;
        let session = registry
            .get(&p.session_id.0)
            .ok_or_else(|| ControlError::not_found(p.session_id.0.clone()))?;
        encode(&session.read_scrollback())
    }

    pub(crate) fn session_read_scrollback_cells(
        &self,
        params: Option<JsonValue>,
    ) -> Result<JsonValue, ControlError> {
        let p: homie_proto::ReadScrollbackCellsParams = decode(params)?;
        let registry = self.registry.lock().map_err(poisoned)?;
        let session = registry
            .get(&p.session_id.0)
            .ok_or_else(|| ControlError::not_found(p.session_id.0.clone()))?;
        encode(&session.read_scrollback_cells(p.first_row, p.max_rows))
    }

    pub(crate) fn session_kill(
        &self,
        params: Option<JsonValue>,
    ) -> Result<JsonValue, ControlError> {
        let p: homie_proto::SessionIdParams = decode(params)?;
        let mut registry = self.registry.lock().map_err(poisoned)?;
        let exit = registry
            .terminate(&p.session_id.0, std::time::Duration::from_secs(3))
            .map_err(|error| ControlError::internal(error.to_string()))?;
        if exit.is_none() {
            return Err(ControlError::not_found(p.session_id.0.clone()));
        }
        let _ = registry.persist();
        if let Some(store) = &self.remote_bindings {
            let _ = store.remove(&p.session_id.0);
        }
        self.publish_updated(&registry, &p.session_id.0);
        Ok(json!({}))
    }

    pub(crate) fn session_remove(
        &self,
        params: Option<JsonValue>,
    ) -> Result<JsonValue, ControlError> {
        let p: homie_proto::SessionIdParams = decode(params)?;
        let mut registry = self.registry.lock().map_err(poisoned)?;
        registry
            .remove(&p.session_id.0, &self.logs_dir)
            .map_err(io_control_error)?;
        let _ = registry.persist();
        if let Some(store) = &self.remote_bindings {
            let _ = store.remove(&p.session_id.0);
        }
        self.events.publish(
            homie_proto::EventName::SESSION_REMOVED,
            json!({ "id": p.session_id.0, "reason": "released" }),
            Some(&p.session_id.0),
        );
        Ok(json!({}))
    }

    pub(crate) fn session_rename(
        &self,
        params: Option<JsonValue>,
    ) -> Result<JsonValue, ControlError> {
        let p: homie_proto::SessionRenameParams = decode(params)?;
        let mut registry = self.registry.lock().map_err(poisoned)?;
        registry
            .rename(&p.session_id.0, &p.title)
            .map_err(io_control_error)?;
        let _ = registry.persist();
        self.publish_updated(&registry, &p.session_id.0);
        Ok(json!({}))
    }

    pub(crate) fn session_mark_seen(
        &self,
        params: Option<JsonValue>,
    ) -> Result<JsonValue, ControlError> {
        let p: homie_proto::SessionIdParams = decode(params)?;
        let mut registry = self.registry.lock().map_err(poisoned)?;
        registry
            .mark_seen(&p.session_id.0)
            .map_err(io_control_error)?;
        let _ = registry.persist();
        self.publish_updated(&registry, &p.session_id.0);
        self.pr_monitor_wake.wake_session(p.session_id.0);
        Ok(json!({}))
    }

    pub(crate) fn client_set_active(
        &self,
        params: Option<JsonValue>,
    ) -> Result<JsonValue, ControlError> {
        let p: homie_proto::ClientActiveParams = decode(params)?;
        self.pr_monitor_wake.set_foreground_active(p.active);
        Ok(json!({}))
    }

    pub(crate) fn session_archive(
        &self,
        params: Option<JsonValue>,
    ) -> Result<JsonValue, ControlError> {
        let p: homie_proto::SessionIdParams = decode(params)?;
        let mut registry = self.registry.lock().map_err(poisoned)?;
        registry
            .archive(&p.session_id.0)
            .map_err(io_control_error)?;
        let _ = registry.persist();
        self.publish_updated(&registry, &p.session_id.0);
        Ok(json!({}))
    }

    pub(crate) fn session_unarchive(
        &self,
        params: Option<JsonValue>,
    ) -> Result<JsonValue, ControlError> {
        let p: homie_proto::SessionIdParams = decode(params)?;
        let mut registry = self.registry.lock().map_err(poisoned)?;
        registry
            .unarchive(&p.session_id.0)
            .map_err(io_control_error)?;
        let _ = registry.persist();
        self.publish_updated(&registry, &p.session_id.0);
        Ok(json!({}))
    }
}

impl ControlServer {
    /// Publishes `session.updated` with the session's current record.
    pub(crate) fn publish_updated(&self, registry: &Registry, id: &str) {
        if let Some(record) = registry
            .records()
            .into_iter()
            .find(|record| record.id.0 == id)
        {
            self.events
                .publish_encoded(homie_proto::EventName::SESSION_UPDATED, &record, Some(id));
        }
    }

    /// Resumable past conversations from the agents' own transcript stores,
    /// excluding ones already represented by live records.
    pub(crate) fn session_history(&self) -> Result<JsonValue, ControlError> {
        let tracked = {
            let registry = self.registry.lock().map_err(poisoned)?;
            registry.tracked_agent_session_ids()
        };
        let home = std::env::var("HOME")
            .map(PathBuf::from)
            .map_err(|_| ControlError::internal("HOME is not set"))?;
        let entries: Vec<homie_proto::HistoryEntry> = crate::history::scan(&home, &tracked)
            .into_iter()
            .map(history_entry_to_wire)
            .collect();
        encode(&homie_proto::SessionHistoryResult { entries })
    }
}
