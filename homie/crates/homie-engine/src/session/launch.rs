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
use crate::session::{HolderConfig, RemoteSessionSpec, SessionSpec};

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
    pub socket_path: PathBuf,
    pub logs_dir: PathBuf,
    pub holder: Option<HolderConfig>,
    pub remote: Option<Arc<RemoteManager>>,
    pub remote_bindings: Option<RemoteBindingStore>,
}

/// A freshly minted remote session token, hex-encoded from 32 random bytes.
pub fn random_session_token() -> Result<homie_proto::remote_pty::SessionToken, ControlError> {
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
