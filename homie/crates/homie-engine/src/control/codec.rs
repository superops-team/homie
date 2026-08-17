//! Proto↔domain projections for the control channel.
//!
//! These map engine domain types onto the shared `homie-proto` wire types the
//! client speaks. They are pure functions with no registry, session, or socket
//! dependency, so they stay unit-testable without a running daemon.

use homie_proto::{AgentKind, DateMillis};

/// Projects one resumable conversation onto its wire form.
pub(super) fn history_entry_to_wire(
    entry: crate::history::HistoryEntry,
) -> homie_proto::HistoryEntry {
    homie_proto::HistoryEntry {
        id: entry.id,
        kind: match entry.kind {
            crate::history::HistoryKind::ClaudeCode => AgentKind::CLAUDE_CODE,
            crate::history::HistoryKind::Codex => AgentKind::CODEX,
        },
        cwd: entry.cwd,
        title: entry.title,
        transcript_path: entry.transcript_path,
        last_active_at: DateMillis::from(entry.last_active_at),
        created_at: entry.created_at.map(DateMillis::from),
        cwd_exists: entry.cwd_exists,
    }
}

/// Projects one `git worktree list` entry onto its wire form.
pub(super) fn worktree_to_wire(info: crate::git::WorktreeInfo) -> homie_proto::WorktreeInfo {
    homie_proto::WorktreeInfo {
        path: info.path,
        branch: info.branch,
        is_bare: info.is_bare,
        is_detached: info.is_detached,
        is_prunable: info.is_prunable,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, UNIX_EPOCH};

    fn entry(kind: crate::history::HistoryKind) -> crate::history::HistoryEntry {
        crate::history::HistoryEntry {
            id: "abc".into(),
            kind,
            cwd: "/w".into(),
            title: Some("Title".into()),
            transcript_path: "/t.jsonl".into(),
            last_active_at: UNIX_EPOCH + Duration::from_millis(1_700_000_000_000),
            created_at: Some(UNIX_EPOCH + Duration::from_millis(1_700_000_000_123)),
            cwd_exists: false,
        }
    }

    #[test]
    fn history_projection_maps_kind() {
        let claude = history_entry_to_wire(entry(crate::history::HistoryKind::ClaudeCode));
        assert_eq!(claude.kind, AgentKind::CLAUDE_CODE);
        let codex = history_entry_to_wire(entry(crate::history::HistoryKind::Codex));
        assert_eq!(codex.kind, AgentKind::CODEX);
    }

    #[test]
    fn history_projection_preserves_scalars() {
        let wire = history_entry_to_wire(entry(crate::history::HistoryKind::Codex));
        assert_eq!(wire.id, "abc");
        assert_eq!(wire.cwd, "/w");
        assert_eq!(wire.title.as_deref(), Some("Title"));
        assert_eq!(wire.transcript_path, "/t.jsonl");
        assert!(!wire.cwd_exists);
    }

    #[test]
    fn history_projection_converts_timestamps_to_millis() {
        let wire = history_entry_to_wire(entry(crate::history::HistoryKind::Codex));
        assert_eq!(wire.last_active_at, DateMillis(1_700_000_000_000.0));
        assert_eq!(wire.created_at, Some(DateMillis(1_700_000_000_123.0)));
    }

    #[test]
    fn history_projection_preserves_missing_created_at() {
        let mut e = entry(crate::history::HistoryKind::Codex);
        e.created_at = None;
        e.title = None;
        let wire = history_entry_to_wire(e);
        assert_eq!(wire.created_at, None);
        assert_eq!(wire.title, None);
    }

    #[test]
    fn worktree_projection_preserves_fields() {
        let wire = worktree_to_wire(crate::git::WorktreeInfo {
            path: "/repo".into(),
            branch: Some("main".into()),
            is_bare: false,
            is_detached: true,
            is_prunable: false,
        });
        assert_eq!(wire.path, "/repo");
        assert_eq!(wire.branch.as_deref(), Some("main"));
        assert!(!wire.is_bare);
        assert!(wire.is_detached);
        assert!(!wire.is_prunable);
    }

    #[test]
    fn worktree_projection_preserves_bare_and_missing_branch() {
        let wire = worktree_to_wire(crate::git::WorktreeInfo {
            path: "/bare".into(),
            branch: None,
            is_bare: true,
            is_detached: false,
            is_prunable: true,
        });
        assert!(wire.is_bare);
        assert!(wire.is_prunable);
        assert!(!wire.is_detached);
        assert_eq!(wire.branch, None);
    }
}
