//! Pure session/status/exit projections for the terminal pane.
//!
//! Map domain records (agent kind, attention/status, exit reason) into
//! display-facing UI state and copy. No `Window`/`Context`/`Entity`/render
//! dependency, so they stay unit-testable in isolation.

use homie_proto::{
    AgentKind as ProtoAgentKind, ExitReason, RiskHint, SessionRecord, SessionStatus,
};
use homie_ui::{AgentKind as UiAgentKind, StatusState};

pub(crate) fn ui_agent_kind(kind: &ProtoAgentKind) -> UiAgentKind {
    // Brand vocabulary, not a protocol type: a manifest agent the client has
    // no hand-drawn mark for falls back to the generic terminal treatment.
    match kind.id() {
        ProtoAgentKind::CLAUDE_CODE_ID => UiAgentKind::ClaudeCode,
        ProtoAgentKind::CODEX_ID => UiAgentKind::Codex,
        ProtoAgentKind::CURSOR_ID => UiAgentKind::Cursor,
        ProtoAgentKind::GEMINI_ID => UiAgentKind::Gemini,
        ProtoAgentKind::SHELL_ID => UiAgentKind::Shell,
        _ => UiAgentKind::Generic,
    }
}

pub(crate) fn status_state(session: &SessionRecord) -> StatusState {
    if session.hibernation.is_some() {
        return StatusState::Hibernated;
    }
    match session.attention() {
        homie_proto::AttentionLevel::Working => StatusState::Working,
        homie_proto::AttentionLevel::NeedsInput => StatusState::NeedsInput {
            destructive: session
                .needs_input
                .as_ref()
                .is_some_and(|detail| detail.risk_hint == RiskHint::Destructive),
        },
        homie_proto::AttentionLevel::DoneUnseen => StatusState::DoneUnseen,
        homie_proto::AttentionLevel::IdleSeen => StatusState::IdleSeen,
        homie_proto::AttentionLevel::None | homie_proto::AttentionLevel::Unknown => {
            StatusState::None
        }
    }
}

pub(crate) fn exit_description(session: &SessionRecord) -> String {
    let SessionStatus::Exited(info) = &session.status else {
        return "Session ended".to_owned();
    };
    match info.reason {
        ExitReason::DaemonRestart => "Session ended when the daemon restarted".to_owned(),
        ExitReason::Signaled => "Agent was stopped".to_owned(),
        ExitReason::Exited if info.code == Some(0) => "Agent exited".to_owned(),
        ExitReason::Exited => format!("Agent exited (code {})", info.code.unwrap_or(-1)),
        ExitReason::External => "Imported session — not started yet".to_owned(),
        ExitReason::Archived => "Archived".to_owned(),
        ExitReason::Unknown => "Session ended".to_owned(),
    }
}
