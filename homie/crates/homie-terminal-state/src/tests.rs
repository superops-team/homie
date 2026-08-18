use super::*;

fn screen_with(input: &[u8]) -> HeadlessScreen {
    let mut screen = HeadlessScreen::new(80, 24);
    screen.feed(input);
    screen
}

#[test]
fn plain_output_lands_on_the_grid() {
    let screen = screen_with(b"hello world\r\nsecond line\r\n");
    assert_eq!(screen.lines(), vec!["hello world", "second line"]);
}

#[test]
fn an_erase_actually_erases() {
    // The whole point of emulating rather than grepping the byte stream:
    // text that was overwritten must not still read as present.
    let screen = screen_with(b"do you want to proceed?\r\n\x1b[2J\x1b[Hall clear\r\n");
    let text = screen.lines().join("\n");
    assert!(
        !text.contains("proceed"),
        "erased text still visible: {text:?}"
    );
    assert!(text.contains("all clear"));
}

#[test]
fn cursor_movement_overwrites_in_place() {
    let screen = screen_with(b"aaaa\rbb");
    assert_eq!(screen.lines(), vec!["bbaa"]);
}

#[test]
fn an_osc_title_is_captured() {
    let screen = screen_with(b"\x1b]0;my-session\x07ready\r\n");
    assert_eq!(screen.title(), Some("my-session"));
}

#[test]
fn osc_9_4_progress_is_scanned_out_of_the_stream() {
    // alacritty does not model this ConEmu extension, so the engine parses
    // it directly. State 1, 40 percent.
    let screen = screen_with(b"\x1b]9;4;1;40\x07working\r\n");
    assert_eq!(screen.progress(), Some((1, 40)));
    assert_eq!(screen.snapshot().osc_progress_state, Some(1));
}

#[test]
fn a_progress_sequence_split_across_reads_is_still_found() {
    let mut screen = HeadlessScreen::new(80, 24);
    screen.feed(b"\x1b]9;4;1");
    screen.feed(b";75\x07");
    assert_eq!(screen.progress(), Some((1, 75)));
}

#[test]
fn content_seq_advances_only_when_the_screen_changes() {
    let mut screen = HeadlessScreen::new(80, 24);
    screen.feed(b"hello\r\n");
    let after_first = screen.content_seq();
    assert!(after_first > 0);

    // A no-op sequence paints nothing.
    screen.feed(b"\x1b[?25l");
    assert_eq!(
        screen.content_seq(),
        after_first,
        "an invisible change must not look like new content"
    );

    screen.feed(b"world\r\n");
    assert!(screen.content_seq() > after_first);
}

#[test]
fn the_alternate_screen_is_detected() {
    let mut screen = HeadlessScreen::new(80, 24);
    assert!(!screen.is_alt_screen());
    screen.feed(b"\x1b[?1049h");
    assert!(screen.is_alt_screen(), "a pager or editor took the screen");
    screen.feed(b"\x1b[?1049l");
    assert!(!screen.is_alt_screen());
}

#[test]
fn a_resize_reflows_to_the_new_width() {
    let mut screen = HeadlessScreen::new(80, 24);
    screen.feed(b"hello\r\n");
    screen.resize(40, 10);
    assert_eq!(screen.lines(), vec!["hello"]);
}

#[test]
fn a_restored_snapshot_reproduces_the_screen_and_modes() {
    let mut original = HeadlessScreen::new(40, 10);
    original.feed(b"\x1b[?2004h\x1b[?1000h\x1b[?1006h");
    original.feed(b"\x1b[1;31mred alert\x1b[0m\r\nplain line\r\n");
    original.feed("wide: ▶ done\r\n".as_bytes());
    let snapshot = original.full_snapshot();

    let mut restored = HeadlessScreen::new(40, 10);
    assert!(
        restored.restore(&[], &snapshot, false, true, true),
        "restorable"
    );
    assert_eq!(restored.lines(), original.lines());
    assert!(restored.bracketed_paste(), "bracketed paste mode carried");
    assert!(restored.mouse_reporting(), "mouse mode carried");
    assert!(!restored.is_alt_screen());
    assert_eq!(restored.cursor(), original.cursor());
    // Attribute fidelity, not just text: the restored grid's cells match.
    assert_eq!(restored.full_snapshot().changed_rows, snapshot.changed_rows);
}

#[test]
fn a_geometry_mismatch_refuses_to_restore() {
    let mut original = HeadlessScreen::new(40, 10);
    original.feed(b"hello\r\n");
    let snapshot = original.full_snapshot();

    let mut smaller = HeadlessScreen::new(39, 10);
    assert!(
        !smaller.restore(&[], &snapshot, false, false, false),
        "a checkpoint from another geometry is a cache miss"
    );
}

/// The extracted crate forked before checkpoints carried scrollback, so
/// this travelled with the implementation. Without it, adopting a session
/// after a daemon restart silently collapses its history to one row.
#[test]
fn a_restored_checkpoint_preserves_scrollback_history() {
    let mut original = HeadlessScreen::new(12, 2);
    original.feed(b"oldest\r\nmiddle\r\nvisible\r\n");
    let history = original.history_snapshot();
    let snapshot = original.full_snapshot();
    assert_eq!(history.len(), 2, "the minimized repro has two history rows");

    let mut restored = HeadlessScreen::new(12, 2);
    assert!(
        restored.restore(&history, &snapshot, false, false, false),
        "restorable"
    );
    assert_eq!(restored.scrollback(), original.scrollback());
}

#[test]
fn a_boxed_prompt_survives_emulation_intact() {
    // What detection actually consumes, end to end.
    let mut screen = HeadlessScreen::new(80, 24);
    screen.feed("╭──────────────────────────╮\r\n".as_bytes());
    screen.feed("│ Do you want to proceed?  │\r\n".as_bytes());
    screen.feed("│ ❯ 1. Yes                 │\r\n".as_bytes());
    screen.feed("╰──────────────────────────╯\r\n".as_bytes());

    let snapshot = screen.snapshot();
    let text = snapshot.lines.join("\n");
    assert!(text.contains("Do you want to proceed?"));
    assert!(text.contains("❯ 1. Yes"), "wide glyphs survive: {text:?}");
}

#[test]
fn grid_mirror_requires_a_snapshot_and_contiguous_deltas() {
    let mut screen = HeadlessScreen::new(8, 2);
    screen.feed(b"one");
    let snapshot = screen.full_snapshot();
    let mut mirror = GridMirror::new();
    mirror
        .apply_snapshot(9, &snapshot, false, true, false)
        .expect("snapshot");
    assert_eq!(mirror.sequence(), Some(9));
    assert_eq!(mirror.size(), (8, 2));
    assert_eq!(mirror.modes(), (false, true, false));

    let mut delta_source = HeadlessScreen::new(8, 2);
    delta_source.feed(b"one");
    let _ = delta_source.grid_update(true);
    delta_source.feed(b" two");
    let delta = delta_source.grid_update(false);
    mirror
        .apply_delta(10, &delta, true, false, true)
        .expect("contiguous delta");
    assert_eq!(mirror.modes(), (true, false, true));
    assert!(matches!(
        mirror.apply_delta(12, &delta, false, false, false),
        Err(MirrorError::SequenceGap {
            expected: 11,
            actual: 12
        })
    ));
}
