use homie_proto::SessionId;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SessionContextSummary {
    pub session_id: SessionId,
    pub title: String,
    pub safe_summary: String,
}

pub fn build_summary(
    session_id: SessionId,
    title: impl Into<String>,
    content: impl AsRef<str>,
) -> SessionContextSummary {
    SessionContextSummary {
        session_id,
        title: title.into(),
        safe_summary: redact_sensitive(content.as_ref()),
    }
}

fn redact_sensitive(input: &str) -> String {
    input
        .split_whitespace()
        .map(|word| {
            let lower = word.to_ascii_lowercase();
            if lower.contains("authorization")
                || lower.contains("bearer")
                || lower.contains("password")
                || lower.contains("secret")
                || lower.contains("token=")
            {
                "[REDACTED]"
            } else {
                word
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
