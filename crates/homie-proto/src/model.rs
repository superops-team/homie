//! Shared domain model types for sessions, agents, and detection.
//!
//! Ported from diri-proto's model module (which mirrors DirijorCore).

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// ---------------------------------------------------------------------------
// AgentKind — identifies what runs inside a session's PTY
// ---------------------------------------------------------------------------

/// Identifies the agent binary running in a session.
///
/// Mirrors diri's `AgentKind`, which is open-ended (adding an agent is a manifest
/// file, not a code change). The well-known ids are kept as constants.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentKind {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
}

impl AgentKind {
    pub const CLAUDE_CODE_ID: &'static str = "claude-code";
    pub const CODEX_ID: &'static str = "codex";
    pub const CURSOR_ID: &'static str = "cursor";
    pub const GEMINI_ID: &'static str = "gemini";
    pub const SHELL_ID: &'static str = "shell";
    pub const GENERIC_ID: &'static str = "generic";

    #[must_use]
    pub fn builtin(id: &'static str) -> Self {
        Self {
            id: id.to_string(),
            command: None,
        }
    }

    #[must_use]
    pub fn generic(command: impl Into<String>) -> Self {
        Self {
            id: Self::GENERIC_ID.to_string(),
            command: Some(command.into()),
        }
    }

    #[must_use]
    pub fn is_builtin(&self) -> bool {
        matches!(
            self.id.as_str(),
            Self::CLAUDE_CODE_ID
                | Self::CODEX_ID
                | Self::CURSOR_ID
                | Self::GEMINI_ID
                | Self::SHELL_ID
        )
    }

    #[must_use]
    pub fn is_shell(&self) -> bool {
        self.id == Self::SHELL_ID
    }

    #[must_use]
    pub fn display_id(&self) -> &str {
        &self.id
    }
}

// ---------------------------------------------------------------------------
// AttentionLevel — how much the user should notice this session
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttentionLevel {
    None,
    NeedsInput,
    DoneUnseen,
    Blocked,
}

impl AttentionLevel {
    #[must_use]
    pub fn is_attention_demanding(self) -> bool {
        matches!(self, Self::NeedsInput | Self::DoneUnseen | Self::Blocked)
    }
}

// ---------------------------------------------------------------------------
// SessionRecord — the full state of one session as seen by the client
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionRecord {
    pub id: String,
    pub agent_kind: AgentKind,
    pub status: crate::SessionStatus,
    pub attention: AttentionLevel,
    pub title: String,
    pub cwd: String,
    #[serde(default)]
    pub branch: Option<String>,
    pub cols: u16,
    pub rows: u16,
    #[serde(default)]
    pub pid: Option<u32>,
    #[serde(default)]
    pub exit_code: Option<i32>,
    #[serde(default)]
    pub created_at: Option<f64>,
    #[serde(default)]
    pub last_output_at: Option<f64>,
    #[serde(default)]
    pub needs_input_destructive: bool,
    #[serde(default)]
    pub is_archived: bool,
    #[serde(default)]
    pub is_hibernated: bool,
    #[serde(default)]
    pub subagent_of: Option<String>,
    #[serde(default)]
    pub project_name: Option<String>,
}

impl SessionRecord {
    #[must_use]
    pub fn is_active(&self) -> bool {
        !self.is_archived && !self.is_hibernated
    }

    #[must_use]
    pub fn is_running(&self) -> bool {
        matches!(
            self.status,
            crate::SessionStatus::Running | crate::SessionStatus::Starting
        )
    }

    #[must_use]
    pub fn needs_attention(&self) -> bool {
        self.attention.is_attention_demanding()
    }
}

// ---------------------------------------------------------------------------
// AgentDescriptor — what the daemon tells the client about each supported agent
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentDescriptor {
    pub id: String,
    pub display_name: String,
    pub short_label: String,
    pub binary: String,
    pub status_authority: StatusAuthority,
    #[serde(default)]
    pub first_class: bool,
    #[serde(default)]
    pub resume: Option<ResumeSpec>,
    #[serde(default)]
    pub approve: Option<AgentKeystroke>,
    #[serde(default)]
    pub deny: Option<AgentKeystroke>,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    #[serde(default)]
    pub env_scrub_prefixes: Vec<String>,
    #[serde(default)]
    pub colour: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StatusAuthority {
    Process,
    Screen,
    Hooks,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResumeSpec {
    pub token: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentKeystroke {
    #[serde(default)]
    pub text: String,
    pub submit: bool,
}

// ---------------------------------------------------------------------------
// Detection types — status detection rules from agent manifests
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectionRule {
    pub pattern: String,
    pub status: String,
    #[serde(default)]
    pub attention: Option<String>,
    #[serde(default)]
    pub destructive: bool,
    #[serde(default)]
    pub subagent: bool,
    #[serde(default)]
    pub title_template: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectionManifest {
    pub id: String,
    #[serde(default)]
    pub rules: Vec<DetectionRule>,
    #[serde(default)]
    pub blockers: Vec<String>,
    #[serde(default)]
    pub startup_grace_secs: f64,
    #[serde(default)]
    pub stale_secs: f64,
    #[serde(default)]
    pub anti_flicker_secs: f64,
}

// ---------------------------------------------------------------------------
// Screen snapshot — a full terminal grid sent from daemon to client
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScreenSnapshot {
    pub session_id: String,
    pub cols: u16,
    pub rows_count: u16,
    /// RLE-encoded rows as base64 strings (each decodes to a RowRle).
    #[serde(default)]
    pub rle_rows: Vec<String>,
    #[serde(default)]
    pub cursor_x: u16,
    #[serde(default)]
    pub cursor_y: u16,
    #[serde(default)]
    pub cursor_visible: bool,
    #[serde(default)]
    pub scrollback_total: u32,
}

// ---------------------------------------------------------------------------
// Session event — broadcast from daemon to all clients
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionUpdatedEvent {
    pub session_id: String,
    #[serde(default)]
    pub status: Option<crate::SessionStatus>,
    #[serde(default)]
    pub attention: Option<AttentionLevel>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub pid: Option<u32>,
    #[serde(default)]
    pub exit_code: Option<i32>,
    #[serde(default)]
    pub needs_input_destructive: Option<bool>,
}

// ---------------------------------------------------------------------------
// Spawn request
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpawnParams {
    pub agent_id: String,
    #[serde(default)]
    pub cwd: Option<String>,
    #[serde(default)]
    pub command: Option<String>,
    #[serde(default)]
    pub cols: Option<u16>,
    #[serde(default)]
    pub rows: Option<u16>,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    #[serde(default)]
    pub title: Option<String>,
}

// ---------------------------------------------------------------------------
// Runtime daemon wire DTOs
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionSummary {
    pub id: String,
    pub title: String,
    pub status: String,
    pub workspace: String,
    pub agent_profile_id: String,
    pub runtime_id: String,
    pub llm_profile_id: String,
    pub permission_profile_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StateSnapshot {
    pub sessions: Vec<SessionSummary>,
    pub event_cursor: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeEvent {
    pub seq: u64,
    pub event: String,
    pub session_id: Option<String>,
    pub status: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeScreenObservation {
    pub state: String,
    pub matched_rule_id: String,
    pub content_seq: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionStatusReport {
    pub status: crate::SessionStatus,
    pub needs_input: Option<crate::NeedsInputDetail>,
    pub turn_completed: bool,
    pub screen_lines: Vec<String>,
    pub screen_observation: Option<RuntimeScreenObservation>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HolderSnapshot {
    pub pid: Option<u32>,
    pub status: Option<String>,
    pub tree_size: Option<usize>,
    pub cols: Option<u16>,
    pub rows: Option<u16>,
    pub log_offset: Option<u64>,
    pub epoch_offset: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionSnapshot {
    pub session: SessionSummary,
    pub status: SessionStatusReport,
    pub output_offset: u64,
    pub output_text: String,
    pub holder: Option<HolderSnapshot>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactKind {
    PullRequest,
    Preview,
    Link,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionArtifact {
    pub kind: ArtifactKind,
    pub url: String,
    pub label: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListeningPort {
    pub port: u16,
    pub url: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactScan {
    pub artifacts: Vec<SessionArtifact>,
    pub ports: Vec<ListeningPort>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PortListRow {
    pub port: u16,
    pub url: String,
    pub session_id: String,
    pub session_title: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionStatusRequest {
    pub session_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionArtifactsRequest {
    pub session_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionPortsRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionSnapshotRequest {
    pub session_id: String,
    #[serde(default)]
    pub output_offset: u64,
    pub max_bytes: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionSetParentRequest {
    pub session_id: String,
    pub parent_session_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionChildrenRequest {
    pub parent_session_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionParentRequest {
    pub session_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionParentResult {
    pub parent_session_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HookReportRequest {
    pub session_id: String,
    pub event: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub needs_input: Option<crate::NeedsInputDetail>,
    #[serde(default)]
    pub turn_completed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorktreeOverviewEntry {
    pub project_root: String,
    pub path: String,
    pub branch: Option<String>,
    pub session_id: Option<String>,
    pub session_status: Option<String>,
    pub dirty: bool,
    pub merged: bool,
    pub age_days: i64,
    pub stale_suggestion: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorktreeOverviewResult {
    pub entries: Vec<WorktreeOverviewEntry>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScannedHistoryEntry {
    pub agent_kind: String,
    pub external_id: String,
    pub cwd: String,
    pub title: Option<String>,
    pub title_source: String,
    pub transcript_path: String,
    pub last_active_at: i64,
    pub created_at: Option<i64>,
    pub cwd_exists: bool,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agent_kind_serde() {
        let kind = AgentKind::builtin(AgentKind::CLAUDE_CODE_ID);
        let json = serde_json::to_string(&kind).expect("serialize");
        let decoded: AgentKind = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded.id, AgentKind::CLAUDE_CODE_ID);
    }

    #[test]
    fn session_record_serde() {
        let record = SessionRecord {
            id: "s1".to_string(),
            agent_kind: AgentKind::builtin(AgentKind::CODEX_ID),
            status: crate::SessionStatus::Running,
            attention: AttentionLevel::None,
            title: "test session".to_string(),
            cwd: "/tmp".to_string(),
            branch: Some("main".to_string()),
            cols: 80,
            rows: 24,
            pid: Some(12345),
            exit_code: None,
            created_at: Some(1700000000.0),
            last_output_at: Some(1700000001.0),
            needs_input_destructive: false,
            is_archived: false,
            is_hibernated: false,
            subagent_of: None,
            project_name: Some("test".to_string()),
        };
        let json = serde_json::to_string(&record).expect("serialize");
        let decoded: SessionRecord = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded.id, "s1");
        assert_eq!(decoded.status, crate::SessionStatus::Running);
    }

    #[test]
    fn attention_level_is_demanding() {
        assert!(AttentionLevel::NeedsInput.is_attention_demanding());
        assert!(AttentionLevel::DoneUnseen.is_attention_demanding());
        assert!(!AttentionLevel::None.is_attention_demanding());
    }
}
