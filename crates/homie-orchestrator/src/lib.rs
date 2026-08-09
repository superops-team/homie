use homie_agents::AgentManifest;
use homie_proto::SessionId;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum IntentSource {
    NewAgent,
    CommandPalette,
    McpTool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct IntentRequest {
    pub source: IntentSource,
    pub text: Option<String>,
    pub parent_session: Option<SessionId>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum IntentDecision {
    SpawnSession {
        prompt: Option<String>,
    },
    SendPrompt {
        session_id: SessionId,
        text: String,
        submit: bool,
    },
    OpenUiSurface {
        surface: String,
    },
}

pub fn route(request: IntentRequest) -> IntentDecision {
    match request.source {
        IntentSource::NewAgent => IntentDecision::SpawnSession {
            prompt: request.text,
        },
        IntentSource::CommandPalette => IntentDecision::OpenUiSurface {
            surface: request
                .text
                .unwrap_or_else(|| "command_palette".to_string()),
        },
        IntentSource::McpTool => IntentDecision::SendPrompt {
            session_id: request
                .parent_session
                .unwrap_or_else(|| SessionId::from("unbound")),
            text: request.text.unwrap_or_default(),
            submit: true,
        },
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpawnPlanInput<'a> {
    pub manifest: &'a AgentManifest,
    pub session_id: &'a str,
    pub socket_path: &'a str,
    pub cli_path: &'a str,
    pub login_shell: &'a str,
    pub login_path: &'a str,
    pub inject_dir: PathBuf,
    pub agent_session_id: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpawnPlan {
    pub argv: Vec<String>,
    pub env: BTreeMap<String, String>,
    pub agent_session_id: Option<String>,
}

#[must_use]
pub fn build_spawn_plan(input: SpawnPlanInput<'_>) -> SpawnPlan {
    let mut env = BTreeMap::from([
        ("HOMIE_SESSION_ID".to_string(), input.session_id.to_string()),
        ("HOMIE_SOCKET".to_string(), input.socket_path.to_string()),
        ("HOMIE_CLI".to_string(), input.cli_path.to_string()),
        ("PATH".to_string(), input.login_path.to_string()),
    ]);
    for (key, value) in &input.manifest.env {
        env.insert(key.clone(), value.clone());
    }

    let mut argv = if let Some(binary) = &input.manifest.binary {
        let mut argv = vec![binary.clone()];
        argv.extend(input.manifest.spawn_args.clone());
        argv
    } else {
        vec![input.login_shell.to_string(), "-l".to_string()]
    };

    let agent_session_id = input.agent_session_id.clone();
    if let (Some(flag), Some(id)) = (&input.manifest.session_id_flag, &agent_session_id) {
        argv.push(flag.clone());
        argv.push(id.clone());
    }

    argv.extend(injection_args(
        &input.manifest.injection,
        &input.inject_dir,
        input.cli_path,
    ));

    if input.manifest.return_to_login_shell {
        argv = return_to_login_shell(&argv, input.login_shell);
    }

    SpawnPlan {
        argv,
        env,
        agent_session_id,
    }
}

fn injection_args(
    injection: &homie_agents::AgentInjection,
    inject_dir: &std::path::Path,
    cli_path: &str,
) -> Vec<String> {
    let mut argv = Vec::new();
    if injection.claude_hooks {
        argv.push("--settings".to_string());
        argv.push(inject_dir.join("claude-hooks.json").display().to_string());
    }
    if injection.claude_mcp {
        argv.push("--mcp-config".to_string());
        argv.push(inject_dir.join("claude-mcp.json").display().to_string());
    }
    if injection.codex_notify {
        argv.push("-c".to_string());
        argv.push(format!("notify=[{}, \"notify\"]", toml_string(cli_path)));
    }
    if injection.codex_mcp {
        argv.push("-c".to_string());
        argv.push(format!(
            "mcp_servers.homie.command={}",
            toml_string(cli_path)
        ));
    }
    argv
}

fn return_to_login_shell(argv: &[String], shell: &str) -> Vec<String> {
    let command = argv
        .iter()
        .map(|arg| shell_quote(arg))
        .collect::<Vec<_>>()
        .join(" ")
        + &format!("; exec {} -i -l", shell_quote(shell));
    vec![
        shell.to_string(),
        "-i".to_string(),
        "-l".to_string(),
        "-c".to_string(),
        command,
    ]
}

fn toml_string(value: &str) -> String {
    format!("{value:?}")
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}
