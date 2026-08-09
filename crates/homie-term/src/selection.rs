//! Scroll-invariant terminal text selection over a grid buffer.
//!
//! Ported from diri-term. Provides word selection, drag-to-select,
//! and text extraction across selection ranges.

use homie_proto::grid::GridCell;

use crate::buffer::GridBuffer;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct SelectionPoint {
    pub row: i64,
    pub col: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SelectionRange {
    pub start: SelectionPoint,
    pub end: SelectionPoint,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SelectionSpan {
    pub row: usize,
    pub start_col: usize,
    pub end_col_exclusive: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum WordClass {
    Word,
    Whitespace,
    Punctuation,
}

#[derive(Clone, Debug, Default)]
pub struct TerminalSelection {
    anchor: Option<SelectionPoint>,
    head: Option<SelectionPoint>,
}

impl TerminalSelection {
    #[must_use]
    pub const fn anchor(&self) -> Option<SelectionPoint> {
        self.anchor
    }

    #[must_use]
    pub const fn head(&self) -> Option<SelectionPoint> {
        self.head
    }

    pub fn clear(&mut self) {
        self.anchor = None;
        self.head = None;
    }

    pub fn begin(&mut self, point: SelectionPoint) {
        self.anchor = Some(point);
        self.head = Some(point);
    }

    pub fn drag_to(&mut self, point: SelectionPoint) {
        if self.anchor.is_some() {
            self.head = Some(point);
        }
    }

    /// Expands a double-click to a word boundary.
    pub fn select_word(&mut self, buffer: &GridBuffer, window_row: usize, col: usize) {
        let Some(row) = buffer.row(window_row) else {
            self.clear();
            return;
        };
        let col = col.min(row.len().saturating_sub(1));
        let class = word_class(row[col]);
        let mut start = col;
        while start > 0 && word_class(row[start - 1]) == class {
            start -= 1;
        }
        let mut end = col + 1;
        while end < row.len() && word_class(row[end]) == class {
            end += 1;
        }
        let absolute_row = window_row as i64;
        self.anchor = Some(SelectionPoint {
            row: absolute_row,
            col: start,
        });
        self.head = Some(SelectionPoint {
            row: absolute_row,
            col: end,
        });
    }

    #[must_use]
    pub fn range(&self) -> Option<SelectionRange> {
        let anchor = self.anchor?;
        let head = self.head?;
        if anchor == head {
            return None;
        }
        let (start, end) = if anchor <= head {
            (anchor, head)
        } else {
            (head, anchor)
        };
        Some(SelectionRange { start, end })
    }

    /// Returns the visible spans of the selection within the given viewport.
    /// Simplified: does not depend on scrollback; always uses live rows.
    #[must_use]
    pub fn visible_spans(&self, visible_rows: usize, cols: usize) -> Vec<SelectionSpan> {
        let Some(range) = self.range() else {
            return Vec::new();
        };
        if cols == 0 || visible_rows == 0 {
            return Vec::new();
        }
        let window_top: i64 = 0;
        let window_bottom =
            window_top.saturating_add(i64::try_from(visible_rows).unwrap_or(i64::MAX));
        let first = range.start.row.max(window_top);
        let last = range.end.row.min(window_bottom.saturating_sub(1));
        if first > last {
            return Vec::new();
        }
        let mut spans = Vec::new();
        for row in first..=last {
            let start_col = if row == range.start.row {
                range.start.col
            } else {
                0
            };
            let end_col = if row == range.end.row {
                range.end.col
            } else {
                cols
            };
            if start_col < end_col {
                spans.push(SelectionSpan {
                    row: row as usize,
                    start_col: start_col.min(cols),
                    end_col_exclusive: end_col.min(cols),
                });
            }
        }
        spans
    }

    /// Extracts selected text, trimming trailing spaces and joining rows with newlines.
    #[must_use]
    pub fn selected_text(&self, buffer: &GridBuffer) -> String {
        let Some(range) = self.range() else {
            return String::new();
        };
        let cols = usize::from(buffer.cols);
        let mut lines = Vec::new();
        for absolute_row in range.start.row..=range.end.row {
            let (start_col, end_col) = columns_for_row(range, absolute_row, cols);
            if start_col >= end_col {
                lines.push(String::new());
                continue;
            }
            let row = buffer.row(absolute_row as usize).unwrap_or(&[]);
            let mut line = String::new();
            for cell in &row[start_col..end_col.min(row.len())] {
                line.push(cell_char(*cell));
            }
            lines.push(line.trim_end_matches(' ').to_owned());
        }
        lines.join("\n")
    }
}

fn columns_for_row(range: SelectionRange, row: i64, cols: usize) -> (usize, usize) {
    let start = if row == range.start.row {
        range.start.col
    } else {
        0
    };
    let end = if row == range.end.row {
        range.end.col
    } else {
        cols
    };
    (start.min(cols), end.min(cols))
}

fn word_class(cell: GridCell) -> WordClass {
    let ch = cell_char(cell);
    if ch.is_alphanumeric() || ch == '_' {
        WordClass::Word
    } else if ch.is_whitespace() {
        WordClass::Whitespace
    } else {
        WordClass::Punctuation
    }
}

pub(crate) fn cell_char(cell: GridCell) -> char {
    if cell.scalar == 0 {
        return ' ';
    }
    char::from_u32(cell.scalar)
        .filter(|ch| *ch != '\n' && *ch != '\r')
        .unwrap_or(' ')
}

#[cfg(test)]
mod tests {
    use homie_proto::grid::{TermColor, TermStyle};

    use super::*;

    fn cell(ch: char) -> GridCell {
        GridCell::new(
            u32::from(ch),
            TermColor::Default,
            TermColor::DefaultInverted,
            TermStyle::empty(),
        )
    }

    fn row(text: &str, cols: usize) -> Vec<GridCell> {
        let mut cells: Vec<_> = text.chars().map(cell).collect();
        cells.resize(cols, GridCell::BLANK);
        cells
    }

    #[test]
    fn reverse_drag_normalizes_in_reading_order() {
        let mut selection = TerminalSelection::default();
        selection.begin(SelectionPoint { row: 1, col: 4 });
        selection.drag_to(SelectionPoint { row: 0, col: 1 });
        assert_eq!(
            selection.range(),
            Some(SelectionRange {
                start: SelectionPoint { row: 0, col: 1 },
                end: SelectionPoint { row: 1, col: 4 },
            })
        );
    }

    #[test]
    fn double_click_selects_word() {
        let mut buffer = GridBuffer::new(14, 1);
        buffer.cells = row("one two_three!", 14);
        let mut selection = TerminalSelection::default();
        selection.select_word(&buffer, 0, 6);
        assert_eq!(selection.selected_text(&buffer), "two_three");
    }

    #[test]
    fn selected_text_trims_trailing_spaces() {
        let mut buffer = GridBuffer::new(10, 1);
        buffer.cells = row("hello     ", 10);
        let mut selection = TerminalSelection::default();
        selection.begin(SelectionPoint { row: 0, col: 0 });
        selection.drag_to(SelectionPoint { row: 0, col: 5 });
        assert_eq!(selection.selected_text(&buffer), "hello");
    }
}
