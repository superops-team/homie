use homie_agents::{ManifestEngine, ManifestState, ScreenSnapshot};

#[test]
fn claude_golden_screens_match_diri_detection() {
    let engine = load_engine();

    let idle = snapshot([
        "Done. Anything else?",
        "╭────────────────────────────────────────────╮",
        "│ ❯                                          │",
        "╰────────────────────────────────────────────╯",
    ]);
    let obs = engine.evaluate(&idle, "claude-code").expect("claude idle");
    assert_eq!(obs.state, ManifestState::Idle);
    assert_eq!(obs.matched_rule_id, "idle-prompt-box");

    let permission = snapshot([
        "╭────────────────────────────────────────────╮",
        "│ Bash command                               │",
        "│                                            │",
        "│ rm -rf build                               │",
        "│                                            │",
        "│ Do you want to proceed?                    │",
        "│ ❯ 1. Yes                                   │",
        "│   2. No, and tell Claude what to do (esc)  │",
        "╰────────────────────────────────────────────╯",
        "esc to cancel",
    ]);
    let obs = engine
        .evaluate(&permission, "claude-code")
        .expect("claude permission");
    assert_eq!(obs.state, ManifestState::BlockedPermission);
    assert_eq!(
        obs.options.as_deref(),
        Some(
            &[
                "Yes".to_string(),
                "No, and tell Claude what to do (esc)".to_string()
            ][..]
        )
    );
    assert!(
        obs.prompt_excerpt
            .as_deref()
            .is_some_and(|excerpt| excerpt.contains("rm -rf build"))
    );

    let mut working = snapshot(["thinking..."]);
    working.osc_title = Some("⠹ Waddling...".into());
    let obs = engine
        .evaluate(&working, "claude-code")
        .expect("claude working");
    assert_eq!(obs.state, ManifestState::Working);
    assert_eq!(obs.matched_rule_id, "working-spinner");

    let transcript = snapshot([
        "Showing detailed transcript · ctrl+r to toggle",
        "╭────────────╮",
        "│ ❯          │",
        "╰────────────╯",
    ]);
    let obs = engine
        .evaluate(&transcript, "claude-code")
        .expect("claude transcript");
    assert_eq!(obs.state, ManifestState::Skip);
    assert_eq!(obs.matched_rule_id, "transcript-viewer");
}

#[test]
fn codex_golden_screens_match_diri_detection() {
    let engine = load_engine();

    let mut action_required = snapshot(["running command...", "npm install"]);
    action_required.osc_title = Some("● Action Required".into());
    let obs = engine
        .evaluate(&action_required, "codex")
        .expect("codex action required");
    assert_eq!(obs.state, ManifestState::BlockedPermission);
    assert_eq!(obs.matched_rule_id, "action-required-title");
    assert!(
        obs.prompt_excerpt
            .as_deref()
            .is_some_and(|excerpt| excerpt.contains("npm install"))
    );

    let confirm = snapshot([
        "╭─ Allow command? ─────────────╮",
        "│ npm install                  │",
        "│ ❯ 1. Yes                     │",
        "│   2. No                      │",
        "╰──────────────────────────────╯",
        "Press enter to confirm or esc to cancel",
    ]);
    let obs = engine.evaluate(&confirm, "codex").expect("codex confirm");
    assert_eq!(obs.state, ManifestState::BlockedPermission);
    assert_eq!(
        obs.options.as_deref(),
        Some(&["Yes".to_string(), "No".to_string()][..])
    );

    let idle = snapshot([
        "╭──────────────────────────────╮",
        "│ › Ask Codex to do something  │",
        "╰──────────────────────────────╯",
    ]);
    let obs = engine.evaluate(&idle, "codex").expect("codex idle");
    assert_eq!(obs.state, ManifestState::Idle);
    assert_eq!(obs.matched_rule_id, "idle-prompt-box");
}

#[test]
fn cursor_golden_screens_match_diri_detection() {
    let engine = load_engine();

    let confirm = snapshot([
        "╭──────────────────────────────╮",
        "│ Run this command?            │",
        "│ npm install                  │",
        "│ Run (y)   Reject (esc/n)     │",
        "╰──────────────────────────────╯",
    ]);
    let obs = engine.evaluate(&confirm, "cursor").expect("cursor confirm");
    assert_eq!(obs.state, ManifestState::BlockedPermission);
    assert_eq!(obs.matched_rule_id, "confirm-dialog");
    assert!(
        obs.prompt_excerpt
            .as_deref()
            .is_some_and(|excerpt| excerpt.contains("npm install"))
    );

    let working = snapshot(["some earlier output", "Generating"]);
    let obs = engine.evaluate(&working, "cursor").expect("cursor working");
    assert_eq!(obs.state, ManifestState::Working);
    assert_eq!(obs.matched_rule_id, "working-status-line");

    let idle = snapshot([
        "╭──────────────────────────────╮",
        "│ → Add a follow-up            │",
        "╰──────────────────────────────╯",
    ]);
    let obs = engine.evaluate(&idle, "cursor").expect("cursor idle");
    assert_eq!(obs.state, ManifestState::Idle);
}

#[test]
fn gemini_golden_screens_match_diri_detection() {
    let engine = load_engine();

    let confirm = snapshot([
        "╭──────────────────────────────────────╮",
        "│ Apply this change?                   │",
        "│ ● 1. Yes, allow once                 │",
        "│   2. Yes, allow always               │",
        "│   3. No, suggest changes (esc)       │",
        "╰──────────────────────────────────────╯",
    ]);
    let obs = engine.evaluate(&confirm, "gemini").expect("gemini confirm");
    assert_eq!(obs.state, ManifestState::BlockedPermission);
    assert_eq!(obs.matched_rule_id, "confirm-dialog");

    let working = snapshot(["⠹ Polishing the code (esc to cancel, 12s)"]);
    let obs = engine.evaluate(&working, "gemini").expect("gemini working");
    assert_eq!(obs.state, ManifestState::Working);
    assert_eq!(obs.matched_rule_id, "working-cancel-timer");

    let idle = snapshot([
        "╭──────────────────────────────────────╮",
        "│ >   Type your message or @path/to/file │",
        "╰──────────────────────────────────────╯",
    ]);
    let obs = engine.evaluate(&idle, "gemini").expect("gemini idle");
    assert_eq!(obs.state, ManifestState::Idle);
}

#[test]
fn unknown_manifest_returns_none() {
    let engine = load_engine();
    assert!(engine.evaluate(&snapshot(["x"]), "nope").is_none());
}

fn load_engine() -> ManifestEngine {
    let (engine, failed) =
        ManifestEngine::load_dir(&manifest_dir()).expect("load bundled manifests");
    assert!(failed.is_empty(), "failed manifests: {failed:?}");
    engine
}

fn snapshot<const N: usize>(lines: [&str; N]) -> ScreenSnapshot {
    ScreenSnapshot {
        lines: lines.into_iter().map(str::to_string).collect(),
        content_seq: 1,
        ..Default::default()
    }
}

fn manifest_dir() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("assets")
        .join("agent-descriptors")
}
