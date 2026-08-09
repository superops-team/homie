use homie_observability::{CommandEvidence, EventName, GateStatus, SafeFields};
use serde_json::json;
use std::path::PathBuf;

#[test]
fn evidence_statuses_preserve_not_run_and_blocked() {
    assert_eq!(GateStatus::NotRun.as_str(), "not_run");
    assert_eq!(GateStatus::Blocked.as_str(), "blocked");
    assert_ne!(GateStatus::NotRun, GateStatus::Pass);
    assert_ne!(GateStatus::Blocked, GateStatus::Pass);
}

#[test]
fn command_evidence_emits_safe_functional_case_event() {
    let evidence = CommandEvidence {
        command: "cargo test --manifest-path crates/homie-observability/Cargo.toml".to_string(),
        exit_code: Some(0),
        status: GateStatus::Pass,
        output_summary: "focused tests passed".to_string(),
        evidence_path: PathBuf::from("docs/verification/diri-observability/report.md"),
        fields: SafeFields::project(&json!({
            "evidence.case_id": "FC-OBS-005",
            "evidence.source": "cargo-test"
        }))
        .expect("safe fields"),
    };

    let event = evidence
        .functional_case_event("FC-OBS-005", 12)
        .expect("safe evidence event");

    assert_eq!(event.name(), EventName::VerificationFunctionalCaseExecuted);
    assert_eq!(event.seq(), 12);
    assert_eq!(
        event.fields().get("evidence.case_id"),
        Some(&json!("FC-OBS-005"))
    );
    assert_eq!(event.fields().get("evidence.status"), Some(&json!("pass")));
    assert_eq!(
        event.fields().get("evidence.path"),
        Some(&json!("docs/verification/diri-observability/report.md"))
    );
    assert_eq!(event.fields().get("raw_response"), None);
}
