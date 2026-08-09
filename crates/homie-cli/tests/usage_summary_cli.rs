use homie_storage::{CreateSession, RecordUsage, StorageConfig, open_or_create};
use serde_json::Value;
use std::process::Command;

#[test]
fn summarizes_usage_records_from_storage() {
    let temp = tempfile::tempdir().expect("tempdir");
    let session = seed_usage(temp.path());

    let output = homie_json([
        "usage",
        "summary",
        "--data-dir",
        temp.path().to_str().unwrap(),
        "--session-id",
        &session.id,
        "--provider-id",
        "provider_local_placeholder",
        "--model",
        "gpt-4o-mini",
        "--json",
    ]);

    assert_eq!(output["events"], 2);
    assert_eq!(output["inputTokens"], 150);
    assert_eq!(output["outputTokens"], 30);
    assert_eq!(output["cacheReadTokens"], 14);
    assert_eq!(output["cacheWriteTokens"], 6);
    assert_eq!(output["totalTokens"], 200);
    assert_eq!(output["authoritativeBillingAvailable"], true);
}

#[test]
fn reports_empty_usage_summary() {
    let temp = tempfile::tempdir().expect("tempdir");
    let output = homie_json([
        "usage",
        "summary",
        "--data-dir",
        temp.path().to_str().unwrap(),
        "--json",
    ]);
    assert_eq!(output["events"], 0);
    assert_eq!(output["totalTokens"], 0);
}

fn seed_usage(data_dir: &std::path::Path) -> homie_storage::SessionSummary {
    let storage = open_or_create(StorageConfig {
        data_dir: data_dir.to_path_buf(),
    })
    .expect("storage");
    storage.migrate().expect("migrate");
    storage.seed_defaults().expect("defaults");
    let session = storage
        .create_session(CreateSession {
            workspace: data_dir.to_path_buf(),
            title: Some("Usage".to_string()),
        })
        .expect("session");
    storage
        .record_usage(usage(&session, "req-1", "event-1", 100, 20, 10, 5))
        .expect("usage 1");
    storage
        .record_usage(usage(&session, "req-2", "event-2", 50, 10, 4, 1))
        .expect("usage 2");
    session
}

fn usage(
    session: &homie_storage::SessionSummary,
    request_id: &str,
    source_event_id: &str,
    input_tokens: i64,
    output_tokens: i64,
    cache_read_tokens: i64,
    cache_write_tokens: i64,
) -> RecordUsage {
    RecordUsage {
        request_id: request_id.to_string(),
        session_id: Some(session.id.clone()),
        agent_profile_id: session.agent_profile_id.clone(),
        runtime_id: session.runtime_id.clone(),
        provider_id: "provider_local_placeholder".to_string(),
        llm_profile_id: session.llm_profile_id.clone(),
        model: "gpt-4o-mini".to_string(),
        request_kind: "chat".to_string(),
        status: "ok".to_string(),
        input_tokens,
        output_tokens,
        cached_input_tokens: 0,
        cache_read_tokens,
        cache_write_tokens,
        cache_write_5m_tokens: 0,
        cache_write_1h_tokens: 1,
        reasoning_tokens: 0,
        unit_price_input: Some("1.0".to_string()),
        unit_price_output: Some("2.0".to_string()),
        currency: Some("USD".to_string()),
        pricing_snapshot_id: None,
        estimated_cost: Some("0.0001".to_string()),
        billed_cost: Some("0.00011".to_string()),
        first_token_latency_ms: Some(20),
        total_latency_ms: Some(200),
        started_at: 200,
        completed_at: 201,
        safe_error_code: None,
        value_kind: "authoritative_billed".to_string(),
        source: "transcript".to_string(),
        source_event_id: source_event_id.to_string(),
    }
}

fn homie_json<const N: usize>(args: [&str; N]) -> Value {
    let output = Command::new(env!("CARGO_BIN_EXE_homie"))
        .args(args)
        .output()
        .expect("homie");
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("json")
}
