use super::*;

impl ControlServer {
    /// One-click handoff of a live Claude session between hosts: WIP commit
    /// plus push plus hard-sync of the target checkout (phase 1, retryable),
    /// stop the source, shuttle the transcript, rewrite the record in place,
    /// and revive on the target through the normal resume path.
    pub(crate) fn session_migrate(
        &self,
        params: Option<JsonValue>,
    ) -> Result<JsonValue, ControlError> {
        let p: homie_proto::SessionMigrateParams = decode(params)?;
        let id = p.session_id.0.clone();
        let record = {
            let registry = self.registry.lock().map_err(poisoned)?;
            registry
                .records()
                .into_iter()
                .find(|record| record.id.0 == id)
                .ok_or_else(|| ControlError::not_found(id.clone()))?
        };
        // Handoff needs no terminal multiplexer of its own. Its phases are
        // git preparation over `hosts::run_shell`, stopping the source through
        // the session's own transport (which signals the remote Agent via its
        // Holder), the transcript shuttle, and a normal resume on the target.
        // Refuse only when a leg is remote and no Helper transport exists to
        // carry it, rather than refusing every call.
        if (record.host.is_some() || p.target_host.is_some()) && self.remote.is_none() {
            return Err(crate::remote::transport_unavailable());
        }
        if record.kind.id() != homie_proto::AgentKind::CLAUDE_CODE_ID {
            return Err(ControlError::bad_request(
                "only Claude Code sessions can move between hosts",
            ));
        }
        if record.host == p.target_host {
            return Err(ControlError::bad_request(match &p.target_host {
                Some(host) => format!("session is already on {host}"),
                None => "session is already local".to_string(),
            }));
        }
        let source_host = record
            .host
            .as_deref()
            .map(|host| self.resolve_host(host))
            .transpose()?;
        let target_host = p
            .target_host
            .as_deref()
            .map(|host| self.resolve_host(host))
            .transpose()?;
        let home = std::env::var("HOME")
            .map(PathBuf::from)
            .map_err(|_| ControlError::internal("HOME is not set"))?;

        // Locate the target checkout by origin (shared with host.locate_repo).
        let origin =
            crate::hosts::origin_of_cwd(&record.cwd, source_host.as_ref()).ok_or_else(|| {
                ControlError::bad_request(format!(
                    "session cwd is not inside a git repository with an 'origin' remote: {}",
                    record.cwd
                ))
            })?;
        let local_roots: Vec<String> = {
            let registry = self.registry.lock().map_err(poisoned)?;
            registry
                .projects_raw()
                .iter()
                .filter_map(|project| project.get("root").and_then(|value| value.as_str()))
                .map(str::to_string)
                .collect()
        };
        let target_repo = crate::hosts::locate(&origin, target_host.as_ref(), &local_roots)
            .ok_or_else(|| match &target_host {
                Some(host) => ControlError::bad_request(format!(
                    "repo not cloned on {} — clone {origin} under {} first",
                    host.display_name(),
                    host.default_cwd.as_deref().unwrap_or("~")
                )),
                None => ControlError::bad_request(format!(
                    "repo not cloned locally — no known project has origin {origin}"
                )),
            })?;

        // Phase 1 (source agent still alive, everything retryable).
        let prepared = crate::migrate::prepare(
            &record.cwd,
            source_host.as_ref(),
            target_host.as_ref(),
            &target_repo,
            target_host
                .as_ref()
                .map(|host| host.display_name())
                .unwrap_or("local"),
        )
        .map_err(migrate_control_error)?;

        // Point of no return: stop the source agent.
        let mut warnings: Vec<String> = Vec::new();
        {
            let mut registry = self.registry.lock().map_err(poisoned)?;
            let _ = registry.terminate(&id, std::time::Duration::from_secs(3));
        }
        // Phase 2: transcript shuttle (source stopped ⇒ the jsonl is final).
        let shuttle = crate::migrate::shuttle_transcript(
            &record.cwd,
            record.transcript_path.as_deref(),
            record.agent_session_id.as_deref(),
            source_host.as_ref(),
            target_host.as_ref(),
            &prepared,
            &home,
        );
        if let Some(warning) = shuttle.warning.clone() {
            warnings.push(warning);
        }

        // Rewrite the record in place: same id/title/sidebar position, new
        // host + cwd.
        {
            let mut registry = self.registry.lock().map_err(poisoned)?;
            let target_id = target_host.as_ref().map(|host| host.id.clone());
            let branch = prepared.branch.clone();
            let cwd = prepared.target_repo_root.clone();
            let transcript = shuttle.local_target_path.clone();
            let local = target_host.is_none();
            registry.ensure_session_project(&cwd, target_id.as_deref());
            registry.update_record(&id, |record| {
                record.host = target_id;
                record.cwd = cwd;
                record.project_id =
                    crate::registry::session_project_id(&record.cwd, record.host.as_deref());
                record.worktree_path = None;
                record.git_branch = Some(branch);
                record.transcript_path = if local { transcript } else { None };
                record.status = homie_proto::SessionStatus::Exited(homie_proto::ExitInfo {
                    reason: homie_proto::ExitReason::Exited,
                    code: Some(0),
                    signal: None,
                });
                record.needs_input = None;
                record.hibernation = None;
                record.memory_bytes = None;
                record.listening_ports = None;
                record.resumability = homie_proto::Resumability::Resumable;
            });
            let _ = registry.persist();
            self.publish_updated(&registry, &id);
        }

        // Cutover: the normal resume path revives the conversation on the
        // target; without a transcript there is nothing to resume, so the
        // record is left revivable and the client's next open resumes fresh.
        let revived = self.session_resume(Some(json!({ "sessionID": id })))?;
        let session: homie_proto::SessionRecord = serde_json::from_value(revived)
            .map_err(|error| ControlError::internal(error.to_string()))?;
        encode(&homie_proto::SessionMigrateResult {
            session,
            transcript_migrated: shuttle.migrated,
            warning: (!warnings.is_empty()).then(|| warnings.join("; ")),
        })
    }
}
