//! The versioned on-disk snapshot and the projection folds that turn a live
//! session view into a stored record.

use homie_proto::{SessionRecord, TitleSource};
use serde::{Deserialize, Serialize};

use crate::session::SessionView;

/// The versioned on-disk snapshot.
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct PersistedState {
    pub version: i64,
    #[serde(default)]
    pub projects: Vec<serde_json::Value>,
    #[serde(default)]
    pub sessions: Vec<SessionRecord>,
}

impl PersistedState {
    pub(crate) fn current(sessions: Vec<SessionRecord>, projects: Vec<serde_json::Value>) -> Self {
        Self {
            version: 1,
            projects,
            sessions,
        }
    }
}

pub(crate) fn fold_session_view(record: &mut SessionRecord, view: &SessionView) {
    fold_session_status(record, view);
    if record.kind == homie_proto::AgentKind::SHELL
        || matches!(
            record.title_source,
            TitleSource::AgentProvided | TitleSource::HomieAssigned | TitleSource::UserRename
        )
    {
        return;
    }
    let Some(title) = view
        .title
        .as_deref()
        .and_then(normalize_agent_title)
        .filter(|title| !is_generic_terminal_title(title, record))
    else {
        return;
    };
    record.title = title;
    record.title_source = view.title_source.unwrap_or(TitleSource::AgentProvided);
}

/// Removes terminal-brand decorations accidentally persisted as conversation
/// titles by older builds. User and Homie-assigned names are intentionally
/// untouched; only titles attributed to the Agent/PTY are safe to repair.
pub(crate) fn repair_persisted_agent_title(record: &mut SessionRecord) -> bool {
    if record.title_source != TitleSource::AgentProvided {
        return false;
    }
    match normalize_agent_title(&record.title)
        .filter(|title| !is_generic_terminal_title(title, record))
    {
        Some(title) if title != record.title => {
            record.title = title;
            true
        }
        Some(_) => false,
        None => {
            record.title = record.kind.id().to_owned();
            record.title_source = TitleSource::Placeholder;
            true
        }
    }
}

pub(crate) fn fold_session_status(record: &mut SessionRecord, view: &SessionView) {
    record.status.clone_from(&view.status);
    record.needs_input.clone_from(&view.needs_input);
}

pub(crate) fn normalize_agent_title(title: &str) -> Option<String> {
    let line = title.lines().map(str::trim).find(|line| !line.is_empty())?;
    let line = line.trim_start_matches(|character: char| {
        character.is_whitespace() || (!character.is_alphanumeric() && character != '_')
    });
    let normalized = line
        .chars()
        .filter(|character| !character.is_control())
        .take(160)
        .collect::<String>();
    let normalized = normalized.trim();
    (!normalized.is_empty()).then(|| normalized.to_owned())
}

fn is_generic_terminal_title(title: &str, record: &SessionRecord) -> bool {
    let title = title.trim().to_ascii_lowercase();
    let compact_title = title
        .chars()
        .filter(|character| character.is_alphanumeric())
        .collect::<String>();
    let cwd = record.cwd.trim_end_matches('/').to_ascii_lowercase();
    let directory = cwd.rsplit('/').next().unwrap_or(&cwd);
    title == cwd
        || title == directory
        || matches!(
            compact_title.as_str(),
            "claude" | "claudecode" | "codex" | "cursor" | "gemini" | "terminal" | "shell"
        )
}
