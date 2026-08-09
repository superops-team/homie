use homie_context::build_summary;
use homie_proto::SessionId;

#[test]
fn session_context_summary_redacts_sensitive_words() {
    let summary = build_summary(
        SessionId::from("session_1"),
        "Work",
        "Authorization Bearer abc password=secret normal",
    );
    assert_eq!(summary.session_id.as_str(), "session_1");
    assert!(summary.safe_summary.contains("[REDACTED]"));
    assert!(summary.safe_summary.contains("normal"));
    assert!(!summary.safe_summary.contains("Bearer"));
    assert!(!summary.safe_summary.contains("secret"));
}
