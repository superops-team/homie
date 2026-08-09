use std::collections::BTreeMap;
use std::path::PathBuf;

use homie_agents::{AgentInjection, AgentManifest, StatusAuthority};
use homie_orchestrator::{SpawnPlanInput, build_spawn_plan};

#[test]
fn builds_claude_hooks_and_mcp_injection_plan() {
    let manifest = AgentManifest {
        id: "claude-code".to_string(),
        display_name: "Claude Code".to_string(),
        short_label: "Claude".to_string(),
        glyph: "C".to_string(),
        aliases: vec![],
        binary: Some("claude".to_string()),
        spawn_args: vec!["--dangerously-skip-permissions".to_string()],
        session_id_flag: Some("--session-id".to_string()),
        status_authority: StatusAuthority::Hooks,
        first_class: true,
        resume: None,
        return_to_login_shell: false,
        approve: None,
        deny: None,
        env: BTreeMap::from([("CLAUDE_CODE_NO_FLICKER".to_string(), "1".to_string())]),
        env_scrub_prefixes: vec![],
        injection: AgentInjection {
            claude_hooks: true,
            claude_mcp: true,
            codex_notify: false,
            codex_mcp: false,
        },
        foreground_exec_names: vec![],
    };
    let plan = build_spawn_plan(SpawnPlanInput {
        manifest: &manifest,
        session_id: "session-1",
        socket_path: "/tmp/homie.sock",
        cli_path: "/usr/local/bin/homie",
        login_shell: "/bin/zsh",
        login_path: "/usr/bin:/bin",
        inject_dir: PathBuf::from("/tmp/inject"),
        agent_session_id: Some("agent-123".to_string()),
    });

    assert_eq!(plan.agent_session_id.as_deref(), Some("agent-123"));
    assert!(plan.argv.starts_with(&[
        "claude".to_string(),
        "--dangerously-skip-permissions".to_string(),
        "--session-id".to_string(),
        "agent-123".to_string(),
    ]));
    assert!(plan.argv.contains(&"--settings".to_string()));
    assert!(
        plan.argv
            .contains(&"/tmp/inject/claude-hooks.json".to_string())
    );
    assert!(plan.argv.iter().any(|arg| arg.contains("claude-mcp.json")));
    assert_eq!(
        plan.env.get("HOMIE_SESSION_ID").map(String::as_str),
        Some("session-1")
    );
    assert_eq!(
        plan.env.get("HOMIE_SOCKET").map(String::as_str),
        Some("/tmp/homie.sock")
    );
    assert_eq!(
        plan.env.get("HOMIE_CLI").map(String::as_str),
        Some("/usr/local/bin/homie")
    );
    assert_eq!(
        plan.env.get("PATH").map(String::as_str),
        Some("/usr/bin:/bin")
    );
    assert_eq!(
        plan.env.get("CLAUDE_CODE_NO_FLICKER").map(String::as_str),
        Some("1")
    );
}

#[test]
fn builds_codex_notify_and_mcp_injection_plan() {
    let manifest = AgentManifest {
        id: "codex".to_string(),
        display_name: "Codex".to_string(),
        short_label: "Codex".to_string(),
        glyph: "X".to_string(),
        aliases: vec![],
        binary: Some("codex".to_string()),
        spawn_args: vec![],
        session_id_flag: None,
        status_authority: StatusAuthority::Hooks,
        first_class: true,
        resume: None,
        return_to_login_shell: false,
        approve: None,
        deny: None,
        env: BTreeMap::new(),
        env_scrub_prefixes: vec![],
        injection: AgentInjection {
            claude_hooks: false,
            claude_mcp: false,
            codex_notify: true,
            codex_mcp: true,
        },
        foreground_exec_names: vec![],
    };
    let plan = build_spawn_plan(SpawnPlanInput {
        manifest: &manifest,
        session_id: "session-2",
        socket_path: "/tmp/homie.sock",
        cli_path: "/usr/local/bin/homie",
        login_shell: "/bin/zsh",
        login_path: "/usr/bin:/bin",
        inject_dir: PathBuf::from("/tmp/inject"),
        agent_session_id: None,
    });

    assert_eq!(plan.argv[0], "codex");
    assert!(
        plan.argv
            .iter()
            .any(|arg| arg == "notify=[\"/usr/local/bin/homie\", \"notify\"]")
    );
    assert!(
        plan.argv
            .iter()
            .any(|arg| arg.contains("mcp_servers.homie"))
    );
}

#[test]
fn wraps_return_to_login_shell_and_generic_command() {
    let manifest = AgentManifest {
        id: "custom".to_string(),
        display_name: "Custom".to_string(),
        short_label: "Custom".to_string(),
        glyph: "*".to_string(),
        aliases: vec![],
        binary: Some("custom-agent".to_string()),
        spawn_args: vec!["--mode".to_string(), "fast".to_string()],
        session_id_flag: None,
        status_authority: StatusAuthority::Process,
        first_class: false,
        resume: None,
        return_to_login_shell: true,
        approve: None,
        deny: None,
        env: BTreeMap::new(),
        env_scrub_prefixes: vec![],
        injection: AgentInjection::default(),
        foreground_exec_names: vec![],
    };
    let plan = build_spawn_plan(SpawnPlanInput {
        manifest: &manifest,
        session_id: "session-3",
        socket_path: "/tmp/homie.sock",
        cli_path: "/usr/local/bin/homie",
        login_shell: "/bin/zsh",
        login_path: "/usr/bin:/bin",
        inject_dir: PathBuf::from("/tmp/inject"),
        agent_session_id: None,
    });

    assert_eq!(plan.argv[0], "/bin/zsh");
    assert_eq!(plan.argv[1], "-i");
    assert_eq!(plan.argv[2], "-l");
    assert_eq!(plan.argv[3], "-c");
    assert!(plan.argv[4].contains("'custom-agent' '--mode' 'fast'; exec '/bin/zsh' -i -l"));
}
