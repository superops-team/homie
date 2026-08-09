use homie_ui::{
    NotificationActionKind, NotificationSession, NotificationSeverity, macos_notification_command,
    notification_rollup, redact_notification_text,
};

#[test]
fn notification_rollup_counts_status_and_exposes_quick_actions() {
    let rollup = notification_rollup(&[
        NotificationSession {
            id: "s1".to_string(),
            title: "Codex".to_string(),
            status: "needs_input".to_string(),
            needs_input: true,
            destructive: false,
            agent_has_approve_deny: true,
        },
        NotificationSession {
            id: "s2".to_string(),
            title: "Shell".to_string(),
            status: "running".to_string(),
            needs_input: false,
            destructive: false,
            agent_has_approve_deny: false,
        },
        NotificationSession {
            id: "s3".to_string(),
            title: "Done".to_string(),
            status: "exited".to_string(),
            needs_input: false,
            destructive: false,
            agent_has_approve_deny: false,
        },
    ]);

    assert_eq!(rollup.total, 3);
    assert_eq!(rollup.needs_input, 1);
    assert_eq!(rollup.running, 1);
    assert_eq!(rollup.exited, 1);
    assert_eq!(rollup.badge(), "1 need input");
    let first = &rollup.items[0];
    assert_eq!(first.severity, NotificationSeverity::Attention);
    assert!(
        first
            .actions
            .iter()
            .any(|action| action.kind == NotificationActionKind::Approve)
    );
    assert!(
        first
            .actions
            .iter()
            .any(|action| action.kind == NotificationActionKind::Deny)
    );
}

#[test]
fn notification_rollup_suppresses_quick_actions_for_unknown_agents() {
    let rollup = notification_rollup(&[NotificationSession {
        id: "s1".to_string(),
        title: "Unknown".to_string(),
        status: "needs_input".to_string(),
        needs_input: true,
        destructive: true,
        agent_has_approve_deny: false,
    }]);

    assert_eq!(rollup.items[0].severity, NotificationSeverity::Critical);
    assert_eq!(rollup.items[0].actions.len(), 1);
    assert_eq!(
        rollup.items[0].actions[0].kind,
        NotificationActionKind::OpenSession
    );
}

#[test]
fn macos_notification_command_escapes_and_redacts_body() {
    let mut rollup = notification_rollup(&[NotificationSession {
        id: "s1".to_string(),
        title: "Codex \"approval\"".to_string(),
        status: "needs_input".to_string(),
        needs_input: true,
        destructive: false,
        agent_has_approve_deny: true,
    }]);
    rollup.items[0].body = "Authorization: Bearer secret".to_string();

    let command = macos_notification_command(&rollup.items[0]);
    assert_eq!(command[0], "/usr/bin/osascript");
    assert_eq!(command[1], "-e");
    assert!(command[2].contains("display notification"));
    assert!(command[2].contains("[redacted]"));
    assert!(!command[2].contains("secret"));
    assert!(command[2].contains("\\\"approval\\\""));
}

#[test]
fn notification_redaction_handles_inline_tokens() {
    assert_eq!(
        redact_notification_text("fetch token=abc123"),
        "fetch token=[redacted]"
    );
    assert_eq!(
        redact_notification_text("cookie=session"),
        "cookie=[redacted]"
    );
}
