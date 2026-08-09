use homie_llm::{TranscriptUsageEvent, UsageProviderKind, UsageSourceKind, UsageValueKind};
use homie_storage::{
    CreateSession, StorageConfig, UsageImportDefaults, UsageQuery, open_or_create,
};

#[test]
fn imports_transcript_usage_events_into_storage_totals() {
    let temp = tempfile::tempdir().expect("tempdir");
    let storage = storage(temp.path());
    let session = storage
        .create_session(CreateSession {
            workspace: temp.path().to_path_buf(),
            title: Some("Usage Import".to_string()),
        })
        .expect("session");
    let defaults = UsageImportDefaults::from_session(&session);
    let events = vec![
        event(
            "transcript:1",
            UsageProviderKind::Claude,
            Some(&session.id),
            "claude-sonnet",
            100,
            20,
            10,
            5,
            3,
            2,
            Some(0.001),
        ),
        event(
            "transcript:2",
            UsageProviderKind::Codex,
            Some(&session.id),
            "codex",
            50,
            10,
            4,
            0,
            0,
            0,
            Some(0.002),
        ),
    ];

    let result = storage
        .record_transcript_usage_events(&events, &defaults)
        .expect("import");
    assert_eq!(result.inserted, 2);
    assert_eq!(result.skipped, 0);

    let totals = storage
        .query_usage_totals(UsageQuery {
            session_id: Some(session.id),
            ..UsageQuery::default()
        })
        .expect("totals");
    assert_eq!(totals.events, 2);
    assert_eq!(totals.input_tokens, 150);
    assert_eq!(totals.output_tokens, 30);
    assert_eq!(totals.cache_read_tokens, 14);
    assert_eq!(totals.cache_write_tokens, 5);
    assert_eq!(totals.cache_write_5m_tokens, 3);
    assert_eq!(totals.cache_write_1h_tokens, 2);
    assert!((totals.estimated_cost - 0.003).abs() < 1e-12);
}

#[test]
fn reimport_deduplicates_source_events() {
    let temp = tempfile::tempdir().expect("tempdir");
    let storage = storage(temp.path());
    let session = storage
        .create_session(CreateSession {
            workspace: temp.path().to_path_buf(),
            title: Some("Usage Dedupe".to_string()),
        })
        .expect("session");
    let defaults = UsageImportDefaults::from_session(&session);
    let event = event(
        "transcript:dedupe",
        UsageProviderKind::Codex,
        Some(&session.id),
        "codex",
        10,
        1,
        2,
        0,
        0,
        0,
        Some(0.0001),
    );

    let first = storage
        .record_transcript_usage_events(std::slice::from_ref(&event), &defaults)
        .expect("first import");
    let second = storage
        .record_transcript_usage_events(&[event], &defaults)
        .expect("second import");

    assert_eq!(first.inserted, 1);
    assert_eq!(first.skipped, 0);
    assert_eq!(second.inserted, 0);
    assert_eq!(second.skipped, 1);
    assert_eq!(
        storage
            .query_usage_totals(UsageQuery::default())
            .expect("totals")
            .events,
        1
    );
}

fn storage(data_dir: &std::path::Path) -> homie_storage::Storage {
    let storage = open_or_create(StorageConfig {
        data_dir: data_dir.to_path_buf(),
    })
    .expect("storage");
    storage.migrate().expect("migrate");
    storage.seed_defaults().expect("defaults");
    storage
}

#[allow(clippy::too_many_arguments)]
fn event(
    source_event_id: &str,
    provider: UsageProviderKind,
    session_id: Option<&str>,
    model: &str,
    input_tokens: i64,
    output_tokens: i64,
    cache_read_tokens: i64,
    cache_write_tokens: i64,
    cache_write_5m_tokens: i64,
    cache_write_1h_tokens: i64,
    estimated_cost: Option<f64>,
) -> TranscriptUsageEvent {
    TranscriptUsageEvent {
        source_event_id: source_event_id.to_string(),
        occurred_at: 200,
        provider,
        profile_id: Some("profile".to_string()),
        session_id: session_id.map(str::to_string),
        model: Some(model.to_string()),
        input_tokens,
        output_tokens,
        cache_read_tokens,
        cache_write_tokens,
        cache_write_5m_tokens,
        cache_write_1h_tokens,
        estimated_cost,
        billed_cost: None,
        value_kind: UsageValueKind::EstimatedApiEquivalent,
        source: UsageSourceKind::Transcript,
    }
}
