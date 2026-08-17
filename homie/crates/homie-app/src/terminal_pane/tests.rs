use gpui::{Image, ImageFormat, KeyDownEvent, Keystroke, Modifiers, TestAppContext};
use homie_proto::{
    ArtifactKind, DateMillis, ExitInfo, ExitReason, NeedsInputDetail, NeedsInputKind,
    NeedsInputSource, PrCheck, PullRequestStatus, RiskHint, SessionArtifact, SessionListResult,
};
use homie_ui::StatusState;

use super::*;

fn sorted_checks(pr: &PullRequestStatus) -> Vec<PrCheck> {
    let mut checks = pr.checks.clone().unwrap_or_default();
    checks.sort_by_key(|check| match check.result.as_str() {
        "fail" => 0,
        "pending" => 1,
        "pass" => 2,
        _ => 3,
    });
    checks
}

/// Replays a drag as the render loop sees it -- a geometry change every
/// `frame`, for `frames` frames -- and returns when each size reached the
/// daemon. Mirrors `update_selected_geometry`: `Arm`/`Fold` hold the size,
/// and an armed tick fires on the cadence carrying the newest one.
fn simulate_drag(frames: u32, frame: Duration) -> Vec<Duration> {
    let mut sent = Vec::new();
    let mut last_sent: Option<Duration> = None;
    let mut armed_at: Option<Duration> = None;
    let mut now = Duration::ZERO;
    for tick in 0..frames {
        now += frame;
        // The armed tick fires on its own, independent of the frame.
        if let Some(at) = armed_at
            && now >= at
        {
            sent.push(at);
            last_sent = Some(at);
            armed_at = None;
        }
        let since = last_sent.map(|at| now.saturating_sub(at));
        match plan_resize(tick == 0, since, armed_at.is_some()) {
            ResizePlan::SendNow => {
                sent.push(now);
                last_sent = Some(now);
            }
            ResizePlan::Arm(delay) => armed_at = Some(now + delay),
            ResizePlan::Fold => {}
        }
    }
    if let Some(at) = armed_at {
        sent.push(at);
    }
    sent
}

#[test]
fn a_live_drag_keeps_resizing_the_pty_at_the_cadence() {
    // One second of dragging at 120Hz. The trailing-edge debounce this
    // replaced sent exactly one resize here -- after the mouse stopped --
    // which is why the terminal appeared to reflow only on drop. The
    // expected count derives from the cadence so it moves with it.
    let sent = simulate_drag(120, Duration::from_millis(8));
    let expected = (1000 / RESIZE_CADENCE.as_millis()) as usize;
    assert!(
        sent.len().abs_diff(expected) <= 3,
        "expected ~{expected} resizes in a second of dragging, got {}",
        sent.len()
    );
    // Leading edge: the drag's first frame is not made to wait.
    assert_eq!(sent[0], Duration::from_millis(8));
    // And no two land closer together than the cadence.
    for pair in sent.windows(2) {
        assert!(
            pair[1].saturating_sub(pair[0]) >= RESIZE_CADENCE,
            "{pair:?} are closer than the cadence"
        );
    }
}

#[test]
fn the_size_a_drag_ends_on_always_reaches_the_daemon() {
    // Three frames then release: the last size must still go out, or the
    // pane keeps painting a grid the daemon has never been told about.
    let sent = simulate_drag(3, Duration::from_millis(8));
    assert!(sent.len() >= 2, "the release size must be sent: {sent:?}");
    let release = Duration::from_millis(3 * 8);
    assert!(
        *sent.last().expect("sent") <= release + RESIZE_CADENCE,
        "the final size lands within one cadence of release: {sent:?}"
    );
}

#[test]
fn an_isolated_resize_never_waits() {
    // A window snap or a session switch is one change after a long idle.
    assert_eq!(
        plan_resize(false, Some(Duration::from_secs(3)), false),
        ResizePlan::SendNow
    );
    assert_eq!(plan_resize(false, None, false), ResizePlan::SendNow);
    // The first measure after attach is what a deferred launch waits for.
    assert_eq!(
        plan_resize(true, Some(Duration::ZERO), true),
        ResizePlan::SendNow
    );
}

fn grid_frame(cols: u16, full: bool) -> GridUpdate {
    GridUpdate {
        cols,
        rows: 40,
        cursor_col: 0,
        cursor_row: 0,
        cursor_visible: true,
        is_full_snapshot: full,
        changed_rows: Vec::new(),
    }
}

fn reflow_hold() -> ReflowHold {
    ReflowHold {
        parked: Vec::new(),
        saw_snapshot: false,
        _release: Task::ready(()),
    }
}

#[test]
fn a_panel_toggle_holds_the_grid_but_a_drag_keeps_reflowing() {
    // ⌘B after any pause: one column change, held so the re-wrap and the
    // program's repaint land together.
    assert!(should_hold_reflow(
        (120, 40),
        (100, 40),
        Some(Duration::from_secs(3))
    ));
    // A drag steps every few frames; freezing it would stop the grid from
    // reflowing under the cursor, which is the whole point of the cadence.
    assert!(!should_hold_reflow(
        (120, 40),
        (119, 40),
        Some(Duration::from_millis(16))
    ));
}

#[test]
fn a_change_with_no_reflow_in_it_is_never_held() {
    // Rows-only: the daemon crops or extends, nothing re-wraps.
    assert!(!should_hold_reflow((120, 40), (120, 30), None));
    // The first measure after attach has nothing on screen to hold.
    assert!(!should_hold_reflow((0, 0), (120, 40), None));
}

#[test]
fn a_hold_ends_on_the_repaint_that_follows_the_re_wrap() {
    let mut hold = reflow_hold();
    // The daemon's re-wrapped snapshot: on its own this is the frame that
    // used to shove the content up, so it must not release the hold.
    assert!(!hold.park(grid_frame(100, true)));
    // The program answering SIGWINCH completes the pair.
    assert!(hold.park(grid_frame(100, false)));
    assert_eq!(hold.parked.len(), 2);
}

#[test]
fn a_re_seed_mid_hold_does_not_stand_in_for_the_repaint() {
    let mut hold = reflow_hold();
    assert!(!hold.park(grid_frame(100, true)));
    assert!(!hold.park(grid_frame(100, true)));
    assert!(hold.park(grid_frame(100, false)));
}

#[test]
fn a_repaint_arriving_before_any_snapshot_keeps_waiting() {
    // Output already in flight when the resize went out is not the answer
    // to it; releasing on it would paint the pre-reflow grid.
    let mut hold = reflow_hold();
    assert!(!hold.park(grid_frame(120, false)));
}

fn fixture_session() -> SessionRecord {
    let envelope: serde_json::Value = serde_json::from_str(include_str!(
        "../../../homie-proto/tests/fixtures/session_list_response.json"
    ))
    .unwrap();
    let list: SessionListResult = serde_json::from_value(envelope["ok"].clone()).unwrap();
    list.sessions[0].clone()
}

fn pull_request(url: &str) -> PullRequestStatus {
    PullRequestStatus {
        url: url.to_owned(),
        number: 42,
        title: Some("Keep terminal resident".to_owned()),
        author: None,
        body: None,
        base_ref_name: None,
        head_ref_name: None,
        state: "OPEN".to_owned(),
        is_draft: false,
        review_decision: Some("APPROVED".to_owned()),
        mergeable: Some("MERGEABLE".to_owned()),
        merge_state_status: Some("CLEAN".to_owned()),
        additions: 45,
        deletions: 12,
        changed_files: 3,
        comment_count: 2,
        review_count: 1,
        resolved_threads: Some(3),
        total_threads: Some(5),
        checks_passed: 3,
        checks_failed: 1,
        checks_pending: 1,
        checks: Some(vec![
            PrCheck {
                name: "build".to_owned(),
                result: "pending".to_owned(),
                detail: None,
                url: None,
            },
            PrCheck {
                name: "lint".to_owned(),
                result: "fail".to_owned(),
                detail: None,
                url: Some("https://example.com/lint".to_owned()),
            },
            PrCheck {
                name: "test".to_owned(),
                result: "pass".to_owned(),
                detail: None,
                url: None,
            },
        ]),
        discussion: None,
        fetched_at: DateMillis(1.0),
    }
}

#[test]
fn chips_follow_swift_artifact_pr_family_then_ports_order() {
    let mut session = fixture_session();
    let url = "https://github.com/homie/homie/pull/42";
    session.artifacts = Some(vec![SessionArtifact {
        kind: ArtifactKind::PullRequest,
        url: url.to_owned(),
        first_seen_at: DateMillis(1.0),
    }]);
    session.pull_requests = Some(vec![pull_request(url)]);
    session.listening_ports = Some(vec![homie_proto::PortInfo {
        port: 3000,
        process_name: "vite".to_owned(),
    }]);

    let chips = PaneChip::for_session(&session);
    assert_eq!(chips.len(), 4);
    assert_eq!(chips[0].label, "PR #42 +45 −12");
    assert_eq!(chips[0].tint, Some(ChipTint::Green));
    assert_eq!(chips[1].label, "3/5");
    assert_eq!(chips[1].tint, Some(ChipTint::Red));
    assert!(chips[1].checks.is_some());
    assert_eq!(chips[2].label, "3/5");
    assert_eq!(chips[2].tint, Some(ChipTint::Orange));
    assert_eq!(chips[3].label, ":3000");
    assert_eq!(chips[3].open_url.as_deref(), Some("http://localhost:3000"));
}

#[test]
fn toolbar_prioritizes_pr_destinations_and_collapses_low_priority_links() {
    let mut session = fixture_session();
    let first_pr = "https://github.com/homie/homie/pull/7";
    let second_pr = "https://github.com/homie/homie/pull/8";
    session.artifacts = Some(vec![
        SessionArtifact {
            kind: ArtifactKind::Link,
            url: "https://docs.example.com/reference".to_owned(),
            first_seen_at: DateMillis(1.0),
        },
        SessionArtifact {
            kind: ArtifactKind::PullRequest,
            url: first_pr.to_owned(),
            first_seen_at: DateMillis(2.0),
        },
        SessionArtifact {
            kind: ArtifactKind::Preview,
            url: "https://preview.example.com".to_owned(),
            first_seen_at: DateMillis(3.0),
        },
        SessionArtifact {
            kind: ArtifactKind::PullRequest,
            url: second_pr.to_owned(),
            first_seen_at: DateMillis(4.0),
        },
    ]);
    session.pull_requests = Some(vec![pull_request(first_pr), pull_request(second_pr)]);

    let chips = PaneChip::for_session(&session);
    assert!(chips[0].label.starts_with("PR #7"));
    assert!(chips[1].label.starts_with("PR #8"));
    assert!(
        chips
            .iter()
            .position(|chip| chip.label == "docs.example.com")
            .is_some_and(|index| index > 1)
    );
}

#[test]
fn check_popover_prioritizes_failure_then_running() {
    let checks = sorted_checks(&pull_request("https://example.com/pull/42"));
    assert_eq!(
        checks
            .iter()
            .map(|check| check.result.as_str())
            .collect::<Vec<_>>(),
        ["fail", "pending", "pass"]
    );
}

#[gpui::test]
fn an_empty_terminal_pane_keeps_the_sidebar_reveal_control(cx: &mut TestAppContext) {
    let runtime = Arc::new(StoreRuntime::inert());
    let tokio = Arc::new(
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("test runtime"),
    );

    let (pane, cx) = cx.add_window_view(move |window, cx| {
        let mut pane = TerminalPane::new(runtime, tokio, window, cx);
        pane.set_shell_chrome(false, false, cx);
        pane
    });

    assert!(
        pane.read_with(cx, |pane, _| pane.selected_session().is_none()),
        "fixture must exercise the empty terminal state"
    );
    assert!(
        cx.debug_bounds("show-sidebar").is_some(),
        "collapsing the sidebar must leave a way to reveal it"
    );
}

#[gpui::test]
fn selecting_a_newly_spawned_session_focuses_its_terminal(cx: &mut TestAppContext) {
    let runtime = Arc::new(StoreRuntime::inert());
    let tokio = Arc::new(
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("test runtime"),
    );
    let existing = fixture_session();
    {
        let mut store = runtime.store.write().expect("session store lock poisoned");
        store.upsert_session(existing.clone());
        store.select(existing.id.clone());
    }

    let runtime_for_view = Arc::clone(&runtime);
    let (pane, cx) = cx
        .add_window_view(move |window, cx| TerminalPane::new(runtime_for_view, tokio, window, cx));
    let _picker_focus = pane.update_in(cx, |pane, window, cx| {
        let picker_focus = cx.focus_handle();
        window.focus(&picker_focus, cx);
        assert!(!pane.is_focused(window));
        picker_focus
    });
    pane.update_in(cx, |pane, window, cx| {
        pane.reconcile_store_change(window, cx);
        assert!(
            !pane.is_focused(window),
            "an unrelated store update must not steal focus from the picker"
        );
    });

    let mut spawned = fixture_session();
    spawned.id = SessionId::new("spawned");
    {
        let mut store = runtime.store.write().expect("session store lock poisoned");
        store.upsert_session(spawned.clone());
        store.select(spawned.id);
    }

    // A successful spawn selects the daemon's new id asynchronously,
    // after the picker owned focus; the follow-selection pane must take
    // focus with that production store-change reconciliation.
    pane.update_in(cx, |pane, window, cx| {
        pane.reconcile_store_change(window, cx);
        assert!(pane.is_focused(window));
    });
}

#[test]
fn needs_input_glyph_preserves_destructive_risk() {
    let mut session = fixture_session();
    session.status = SessionStatus::NeedsInput(NeedsInputKind::Permission);
    session.needs_input = Some(NeedsInputDetail {
        kind: NeedsInputKind::Permission,
        source: NeedsInputSource::ClaudePermissionHook,
        tool_name: Some("Bash".to_owned()),
        summary: "Approve command".to_owned(),
        prompt_excerpt: None,
        options: None,
        risk_hint: RiskHint::Destructive,
        occurred_at: DateMillis(2.0),
    });
    assert_eq!(
        status_state(&session),
        StatusState::NeedsInput { destructive: true }
    );
}

#[test]
fn daemon_restart_exit_copy_matches_reference() {
    let mut session = fixture_session();
    session.status = SessionStatus::Exited(ExitInfo {
        reason: ExitReason::DaemonRestart,
        code: None,
        signal: None,
    });
    assert_eq!(
        exit_description(&session),
        "Session ended when the daemon restarted"
    );
}

#[test]
fn gpui_key_adapter_feeds_existing_terminal_encoder() {
    let event = KeyDownEvent {
        keystroke: Keystroke::parse("up").unwrap(),
        is_held: false,
        prefer_character_input: false,
    };
    let mapped = terminal_key_event(&event).unwrap();
    assert_eq!(
        encode_key(&mapped, TermModifiers::default(), TermInputModes::default()),
        b"\x1b[A"
    );

    let command_backspace = KeyDownEvent {
        keystroke: Keystroke {
            modifiers: Modifiers {
                platform: true,
                ..Modifiers::default()
            },
            key: "backspace".to_owned(),
            key_char: None,
        },
        is_held: false,
        prefer_character_input: false,
    };
    let mapped = terminal_key_event(&command_backspace).unwrap();
    assert_eq!(
        encode_key(
            &mapped,
            TermModifiers {
                cmd: true,
                ..TermModifiers::default()
            },
            TermInputModes::default()
        ),
        [0x15]
    );
}

#[test]
fn clipboard_image_entries_are_detected_before_text_paste() {
    let item = ClipboardItem::new_image(&Image {
        format: ImageFormat::Png,
        bytes: b"clipboard png".to_vec(),
        id: 7,
    });

    let (bytes, extension) = clipboard_image(&item).expect("image payload");
    assert_eq!(bytes, b"clipboard png");
    assert_eq!(extension, "png");
    assert_eq!(item.text(), None);
}

#[test]
fn offscreen_terminal_damage_updates_its_buffer_without_repainting_the_window() {
    let selected = SessionId::new("selected");
    let background = SessionId::new("background");

    assert!(terminal_damage_should_repaint(
        true,
        Some(&selected),
        &selected,
        true
    ));
    assert!(!terminal_damage_should_repaint(
        true,
        Some(&selected),
        &background,
        true
    ));
    assert!(!terminal_damage_should_repaint(
        true,
        Some(&selected),
        &selected,
        false
    ));
    assert!(!terminal_damage_should_repaint(
        false,
        Some(&selected),
        &selected,
        true
    ));
}

#[test]
fn protocol_grid_never_exceeds_the_columns_that_can_be_painted() {
    let metrics =
        CellMetrics::from_measurements(px(7.75), px(10.0), px(3.0), px(1.0), gpui::FontId(7));
    // A fractional-width boundary where the window estimate reports ten
    // columns, but the actual grid content box is three border pixels
    // narrower and can paint only nine.
    let reported = estimated_grid_size(101.5, 100.0, 0.0, metrics);
    let painted = metrics.cols_for_width(px(101.5
        - GRID_HORIZONTAL_PADDING
        - GRID_LAYOUT_HORIZONTAL_CHROME));

    assert!(
        reported.0 <= painted,
        "reported {} columns but only {painted} fit",
        reported.0
    );
}
