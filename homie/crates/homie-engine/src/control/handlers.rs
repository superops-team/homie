//! Control-channel method handlers.
//!
//! The per-method business logic (handshake, session spawn/list/resume, host and
//! worktree operations, hook reporting, governance, browser calls). These stay as
//! `impl ControlServer` methods so they can reach the private fields, but they live
//! apart from the transport layer (serve/handle_line/dispatch) and the wire codec
//! because they change for protocol/business reasons, not framing ones.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use homie_proto::{ControlError, JsonValue, WIRE_VERSION};
use serde_json::{Value, json};

use crate::registry::Registry;

use super::ControlServer;
use super::codec::{history_entry_to_wire, worktree_to_wire};
use super::inject::prepare_agent_input;
use super::wire::{
    decode, encode, io_control_error, migrate_control_error, poisoned, resolve_on_path,
};
use super::{BUILD, next_session_id, process_executable_hash};

impl ControlServer {
    pub(super) fn hello(&self, params: Option<JsonValue>) -> Result<JsonValue, ControlError> {
        let proto = params
            .as_ref()
            .and_then(|value| value.get("proto"))
            .and_then(Value::as_u64)
            .unwrap_or(WIRE_VERSION as u64);
        if proto != WIRE_VERSION as u64 {
            return Err(ControlError::version_mismatch(format!(
                "client speaks protocol {proto}, this engine speaks {WIRE_VERSION}"
            )));
        }
        Ok(json!({
            "proto": WIRE_VERSION,
            "build": BUILD,
            "engineKind": homie_proto::RUST_ENGINE_KIND,
            "pid": std::process::id() as i32,
            "executableHash": process_executable_hash(),
        }))
    }

    pub(super) fn session_capabilities(
        &self,
        params: Option<JsonValue>,
    ) -> Result<JsonValue, ControlError> {
        let p: homie_proto::SessionCapabilitiesParams = decode(params)?;
        let registry = self.registry.lock().map_err(poisoned)?;
        let record = registry
            .record(&p.session_id.0)
            .ok_or_else(|| ControlError::not_found(p.session_id.0.clone()))?;
        let capabilities =
            crate::driver::capabilities_for_manifest_id(record.effective_kind().id());
        encode(&homie_proto::SessionCapabilitiesResult {
            session_id: p.session_id,
            capabilities,
        })
    }

    /// Starts an agent and begins watching it.
    ///
    /// The command line comes from the manifest's agent descriptor, so this
    /// works for any agent that has one without code changes. Two limits worth
    /// stating: hook and MCP injection are not ported yet, so a Claude session
    /// started here is screen-detected rather than hook-driven; and `shell` and
    /// `generic` need an explicit `argv`, since their manifests declare no
    /// binary.
    pub(super) fn session_spawn(
        &self,
        params: Option<JsonValue>,
    ) -> Result<JsonValue, ControlError> {
        let raw = params.ok_or_else(|| ControlError::bad_request("params are required"))?;
        // Tests and scripts may pass a raw argv; the app never does. Read it
        // before the typed decode consumes the value.
        let argv: Vec<String> = raw
            .get("argv")
            .and_then(Value::as_array)
            .map(|values| {
                values
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default();
        let p: homie_proto::SessionSpawnParams = decode(Some(raw))?;
        if p.host.is_some() {
            return self.session_spawn_remote(p, argv);
        }
        let kind = p.kind.id().to_string();
        // A generic kind carries the user's command line inside itself.
        let argv = if argv.is_empty() {
            match p.kind.command() {
                Some(command) if !command.is_empty() => {
                    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".into());
                    vec![shell, "-lc".into(), command.to_string()]
                }
                _ if kind == homie_proto::AgentKind::SHELL_ID => {
                    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".into());
                    vec![shell, "-l".into()]
                }
                _ => Vec::new(),
            }
        } else {
            argv
        };

        // A worktree spawn creates the checkout first, then lands in it.
        let mut cwd = p.cwd.clone();
        let mut worktree_path = None;
        let mut git_branch = None;
        if p.new_worktree.unwrap_or(false) {
            let info =
                crate::git::create_worktree(Path::new(&p.cwd), p.worktree_branch.as_deref(), None)
                    .map_err(io_control_error)?;
            git_branch.clone_from(&info.branch);
            cwd.clone_from(&info.path);
            worktree_path = Some(info.path);
        }
        let cwd_path = PathBuf::from(&cwd);
        if !cwd_path.is_dir() {
            return Err(ControlError::bad_request(format!(
                "cwd {cwd:?} is not a directory"
            )));
        }

        let mut registry = self.registry.lock().map_err(poisoned)?;
        let engine = registry.engine();
        let manifest = engine
            .manifest(&kind)
            .ok_or_else(|| ControlError::not_found(format!("no manifest for agent {kind:?}")))?;
        let descriptor = manifest.agent.clone().unwrap_or_default();
        let authority = descriptor.authority();

        let id = next_session_id();
        // Build the complete agent argv before `spawn_spec`: agents declaring
        // `returnToLoginShell` need every manifest and injection argument
        // quoted inside the shell's `-c` command.
        let mut launch_args = argv.clone();
        let mut agent_session_id = None;
        if descriptor.binary.is_some() {
            launch_args.extend(descriptor.spawn_args.iter().cloned());
            agent_session_id = descriptor.session_id_flag.as_ref().map(|flag| {
                let uuid = crate::inject::uuid_v4();
                launch_args.push(flag.clone());
                launch_args.push(uuid.clone());
                uuid
            });
            if let Some(injection) = &self.injection {
                launch_args.extend(crate::inject::injection_args(
                    &descriptor.injection,
                    &injection.inject_dir,
                    &injection.cli_path,
                ));
            }
        }

        let inherited: Vec<(String, String)> = std::env::vars().collect();
        let mut pty = match descriptor.spawn_spec(&cwd_path, inherited.clone(), &launch_args) {
            Some(spec) => spec,
            // No binary in the manifest: the caller has to say what to run.
            None if !argv.is_empty() => {
                let mut spec = crate::pty::PtySpec::new(argv.clone(), &cwd_path);
                spec.env = shell_pty_environment(inherited);
                spec
            }
            None => {
                return Err(ControlError::bad_request(format!(
                    "agent {kind:?} declares no binary, so argv is required"
                )));
            }
        };

        let mut record = new_record(&id, &kind, &cwd);
        // A linked worktree is an execution cwd inside the project selected
        // by the user; it does not become a new first-level sidebar project.
        record.project_id = crate::registry::session_project_id(&p.cwd, None);
        registry.ensure_session_project(&p.cwd, None);
        if let Some(title) = &p.title {
            record.title = title.clone();
            record.title_source = homie_proto::TitleSource::HomieAssigned;
        }
        record.worktree_path = worktree_path;
        record.git_branch = git_branch.or_else(|| crate::git::branch(&cwd_path));
        record.parent = p.parent.clone();
        if let (Some(cols), Some(rows)) = (p.initial_cols, p.initial_rows) {
            pty.cols = cols.clamp(2, u16::MAX as i64) as u16;
            pty.rows = rows.clamp(2, u16::MAX as i64) as u16;
        }

        // Injection environment and the caller-minted conversation UUID. The
        // argv side was assembled before `spawn_spec` so its shell wrapper
        // contains the complete command.
        if descriptor.binary.is_some() {
            if let Some(injection) = &self.injection {
                pty.env
                    .push((crate::inject::SESSION_ID_ENV.into(), id.clone()));
                pty.env.push((
                    crate::inject::SOCKET_ENV.into(),
                    self.socket_path.to_string_lossy().into_owned(),
                ));
                pty.env.push((
                    crate::inject::CLI_ENV.into(),
                    injection.cli_path.to_string_lossy().into_owned(),
                ));
            }
            if let Some(uuid) = &agent_session_id {
                record.agent_session_id = Some(uuid.clone());
                if descriptor.injection.claude_hooks
                    && let Ok(home) = std::env::var("HOME")
                {
                    record.transcript_path = Some(
                        crate::inject::claude_transcript_path(Path::new(&home), &cwd, uuid)
                            .to_string_lossy()
                            .into_owned(),
                    );
                }
            }
        }
        let spec = crate::session::SessionSpec {
            id: id.clone(),
            pty,
            manifest_id: kind.clone(),
            authority,
            logs_dir: self.logs_dir.clone(),
            holder: self.holder.clone(),
            remote: None,
            defer_launch: true,
        };
        registry
            .spawn(spec, record)
            .map_err(|error| ControlError::internal(error.to_string()))?;
        let _ = registry.persist();
        self.publish_updated(&registry, &id);

        // An initial prompt is typed once the TUI can actually receive input,
        // and verified on screen afterward — ported from the Swift
        // `injectInitialPrompt`, which replaced a blind fixed delay that
        // raced Claude Code's boot and lost keystrokes into a composer that
        // did not exist yet.
        let prompt = p.initial_prompt.clone().filter(|prompt| !prompt.is_empty());
        if kind == homie_proto::AgentKind::CLAUDE_CODE_ID || prompt.is_some() {
            let registry = Arc::clone(&self.registry);
            let session_id = id.clone();
            std::thread::spawn(move || {
                prepare_agent_input(
                    &registry,
                    &session_id,
                    kind == homie_proto::AgentKind::CLAUDE_CODE_ID,
                    prompt.as_deref(),
                );
            });
        }

        let record = registry
            .records()
            .into_iter()
            .find(|record| record.id.0 == id)
            .ok_or_else(|| ControlError::internal("the new session vanished"))?;
        // SessionSpawnResult is the record itself, as the reference implementation
        // answers — not wrapped.
        serde_json::to_value(&record).map_err(|error| ControlError::internal(error.to_string()))
    }

    pub(super) fn session_spawn_remote(
        &self,
        p: homie_proto::SessionSpawnParams,
        caller_argv: Vec<String>,
    ) -> Result<JsonValue, ControlError> {
        let manager = self
            .remote
            .as_ref()
            .cloned()
            .ok_or_else(crate::remote::transport_unavailable)?;
        let binding_store = self.remote_bindings.clone().ok_or_else(|| {
            ControlError::internal("owner-only remote binding store is unavailable")
        })?;
        let host_id = p
            .host
            .as_deref()
            .ok_or_else(|| ControlError::bad_request("remote host is required"))?;
        let host = self.resolve_host(host_id)?;
        if p.new_worktree.unwrap_or(false) {
            return Err(ControlError::bad_request(
                "remote worktree creation requires the structured workspace RPC",
            ));
        }
        if p.same_repo_as.is_some() {
            return Err(ControlError::bad_request(
                "sameRepoAs requires the structured remote workspace RPC",
            ));
        }

        let helper = manager.ensure_helper(&host).map_err(io_control_error)?;
        let persistence = manager
            .probe_persistence(&host, &helper)
            .map_err(io_control_error)?;
        let requested_cwd = if p.cwd.trim().is_empty() {
            host.default_cwd.clone().unwrap_or_else(|| "~".into())
        } else {
            p.cwd.clone()
        };
        let captured = manager
            .capture_environment(
                &helper,
                &homie_proto::remote_pty::EnvironmentCaptureRequest {
                    cwd: Some(requested_cwd),
                    timeout_millis: 10_000,
                },
            )
            .map_err(io_control_error)?;
        let cwd = PathBuf::from(&captured.cwd);
        if !cwd.is_absolute() {
            return Err(ControlError::internal(
                "remote Helper returned a non-absolute cwd",
            ));
        }

        let kind = p.kind.id().to_string();
        let (descriptor, engine) = {
            let registry = self.registry.lock().map_err(poisoned)?;
            let engine = registry.engine();
            let manifest = engine.manifest(&kind).ok_or_else(|| {
                ControlError::not_found(format!("no manifest for agent {kind:?}"))
            })?;
            (manifest.agent.clone().unwrap_or_default(), engine)
        };
        drop(engine);
        let authority = descriptor.authority();
        let inherited = captured
            .environment
            .into_iter()
            .map(|variable| (variable.name, variable.value))
            .collect::<Vec<_>>();

        let id = next_session_id();
        let mut agent_session_id = None;
        let mut launch_args = caller_argv.clone();
        if descriptor.binary.is_some() {
            launch_args.extend(descriptor.spawn_args.iter().cloned());
            agent_session_id = descriptor.session_id_flag.as_ref().map(|flag| {
                let uuid = crate::inject::uuid_v4();
                launch_args.push(flag.clone());
                launch_args.push(uuid.clone());
                uuid
            });
        }

        let argv = if descriptor.binary.is_some() {
            descriptor
                .remote_spawn_spec(&cwd, inherited.clone(), &launch_args)
                .ok_or_else(|| ControlError::internal("remote descriptor has no binary"))?
                .argv
        } else if !caller_argv.is_empty() {
            caller_argv
        } else if let Some(command) = p.kind.command().filter(|command| !command.is_empty()) {
            vec![captured.shell.clone(), "-lc".into(), command.to_string()]
        } else if kind == homie_proto::AgentKind::SHELL_ID {
            vec![captured.shell.clone(), "-l".into()]
        } else {
            return Err(ControlError::bad_request(format!(
                "agent {kind:?} declares no binary, so argv is required"
            )));
        };
        let mut pty = if descriptor.binary.is_some() {
            descriptor
                .remote_spawn_spec(&cwd, inherited, &launch_args)
                .ok_or_else(|| ControlError::internal("remote descriptor has no binary"))?
        } else {
            let mut spec = crate::pty::PtySpec::new(argv, &cwd);
            spec.env = shell_pty_environment(inherited);
            spec
        };
        if let (Some(cols), Some(rows)) = (p.initial_cols, p.initial_rows) {
            pty.cols = cols.clamp(2, u16::MAX as i64) as u16;
            pty.rows = rows.clamp(2, u16::MAX as i64) as u16;
        }

        let token = random_session_token()?;
        let launch = homie_proto::remote_pty::LaunchRequest {
            session_id: id.clone(),
            session_token: token,
            argv: pty.argv.clone(),
            cwd: captured.cwd.clone(),
            environment: pty
                .env
                .iter()
                .map(
                    |(name, value)| homie_proto::remote_pty::EnvironmentVariable {
                        name: name.clone(),
                        value: value.clone(),
                    },
                )
                .collect(),
            cols: pty.cols,
            rows: pty.rows,
            persistence,
        };

        let mut record = new_record(&id, &kind, &captured.cwd);
        record.host = Some(host.id.clone());
        record.project_id = crate::registry::session_project_id(&captured.cwd, Some(&host.id));
        record.remote_persistence = Some(persistence);
        record.parent = p.parent.clone();
        record.agent_session_id = agent_session_id;
        if let Some(title) = &p.title {
            record.title = title.clone();
            record.title_source = homie_proto::TitleSource::HomieAssigned;
        }
        let spec = crate::session::SessionSpec {
            id: id.clone(),
            pty,
            manifest_id: kind.clone(),
            authority,
            logs_dir: self.logs_dir.clone(),
            holder: None,
            remote: Some(crate::session::RemoteSessionSpec {
                manager,
                helper,
                launch,
                host_id: host.id.clone(),
                binding_store,
            }),
            defer_launch: false,
        };
        let mut registry = self.registry.lock().map_err(poisoned)?;
        registry.ensure_session_project(&captured.cwd, Some(&host.id));
        registry
            .spawn(spec, record)
            .map_err(|error| ControlError::internal(error.to_string()))?;
        let _ = registry.persist();
        self.publish_updated(&registry, &id);

        let prompt = p.initial_prompt.filter(|prompt| !prompt.is_empty());
        if kind == homie_proto::AgentKind::CLAUDE_CODE_ID || prompt.is_some() {
            let registry = Arc::clone(&self.registry);
            let session_id = id.clone();
            std::thread::spawn(move || {
                prepare_agent_input(
                    &registry,
                    &session_id,
                    kind == homie_proto::AgentKind::CLAUDE_CODE_ID,
                    prompt.as_deref(),
                );
            });
        }
        let record = registry
            .records()
            .into_iter()
            .find(|record| record.id.0 == id)
            .ok_or_else(|| ControlError::internal("the new remote session vanished"))?;
        serde_json::to_value(&record).map_err(|error| ControlError::internal(error.to_string()))
    }

    /// `test.run` / `browser.act`: the Playwright sidecar, launched lazily.
    pub(super) fn browser_call(
        &self,
        method: &str,
        params: Option<JsonValue>,
    ) -> Result<JsonValue, ControlError> {
        let params = params.ok_or_else(|| ControlError::bad_request("params are required"))?;
        let pool = self
            .browser
            .get_or_init(|| crate::browser::BrowserPool::new(&self.logs_dir));
        let result = if method == "run" {
            pool.run(params)
        } else {
            pool.browse(params)
        };
        result.map_err(|error| ControlError {
            code: "browser_pool".into(),
            message: error,
        })
    }

    /// The aggregated staleness view: every worktree of every project,
    /// joined with the session (live wins) occupying it, its dirtiness,
    /// merged-ness into the default branch, and age — plus the "safe to
    /// clean up" suggestion. The staleness join itself lives in `crate::git`.
    pub(super) fn worktree_overview(&self) -> Result<JsonValue, ControlError> {
        let (records, roots) = {
            let registry = self.registry.lock().map_err(poisoned)?;
            let roots: Vec<String> = registry
                .projects_raw()
                .iter()
                .filter_map(|project| project.get("root").and_then(|value| value.as_str()))
                .map(str::to_string)
                .collect();
            (registry.records(), roots)
        };
        encode(&crate::git::worktree_overview(&records, roots))
    }

    /// One-click handoff of a live Claude session between hosts: WIP commit
    /// plus push plus hard-sync of the target checkout (phase 1, retryable),
    /// stop the source, shuttle the transcript, rewrite the record in place,
    /// and revive on the target through the normal resume path.
    pub(super) fn session_migrate(
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

    /// `host.sync_prefs`: push the local agent preferences to a host so
    /// agents there behave like local ones. Additive rsync, fixed include
    /// list, per-tool reporting.
    pub(super) fn host_sync_prefs(
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
    pub(super) fn host_initialize(
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
    pub(super) fn host_list_directories(
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
    pub(super) fn host_locate_repo(
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
    pub(super) fn resolve_host(
        &self,
        host_id: &str,
    ) -> Result<homie_proto::HostEntry, ControlError> {
        homie_proto::HostsConfig::load(self.hosts_file())
            .hosts
            .into_iter()
            .find(|entry| entry.id == host_id)
            .ok_or_else(|| {
                ControlError::bad_request(format!("unknown host {host_id:?}; check hosts.json"))
            })
    }

    /// Applies the current application build's remote environment gate before
    /// a stateless SSH action. Live Holder operations deliberately use their
    /// session binding's creation-time Helper instead.
    pub(super) fn hosts_file(&self) -> PathBuf {
        self.socket_path
            .parent()
            .map(|parent| parent.join("hosts.json"))
            .unwrap_or_else(|| PathBuf::from("hosts.json"))
    }

    /// `session.list` and `state.snapshot` are the same view: every record
    /// plus the project list, exactly as the reference implementation answers them.
    pub(super) fn session_list(&self) -> Result<JsonValue, ControlError> {
        let registry = self.registry.lock().map_err(poisoned)?;
        serde_json::to_value(json!({
            "sessions": registry.records(),
            "projects": registry.projects_raw(),
        }))
        .map_err(|error| ControlError::internal(error.to_string()))
    }

    pub(super) fn session_send_text(
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

    pub(super) fn session_resize(
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

    pub(super) fn session_read_screen(
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

    pub(super) fn session_read_scrollback(
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

    pub(super) fn session_read_scrollback_cells(
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

    pub(super) fn session_kill(
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

    pub(super) fn session_remove(
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

    pub(super) fn session_rename(
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

    pub(super) fn session_mark_seen(
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

    pub(super) fn client_set_active(
        &self,
        params: Option<JsonValue>,
    ) -> Result<JsonValue, ControlError> {
        let p: homie_proto::ClientActiveParams = decode(params)?;
        self.pr_monitor_wake.set_foreground_active(p.active);
        Ok(json!({}))
    }

    pub(super) fn session_archive(
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

    pub(super) fn session_unarchive(
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

    /// A hook or notify callback from inside an agent session: the signal
    /// that makes hook-authority agents' status precise. Parsed by the same
    /// rules the reference implementation used, metadata folded into the record, signal
    /// fed to the session's reducer.
    pub(super) fn hook_report(&self, params: Option<JsonValue>) -> Result<JsonValue, ControlError> {
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

    /// Revives an exited session's conversation under the SAME record id.
    pub(super) fn session_resume(
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

    pub(super) fn remote_resume_spec(
        &self,
        record: &homie_proto::SessionRecord,
    ) -> Result<crate::session::SessionSpec, ControlError> {
        let manager = self
            .remote
            .as_ref()
            .cloned()
            .ok_or_else(crate::remote::transport_unavailable)?;
        let binding_store = self.remote_bindings.clone().ok_or_else(|| {
            ControlError::internal("owner-only remote binding store is unavailable")
        })?;
        let host_id = record
            .host
            .as_deref()
            .ok_or_else(|| ControlError::bad_request("remote record has no host"))?;
        let host = self.resolve_host(host_id)?;
        let helper = manager.ensure_helper(&host).map_err(io_control_error)?;
        let persistence = manager
            .probe_persistence(&host, &helper)
            .map_err(io_control_error)?;
        let captured = manager
            .capture_environment(
                &helper,
                &homie_proto::remote_pty::EnvironmentCaptureRequest {
                    cwd: Some(record.cwd.clone()),
                    timeout_millis: 10_000,
                },
            )
            .map_err(io_control_error)?;
        let cwd = PathBuf::from(&captured.cwd);
        if !cwd.is_absolute() {
            return Err(ControlError::internal(
                "remote Helper returned a non-absolute cwd",
            ));
        }
        let (descriptor, authority) = {
            let registry = self.registry.lock().map_err(poisoned)?;
            let engine = registry.engine();
            let manifest = engine.manifest(record.kind.id()).ok_or_else(|| {
                ControlError::not_found(format!("no manifest for agent {}", record.kind.id()))
            })?;
            let descriptor = manifest.agent.clone().unwrap_or_default();
            let authority = descriptor.authority();
            (descriptor, authority)
        };
        let mut launch_args = descriptor.spawn_args.clone();
        launch_args.extend(
            descriptor
                .resume_args(record.agent_session_id.as_deref())
                .ok_or_else(|| {
                    ControlError::bad_request(format!(
                        "agent {} does not support resume",
                        record.kind.id()
                    ))
                })?,
        );
        let inherited = captured
            .environment
            .into_iter()
            .map(|variable| (variable.name, variable.value));
        let pty = descriptor
            .remote_spawn_spec(&cwd, inherited, &launch_args)
            .ok_or_else(|| {
                ControlError::bad_request(format!("agent {} declares no binary", record.kind.id()))
            })?;
        let launch = homie_proto::remote_pty::LaunchRequest {
            session_id: record.id.0.clone(),
            session_token: random_session_token()?,
            argv: pty.argv.clone(),
            cwd: captured.cwd,
            environment: pty
                .env
                .iter()
                .map(
                    |(name, value)| homie_proto::remote_pty::EnvironmentVariable {
                        name: name.clone(),
                        value: value.clone(),
                    },
                )
                .collect(),
            cols: pty.cols,
            rows: pty.rows,
            persistence,
        };
        Ok(crate::session::SessionSpec {
            id: record.id.0.clone(),
            pty,
            manifest_id: record.kind.id().to_string(),
            authority,
            logs_dir: self.logs_dir.clone(),
            holder: None,
            remote: Some(crate::session::RemoteSessionSpec {
                manager,
                helper,
                launch,
                host_id: host.id,
                binding_store,
            }),
            defer_launch: false,
        })
    }

    /// Revives a conversation found in an agent's own history: a NEW record
    /// whose agent-side id is the transcript's.
    pub(super) fn session_resume_from_history(
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
    pub(super) fn resume_spec(
        &self,
        registry: &Registry,
        id: &str,
        kind: &str,
        cwd: &str,
        agent_session_id: Option<&str>,
    ) -> Result<crate::session::SessionSpec, ControlError> {
        let engine = registry.engine();
        let manifest = engine
            .manifest(kind)
            .ok_or_else(|| ControlError::not_found(format!("no manifest for agent {kind}")))?;
        let descriptor = manifest.agent.clone().unwrap_or_default();
        descriptor
            .binary
            .as_ref()
            .ok_or_else(|| ControlError::bad_request(format!("agent {kind} declares no binary")))?;
        let tail = descriptor.resume_args(agent_session_id).ok_or_else(|| {
            ControlError::bad_request(format!("agent {kind} does not support resume"))
        })?;

        let mut launch_args = descriptor.spawn_args.clone();
        launch_args.extend(tail);
        if let Some(injection) = &self.injection {
            // Only the appendable flag mechanisms replay on resume, exactly
            // as in Swift: Codex's global `-c` overrides must precede the
            // resume SUBCOMMAND and are deliberately not replayed.
            let claude_only = crate::agent::InjectionSpec {
                claude_hooks: descriptor.injection.claude_hooks,
                claude_mcp: descriptor.injection.claude_mcp,
                ..Default::default()
            };
            launch_args.extend(crate::inject::injection_args(
                &claude_only,
                &injection.inject_dir,
                &injection.cli_path,
            ));
        }

        let inherited: Vec<(String, String)> = std::env::vars().collect();
        let mut pty = descriptor
            .spawn_spec(Path::new(cwd), inherited, &launch_args)
            .ok_or_else(|| ControlError::internal("resume spec without a binary"))?;
        if let Some(injection) = &self.injection {
            pty.env
                .push((crate::inject::SESSION_ID_ENV.into(), id.to_string()));
            pty.env.push((
                crate::inject::SOCKET_ENV.into(),
                self.socket_path.to_string_lossy().into_owned(),
            ));
            pty.env.push((
                crate::inject::CLI_ENV.into(),
                injection.cli_path.to_string_lossy().into_owned(),
            ));
        }
        Ok(crate::session::SessionSpec {
            id: id.to_string(),
            pty,
            manifest_id: kind.to_string(),
            authority: descriptor.authority(),
            logs_dir: self.logs_dir.clone(),
            holder: self.holder.clone(),
            remote: None,
            defer_launch: true,
        })
    }

    /// Pops the most recently closed session whose folder still exists and
    /// re-lists it (exited), ready for the resume path.
    pub(super) fn session_reopen_last(&self) -> Result<JsonValue, ControlError> {
        let mut registry = self.registry.lock().map_err(poisoned)?;
        let record = registry
            .reopen_last_closed()
            .ok_or_else(|| ControlError::bad_request("no recently closed session"))?;
        let _ = registry.persist();
        self.publish_updated(&registry, &record.id.0);
        serde_json::to_value(&record).map_err(|error| ControlError::internal(error.to_string()))
    }

    /// Which agent binaries actually resolve, plus each manifest's descriptor
    /// — this doubles as the agent catalog the client's picker renders.
    pub(super) fn agent_readiness(&self) -> Result<JsonValue, ControlError> {
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

    pub(super) fn environment_refresh_path(&self) -> Result<JsonValue, ControlError> {
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

    pub(super) fn project_add(&self, params: Option<JsonValue>) -> Result<JsonValue, ControlError> {
        let p: homie_proto::ProjectAddParams = decode(params)?;
        let mut registry = self.registry.lock().map_err(poisoned)?;
        let project = registry.add_project(&p.root);
        let _ = registry.persist();
        Ok(project)
    }

    /// The working tree's diff against a base ref, for the app's diff pane.
    pub(super) fn session_read_diff(
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

    /// SIGSTOPs the session's whole tree and records it as hibernated. The
    /// PTY and holder stay alive; wake is one SIGCONT away.
    /// Updates the two governor tunables the app exposes; the rest keep the
    /// Swift defaults. Applies on the governor's next sweep.
    pub(super) fn governor_configure(
        &self,
        params: Option<JsonValue>,
    ) -> Result<JsonValue, ControlError> {
        let p: homie_proto::GovernorSettingsParams = decode(params)?;
        let mut config = self.governor.lock().map_err(poisoned)?;
        config.idle_threshold_seconds = p.idle_threshold_seconds.max(0.0);
        config.hard_memory_bytes = p.hard_memory_bytes;
        Ok(json!({}))
    }

    pub(super) fn session_hibernate(
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

    pub(super) fn session_wake(
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

    /// Publishes `session.updated` with the session's current record.
    pub(super) fn publish_updated(&self, registry: &Registry, id: &str) {
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
    pub(super) fn session_history(&self) -> Result<JsonValue, ControlError> {
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

    pub(super) fn worktree_create(
        &self,
        params: Option<JsonValue>,
    ) -> Result<JsonValue, ControlError> {
        let p: homie_proto::WorktreeCreateParams = decode(params)?;
        let info = crate::git::create_worktree(
            Path::new(&p.repo_path),
            p.branch.as_deref(),
            p.base.as_deref(),
        )
        .map_err(io_control_error)?;
        self.events.publish(
            "worktree.created",
            json!({ "repoPath": p.repo_path, "path": info.path, "branch": info.branch }),
            None,
        );
        encode(&worktree_to_wire(info))
    }

    pub(super) fn worktree_list(
        &self,
        params: Option<JsonValue>,
    ) -> Result<JsonValue, ControlError> {
        let p: homie_proto::WorktreeListParams = decode(params)?;
        let list = crate::git::list_worktrees(Path::new(&p.repo_path)).map_err(io_control_error)?;
        encode(&list.into_iter().map(worktree_to_wire).collect::<Vec<_>>())
    }

    pub(super) fn worktree_remove(
        &self,
        params: Option<JsonValue>,
    ) -> Result<JsonValue, ControlError> {
        let p: homie_proto::WorktreeRemoveParams = decode(params)?;
        crate::git::remove_worktree(Path::new(&p.repo_path), &p.worktree_path, p.force)
            .map_err(io_control_error)?;
        self.events.publish(
            "worktree.removed",
            json!({ "repoPath": p.repo_path, "path": p.worktree_path }),
            None,
        );
        Ok(json!({}))
    }
}

fn random_session_token() -> Result<homie_proto::remote_pty::SessionToken, ControlError> {
    let mut bytes = [0_u8; 32];
    getrandom::fill(&mut bytes)
        .map_err(|error| ControlError::internal(format!("secure random source failed: {error}")))?;
    let encoded = bytes
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    homie_proto::remote_pty::SessionToken::new(encoded)
        .map_err(|error| ControlError::internal(error.to_string()))
}

pub(crate) fn new_record(id: &str, kind: &str, cwd: &str) -> homie_proto::SessionRecord {
    use homie_proto::{AgentKind, DateMillis, Resumability, SessionId, TitleSource};
    let now: DateMillis = std::time::SystemTime::now().into();
    homie_proto::SessionRecord {
        id: SessionId(id.to_string()),
        kind: AgentKind::new(kind),
        cwd: cwd.to_string(),
        project_id: crate::registry::session_project_id(cwd, None),
        worktree_path: None,
        git_branch: None,
        title: kind.to_string(),
        title_source: TitleSource::Placeholder,
        agent_session_id: None,
        transcript_path: None,
        status: homie_proto::SessionStatus::Starting,
        needs_input: None,
        resumability: Resumability::Live,
        parent: None,
        created_at: now,
        updated_at: now,
        last_turn_completed_at: None,
        last_seen_at: None,
        pinned: false,
        archived_at: None,
        host: None,
        remote_persistence: None,
        hibernation: None,
        memory_bytes: None,
        artifacts: None,
        pull_requests: None,
        listening_ports: None,
        foreground_agent: None,
    }
}

pub(super) fn shell_pty_environment(mut inherited: Vec<(String, String)>) -> Vec<(String, String)> {
    inherited.retain(|(key, _)| key != "TERM" && key != "NO_COLOR");
    inherited.push(("TERM".into(), "xterm-256color".into()));
    inherited
}
