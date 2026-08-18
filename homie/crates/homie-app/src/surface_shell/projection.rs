use super::*;

pub(crate) fn ui_agent(kind: &ProtoAgentKind) -> homie_ui::AgentKind {
    // Brand vocabulary, not a protocol type: a manifest agent the client has
    // no hand-drawn mark for falls back to the generic terminal treatment.
    match kind.id() {
        ProtoAgentKind::CLAUDE_CODE_ID => homie_ui::AgentKind::ClaudeCode,
        ProtoAgentKind::CODEX_ID => homie_ui::AgentKind::Codex,
        ProtoAgentKind::CURSOR_ID => homie_ui::AgentKind::Cursor,
        ProtoAgentKind::GEMINI_ID => homie_ui::AgentKind::Gemini,
        ProtoAgentKind::SHELL_ID => homie_ui::AgentKind::Shell,
        _ => homie_ui::AgentKind::Generic,
    }
}

pub(crate) fn ui_default_agent(agent: DefaultAgent) -> homie_ui::AgentKind {
    match agent {
        DefaultAgent::ClaudeCode => homie_ui::AgentKind::ClaudeCode,
        DefaultAgent::Codex => homie_ui::AgentKind::Codex,
        DefaultAgent::Cursor => homie_ui::AgentKind::Cursor,
        DefaultAgent::Gemini => homie_ui::AgentKind::Gemini,
    }
}

pub(crate) fn folder_name(path: &str) -> &str {
    Path::new(path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(path)
}

pub(crate) fn relative_parent(path: &str) -> String {
    let Some(parent) = Path::new(path).parent() else {
        return String::new();
    };
    let home = std::env::var_os("HOME").map(PathBuf::from);
    if home.as_deref() == Some(parent) {
        return "~".to_owned();
    }
    if let Some(home) = home
        && let Ok(relative) = parent.strip_prefix(home)
    {
        return format!("~/{}", relative.display());
    }
    parent.to_string_lossy().into_owned()
}

/// Second line under the update summary: why updates are off, or when the last
/// check ran.
pub(crate) fn update_detail(state: &crate::updates::UpdateState, unsupported: bool) -> String {
    if unsupported {
        // Verbatim reason ("not running from an app bundle", "not signed with
        // a Developer ID") so a dev build explains itself instead of looking
        // broken.
        return match &state.phase {
            UpdatePhase::Unsupported(reason) => format!("Updates off — {reason}"),
            _ => "Updates off for this build".to_owned(),
        };
    }
    let Some(checked) = state.last_checked_unix else {
        return "Not checked yet".to_owned();
    };
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or(checked);
    let seconds = now.saturating_sub(checked);
    let ago = match seconds {
        0..=59 => "just now".to_owned(),
        60..=3599 => format!("{}m ago", seconds / 60),
        3600..=86_399 => format!("{}h ago", seconds / 3600),
        _ => format!("{}d ago", seconds / 86_400),
    };
    format!("Last checked {ago}")
}

pub(crate) fn relative_time(milliseconds: f64) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0.0, |duration| duration.as_secs_f64() * 1000.0);
    let seconds = ((now - milliseconds).max(0.0) / 1000.0) as u64;
    match seconds {
        0..=59 => "now".to_owned(),
        60..=3_599 => format!("{}m", seconds / 60),
        3_600..=86_399 => format!("{}h", seconds / 3_600),
        _ => format!("{}d", seconds / 86_400),
    }
}
