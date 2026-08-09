use homie_term::buffer::GridBuffer;
use homie_term::find::TerminalFindModel;
use homie_term::keys::{KeyEvent, Modifiers, NamedKey, TermInputModes, encode_key};
use std::time::Duration;

#[test]
fn grid_is_created_with_dimensions() {
    let grid = GridBuffer::new(80, 24);
    assert_eq!(grid.cols, 80);
    assert_eq!(grid.rows, 24);
    assert!(grid.is_blank());
}

#[test]
fn find_returns_row_and_column_matches() {
    use homie_proto::grid::{GridCell, TermColor, TermStyle};

    fn cell(ch: char) -> GridCell {
        GridCell::new(
            u32::from(ch),
            TermColor::Default,
            TermColor::DefaultInverted,
            TermStyle::empty(),
        )
    }

    let mut grid = GridBuffer::new(80, 2);
    let text = "hello";
    for (col, ch) in text.chars().enumerate() {
        grid.cells[col] = cell(ch);
    }
    let text2 = "well hello";
    for (col, ch) in text2.chars().enumerate() {
        grid.cells[80 + col] = cell(ch);
    }

    let mut model = TerminalFindModel::default();
    model.set_query("hello", Duration::ZERO);
    let (query, _, generation) = model
        .take_due_search(Duration::from_millis(200))
        .expect("search due");
    model.apply_snapshot(&query, generation, &grid);

    let matches = model.matches();
    assert_eq!(matches.len(), 2);
    assert_eq!(matches[0].absolute_row, 0);
    assert_eq!(matches[0].start_col, 0);
    assert_eq!(matches[1].absolute_row, 1);
    assert_eq!(matches[1].start_col, 5);
}

#[test]
fn named_keys_encode_to_terminal_sequences() {
    let modes = TermInputModes::default();

    assert_eq!(
        encode_key(
            &KeyEvent::named(NamedKey::Enter),
            Modifiers::default(),
            modes
        ),
        b"\r"
    );
    assert_eq!(
        encode_key(
            &KeyEvent::named(NamedKey::Escape),
            Modifiers::default(),
            modes
        ),
        b"\x1b"
    );
    // Arrow keys use xterm-style CSI 1;mod format with modifier=1 when no modifier pressed
    assert_eq!(
        encode_key(
            &KeyEvent::named(NamedKey::ArrowUp),
            Modifiers::default(),
            modes
        ),
        b"\x1b[1;1A"
    );
    assert_eq!(
        encode_key(
            &KeyEvent::named(NamedKey::ArrowDown),
            Modifiers::default(),
            modes
        ),
        b"\x1b[1;1B"
    );
    assert_eq!(
        encode_key(
            &KeyEvent::named(NamedKey::ArrowRight),
            Modifiers::default(),
            modes
        ),
        b"\x1b[1;1C"
    );
    assert_eq!(
        encode_key(
            &KeyEvent::named(NamedKey::ArrowLeft),
            Modifiers::default(),
            modes
        ),
        b"\x1b[1;1D"
    );
}
