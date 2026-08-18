use homie_proto::grid::{ChangedRow, GridCell, GridUpdate};

/// Receiver-side authority for a remote terminal stream. A mirror accepts a
/// full snapshot as a new baseline and then only contiguous sequenced diffs;
/// a gap forces the caller to request another full snapshot instead of
/// silently displaying a plausible but incorrect screen.
#[derive(Clone, Debug, Default)]
pub struct GridMirror {
    cells: Vec<GridCell>,
    cols: u16,
    rows: u16,
    cursor_col: u16,
    cursor_row: u16,
    cursor_visible: bool,
    sequence: Option<u64>,
    alt_screen: bool,
    bracketed_paste: bool,
    mouse_reporting: bool,
}

impl GridMirror {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn apply_snapshot(
        &mut self,
        sequence: u64,
        grid: &GridUpdate,
        alt_screen: bool,
        bracketed_paste: bool,
        mouse_reporting: bool,
    ) -> Result<(), MirrorError> {
        if !grid.is_full_snapshot {
            return Err(MirrorError::SnapshotRequired);
        }
        validate_grid(grid)?;
        grid.apply(&mut self.cells);
        self.update_metadata(grid);
        self.sequence = Some(sequence);
        self.alt_screen = alt_screen;
        self.bracketed_paste = bracketed_paste;
        self.mouse_reporting = mouse_reporting;
        Ok(())
    }

    pub fn apply_delta(
        &mut self,
        sequence: u64,
        grid: &GridUpdate,
        alt_screen: bool,
        bracketed_paste: bool,
        mouse_reporting: bool,
    ) -> Result<(), MirrorError> {
        let Some(previous) = self.sequence else {
            return Err(MirrorError::SnapshotRequired);
        };
        let expected = previous
            .checked_add(1)
            .ok_or(MirrorError::SequenceOverflow)?;
        if sequence != expected {
            return Err(MirrorError::SequenceGap {
                expected,
                actual: sequence,
            });
        }
        if grid.is_full_snapshot || grid.cols != self.cols || grid.rows != self.rows {
            return Err(MirrorError::SnapshotRequired);
        }
        validate_grid(grid)?;
        grid.apply(&mut self.cells);
        self.update_metadata(grid);
        self.sequence = Some(sequence);
        self.alt_screen = alt_screen;
        self.bracketed_paste = bracketed_paste;
        self.mouse_reporting = mouse_reporting;
        Ok(())
    }

    fn update_metadata(&mut self, grid: &GridUpdate) {
        self.cols = grid.cols;
        self.rows = grid.rows;
        self.cursor_col = grid.cursor_col;
        self.cursor_row = grid.cursor_row;
        self.cursor_visible = grid.cursor_visible;
    }

    #[must_use]
    pub fn cells(&self) -> &[GridCell] {
        &self.cells
    }

    #[must_use]
    pub const fn size(&self) -> (u16, u16) {
        (self.cols, self.rows)
    }

    #[must_use]
    pub const fn cursor(&self) -> (u16, u16, bool) {
        (self.cursor_col, self.cursor_row, self.cursor_visible)
    }

    #[must_use]
    pub const fn sequence(&self) -> Option<u64> {
        self.sequence
    }

    #[must_use]
    pub const fn modes(&self) -> (bool, bool, bool) {
        (self.alt_screen, self.bracketed_paste, self.mouse_reporting)
    }

    #[must_use]
    pub fn full_update(&self) -> Option<GridUpdate> {
        self.sequence?;
        let cols = usize::from(self.cols);
        let rows = usize::from(self.rows);
        if self.cells.len() != cols.saturating_mul(rows) {
            return None;
        }
        let changed_rows = (0..rows)
            .map(|row| {
                ChangedRow::new(
                    row as u16,
                    self.cells[row * cols..(row + 1) * cols].to_vec(),
                )
            })
            .collect();
        Some(GridUpdate {
            cols: self.cols,
            rows: self.rows,
            cursor_col: self.cursor_col,
            cursor_row: self.cursor_row,
            cursor_visible: self.cursor_visible,
            is_full_snapshot: true,
            changed_rows,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MirrorError {
    SnapshotRequired,
    SequenceGap { expected: u64, actual: u64 },
    SequenceOverflow,
    InvalidGrid(&'static str),
}

impl std::fmt::Display for MirrorError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SnapshotRequired => formatter.write_str("a full terminal snapshot is required"),
            Self::SequenceGap { expected, actual } => {
                write!(
                    formatter,
                    "terminal sequence gap: expected {expected}, got {actual}"
                )
            }
            Self::SequenceOverflow => formatter.write_str("terminal sequence overflow"),
            Self::InvalidGrid(detail) => write!(formatter, "invalid terminal grid: {detail}"),
        }
    }
}

impl std::error::Error for MirrorError {}

fn validate_grid(grid: &GridUpdate) -> Result<(), MirrorError> {
    if grid.cols == 0 || grid.rows == 0 {
        return Err(MirrorError::InvalidGrid("dimensions must be non-zero"));
    }
    if grid.cursor_col >= grid.cols || grid.cursor_row >= grid.rows {
        return Err(MirrorError::InvalidGrid("cursor is outside the grid"));
    }
    for row in &grid.changed_rows {
        if row.y >= grid.rows || row.cells.len() != usize::from(grid.cols) {
            return Err(MirrorError::InvalidGrid(
                "changed row is outside the grid or has the wrong width",
            ));
        }
    }
    Ok(())
}
