//! `inject --agent` preview: reuses `homie-engine::inject` so the CLI preview
//! and real spawn-time injection share one source of truth (`injection_args`).

use serde_json::json;

use homie_engine::inject::{
    GatewayRuntime, claude_gateway_env, codex_gateway_args, codex_gateway_env,
};

/// Placeholder shown at preview time. No session exists yet, so no real key is
/// minted; the daemon issues an `sk-…` only at spawn.
pub const VIRTUAL_KEY_PLACEHOLDER: &str = "<virtual-key-issued-at-spawn>";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Agent {
    Codex,
    Claude,
}

impl Agent {
    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "codex" => Some(Self::Codex),
            "claude" => Some(Self::Claude),
            _ => None,
        }
    }
}

/// Build the exact injection preview JSON for an agent given the gateway's
/// listen address (already turned into an `http://` base URL by the caller).
pub fn preview(agent: &Agent, base_url: &str) -> serde_json::Value {
    let runtime = GatewayRuntime {
        base_url: base_url.to_owned(),
        virtual_key: VIRTUAL_KEY_PLACEHOLDER.to_owned(),
    };
    match agent {
        Agent::Codex => json!({
            "agent": "codex",
            "args": codex_gateway_args(&runtime),
            "env": codex_gateway_env(&runtime),
        }),
        Agent::Claude => json!({
            "agent": "claude",
            "args": [],
            "env": claude_gateway_env(&runtime),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_known_agents_only() {
        assert_eq!(Agent::parse("codex"), Some(Agent::Codex));
        assert_eq!(Agent::parse("claude"), Some(Agent::Claude));
        assert_eq!(Agent::parse("gpt"), None);
    }

    #[test]
    fn codex_preview_emits_route_args_and_env_key() {
        let value = preview(&Agent::Codex, "http://127.0.0.1:7338");
        assert_eq!(value["agent"], "codex");
        let args = value["args"].as_array().expect("args array");
        assert!(
            args.iter().any(|a| a == "model_provider=\"homie\""),
            "{args:?}"
        );
        assert!(
            args.iter()
                .any(|a| a == "model_providers.homie.base_url=\"http://127.0.0.1:7338/v1\""),
            "{args:?}"
        );
        let env = value["env"].as_array().expect("env array");
        assert_eq!(env.len(), 1);
        assert_eq!(env[0][0], "HOMIE_CODEX_GATEWAY_KEY");
        assert_eq!(env[0][1], VIRTUAL_KEY_PLACEHOLDER);
    }

    #[test]
    fn claude_preview_emits_anthropic_env() {
        let value = preview(&Agent::Claude, "http://127.0.0.1:7338");
        assert_eq!(value["agent"], "claude");
        assert!(value["args"].as_array().expect("args").is_empty());
        let env = value["env"].as_array().expect("env array");
        assert_eq!(env.len(), 2);
        assert_eq!(env[0][0], "ANTHROPIC_BASE_URL");
        assert_eq!(env[0][1], "http://127.0.0.1:7338");
        assert_eq!(env[1][0], "ANTHROPIC_AUTH_TOKEN");
        assert_eq!(env[1][1], VIRTUAL_KEY_PLACEHOLDER);
    }
}
