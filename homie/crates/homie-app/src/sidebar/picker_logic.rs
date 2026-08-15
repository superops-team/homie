use homie_proto::AgentKind;

pub(super) fn remote_picker_target(
    explicit_directory: Option<&str>,
    host_default: Option<&str>,
) -> String {
    explicit_directory
        .or(host_default)
        .map(normalize_remote_picker_path)
        .unwrap_or_else(|| "~".to_owned())
}

fn normalize_remote_picker_path(path: &str) -> String {
    let path = path.trim();
    if path.is_empty() {
        return "~".to_owned();
    }
    let without_trailing_slashes = path.trim_end_matches('/');
    if without_trailing_slashes.is_empty() {
        "/".to_owned()
    } else {
        without_trailing_slashes.to_owned()
    }
}

pub(super) fn should_resolve_active_repo(
    explicit_directory: Option<&str>,
    target_host: Option<&str>,
    active_host: Option<&str>,
) -> bool {
    explicit_directory.is_none() && target_host.is_none() && active_host.is_some()
}

pub(super) fn agent_picker_shortcut(
    kind: &AgentKind,
    default_kind: &AgentKind,
    fallback: &'static str,
) -> &'static str {
    if kind == default_kind {
        "⌘T"
    } else {
        fallback
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picker_logic_remote_target_prefers_explicit_directory() {
        assert_eq!(
            remote_picker_target(Some("/Users/remote/code/homie"), Some("~")),
            "/Users/remote/code/homie"
        );
    }

    #[test]
    fn picker_logic_remote_default_directory_is_normalized() {
        assert_eq!(remote_picker_target(None, Some("~/")), "~");
        assert_eq!(remote_picker_target(None, Some("/srv/app/")), "/srv/app");
        assert_eq!(remote_picker_target(None, Some("/")), "/");
        assert_eq!(remote_picker_target(None, Some("")), "~");
        assert_eq!(remote_picker_target(None, None), "~");
    }

    #[test]
    fn picker_logic_repo_resolution_only_applies_remote_to_local_without_explicit_directory() {
        assert!(!should_resolve_active_repo(None, Some("forge"), None));
        assert!(!should_resolve_active_repo(
            None,
            Some("forge"),
            Some("studio")
        ));
        assert!(should_resolve_active_repo(None, None, Some("studio")));
        assert!(!should_resolve_active_repo(
            Some("/Users/me/code"),
            None,
            Some("studio")
        ));
    }

    #[test]
    fn picker_logic_shortcut_marks_default_agent() {
        assert_eq!(
            agent_picker_shortcut(&AgentKind::CLAUDE_CODE, &AgentKind::CLAUDE_CODE, ""),
            "⌘T"
        );
        assert_eq!(
            agent_picker_shortcut(&AgentKind::CODEX, &AgentKind::CLAUDE_CODE, "⌘⇧N"),
            "⌘⇧N"
        );
        assert_eq!(
            agent_picker_shortcut(&AgentKind::SHELL, &AgentKind::CLAUDE_CODE, "⌥⌘T"),
            "⌥⌘T"
        );
    }
}
