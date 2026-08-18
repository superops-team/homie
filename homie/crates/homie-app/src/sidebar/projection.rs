use super::*;

pub(crate) fn count_label(verb: &str, count: usize) -> String {
    if count == 1 {
        format!("{verb} 1 Session")
    } else {
        format!("{verb} {count} Sessions")
    }
}

pub(crate) fn display_title(session: &SessionRecord) -> String {
    if session.title_source == homie_proto::TitleSource::Placeholder {
        if matches!(
            session.status,
            homie_proto::SessionStatus::Starting
                | homie_proto::SessionStatus::Working
                | homie_proto::SessionStatus::NeedsInput(_)
        ) {
            "Untitled".into()
        } else {
            "Ended".into()
        }
    } else {
        session.title.clone()
    }
}

pub(crate) fn status_state(session: &SessionRecord, migrating: bool) -> StatusState {
    if migrating {
        return StatusState::Working;
    }
    if session.hibernation.is_some() {
        return StatusState::Hibernated;
    }
    match session.attention() {
        ProtoAttentionLevel::NeedsInput => StatusState::NeedsInput {
            destructive: session
                .needs_input
                .as_ref()
                .is_some_and(|detail| detail.risk_hint == homie_proto::RiskHint::Destructive),
        },
        ProtoAttentionLevel::DoneUnseen => StatusState::DoneUnseen,
        ProtoAttentionLevel::Working => StatusState::Working,
        ProtoAttentionLevel::IdleSeen => StatusState::IdleSeen,
        ProtoAttentionLevel::None | ProtoAttentionLevel::Unknown => StatusState::None,
    }
}

/// Rows for the new-agent picker: the hand-branded agents in their pinned
/// order, then every OTHER catalog agent whose CLI is actually installed.
///
/// Sourcing the tail from the daemon's catalog is what makes a new agent
/// manifest reachable without a client release. Gating it on `available()` is
/// what keeps the menu from becoming a nineteen-row wall of CLIs the user has
/// never installed — the four pinned rows stay visible either way because they
/// are what the app is *about*.
pub(crate) fn agent_picker_options(
    catalog: &homie_proto::AgentReadinessResult,
) -> Vec<(String, ProtoAgentKind, &'static str)> {
    let pinned = [
        ("Claude Code", ProtoAgentKind::CLAUDE_CODE, ""),
        ("Codex", ProtoAgentKind::CODEX, "⌘⇧N"),
        ("Cursor", ProtoAgentKind::CURSOR, ""),
        ("Gemini", ProtoAgentKind::GEMINI, ""),
    ];
    let mut options: Vec<(String, ProtoAgentKind, &'static str)> = pinned
        .iter()
        .map(|(title, kind, shortcut)| ((*title).to_owned(), kind.clone(), *shortcut))
        .collect();
    for item in &catalog.agents {
        if pinned.iter().any(|(_, kind, _)| kind == &item.kind) || !item.available() {
            continue;
        }
        let title = item
            .descriptor
            .as_ref()
            .map_or_else(|| item.kind.id().to_owned(), |d| d.display_name.clone());
        options.push((title, item.kind.clone(), ""));
    }
    // Terminal is last on purpose: it is the escape hatch, not an agent.
    options.push(("Terminal".to_owned(), ProtoAgentKind::SHELL, "⌥⌘T"));
    options
}

pub(crate) fn ui_agent_kind(kind: &ProtoAgentKind) -> AgentKind {
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

pub(crate) fn rollup_attention(sessions: &[Arc<SessionRecord>]) -> AttentionLevel {
    sessions
        .iter()
        .fold(AttentionLevel::None, |rollup, session| {
            let state = match status_state(session, false) {
                StatusState::NeedsInput { destructive } => {
                    AttentionLevel::NeedsInput { destructive }
                }
                StatusState::DoneUnseen => AttentionLevel::DoneUnseen,
                StatusState::Working => AttentionLevel::Working,
                StatusState::IdleSeen => AttentionLevel::IdleSeen,
                StatusState::Hibernated => AttentionLevel::Hibernated,
                StatusState::None => AttentionLevel::None,
            };
            if attention_rank(state) > attention_rank(rollup) {
                state
            } else {
                rollup
            }
        })
}

pub(crate) const fn attention_rank(level: AttentionLevel) -> u8 {
    match level {
        AttentionLevel::None | AttentionLevel::Hibernated => 0,
        AttentionLevel::IdleSeen => 1,
        AttentionLevel::Working => 2,
        AttentionLevel::DoneUnseen => 3,
        AttentionLevel::NeedsInput { .. } => 4,
    }
}

pub(crate) fn retain_live_glyphs<T>(glyphs: &mut HashMap<SessionId, T>, live: &[SessionId]) {
    let live: std::collections::HashSet<_> = live.iter().collect();
    glyphs.retain(|id, _| live.contains(id));
}

pub(crate) fn shortcut_ranks(sessions: &[Arc<SessionRecord>]) -> HashMap<SessionId, usize> {
    let session_count = sessions.len();
    sessions
        .iter()
        .enumerate()
        .filter_map(|(index, session)| {
            let shortcut = if index < 8 {
                Some(index + 1)
            } else if index + 1 == session_count {
                Some(9)
            } else {
                None
            }?;
            Some((session.id.clone(), shortcut))
        })
        .collect()
}

pub(crate) fn clamp_path(path: &str) -> String {
    if path.chars().count() <= 40 {
        return path.into();
    }
    let last = path.rsplit('/').next().unwrap_or(path);
    let head_budget = 40usize.saturating_sub(last.chars().count() + 2).max(4);
    format!(
        "{}…/{last}",
        path.chars().take(head_budget).collect::<String>()
    )
}

/// Overflow threshold for a session title. Individual badges reserve their
/// content estimate, padding, and following gap; HoverMarquee shapes the title
/// itself exactly. Rows carry a fixed disclosure column and one indent column
/// per ancestor, so nesting costs title width and has to be counted here or a
/// deep row marquees a title that was never actually clipped.
#[allow(clippy::too_many_arguments)]
pub(crate) fn session_title_available_width(
    sidebar_width: f32,
    depth: u16,
    migrating: bool,
    non_persistent: bool,
    ended: bool,
    host_label: Option<&str>,
    hibernated: bool,
    pinned: bool,
    shortcut_visible: bool,
) -> f32 {
    // Row insets + fold column + identity glyph + the gaps between them.
    let mut available = sidebar_width - 68.0 - f32::from(depth) * (Space::INDENT + 8.0);
    if migrating {
        available -= 66.0;
    }
    if non_persistent {
        available -= 72.0;
    }
    if ended {
        available -= 48.0;
    }
    if let Some(host) = host_label {
        available -= host.chars().count() as f32 * 6.2 + 18.0;
    }
    if hibernated {
        available -= 42.0;
    }
    if pinned {
        available -= 18.0;
    }
    // The close button and the shortcut hint share the trailing slot and are
    // near enough the same width that one reservation covers both.
    if shortcut_visible {
        available -= 28.0;
    }
    available.max(36.0)
}

pub(crate) fn compact_duration(seconds: i64) -> String {
    let minutes = (seconds / 60).max(0);
    if minutes >= 60 {
        format!("{}h {}m", minutes / 60, minutes % 60)
    } else {
        format!("{minutes}m")
    }
}
