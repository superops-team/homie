//! Launch/spec assembly for sessions: turning a session record or spawn
//! request into the [`SessionSpec`] that starts the agent.
//!
//! This lives in the session domain instead of `control/handlers.rs` so the
//! transport layer stays a thin decode → domain → encode adapter. The pieces
//! it needs that a `ControlServer` owns (injection wiring, socket path, logs
//! dir, holder, remote transport) are bundled into [`LaunchContext`], which the
//! handler assembles once and passes in.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use homie_proto::ControlError;

use crate::registry::Registry;
use crate::remote::binding::RemoteBindingStore;
use crate::remote::manager::RemoteManager;
use crate::session::{HolderConfig, RemoteSessionSpec, SessionSpec, SpawnPlan};

/// The launch-side dependencies a `ControlServer` owns, bundled so the
/// session domain can assemble specs without reaching back into the transport
/// layer. Every field is optional or plain data: the session domain never
/// locks the registry mutex through this context.
pub struct LaunchContext {
    /// Injection wiring (hook/MCP shims and the CLI they point at). Present
    /// when the daemon was started with injection enabled.
    pub inject_dir: Option<PathBuf>,
    pub cli_path: Option<PathBuf>,
    /// The daemon-embedded MCP runtime, when agents opt into `codexMcp` /
    /// `claudeMcp` via a streamable-http endpoint.
    pub mcp: Option<crate::mcp::McpRuntime>,
    /// Per-session virtual-key issuer for agents opting into `codexGateway`.
    pub gateway: Option<crate::inject::GatewayIssuer>,
    pub socket_path: PathBuf,
    pub logs_dir: PathBuf,
    pub holder: Option<HolderConfig>,
    pub remote: Option<Arc<RemoteManager>>,
    pub remote_bindings: Option<RemoteBindingStore>,
}

/// A freshly minted remote session token, hex-encoded from 32 random bytes.
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

/// Resolves a host id against the daemon's `hosts.json`, exactly as the
/// transport layer did.
pub fn resolve_host(
    ctx: &LaunchContext,
    host_id: &str,
) -> Result<homie_proto::HostEntry, ControlError> {
    let hosts_file = ctx
        .socket_path
        .parent()
        .map(|parent| parent.join("hosts.json"))
        .unwrap_or_else(|| PathBuf::from("hosts.json"));
    homie_proto::HostsConfig::load(hosts_file)
        .hosts
        .into_iter()
        .find(|entry| entry.id == host_id)
        .ok_or_else(|| {
            ControlError::bad_request(format!("unknown host {host_id:?}; check hosts.json"))
        })
}

/// Maps an I/O error onto the control channel's error vocabulary, mirroring
/// the transport layer so remote spec errors keep identical codes/messages.
fn io_control_error(error: std::io::Error) -> ControlError {
    match error.kind() {
        std::io::ErrorKind::NotFound => ControlError::not_found(error.to_string()),
        _ => ControlError::internal(error.to_string()),
    }
}

/// The spawn spec that re-enters a local conversation: the manifest's resume
/// argv plus the same hook/MCP wiring a fresh spawn gets — a resumed Claude
/// must not silently lose status detection or the homie tools.
pub fn resume_spec(
    ctx: &LaunchContext,
    registry: &Registry,
    id: &str,
    kind: &str,
    cwd: &str,
    agent_session_id: Option<&str>,
) -> Result<SessionSpec, ControlError> {
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
    if let (Some(inject_dir), Some(cli_path)) = (&ctx.inject_dir, &ctx.cli_path) {
        // Only the appendable flag mechanisms replay on resume, exactly as in
        // Swift: Codex's global `-c` overrides must precede the resume
        // SUBCOMMAND and are deliberately not replayed.
        let claude_only = crate::agent::InjectionSpec {
            claude_hooks: descriptor.injection.claude_hooks,
            claude_mcp: descriptor.injection.claude_mcp,
            ..Default::default()
        };
        launch_args.extend(crate::inject::injection_args(
            &claude_only,
            inject_dir,
            cli_path,
            None,
            ctx.mcp.as_ref(),
        ));
    }

    let inherited: Vec<(String, String)> = std::env::vars().collect();
    let mut pty = descriptor
        .spawn_spec(Path::new(cwd), inherited, &launch_args)
        .ok_or_else(|| ControlError::internal("resume spec without a binary"))?;
    if let (Some(_inject_dir), Some(cli_path)) = (&ctx.inject_dir, &ctx.cli_path) {
        pty.env
            .push((crate::inject::SESSION_ID_ENV.into(), id.to_string()));
        pty.env.push((
            crate::inject::SOCKET_ENV.into(),
            ctx.socket_path.to_string_lossy().into_owned(),
        ));
        pty.env.push((
            crate::inject::CLI_ENV.into(),
            cli_path.to_string_lossy().into_owned(),
        ));
        if let Some(mcp) = ctx.mcp.as_ref() {
            pty.env.extend(crate::inject::mcp_env(mcp));
        }
    }
    Ok(SessionSpec {
        id: id.to_string(),
        pty,
        manifest_id: kind.to_string(),
        authority: descriptor.authority(),
        logs_dir: ctx.logs_dir.clone(),
        holder: ctx.holder.clone(),
        remote: None,
        defer_launch: true,
    })
}

/// The spawn spec that re-enters a remote (holder-backed) conversation: probe
/// the host's persistence, capture its environment, and build the launch
/// request with the manifest's resume argv.
pub fn remote_resume_spec(
    ctx: &LaunchContext,
    registry: &Registry,
    record: &homie_proto::SessionRecord,
) -> Result<SessionSpec, ControlError> {
    let manager = ctx
        .remote
        .as_ref()
        .cloned()
        .ok_or_else(crate::remote::transport_unavailable)?;
    let binding_store = ctx
        .remote_bindings
        .clone()
        .ok_or_else(|| ControlError::internal("owner-only remote binding store is unavailable"))?;
    let host_id = record
        .host
        .as_deref()
        .ok_or_else(|| ControlError::bad_request("remote record has no host"))?;
    let host = resolve_host(ctx, host_id)?;
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
    Ok(SessionSpec {
        id: record.id.0.clone(),
        pty,
        manifest_id: record.kind.id().to_string(),
        authority,
        logs_dir: ctx.logs_dir.clone(),
        holder: None,
        remote: Some(RemoteSessionSpec {
            manager,
            helper,
            launch,
            host_id: host.id,
            binding_store,
        }),
        defer_launch: false,
    })
}

/// Assembles the spawn spec and record for a local (`host: null`) spawn.
///
/// Worktree creation and cwd validation happen in the handler, *before* the
/// registry lock, because `git worktree add` is a slow filesystem operation
/// that must not stall every other control request. This function therefore
/// receives the already-resolved `cwd`, `worktree_path`, and `git_branch` and
/// only performs in-memory assembly (manifest lookup, argv build, virtual-key
/// mint, spec construction) under the lock.
#[allow(clippy::too_many_arguments)]
pub fn spawn_spec(
    ctx: &LaunchContext,
    registry: &Registry,
    params: &homie_proto::SessionSpawnParams,
    caller_argv: Vec<String>,
    cwd: &str,
    worktree_path: Option<String>,
    git_branch: Option<String>,
) -> Result<SpawnPlan, ControlError> {
    let p = params;
    let kind = p.kind.id().to_string();
    // A generic kind carries the user's command line inside itself.
    let argv = if caller_argv.is_empty() {
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
        caller_argv
    };

    let cwd_path = PathBuf::from(cwd);
    let engine = registry.engine();
    let manifest = engine
        .manifest(&kind)
        .ok_or_else(|| ControlError::not_found(format!("no manifest for agent {kind:?}")))?;
    let descriptor = manifest.agent.clone().unwrap_or_default();
    let authority = descriptor.authority();

    let id = crate::control::next_session_id();
    // Build the complete agent argv before `spawn_spec`: agents declaring
    // `returnToLoginShell` need every manifest and injection argument quoted
    // inside the shell's `-c` command.
    let mut launch_args = argv.clone();
    let mut agent_session_id = None;
    let mut gateway_runtime = None;
    if descriptor.binary.is_some() {
        launch_args.extend(descriptor.spawn_args.iter().cloned());
        agent_session_id = descriptor.session_id_flag.as_ref().map(|flag| {
            let uuid = crate::inject::uuid_v4();
            launch_args.push(flag.clone());
            launch_args.push(uuid.clone());
            uuid
        });
        if let (Some(inject_dir), Some(cli_path)) = (&ctx.inject_dir, &ctx.cli_path) {
            // Mint a per-session virtual key before assembling argv so the
            // same runtime feeds both the Codex `-c` overrides and env.
            if descriptor.injection.codex_gateway
                && let Some(issuer) = &ctx.gateway
            {
                match issuer.mint(Some(id.clone())) {
                    Ok(runtime) => gateway_runtime = Some(runtime),
                    Err(error) => eprintln!("homied-rs: virtual key mint failed: {error}"),
                }
            }
            launch_args.extend(crate::inject::injection_args(
                &descriptor.injection,
                inject_dir,
                cli_path,
                gateway_runtime.as_ref(),
                ctx.mcp.as_ref(),
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

    let mut record = crate::control::new_record(&id, &kind, cwd);
    // A linked worktree is an execution cwd inside the project selected by
    // the user; it does not become a new first-level sidebar project.
    record.project_id = crate::registry::session_project_id(&p.cwd, None);
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

    // Injection environment and the caller-minted conversation UUID. The argv
    // side was assembled before `spawn_spec` so its shell wrapper contains the
    // complete command.
    if descriptor.binary.is_some() {
        if let (Some(_), Some(cli_path)) = (&ctx.inject_dir, &ctx.cli_path) {
            pty.env
                .push((crate::inject::SESSION_ID_ENV.into(), id.clone()));
            pty.env.push((
                crate::inject::SOCKET_ENV.into(),
                ctx.socket_path.to_string_lossy().into_owned(),
            ));
            pty.env.push((
                crate::inject::CLI_ENV.into(),
                cli_path.to_string_lossy().into_owned(),
            ));
            if let Some(runtime) = &gateway_runtime {
                pty.env
                    .extend(crate::inject::gateway_env(&descriptor.injection, runtime));
            }
            if let Some(mcp) = ctx.mcp.as_ref() {
                pty.env.extend(crate::inject::mcp_env(mcp));
            }
        }
        if let Some(uuid) = &agent_session_id {
            record.agent_session_id = Some(uuid.clone());
            if descriptor.injection.claude_hooks
                && let Ok(home) = std::env::var("HOME")
            {
                record.transcript_path = Some(
                    crate::inject::claude_transcript_path(Path::new(&home), cwd, uuid)
                        .to_string_lossy()
                        .into_owned(),
                );
            }
        }
    }

    let spec = SessionSpec {
        id: id.clone(),
        pty,
        manifest_id: kind,
        authority,
        logs_dir: ctx.logs_dir.clone(),
        holder: ctx.holder.clone(),
        remote: None,
        defer_launch: true,
    };

    let prompt = p.initial_prompt.clone().filter(|prompt| !prompt.is_empty());

    Ok(SpawnPlan {
        spec,
        record,
        prompt,
        project_root: p.cwd.clone(),
        host_id: None,
    })
}

/// Assembles the spawn spec and record for a remote (holder-backed) spawn:
/// probe the host's persistence, capture its environment, and build the
/// launch request with the manifest's spawn argv.
///
/// The `descriptor` is resolved by the handler under a brief registry lock and
/// passed in, so the slow remote transport calls (helper install, persistence
/// probe, environment capture) run without holding the registry mutex.
pub fn remote_spawn_spec(
    ctx: &LaunchContext,
    descriptor: crate::agent::AgentDescriptor,
    params: homie_proto::SessionSpawnParams,
    caller_argv: Vec<String>,
) -> Result<SpawnPlan, ControlError> {
    let p = params;
    let manager = ctx
        .remote
        .as_ref()
        .cloned()
        .ok_or_else(crate::remote::transport_unavailable)?;
    let binding_store = ctx
        .remote_bindings
        .clone()
        .ok_or_else(|| ControlError::internal("owner-only remote binding store is unavailable"))?;
    let host_id = p
        .host
        .as_deref()
        .ok_or_else(|| ControlError::bad_request("remote host is required"))?;
    let host = resolve_host(ctx, host_id)?;
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
    let authority = descriptor.authority();
    let inherited = captured
        .environment
        .into_iter()
        .map(|variable| (variable.name, variable.value))
        .collect::<Vec<_>>();

    let id = crate::control::next_session_id();
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

    let mut record = crate::control::new_record(&id, &kind, &captured.cwd);
    record.host = Some(host.id.clone());
    record.project_id = crate::registry::session_project_id(&captured.cwd, Some(&host.id));
    record.remote_persistence = Some(persistence);
    record.parent = p.parent.clone();
    record.agent_session_id = agent_session_id;
    if let Some(title) = &p.title {
        record.title = title.clone();
        record.title_source = homie_proto::TitleSource::HomieAssigned;
    }
    let spec = SessionSpec {
        id: id.clone(),
        pty,
        manifest_id: kind,
        authority,
        logs_dir: ctx.logs_dir.clone(),
        holder: None,
        remote: Some(RemoteSessionSpec {
            manager,
            helper,
            launch,
            host_id: host.id.clone(),
            binding_store,
        }),
        defer_launch: false,
    };

    let prompt = p.initial_prompt.filter(|prompt| !prompt.is_empty());

    Ok(SpawnPlan {
        spec,
        record,
        prompt,
        project_root: captured.cwd,
        host_id: Some(host.id),
    })
}

/// The PTY environment for a bare shell/generic spawn: asserts `TERM` and
/// removes `NO_COLOR`, mirroring what `spawn_spec` does for manifest agents.
fn shell_pty_environment(mut inherited: Vec<(String, String)>) -> Vec<(String, String)> {
    inherited.retain(|(key, _)| key != "TERM" && key != "NO_COLOR");
    inherited.push(("TERM".into(), "xterm-256color".into()));
    inherited
}

#[cfg(test)]
mod tests {
    use super::shell_pty_environment;

    #[test]
    fn local_pty_environment_sets_term_and_removes_no_color() {
        let env = shell_pty_environment(vec![
            ("TERM".into(), "dumb".into()),
            ("NO_COLOR".into(), "1".into()),
            ("PATH".into(), "/bin".into()),
        ]);

        assert_eq!(
            env.iter()
                .filter(|(key, _)| key == "TERM")
                .map(|(_, value)| value.as_str())
                .collect::<Vec<_>>(),
            vec!["xterm-256color"]
        );
        assert!(!env.iter().any(|(key, _)| key == "NO_COLOR"));
        assert!(
            env.iter()
                .any(|(key, value)| key == "PATH" && value == "/bin")
        );
    }
}
