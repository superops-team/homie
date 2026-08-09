use std::fs;

use homie_llm::{UsageProviderKind, parse_transcript_usage_events};
use serde_json::json;

#[test]
fn parses_claude_usage_events() {
    let temp = tempfile::tempdir().expect("tempdir");
    let transcript = temp.path().join("claude.jsonl");
    write_lines(
        &transcript,
        &[
            json!({"type":"user","timestamp":"2026-07-22T10:00:00Z","message":{"content":"ignored"}}),
            json!({
                "type": "assistant",
                "timestamp": "2026-07-22T10:12:00Z",
                "requestId": "request-1",
                "sessionId": "session-claude",
                "message": {
                    "id": "message-1",
                    "model": "claude-sonnet",
                    "usage": {
                        "input_tokens": 1_000_000,
                        "output_tokens": 1_000,
                        "cache_read_input_tokens": 2_000,
                        "cache_creation_input_tokens": 3_000,
                        "cache_creation": {
                            "ephemeral_5m_input_tokens": 2_500,
                            "ephemeral_1h_input_tokens": 500
                        }
                    }
                }
            }),
        ],
    );

    let events =
        parse_transcript_usage_events(&transcript, UsageProviderKind::Claude, Some("work"))
            .expect("parse");
    assert_eq!(events.len(), 1);
    let event = &events[0];
    assert_eq!(event.provider, UsageProviderKind::Claude);
    assert_eq!(event.profile_id.as_deref(), Some("work"));
    assert_eq!(event.session_id.as_deref(), Some("session-claude"));
    assert_eq!(event.model.as_deref(), Some("claude-sonnet"));
    assert_eq!(event.input_tokens, 1_000_000);
    assert_eq!(event.output_tokens, 1_000);
    assert_eq!(event.cache_read_tokens, 2_000);
    assert_eq!(event.cache_write_tokens, 3_000);
    assert_eq!(event.cache_write_5m_tokens, 2_500);
    assert_eq!(event.cache_write_1h_tokens, 500);
    assert!(event.source_event_id.starts_with("transcript:"));
    assert!(event.estimated_cost.expect("cost") > 3.0);
}

#[test]
fn parses_codex_usage_events_with_model_context() {
    let temp = tempfile::tempdir().expect("tempdir");
    let transcript = temp.path().join("codex.jsonl");
    write_lines(
        &transcript,
        &[
            json!({"type":"session_meta","payload":{"model":"gpt-5.4-mini"}}),
            json!({
                "type": "event_msg",
                "timestamp": "2026-07-22T11:10:00Z",
                "payload": {
                    "type": "token_count",
                    "thread_id": "thread-1",
                    "info": {
                        "last_token_usage": {
                            "input_tokens": 1_000,
                            "cached_input_tokens": 400,
                            "output_tokens": 100
                        }
                    }
                }
            }),
        ],
    );

    let events =
        parse_transcript_usage_events(&transcript, UsageProviderKind::Codex, None).expect("parse");
    assert_eq!(events.len(), 1);
    let event = &events[0];
    assert_eq!(event.provider, UsageProviderKind::Codex);
    assert_eq!(event.session_id.as_deref(), Some("thread-1"));
    assert_eq!(event.model.as_deref(), Some("gpt-5.4-mini"));
    assert_eq!(event.input_tokens, 1_000);
    assert_eq!(event.output_tokens, 100);
    assert_eq!(event.cache_read_tokens, 400);
    assert_eq!(event.cache_write_tokens, 0);
    assert!(event.estimated_cost.expect("cost") > 0.0);
}

#[test]
fn bad_unknown_and_negative_inputs_are_safe() {
    let temp = tempfile::tempdir().expect("tempdir");
    let transcript = temp.path().join("mixed.jsonl");
    fs::write(
        &transcript,
        format!(
            "{}\nnot-json\n{}\n",
            json!({"type":"event_msg","timestamp":"2026-07-22T11:10:00Z","payload":{"type":"token_count","thread_id":"thread-negative","info":{"last_token_usage":{"input_tokens":-10,"cached_input_tokens":-5,"output_tokens":-2}}}}),
            json!({"type":"event_msg","timestamp":"2026-07-22T11:11:00Z","payload":{"type":"token_count","thread_id":"thread-unknown","info":{"last_token_usage":{"input_tokens":9,"cached_input_tokens":0,"output_tokens":1}}}})
        ),
    )
    .expect("write transcript");

    let first = parse_transcript_usage_events(&transcript, UsageProviderKind::Codex, None)
        .expect("parse first");
    let second = parse_transcript_usage_events(&transcript, UsageProviderKind::Codex, None)
        .expect("parse second");
    assert_eq!(first.len(), 2);
    assert_eq!(first[0].input_tokens, 0);
    assert_eq!(first[0].output_tokens, 0);
    assert_eq!(first[0].cache_read_tokens, 0);
    assert_eq!(first[0].estimated_cost, None);
    assert_eq!(first[1].estimated_cost, None);
    assert_eq!(first[0].source_event_id, second[0].source_event_id);
    assert_eq!(first[1].source_event_id, second[1].source_event_id);
}

fn write_lines(path: &std::path::Path, values: &[serde_json::Value]) {
    let mut output = String::new();
    for value in values {
        output.push_str(&value.to_string());
        output.push('\n');
    }
    fs::write(path, output).expect("write transcript");
}
