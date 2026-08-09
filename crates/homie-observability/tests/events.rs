use homie_observability::{EventFilter, EventName, SafeEvent, SafeFields};
use serde_json::json;

#[test]
fn event_filter_matches_session_and_kind_like_diri_eventbus() {
    let session_status = SafeEvent::new(
        EventName::SessionStatus,
        9,
        Some("s_a".to_string()),
        SafeFields::project(&json!({
            "session.id": "s_a",
            "session.status": "needs_input"
        }))
        .expect("safe fields"),
    );
    let session_output = SafeEvent::new(
        EventName::SessionOutput,
        10,
        Some("s_b".to_string()),
        SafeFields::default(),
    );
    let worktree = SafeEvent::new(EventName::WorktreeCreated, 11, None, SafeFields::default());

    let only_session_a = EventFilter::new()
        .with_session("s_a")
        .with_kind(EventName::SessionStatus);

    assert!(session_status.visible_to(&only_session_a));
    assert!(!session_output.visible_to(&only_session_a));
    assert!(!worktree.visible_to(&only_session_a));
}

#[test]
fn events_dropped_marker_is_visible_through_every_filter_and_uses_seq_zero() {
    let marker = SafeEvent::events_dropped(6, 1, 6).expect("drop marker");
    let narrow = EventFilter::new()
        .with_session("s_a")
        .with_kind(EventName::SessionStatus);

    assert_eq!(marker.name(), EventName::EventsDropped);
    assert_eq!(marker.seq(), 0);
    assert!(marker.visible_to(&narrow));
    assert_eq!(marker.fields().get("event.dropped"), Some(&json!(6)));
    assert_eq!(marker.fields().get("event.from_seq"), Some(&json!(1)));
    assert_eq!(marker.fields().get("event.to_seq"), Some(&json!(6)));
}

#[test]
fn event_names_roundtrip_to_diri_wire_names() {
    assert_eq!(EventName::SessionUpdated.as_str(), "session.updated");
    assert_eq!(EventName::SessionResources.as_str(), "session.resources");
    assert_eq!(EventName::SessionRemoved.as_str(), "session.removed");
    assert_eq!(EventName::ProjectUpdated.as_str(), "project.updated");
    assert_eq!(EventName::SessionSpawned.as_str(), "session.spawned");
    assert_eq!(EventName::SessionStatus.as_str(), "session.status");
    assert_eq!(EventName::SessionNeedsInput.as_str(), "session.needs_input");
    assert_eq!(EventName::SessionOutput.as_str(), "session.output");
    assert_eq!(EventName::SessionArtifact.as_str(), "session.artifact");
    assert_eq!(EventName::SessionArchived.as_str(), "session.archived");
    assert_eq!(EventName::WorktreeCreated.as_str(), "worktree.created");
    assert_eq!(EventName::WorktreeRemoved.as_str(), "worktree.removed");
    assert_eq!(EventName::EventsDropped.as_str(), "events.dropped");
    assert_eq!(
        EventName::MetricsWriteFailed.as_str(),
        "metrics.write_failed"
    );
    assert_eq!(
        EventName::VerificationFunctionalCaseExecuted.as_str(),
        "verification.functional_case_executed"
    );
}
