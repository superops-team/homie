//! Pure projection helpers for the inspector.
//!
//! These map domain records (PR status, session record, artifacts, diffs) into
//! display-facing strings and colors. No `Window`/`Context`/`Entity`/render
//! dependency, so they stay unit-testable in isolation.

use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use gpui::rgba;
use homie_proto::{
    AgentKind as ProtoAgentKind, ArtifactKind, PrCheck, PrDiscussionItem, PullRequestStatus,
    SessionArtifact, SessionRecord, SessionStatus,
};
use homie_ui::{AgentKind, Ink, SemanticColors};

use crate::diff::DiffLayer;
use crate::review_prompt::ReviewLayer;

pub(crate) fn sorted_pr_checks(pull_request: &PullRequestStatus) -> Vec<PrCheck> {
    let mut checks = pull_request.checks.clone().unwrap_or_default();
    checks.sort_by_key(|check| match check.result.as_str() {
        "fail" => 0,
        "pending" => 1,
        "pass" => 2,
        _ => 3,
    });
    checks
}

pub(crate) fn checks_rollup(pull_request: &PullRequestStatus) -> (String, gpui::Rgba) {
    if pull_request.checks_failed > 0 {
        return (
            format!("{} failed", pull_request.checks_failed),
            Ink::DANGER,
        );
    }
    if pull_request.checks_pending > 0 {
        return (
            format!("{} running", pull_request.checks_pending),
            Ink::ATTENTION,
        );
    }
    ("All passed".to_owned(), Ink::FRESH)
}

pub(crate) fn discussion_state(
    item: &PrDiscussionItem,
    colors: SemanticColors,
) -> (Option<String>, gpui::Rgba) {
    match item.state.as_deref() {
        Some("APPROVED") => (Some("Approved".to_owned()), Ink::FRESH),
        Some("CHANGES_REQUESTED") => (Some("Requested changes".to_owned()), Ink::DANGER),
        Some("COMMENTED") => (Some("Reviewed".to_owned()), colors.secondary),
        Some(state) => (Some(humanize_github_state(state)), colors.secondary),
        None => (None, colors.secondary),
    }
}

pub(crate) fn pull_request_can_merge(pull_request: &PullRequestStatus) -> bool {
    pull_request.state == "OPEN"
        && !pull_request.is_draft
        && pull_request.mergeable.as_deref() != Some("CONFLICTING")
        && pull_request.checks_failed == 0
        && pull_request.checks_pending == 0
        && !matches!(
            pull_request.review_decision.as_deref(),
            Some("CHANGES_REQUESTED") | Some("REVIEW_REQUIRED")
        )
        && !matches!(
            pull_request.merge_state_status.as_deref(),
            Some("BLOCKED") | Some("DIRTY") | Some("DRAFT")
        )
}

pub(crate) fn merge_blocker_label(pull_request: &PullRequestStatus) -> &'static str {
    if pull_request.checks_failed > 0 {
        "Checks are failing"
    } else if pull_request.checks_pending > 0 {
        "Checks are still running"
    } else if pull_request.mergeable.as_deref() == Some("CONFLICTING") {
        "Resolve merge conflicts"
    } else if pull_request.review_decision.as_deref() == Some("CHANGES_REQUESTED") {
        "Changes were requested"
    } else if pull_request.review_decision.as_deref() == Some("REVIEW_REQUIRED") {
        "Review is required"
    } else {
        "GitHub is blocking the merge"
    }
}

pub(crate) fn humanize_github_state(value: &str) -> String {
    let lower = value.replace('_', " ").to_ascii_lowercase();
    let mut chars = lower.chars();
    chars
        .next()
        .map(|first| first.to_uppercase().collect::<String>() + chars.as_str())
        .unwrap_or_default()
}

pub(crate) fn artifact_count(session: &SessionRecord) -> usize {
    let artifacts = session.artifacts.as_deref().unwrap_or_default();
    let visible_artifacts = artifacts
        .iter()
        .filter(|artifact| artifact_visible(artifact))
        .count();
    let ports = session.listening_ports.as_deref().unwrap_or_default();
    let status_only_pull_requests = session
        .pull_requests
        .as_deref()
        .unwrap_or_default()
        .iter()
        .filter(|status| {
            !artifacts.iter().any(|artifact| {
                artifact.kind == ArtifactKind::PullRequest && artifact.url == status.url
            })
        })
        .count();
    visible_artifacts + ports.len() + status_only_pull_requests
}

pub(crate) fn artifact_visible(artifact: &SessionArtifact) -> bool {
    !matches!(artifact.kind, ArtifactKind::Link | ArtifactKind::Unknown)
}

pub(crate) fn ui_agent_kind(kind: &ProtoAgentKind) -> AgentKind {
    match kind.id() {
        ProtoAgentKind::CLAUDE_CODE_ID => AgentKind::ClaudeCode,
        ProtoAgentKind::CODEX_ID => AgentKind::Codex,
        ProtoAgentKind::CURSOR_ID => AgentKind::Cursor,
        ProtoAgentKind::GEMINI_ID => AgentKind::Gemini,
        ProtoAgentKind::SHELL_ID => AgentKind::Shell,
        _ => AgentKind::Generic,
    }
}

pub(crate) fn session_status(
    session: &SessionRecord,
    colors: SemanticColors,
) -> (&'static str, gpui::Rgba) {
    if session.hibernation.is_some() {
        return ("Sleeping", colors.secondary);
    }
    match session.status {
        SessionStatus::Starting => (
            "Starting",
            Ink::working(ui_agent_kind(session.effective_kind()), colors),
        ),
        SessionStatus::Working => (
            "Working",
            Ink::working(ui_agent_kind(session.effective_kind()), colors),
        ),
        SessionStatus::NeedsInput(_) => {
            let destructive = session
                .needs_input
                .as_ref()
                .is_some_and(|detail| detail.risk_hint == homie_proto::RiskHint::Destructive);
            (
                "Needs input",
                if destructive {
                    Ink::DANGER
                } else {
                    Ink::ATTENTION
                },
            )
        }
        SessionStatus::Idle if session.attention() == homie_proto::AttentionLevel::DoneUnseen => {
            ("Finished", Ink::FRESH)
        }
        SessionStatus::Idle => ("Idle", colors.secondary),
        SessionStatus::Exited(_) => ("Ended", colors.tertiary),
        SessionStatus::Unknown => ("Unknown", colors.tertiary),
    }
}

pub(crate) fn pull_request_state(
    pull_request: &PullRequestStatus,
    colors: SemanticColors,
) -> (&'static str, gpui::Rgba) {
    if pull_request.state == "MERGED" {
        return ("Merged", rgba(0xaf7cf7ff));
    }
    if pull_request.state == "CLOSED" {
        return ("Closed", Ink::DANGER);
    }
    if pull_request.is_draft {
        return ("Draft", colors.secondary);
    }
    if pull_request.mergeable.as_deref() == Some("CONFLICTING") {
        return ("Conflicts", Ink::DANGER);
    }
    match pull_request.review_decision.as_deref() {
        Some("APPROVED") => ("Approved", Ink::FRESH),
        Some("CHANGES_REQUESTED") => ("Needs work", Ink::DANGER),
        Some("REVIEW_REQUIRED") => ("Review needed", Ink::ATTENTION),
        _ => ("Open", colors.secondary),
    }
}

pub(crate) fn pull_request_discussion(pull_request: &PullRequestStatus) -> Option<String> {
    let mut parts = Vec::new();
    if pull_request.comment_count > 0 {
        parts.push(format!(
            "{} {}",
            pull_request.comment_count,
            if pull_request.comment_count == 1 {
                "comment"
            } else {
                "comments"
            }
        ));
    }
    if pull_request.review_count > 0 {
        parts.push(format!(
            "{} {}",
            pull_request.review_count,
            if pull_request.review_count == 1 {
                "review"
            } else {
                "reviews"
            }
        ));
    }
    if let Some(total) = pull_request.total_threads.filter(|total| *total > 0) {
        parts.push(format!(
            "{} of {total} threads resolved",
            pull_request.resolved_threads.unwrap_or(0)
        ));
    }
    (!parts.is_empty()).then(|| parts.join(" · "))
}

pub(crate) fn artifact_title(artifact: &SessionArtifact) -> String {
    match artifact.kind {
        ArtifactKind::PullRequest => pr_number(&artifact.url)
            .map(|number| format!("PR #{number}"))
            .unwrap_or_else(|| "Pull request".to_owned()),
        ArtifactKind::LinearIssue => {
            linear_key(&artifact.url).unwrap_or_else(|| "Linear issue".to_owned())
        }
        ArtifactKind::Preview => url_authority(&artifact.url),
        ArtifactKind::Link | ArtifactKind::Unknown => url_authority(&artifact.url),
    }
}

pub(crate) fn pr_number(url: &str) -> Option<String> {
    let parts = url
        .split('/')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    if let Some(index) = parts.iter().position(|part| *part == "pull") {
        return parts
            .get(index + 1)
            .map(|part| part.chars().take_while(char::is_ascii_digit).collect())
            .filter(|part: &String| !part.is_empty());
    }
    parts
        .last()
        .filter(|part| part.chars().all(|character| character.is_ascii_digit()))
        .map(|part| (*part).to_owned())
}

pub(crate) fn linear_key(url: &str) -> Option<String> {
    let parts = url.split('/').collect::<Vec<_>>();
    let index = parts.iter().position(|part| *part == "issue")?;
    parts.get(index + 1).map(|part| (*part).to_owned())
}

pub(crate) fn url_authority(url: &str) -> String {
    url.split_once("://")
        .map_or(url, |(_, remainder)| remainder)
        .split('/')
        .next()
        .filter(|authority| !authority.is_empty())
        .unwrap_or(url)
        .to_owned()
}

pub(crate) fn folder_name(path: &str) -> String {
    PathBuf::from(path)
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .unwrap_or(path)
        .to_owned()
}

pub(crate) fn format_bytes(bytes: u64) -> String {
    const GIB: f64 = 1_073_741_824.0;
    const MIB: f64 = 1_048_576.0;
    if bytes >= 1_073_741_824 {
        format!("{:.1} GB", bytes as f64 / GIB)
    } else {
        format!("{:.0} MB", bytes as f64 / MIB)
    }
}

pub(crate) fn relative_time(milliseconds: f64) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0.0, |duration| duration.as_secs_f64() * 1000.0);
    let seconds = ((now - milliseconds).max(0.0) / 1000.0) as u64;
    match seconds {
        0..=59 => "now".to_owned(),
        60..=3_599 => format!("{}m ago", seconds / 60),
        3_600..=86_399 => format!("{}h ago", seconds / 3_600),
        _ => format!("{}d ago", seconds / 86_400),
    }
}

pub(crate) fn prompt_layer(layer: DiffLayer) -> ReviewLayer {
    match layer {
        DiffLayer::Branch => ReviewLayer::Branch,
        DiffLayer::Staged => ReviewLayer::Staged,
        DiffLayer::Working => ReviewLayer::Working,
    }
}

pub(crate) fn patch_creates_file(patch: &[u8]) -> bool {
    patch
        .windows(b"--- /dev/null".len())
        .any(|window| window == b"--- /dev/null")
}
