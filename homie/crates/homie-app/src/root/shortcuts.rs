use gpui::Modifiers;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NewSessionShortcut {
    Default,
    Shell,
    Codex,
}

/// The session-creation shortcut policy, kept separate from dispatch so it can
/// be regression-tested without constructing the full daemon-backed root view.
pub(crate) fn new_session_shortcut(key: &str, modifiers: Modifiers) -> Option<NewSessionShortcut> {
    if !modifiers.platform {
        return None;
    }
    match key {
        "t" if modifiers.alt => Some(NewSessionShortcut::Shell),
        "t" if !modifiers.shift => Some(NewSessionShortcut::Default),
        "n" if modifiers.shift => Some(NewSessionShortcut::Codex),
        _ => None,
    }
}

/// Session navigation owns only its explicit shortcut. Returning `None` leaves
/// arrow keys available to the focused text field or terminal.
pub(crate) fn session_navigation_delta(
    key: &str,
    modifiers: Modifiers,
    arrow_surface_visible: bool,
) -> Option<isize> {
    if !modifiers.platform || arrow_surface_visible {
        return None;
    }
    match key {
        "up" | "left" if modifiers.alt => Some(-1),
        "down" | "right" if modifiers.alt => Some(1),
        "[" | "{" => Some(-1),
        "]" | "}" => Some(1),
        _ => None,
    }
}
