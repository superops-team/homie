use std::time::{Duration, SystemTime, UNIX_EPOCH};

use homie_agents::{
    Authority, ClaudeHook, ManifestState, ReducerTiming, ScreenObservation, StatusReducer,
    StatusSignal,
};
use homie_proto::{NeedsInputKind, SessionStatus};

fn t0() -> SystemTime {
    UNIX_EPOCH + Duration::from_secs(1_700_000_000)
}

fn observation(state: ManifestState, seq: u64) -> ScreenObservation {
    ScreenObservation {
        state,
        matched_rule_id: "test".into(),
        priority: 100,
        content_seq: seq,
        prompt_excerpt: None,
        options: None,
    }
}

fn blocker(seq: u64, excerpt: &str) -> ScreenObservation {
    ScreenObservation {
        state: ManifestState::BlockedPermission,
        matched_rule_id: "permission".into(),
        priority: 1000,
        content_seq: seq,
        prompt_excerpt: Some(excerpt.into()),
        options: Some(vec!["Yes".into(), "No".into()]),
    }
}

fn hook(hook: ClaudeHook) -> StatusSignal {
    StatusSignal::ClaudeHook {
        hook,
        is_subagent: false,
    }
}

#[test]
fn status_reducer_claude_turn_completes_once() {
    let mut reducer = StatusReducer::new(Authority::HooksPrimary, t0());
    let now = t0() + Duration::from_secs(5);

    reducer.reduce(hook(ClaudeHook::UserPromptSubmit), now);
    assert_eq!(*reducer.status(), SessionStatus::Running);

    let outcome = reducer.reduce(hook(ClaudeHook::Stop), now + Duration::from_millis(100));
    let outcome = if outcome.status_change.is_none() {
        reducer.reduce(StatusSignal::Tick, now + Duration::from_millis(200))
    } else {
        outcome
    };
    assert_eq!(outcome.status_change, Some(SessionStatus::Idle));
    assert!(outcome.turn_completed);

    let again = reducer.reduce(hook(ClaudeHook::Stop), now + Duration::from_millis(300));
    assert!(!again.turn_completed);
}

#[test]
fn status_reducer_screen_idle_uses_anti_flicker() {
    let mut reducer = StatusReducer::new(Authority::ScreenPrimary, t0());
    let mut now = t0() + Duration::from_secs(5);

    reducer.reduce(
        StatusSignal::Screen(observation(ManifestState::Working, 1)),
        now,
    );
    assert_eq!(*reducer.status(), SessionStatus::Running);

    for seq in 2..=3 {
        now += Duration::from_millis(100);
        reducer.reduce(
            StatusSignal::Screen(observation(ManifestState::Idle, seq)),
            now,
        );
        assert_eq!(*reducer.status(), SessionStatus::Running);
    }

    now += Duration::from_millis(100);
    let outcome = reducer.reduce(
        StatusSignal::Screen(observation(ManifestState::Idle, 4)),
        now,
    );
    assert_eq!(outcome.status_change, Some(SessionStatus::Idle));
}

#[test]
fn status_reducer_visible_blocker_produces_needs_input_detail() {
    let mut reducer = StatusReducer::new(Authority::HooksPrimary, t0());
    let now = t0() + Duration::from_secs(5);

    reducer.reduce(hook(ClaudeHook::UserPromptSubmit), now);
    let outcome = reducer.reduce(
        StatusSignal::Screen(blocker(1, "Do you want to proceed?")),
        now + Duration::from_millis(50),
    );

    assert_eq!(outcome.status_change, Some(SessionStatus::NeedsInput));
    let detail = outcome.needs_input.expect("needs-input detail");
    assert_eq!(detail.kind, NeedsInputKind::Approval);
    assert_eq!(detail.summary, "Do you want to proceed?");
    assert_eq!(
        detail.options.as_deref(),
        Some(&["Yes".to_string(), "No".to_string()][..])
    );
}

#[test]
fn status_reducer_subagent_events_do_not_move_parent() {
    let mut reducer = StatusReducer::new(Authority::HooksPrimary, t0());
    let now = t0() + Duration::from_secs(5);
    reducer.reduce(hook(ClaudeHook::UserPromptSubmit), now);

    let outcome = reducer.reduce(
        StatusSignal::ClaudeHook {
            hook: ClaudeHook::Stop,
            is_subagent: true,
        },
        now + Duration::from_millis(100),
    );
    assert_eq!(outcome.status_change, None);
    assert_eq!(*reducer.status(), SessionStatus::Running);
}

#[test]
fn status_reducer_process_only_output_then_exit() {
    let timing = ReducerTiming {
        staleness_timeout: Duration::from_secs(10),
        ..ReducerTiming::default()
    };
    let mut reducer = StatusReducer::new(Authority::ProcessOnly, t0()).with_timing(timing);
    let now = t0() + Duration::from_secs(1);

    let outcome = reducer.reduce(StatusSignal::PtyOutputActivity, now);
    assert_eq!(outcome.status_change, Some(SessionStatus::Running));

    let outcome = reducer.reduce(
        StatusSignal::Screen(observation(ManifestState::Idle, 1)),
        now + Duration::from_secs(1),
    );
    assert_eq!(outcome.status_change, None);

    let outcome = reducer.reduce(
        StatusSignal::ProcessExit {
            code: Some(0),
            signal: None,
        },
        now + Duration::from_secs(2),
    );
    assert_eq!(outcome.status_change, Some(SessionStatus::Exited));
}
