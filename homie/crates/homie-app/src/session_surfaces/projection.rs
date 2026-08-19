//! Pure projection helpers mapping store/protocol types onto UI vocabulary.

use super::*;

pub(crate) fn switcher_key(event: &KeyDownEvent) -> SwitcherKey {
    match event.keystroke.key.as_str() {
        "tab" => SwitcherKey::Tab {
            control: event.keystroke.modifiers.control,
            shift: event.keystroke.modifiers.shift,
        },
        "escape" => SwitcherKey::Escape,
        "enter" => SwitcherKey::Enter,
        "left" => SwitcherKey::ArrowLeft,
        "right" => SwitcherKey::ArrowRight,
        "up" => SwitcherKey::ArrowUp,
        "down" => SwitcherKey::ArrowDown,
        _ => SwitcherKey::Other,
    }
}

pub(super) fn ui_agent_kind(kind: &ProtoAgentKind) -> AgentKind {
    // Brand vocabulary, not a protocol type: a manifest agent the client has
    // no hand-drawn mark for falls back to the generic terminal treatment.
    match kind.id() {
        ProtoAgentKind::CLAUDE_CODE_ID => AgentKind::ClaudeCode,
        ProtoAgentKind::CODEX_ID => AgentKind::Codex,
        ProtoAgentKind::CURSOR_ID => AgentKind::Cursor,
        ProtoAgentKind::GEMINI_ID => AgentKind::Gemini,
        ProtoAgentKind::SHELL_ID => AgentKind::Shell,
        _ => AgentKind::Generic,
    }
}

pub(super) fn ui_status_state(session: &SessionRecord) -> StatusState {
    if session.hibernation.is_some() {
        return StatusState::Hibernated;
    }
    match session.attention() {
        AttentionLevel::Working => StatusState::Working,
        AttentionLevel::NeedsInput => StatusState::NeedsInput {
            destructive: session
                .needs_input
                .as_ref()
                .is_some_and(|detail| detail.risk_hint == RiskHint::Destructive),
        },
        AttentionLevel::DoneUnseen => StatusState::DoneUnseen,
        AttentionLevel::IdleSeen => StatusState::IdleSeen,
        AttentionLevel::None | AttentionLevel::Unknown => StatusState::None,
    }
}

pub(super) fn status_color(session: &SessionRecord, colors: SemanticColors) -> gpui::Rgba {
    match ui_status_state(session) {
        StatusState::Working => Ink::working(ui_agent_kind(session.effective_kind()), colors),
        StatusState::NeedsInput { destructive: true } => Ink::DANGER,
        StatusState::NeedsInput { destructive: false } => Ink::ATTENTION,
        StatusState::DoneUnseen => Ink::FRESH,
        StatusState::IdleSeen | StatusState::None | StatusState::Hibernated => colors.secondary,
    }
}

pub(super) fn state_badge(session: &SessionRecord) -> String {
    if session.hibernation.is_some() {
        "asleep".to_owned()
    } else if matches!(session.status, SessionStatus::Exited(_)) {
        "ended".to_owned()
    } else if let Some(bytes) = session
        .memory_bytes
        .filter(|bytes| *bytes > 2 * 1_073_741_824)
    {
        format!("{:.1} GB", bytes as f64 / 1_073_741_824.0)
    } else {
        session
            .git_branch
            .as_deref()
            .map(clamp_branch)
            .unwrap_or_default()
    }
}

pub(super) fn state_badge_color(session: &SessionRecord, colors: SemanticColors) -> gpui::Rgba {
    if session
        .hibernation
        .as_ref()
        .is_some_and(|info| info.reason == homie_proto::HibernationReason::MemoryPressure)
        || session
            .memory_bytes
            .is_some_and(|bytes| bytes > 6 * 1_073_741_824)
    {
        Ink::ATTENTION
    } else {
        colors.tertiary
    }
}

fn clamp_branch(branch: &str) -> String {
    let characters: Vec<_> = branch.chars().collect();
    if characters.len() <= 18 {
        branch.to_owned()
    } else {
        format!(
            "{}…{}",
            characters[..8].iter().collect::<String>(),
            characters[characters.len() - 8..]
                .iter()
                .collect::<String>()
        )
    }
}
