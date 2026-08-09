use homie_orchestrator::{IntentDecision, IntentRequest, IntentSource, route};
use homie_proto::SessionId;

#[test]
fn routes_new_agent_to_spawn_session() {
    let decision = route(IntentRequest {
        source: IntentSource::NewAgent,
        text: Some("build feature".to_string()),
        parent_session: None,
    });
    assert_eq!(
        decision,
        IntentDecision::SpawnSession {
            prompt: Some("build feature".to_string())
        }
    );
}

#[test]
fn routes_palette_to_ui_surface() {
    let decision = route(IntentRequest {
        source: IntentSource::CommandPalette,
        text: Some("history".to_string()),
        parent_session: None,
    });
    assert_eq!(
        decision,
        IntentDecision::OpenUiSurface {
            surface: "history".to_string()
        }
    );
}

#[test]
fn routes_mcp_tool_to_parent_session_prompt() {
    let decision = route(IntentRequest {
        source: IntentSource::McpTool,
        text: Some("continue".to_string()),
        parent_session: Some(SessionId::from("session_1")),
    });
    assert_eq!(
        decision,
        IntentDecision::SendPrompt {
            session_id: SessionId::from("session_1"),
            text: "continue".to_string(),
            submit: true
        }
    );
}
