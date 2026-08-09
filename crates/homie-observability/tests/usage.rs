use homie_observability::{UsageEvidence, UsageEvidenceError, UsageSource, UsageValueKind};
use serde_json::json;

#[test]
fn usage_evidence_projects_diri_usage_summary_fields() {
    let usage = UsageEvidence {
        provider: "claude".to_string(),
        profile_id: Some("profile-default".to_string()),
        session_id: Some("s_123".to_string()),
        model: Some("claude-sonnet".to_string()),
        input_tokens: 1000,
        output_tokens: 250,
        cache_read_tokens: 50,
        cache_write_tokens: 25,
        estimated_usd: Some(0.0125),
        billed_usd: None,
        value_kind: UsageValueKind::EstimatedApiEquivalent,
        source: UsageSource::Transcript,
        occurred_at: 1_800_000_000,
    };

    let fields = usage.to_safe_fields().expect("safe usage fields");

    assert_eq!(fields.get("usage.provider"), Some(&json!("claude")));
    assert_eq!(
        fields.get("usage.profile_id"),
        Some(&json!("profile-default"))
    );
    assert_eq!(fields.get("usage.session_id"), Some(&json!("s_123")));
    assert_eq!(fields.get("usage.model"), Some(&json!("claude-sonnet")));
    assert_eq!(fields.get("usage.input_tokens"), Some(&json!(1000)));
    assert_eq!(fields.get("usage.output_tokens"), Some(&json!(250)));
    assert_eq!(fields.get("usage.cache_read_tokens"), Some(&json!(50)));
    assert_eq!(fields.get("usage.cache_write_tokens"), Some(&json!(25)));
    assert_eq!(fields.get("usage.estimated_usd"), Some(&json!(0.0125)));
    assert_eq!(
        fields.get("usage.value_kind"),
        Some(&json!("estimated_api_equivalent"))
    );
    assert_eq!(fields.get("usage.source"), Some(&json!("transcript")));
    assert_eq!(fields.get("raw_prompt"), None);
    assert_eq!(fields.get("tool_result"), None);
}

#[test]
fn usage_evidence_rejects_negative_tokens() {
    let usage = UsageEvidence {
        provider: "codex".to_string(),
        profile_id: None,
        session_id: None,
        model: None,
        input_tokens: -1,
        output_tokens: 0,
        cache_read_tokens: 0,
        cache_write_tokens: 0,
        estimated_usd: None,
        billed_usd: None,
        value_kind: UsageValueKind::EstimatedApiEquivalent,
        source: UsageSource::Transcript,
        occurred_at: 1_800_000_000,
    };

    assert_eq!(
        usage.to_safe_fields().expect_err("negative token rejected"),
        UsageEvidenceError::NegativeTokens
    );
}

#[test]
fn usage_evidence_rejects_non_finite_or_negative_cost() {
    let mut usage = UsageEvidence {
        provider: "codex".to_string(),
        profile_id: None,
        session_id: None,
        model: None,
        input_tokens: 0,
        output_tokens: 0,
        cache_read_tokens: 0,
        cache_write_tokens: 0,
        estimated_usd: Some(f64::INFINITY),
        billed_usd: None,
        value_kind: UsageValueKind::EstimatedApiEquivalent,
        source: UsageSource::Transcript,
        occurred_at: 1_800_000_000,
    };

    assert_eq!(
        usage.to_safe_fields().expect_err("infinite cost rejected"),
        UsageEvidenceError::InvalidCost
    );

    usage.estimated_usd = Some(-0.01);
    assert_eq!(
        usage.to_safe_fields().expect_err("negative cost rejected"),
        UsageEvidenceError::InvalidCost
    );
}
