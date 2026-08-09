use homie_proto::grid::{GridCell, TermColor, TermStyle};
use homie_term::buffer::GridBuffer;
use homie_term::scrollback::{
    ReadScrollbackCellsResult, ScrollRouter, ScrollbackApplyError, ScrollbackViewport,
    TerminalModes, WheelDelta, WheelEvent, WheelRoute,
};

fn cell(ch: char) -> GridCell {
    GridCell::new(
        u32::from(ch),
        TermColor::Default,
        TermColor::DefaultInverted,
        TermStyle::empty(),
    )
}

fn row(text: &str, cols: usize) -> Vec<GridCell> {
    let mut cells = text.chars().map(cell).collect::<Vec<_>>();
    cells.resize(cols, GridCell::BLANK);
    cells
}

#[test]
fn scrollback_fetches_missing_window_and_composes_with_live_rows() {
    let mut viewport = ScrollbackViewport::default();
    viewport.apply_geometry(8, 8);
    assert!(viewport.set_view_offset(2, 3));

    let request = viewport.begin_fetch(3).expect("fetch request");
    assert_eq!(request.first_row, 3);
    assert_eq!(request.max_rows, 8);

    viewport
        .complete_fetch(
            ReadScrollbackCellsResult {
                first_row: 3,
                row_count: 5,
                rows: vec![
                    row("h3", 3),
                    row("h4", 3),
                    row("h5", 3),
                    row("h6", 3),
                    row("h7", 3),
                ],
                total_rows: 8,
                live_start_row: 8,
                content_seq: 1,
            },
            3,
        )
        .expect("fetch applies");

    let mut live = GridBuffer::new(3, 3);
    live.cells = [row("L0", 3), row("L1", 3), row("L2", 3)].concat();
    let composed = viewport.compose(&live, 3);
    assert_eq!(composed, vec![row("h6", 3), row("h7", 3), row("L0", 3)]);
}

#[test]
fn scrollback_rejects_mismatched_fetch_results() {
    let mut viewport = ScrollbackViewport::default();
    viewport.apply_geometry(8, 8);
    viewport.set_view_offset(2, 3);
    viewport.begin_fetch(3).expect("fetch request");

    let error = viewport
        .complete_fetch(
            ReadScrollbackCellsResult {
                first_row: 3,
                row_count: 5,
                rows: vec![row("h3", 3)],
                total_rows: 0,
                live_start_row: 8,
                content_seq: 1,
            },
            3,
        )
        .expect_err("mismatch");
    assert!(matches!(
        error,
        ScrollbackApplyError::RowCountMismatch {
            requested: 5,
            received: 1
        }
    ));
}

#[test]
fn scrollback_alt_screen_returns_to_live_and_clears_requests() {
    let mut viewport = ScrollbackViewport::default();
    viewport.apply_geometry(10, 10);
    viewport.set_view_offset(4, 3);
    assert!(viewport.begin_fetch(3).is_some());

    assert!(viewport.enter_alt_screen());
    assert_eq!(viewport.view_offset(), 0);
    assert!(viewport.begin_fetch(3).is_none());
}

#[test]
fn scroll_router_is_local_only_outside_alt_or_mouse_modes() {
    let mut router = ScrollRouter::default();
    let event = WheelEvent {
        delta: WheelDelta::Lines(2),
        visible_rows: 20,
        phase: Default::default(),
    };

    assert_eq!(
        router.route(TerminalModes::default(), event),
        Some(WheelRoute::Local { lines: 2 })
    );

    assert!(matches!(
        router.route(
            TerminalModes {
                alt_screen: true,
                ..Default::default()
            },
            event,
        ),
        Some(WheelRoute::Passthrough { lines: 2 })
    ));
}
