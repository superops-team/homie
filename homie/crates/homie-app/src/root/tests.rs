use gpui::Modifiers;

use super::*;

fn command_modifiers() -> Modifiers {
    Modifiers {
        platform: true,
        ..Modifiers::default()
    }
}

#[test]
fn command_t_launches_the_configured_default_agent() {
    assert_eq!(
        new_session_shortcut("t", command_modifiers()),
        Some(NewSessionShortcut::Default)
    );
}

#[test]
fn session_navigation_requires_command_option_arrows() {
    let command = command_modifiers();
    assert_eq!(session_navigation_delta("left", command, false), None);

    let command_option = Modifiers {
        alt: true,
        ..command
    };
    assert_eq!(
        session_navigation_delta("left", command_option, false),
        Some(-1)
    );
    assert_eq!(
        session_navigation_delta("right", command_option, false),
        Some(1)
    );
}
