use homie_observability::{EventName, MetricsWriteFailure};
use serde_json::json;

#[test]
fn metrics_write_failure_projects_to_safe_event_without_changing_business_result() {
    let business_result: Result<&str, &str> = Ok("llm-response-delivered");
    let failure = MetricsWriteFailure {
        metrics_kind: "llm.usage".to_string(),
        metrics_scope: "session:s_123".to_string(),
        component: "homie-llm".to_string(),
        operation: "record_usage".to_string(),
        safe_error_code: "sqlite_busy".to_string(),
        retryable: true,
        occurred_at: 1_800_000_000,
    };

    let event = failure.to_event(12).expect("safe metrics event");

    assert_eq!(business_result, Ok("llm-response-delivered"));
    assert_eq!(event.name(), EventName::MetricsWriteFailed);
    assert_eq!(event.seq(), 12);
    assert_eq!(
        event.fields().get("metrics.kind"),
        Some(&json!("llm.usage"))
    );
    assert_eq!(
        event.fields().get("metrics.scope"),
        Some(&json!("session:s_123"))
    );
    assert_eq!(event.fields().get("component"), Some(&json!("homie-llm")));
    assert_eq!(
        event.fields().get("operation"),
        Some(&json!("record_usage"))
    );
    assert_eq!(
        event.fields().get("safe_error_code"),
        Some(&json!("sqlite_busy"))
    );
    assert_eq!(event.fields().get("retryable"), Some(&json!(true)));
    assert_eq!(event.fields().get("raw_request"), None);
    assert_eq!(event.fields().get("authorization"), None);
}
