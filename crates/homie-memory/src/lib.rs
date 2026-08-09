use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MemoryCandidate {
    pub id: String,
    pub source_event_id: String,
    pub content: String,
    pub status: MemoryCandidateStatus,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum MemoryCandidateStatus {
    Created,
    Rejected,
    Approved,
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum MemoryError {
    #[error("memory candidate requires a source event")]
    MissingSource,
    #[error("memory candidate content failed redaction policy")]
    UnsafeContent,
}

pub fn write_candidate(
    id: impl Into<String>,
    source_event_id: impl Into<String>,
    content: impl Into<String>,
) -> Result<MemoryCandidate, MemoryError> {
    let source_event_id = source_event_id.into();
    if source_event_id.trim().is_empty() {
        return Err(MemoryError::MissingSource);
    }
    let content = content.into();
    if contains_sensitive_marker(&content) {
        return Err(MemoryError::UnsafeContent);
    }
    Ok(MemoryCandidate {
        id: id.into(),
        source_event_id,
        content,
        status: MemoryCandidateStatus::Created,
    })
}

fn contains_sensitive_marker(content: &str) -> bool {
    let lower = content.to_ascii_lowercase();
    lower.contains("authorization")
        || lower.contains("bearer ")
        || lower.contains("password=")
        || lower.contains("provider_key")
}
