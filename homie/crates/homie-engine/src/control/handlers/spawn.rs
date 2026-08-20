use super::*;

impl ControlServer {
    /// Starts an agent and begins watching it.
    ///
    /// The command line comes from the manifest's agent descriptor, so this
    /// works for any agent that has one without code changes. Two limits worth
    /// stating: hook and MCP injection are not ported yet, so a Claude session
    /// started here is screen-detected rather than hook-driven; and `shell` and
    /// `generic` need an explicit `argv`, since their manifests declare no
    /// binary.
    pub(crate) fn session_spawn(
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

        // A worktree spawn creates the checkout first, then lands in it. This
        // stays in the handler, before the registry lock, because
        // `git worktree add` is a slow filesystem call that must not stall
        // every other control request.
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
        let plan = crate::session::spawn_spec(
            &self.launch_context(),
            &registry,
            &p,
            argv,
            &cwd,
            worktree_path,
            git_branch,
        )?;
        let id = plan.spec.id.clone();
        let kind = plan.spec.manifest_id.clone();
        registry.ensure_session_project(&plan.project_root, plan.host_id.as_deref());
        registry
            .spawn(plan.spec, plan.record)
            .map_err(|error| ControlError::internal(error.to_string()))?;
        let _ = registry.persist();
        self.publish_updated(&registry, &id);

        // An initial prompt is typed once the TUI can actually receive input,
        // and verified on screen afterward — ported from the Swift
        // `injectInitialPrompt`, which replaced a blind fixed delay that
        // raced Claude Code's boot and lost keystrokes into a composer that
        // did not exist yet.
        self.schedule_initial_prompt(&kind, plan.prompt, &id);

        let record = registry
            .records()
            .into_iter()
            .find(|record| record.id.0 == id)
            .ok_or_else(|| ControlError::internal("the new session vanished"))?;
        // SessionSpawnResult is the record itself, as the reference implementation
        // answers — not wrapped.
        serde_json::to_value(&record).map_err(|error| ControlError::internal(error.to_string()))
    }

    pub(crate) fn session_spawn_remote(
        &self,
        p: homie_proto::SessionSpawnParams,
        caller_argv: Vec<String>,
    ) -> Result<JsonValue, ControlError> {
        let kind = p.kind.id().to_string();
        // Resolve the manifest under a brief lock, then release it so the slow
        // remote transport calls below run without stalling other requests.
        let descriptor = {
            let registry = self.registry.lock().map_err(poisoned)?;
            let engine = registry.engine();
            let manifest = engine.manifest(&kind).ok_or_else(|| {
                ControlError::not_found(format!("no manifest for agent {kind:?}"))
            })?;
            manifest.agent.clone().unwrap_or_default()
        };
        let plan =
            crate::session::remote_spawn_spec(&self.launch_context(), descriptor, p, caller_argv)?;
        let id = plan.spec.id.clone();
        let mut registry = self.registry.lock().map_err(poisoned)?;
        registry.ensure_session_project(&plan.project_root, plan.host_id.as_deref());
        registry
            .spawn(plan.spec, plan.record)
            .map_err(|error| ControlError::internal(error.to_string()))?;
        let _ = registry.persist();
        self.publish_updated(&registry, &id);

        self.schedule_initial_prompt(&kind, plan.prompt, &id);
        let record = registry
            .records()
            .into_iter()
            .find(|record| record.id.0 == id)
            .ok_or_else(|| ControlError::internal("the new remote session vanished"))?;
        serde_json::to_value(&record).map_err(|error| ControlError::internal(error.to_string()))
    }

    /// Types the initial prompt once the TUI can actually receive input, and
    /// verifies it on screen afterward. Shared by local and remote spawn.
    fn schedule_initial_prompt(&self, kind: &str, prompt: Option<String>, id: &str) {
        if kind == homie_proto::AgentKind::CLAUDE_CODE_ID || prompt.is_some() {
            let registry = Arc::clone(&self.registry);
            let session_id = id.to_string();
            let is_claude = kind == homie_proto::AgentKind::CLAUDE_CODE_ID;
            std::thread::spawn(move || {
                prepare_agent_input(&registry, &session_id, is_claude, prompt.as_deref());
            });
        }
    }

    /// `test.run` / `browser.act`: the Playwright sidecar, launched lazily.
    pub(crate) fn browser_call(
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

    /// Bundles the launch-side dependencies the session domain needs to build
    /// specs, so spawn/resume logic stays out of the transport layer.
    pub(crate) fn launch_context(&self) -> crate::session::LaunchContext {
        crate::session::LaunchContext {
            inject_dir: self.injection.as_ref().map(|i| i.inject_dir.clone()),
            cli_path: self.injection.as_ref().map(|i| i.cli_path.clone()),
            mcp: self.injection.as_ref().and_then(|i| i.mcp.clone()),
            gateway: self.injection.as_ref().and_then(|i| i.gateway.clone()),
            socket_path: self.socket_path.clone(),
            logs_dir: self.logs_dir.clone(),
            holder: self.holder.clone(),
            remote: self.remote.clone(),
            remote_bindings: self.remote_bindings.clone(),
        }
    }
}
