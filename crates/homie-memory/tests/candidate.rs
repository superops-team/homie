use homie_memory::{MemoryCandidateStatus, MemoryError, write_candidate};

#[test]
fn memory_candidate_requires_source_event() {
    assert_eq!(
        write_candidate("mem_1", "", "safe fact").unwrap_err(),
        MemoryError::MissingSource
    );
}

#[test]
fn memory_candidate_rejects_sensitive_content() {
    assert_eq!(
        write_candidate("mem_1", "event_1", "Authorization Bearer abc").unwrap_err(),
        MemoryError::UnsafeContent
    );
}

#[test]
fn memory_candidate_accepts_safe_sourced_content() {
    let candidate = write_candidate("mem_1", "event_1", "safe fact").expect("candidate");
    assert_eq!(candidate.source_event_id, "event_1");
    assert_eq!(candidate.status, MemoryCandidateStatus::Created);
}
