//! Spawn-time config injection: what turns a bare agent CLI into a
//! Homie-connected session.
//!
//! Ported from the local half of `InjectionBuilder`. The daemon writes two
//! files at startup — a Claude hooks settings file and a Claude MCP config —
//! whose contents reference `$HOMIE_CLI` / the CLI's sibling `homie-mcp`,
//! then appends per-launch flags (`--settings`, `--mcp-config`, Codex `-c`
//! overrides) for whichever mechanisms the agent's manifest opted into. This
//! is what makes a Claude session hook-driven rather than screen-detected,
//! and what gives every agent the `homie` MCP tools.

use std::io;
use std::path::{Path, PathBuf};

use serde_json::json;

use crate::agent::InjectionSpec;

/// Environment variable names shared with every hook and MCP shim.
pub const SESSION_ID_ENV: &str = "HOMIE_SESSION_ID";
pub const SOCKET_ENV: &str = "HOMIE_SOCKET";
pub const CLI_ENV: &str = "HOMIE_CLI";

/// Environment variable Codex reads (via `model_providers.homie.env_key`) for
/// the gateway virtual key. Named under `HOMIE_` so the daemon owns it, but
/// injected *after* env scrubbing so it survives into the session.
pub const CODEX_GATEWAY_ENV: &str = "HOMIE_CODEX_GATEWAY_KEY";

/// Runtime facts about the local LLM gateway injected into agents that opt in.
///
/// `base_url` is the gateway root (e.g. `http://127.0.0.1:7338`); Codex gets
/// `<base_url>/v1` as its OpenAI-compatible base, Claude Code gets `base_url`
/// directly as `ANTHROPIC_BASE_URL`. `virtual_key` is a `sk-…` issued to the
/// session and never shown after issuance.
#[derive(Clone, Debug)]
pub struct GatewayRuntime {
    pub base_url: String,
    pub virtual_key: String,
}

impl GatewayRuntime {
    fn codex_base_url(&self) -> String {
        format!("{}/v1", self.base_url.trim_end_matches('/'))
    }
}

/// Codex `-c` overrides routing it through the local gateway.
pub fn codex_gateway_args(runtime: &GatewayRuntime) -> Vec<String> {
    vec![
        "-c".into(),
        "model_provider=\"homie\"".into(),
        "-c".into(),
        format!(
            "model_providers.homie.base_url=\"{}\"",
            runtime.codex_base_url()
        ),
        "-c".into(),
        "model_providers.homie.wire_api=\"responses\"".into(),
        "-c".into(),
        format!("model_providers.homie.env_key=\"{}\"", CODEX_GATEWAY_ENV),
    ]
}

/// Environment a Claude Code session needs to route through the gateway.
pub fn claude_gateway_env(runtime: &GatewayRuntime) -> Vec<(String, String)> {
    vec![
        ("ANTHROPIC_BASE_URL".into(), runtime.base_url.clone()),
        ("ANTHROPIC_AUTH_TOKEN".into(), runtime.virtual_key.clone()),
    ]
}

/// Environment a Codex session needs to supply the virtual key.
pub fn codex_gateway_env(runtime: &GatewayRuntime) -> Vec<(String, String)> {
    vec![(CODEX_GATEWAY_ENV.into(), runtime.virtual_key.clone())]
}

/// Combined session env for gateway routing, gated by each agent's opt-in.
pub fn gateway_env(injection: &InjectionSpec, runtime: &GatewayRuntime) -> Vec<(String, String)> {
    let mut env = Vec::new();
    if injection.claude_gateway {
        env.extend(claude_gateway_env(runtime));
    }
    if injection.codex_gateway {
        env.extend(codex_gateway_env(runtime));
    }
    env
}

/// A random v4 UUID in the lowercase-hex form Claude accepts as
/// `--session-id`. Minting it ourselves is what makes resume possible later
/// without the agent ever reporting an id.
pub fn uuid_v4() -> String {
    let mut bytes = [0u8; 16];
    getrandom::fill(&mut bytes).expect("random");
    bytes[6] = (bytes[6] & 0x0f) | 0x40; // version 4
    bytes[8] = (bytes[8] & 0x3f) | 0x80; // RFC 4122 variant
    let h = |range: std::ops::Range<usize>| {
        bytes[range]
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    };
    format!(
        "{}-{}-{}-{}-{}",
        h(0..4),
        h(4..6),
        h(6..8),
        h(8..10),
        h(10..16)
    )
}

/// The static hooks file injected into every Claude session via `--settings`.
/// Commands read `$HOMIE_CLI` / `$HOMIE_SOCKET` from the PTY env, so the
/// file content is identical for all sessions and safe to write once.
pub fn write_claude_hooks_file(inject_dir: &Path) -> io::Result<()> {
    const EVENTS: [&str; 9] = [
        "SessionStart",
        "UserPromptSubmit",
        "PreToolUse",
        "PermissionRequest",
        "Notification",
        "Stop",
        "SubagentStart",
        "SubagentStop",
        "SessionEnd",
    ];
    let mut hooks = serde_json::Map::new();
    for event in EVENTS {
        hooks.insert(
            event.to_string(),
            json!([{
                "hooks": [{
                    "type": "command",
                    "command": format!("\"${CLI_ENV}\" hook {event}"),
                    "timeout": 10,
                }]
            }]),
        );
    }
    write_atomic(
        &inject_dir.join("claude-hooks.json"),
        &serde_json::to_vec(&json!({ "hooks": hooks }))?,
    )
}

/// The Claude `--mcp-config` file: the `homie` stdio server backed by the
/// CLI's sibling `homie-mcp` proxy (or the CLI itself as a fallback).
pub fn write_claude_mcp_file(inject_dir: &Path, cli_path: &Path) -> io::Result<()> {
    let (command, args) = mcp_launch(cli_path);
    write_atomic(
        &inject_dir.join("claude-mcp.json"),
        &serde_json::to_vec_pretty(&json!({
            "mcpServers": {
                "homie": { "type": "stdio", "command": command, "args": args }
            }
        }))?,
    )
}

fn mcp_launch(cli_path: &Path) -> (String, Vec<String>) {
    let proxy = cli_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("homie-mcp");
    if is_executable(&proxy) {
        (proxy.to_string_lossy().into_owned(), Vec::new())
    } else {
        (
            cli_path.to_string_lossy().into_owned(),
            vec!["mcp-stdio".into()],
        )
    }
}

/// Per-launch injection arguments for the mechanisms a manifest opted into.
pub fn injection_args(
    injection: &InjectionSpec,
    inject_dir: &Path,
    cli_path: &Path,
    gateway: Option<&GatewayRuntime>,
) -> Vec<String> {
    let mut argv = Vec::new();
    if injection.claude_hooks {
        let hooks = inject_dir.join("claude-hooks.json");
        if hooks.exists() {
            argv.push("--settings".into());
            argv.push(hooks.to_string_lossy().into_owned());
        }
    }
    if injection.claude_mcp {
        let mcp = inject_dir.join("claude-mcp.json");
        if mcp.exists() {
            argv.push("--mcp-config".into());
            argv.push(mcp.to_string_lossy().into_owned());
        }
    }
    if injection.codex_notify {
        argv.push("-c".into());
        argv.push(format!(
            "notify=[{}, \"notify\"]",
            toml_string(&cli_path.to_string_lossy())
        ));
    }
    if injection.codex_mcp {
        let (command, args) = mcp_launch(cli_path);
        let encoded_args = args
            .iter()
            .map(|arg| toml_string(arg))
            .collect::<Vec<_>>()
            .join(",");
        argv.push("-c".into());
        argv.push(format!(
            "mcp_servers.homie.command={}",
            toml_string(&command)
        ));
        argv.push("-c".into());
        argv.push(format!("mcp_servers.homie.args=[{encoded_args}]"));
    }
    if injection.codex_gateway {
        if let Some(runtime) = gateway {
            argv.extend(codex_gateway_args(runtime));
        }
    }
    argv
}

/// Claude Code's project-directory slug for a working directory: `/` and `.`
/// replaced by `-`, verified against real dirs under `~/.claude/projects`.
pub fn claude_project_slug(cwd: &str) -> String {
    cwd.replace(['/', '.'], "-")
}

/// `~/.claude/projects/<slug>/<uuid>.jsonl` — predictable only for Claude,
/// because only Claude lets the caller choose the session UUID *and* derives
/// its jsonl path from the cwd.
pub fn claude_transcript_path(home: &Path, cwd: &str, session_uuid: &str) -> PathBuf {
    home.join(".claude/projects")
        .join(claude_project_slug(cwd))
        .join(format!("{session_uuid}.jsonl"))
}

fn toml_string(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}

fn is_executable(path: &Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::metadata(path)
            .map(|meta| meta.is_file() && meta.permissions().mode() & 0o111 != 0)
            .unwrap_or(false)
    }
    #[cfg(not(unix))]
    {
        path.is_file()
    }
}

fn write_atomic(path: &Path, contents: &[u8]) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, contents)?;
    std::fs::rename(&tmp, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uuids_are_v4_and_unique() {
        let a = uuid_v4();
        let b = uuid_v4();
        assert_ne!(a, b);
        assert_eq!(a.len(), 36);
        assert_eq!(a.as_bytes()[14], b'4', "version nibble: {a}");
        assert!(
            matches!(a.as_bytes()[19], b'8' | b'9' | b'a' | b'b'),
            "variant nibble: {a}"
        );
    }

    #[test]
    fn the_hooks_file_matches_the_swift_shape() {
        let temp = tempfile::tempdir().expect("temp");
        write_claude_hooks_file(temp.path()).expect("write");
        let parsed: serde_json::Value = serde_json::from_slice(
            &std::fs::read(temp.path().join("claude-hooks.json")).expect("read"),
        )
        .expect("parse");
        let stop = &parsed["hooks"]["Stop"][0]["hooks"][0];
        assert_eq!(stop["type"], "command");
        assert_eq!(stop["command"], "\"$HOMIE_CLI\" hook Stop");
        assert_eq!(stop["timeout"], 10);
        assert!(parsed["hooks"]["SubagentStop"].is_array());
    }

    #[test]
    fn the_mcp_file_prefers_the_sibling_proxy() {
        let temp = tempfile::tempdir().expect("temp");
        let cli = temp.path().join("bin/homie");
        std::fs::create_dir_all(cli.parent().unwrap()).expect("mkdir");
        std::fs::write(&cli, "#!/bin/sh\n").expect("cli");

        // No proxy: fall back to `homie mcp-stdio`.
        write_claude_mcp_file(temp.path(), &cli).expect("write");
        let parsed: serde_json::Value = serde_json::from_slice(
            &std::fs::read(temp.path().join("claude-mcp.json")).expect("read"),
        )
        .expect("parse");
        assert_eq!(parsed["mcpServers"]["homie"]["args"][0], "mcp-stdio");

        // With an executable sibling, it becomes the command.
        let proxy = temp.path().join("bin/homie-mcp");
        std::fs::write(&proxy, "#!/bin/sh\n").expect("proxy");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&proxy, std::fs::Permissions::from_mode(0o755))
                .expect("chmod");
        }
        write_claude_mcp_file(temp.path(), &cli).expect("write");
        let parsed: serde_json::Value = serde_json::from_slice(
            &std::fs::read(temp.path().join("claude-mcp.json")).expect("read"),
        )
        .expect("parse");
        assert_eq!(
            parsed["mcpServers"]["homie"]["command"],
            proxy.to_string_lossy().as_ref()
        );
        assert_eq!(
            parsed["mcpServers"]["homie"]["args"]
                .as_array()
                .map(Vec::len),
            Some(0)
        );
    }

    #[test]
    fn injection_args_cover_all_four_mechanisms() {
        let temp = tempfile::tempdir().expect("temp");
        let cli = temp.path().join("homie");
        write_claude_hooks_file(temp.path()).expect("hooks");
        write_claude_mcp_file(temp.path(), &cli).expect("mcp");

        let claude = InjectionSpec {
            claude_hooks: true,
            claude_mcp: true,
            ..Default::default()
        };
        let args = injection_args(&claude, temp.path(), &cli, None);
        assert_eq!(args[0], "--settings");
        assert!(args[1].ends_with("claude-hooks.json"));
        assert_eq!(args[2], "--mcp-config");
        assert!(args[3].ends_with("claude-mcp.json"));

        let codex = InjectionSpec {
            codex_notify: true,
            codex_mcp: true,
            ..Default::default()
        };
        let args = injection_args(&codex, temp.path(), &cli, None);
        assert_eq!(args[0], "-c");
        assert!(args[1].starts_with("notify=["), "{args:?}");
        assert!(
            args.iter()
                .any(|arg| arg.starts_with("mcp_servers.homie.command=")),
            "{args:?}"
        );
    }

    #[test]
    fn codex_gateway_args_route_through_local_gateway() {
        let runtime = GatewayRuntime {
            base_url: "http://127.0.0.1:7338".into(),
            virtual_key: "sk-abc123".into(),
        };
        let injection = InjectionSpec {
            codex_gateway: true,
            ..Default::default()
        };
        let temp = tempfile::tempdir().expect("temp");
        let cli = temp.path().join("homie");
        let args = injection_args(&injection, temp.path(), &cli, Some(&runtime));
        assert!(args.iter().any(|a| a == "-c"), "{args:?}");
        assert!(
            args.iter().any(|a| a == "model_provider=\"homie\""),
            "{args:?}"
        );
        assert!(
            args.iter()
                .any(|a| a == "model_providers.homie.base_url=\"http://127.0.0.1:7338/v1\""),
            "{args:?}"
        );
        assert!(
            args.iter()
                .any(|a| a == "model_providers.homie.wire_api=\"responses\""),
            "{args:?}"
        );
        assert!(
            args.iter()
                .any(|a| *a == format!("model_providers.homie.env_key=\"{}\"", CODEX_GATEWAY_ENV)),
            "{args:?}"
        );
    }

    #[test]
    fn gateway_args_absent_without_opt_in_or_runtime() {
        let temp = tempfile::tempdir().expect("temp");
        let cli = temp.path().join("homie");
        let runtime = GatewayRuntime {
            base_url: "http://127.0.0.1:7338".into(),
            virtual_key: "sk-x".into(),
        };
        // Not opted in → no gateway args even with a runtime.
        let no_opt = InjectionSpec::default();
        assert!(injection_args(&no_opt, temp.path(), &cli, Some(&runtime)).is_empty());
        // Opted in but no runtime → no gateway args.
        let opt = InjectionSpec {
            codex_gateway: true,
            ..Default::default()
        };
        assert!(injection_args(&opt, temp.path(), &cli, None).is_empty());
    }

    #[test]
    fn claude_gateway_env_sets_anthropic_base_and_token() {
        let runtime = GatewayRuntime {
            base_url: "http://127.0.0.1:7338".into(),
            virtual_key: "sk-abc123".into(),
        };
        let env = claude_gateway_env(&runtime);
        assert_eq!(env.len(), 2);
        assert_eq!(
            env[0],
            (
                "ANTHROPIC_BASE_URL".to_owned(),
                "http://127.0.0.1:7338".to_owned()
            )
        );
        assert_eq!(
            env[1],
            ("ANTHROPIC_AUTH_TOKEN".to_owned(), "sk-abc123".to_owned())
        );
    }

    #[test]
    fn codex_gateway_env_exposes_key_via_env_key() {
        let runtime = GatewayRuntime {
            base_url: "http://127.0.0.1:7338".into(),
            virtual_key: "sk-abc123".into(),
        };
        let env = codex_gateway_env(&runtime);
        assert_eq!(env.len(), 1);
        assert_eq!(env[0].0, CODEX_GATEWAY_ENV);
        assert_eq!(env[0].1, "sk-abc123");
    }

    #[test]
    fn the_transcript_slug_matches_claudes_rule() {
        assert_eq!(
            claude_project_slug("/Users/giga/.claude/worktrees/x"),
            "-Users-giga--claude-worktrees-x"
        );
        let path = claude_transcript_path(Path::new("/Users/giga"), "/tmp/repo", "abc-123");
        assert_eq!(
            path,
            Path::new("/Users/giga/.claude/projects/-tmp-repo/abc-123.jsonl")
        );
    }
}
