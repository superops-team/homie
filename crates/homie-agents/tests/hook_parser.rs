use homie_agents::{HookEvent, NotifyEvent, parse_claude_hook, parse_codex_notify};
use homie_proto::{NeedsInputKind, RiskHint};
use serde_json::json;

#[test]
fn hook_parser_claude_permission_request_redacts_secret_command() {
    let payload = json!({
        "session_id": "abc-123",
        "transcript_path": "/tmp/abc-123.jsonl",
        "tool_name": "Bash",
        "tool_input": { "command": "deploy --token=example-token-value" }
    });

    let parsed = parse_claude_hook("PermissionRequest", &payload).expect("parsed");
    assert_eq!(parsed.session_id.as_deref(), Some("abc-123"));
    assert_eq!(
        parsed.event,
        HookEvent::PermissionRequest {
            tool_name: Some("Bash".into()),
            summary: Some("deploy --token=•••".into()),
            is_subagent: false,
        }
    );
    let detail = parsed.needs_input.expect("needs input detail");
    assert_eq!(detail.kind, NeedsInputKind::Approval);
    assert_eq!(detail.risk_hint, RiskHint::Neutral);
    assert!(!detail.summary.contains("example-token-value"));
}

#[test]
fn hook_parser_subagent_prompt_does_not_retitle_parent() {
    let payload = json!({
        "session_id": "parent",
        "agent_id": "child",
        "prompt": "Go do a subtask"
    });

    let parsed = parse_claude_hook("UserPromptSubmit", &payload).expect("parsed");
    assert_eq!(
        parsed.event,
        HookEvent::UserPromptSubmit { is_subagent: true }
    );
    assert_eq!(parsed.first_prompt_title, None);
}

#[test]
fn hook_parser_unknown_event_fails_open() {
    let payload = json!({
        "session_id": "s1",
        "authorization": "Bearer example-token"
    });

    let parsed = parse_claude_hook("FutureHook", &payload).expect("fail-open event");
    assert_eq!(parsed.event, HookEvent::Unknown("FutureHook".into()));
    assert_eq!(parsed.session_id.as_deref(), Some("s1"));
    assert!(!parsed.safe_summary.contains("example-token"));
}

#[test]
fn hook_parser_redacts_nested_headers_and_url_query_secrets() {
    let payload = json!({
        "session_id": "abc-123",
        "tool_name": "WebFetch",
        "tool_input": {
            "url": "https://example.test/deploy?token=example-url-token&safe=ok&api_key=example-api-key",
            "headers": {
                "Authorization": "Bearer example-auth-token",
                "Cookie": "sid=example-cookie",
                "X-Trace": "ordinary"
            },
            "nested": {
                "password": "example-password",
                "values": [
                    {"secret": "example-nested-secret"},
                    "Authorization: Bearer example-inline-token"
                ]
            }
        }
    });

    let parsed = parse_claude_hook("PermissionRequest", &payload).expect("parsed");
    let summary = parsed.safe_summary;
    for leaked in [
        "example-url-token",
        "example-api-key",
        "example-auth-token",
        "example-cookie",
        "example-password",
        "example-nested-secret",
        "example-inline-token",
    ] {
        assert!(
            !summary.contains(leaked),
            "safe summary leaked {leaked}: {summary}"
        );
    }
    assert!(summary.contains("ordinary"));

    let detail = parsed.needs_input.expect("needs input");
    assert_eq!(detail.kind, NeedsInputKind::Approval);
    assert!(!detail.summary.contains("example-url-token"));
    assert!(!detail.summary.contains("example-api-key"));
    assert!(
        !detail
            .prompt_excerpt
            .unwrap_or_default()
            .contains("example-url-token")
    );
}

#[test]
fn hook_parser_codex_turn_complete_is_stable_event() {
    let payload = json!({
        "type": "agent-turn-complete",
        "thread-id": "thread-1",
        "input-messages": ["Add status reducer tests"],
        "last-assistant-message": "done"
    });

    let parsed = parse_codex_notify(&payload).expect("parsed");
    assert_eq!(parsed.event, NotifyEvent::CodexTurnComplete);
    assert_eq!(parsed.session_id.as_deref(), Some("thread-1"));
    assert_eq!(
        parsed.first_prompt_title.as_deref(),
        Some("Add status reducer tests")
    );
}
