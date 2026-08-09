//! Scrollback viewport with wheel routing and mode tracking.
//!
//! Rows are addressed by absolute terminal row. The live grid starts at
//! `live_start_row`, and historical rows are cached by absolute row.

use std::collections::BTreeMap;
use std::fmt;
use std::ops::Range;

use homie_proto::grid::GridCell;

use crate::buffer::GridBuffer;

const MAX_SCROLLBACK_CACHE_ROWS: usize = 512;

pub type SessionId = String;

#[derive(Clone, Debug, PartialEq)]
pub struct ReadScrollbackCellsResult {
    pub first_row: i64,
    pub row_count: i64,
    pub rows: Vec<Vec<GridCell>>,
    pub total_rows: i64,
    pub live_start_row: i64,
    pub content_seq: u64,
}

#[derive(Clone, Debug)]
pub struct GridCodecError(String);

impl fmt::Display for GridCodecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl std::error::Error for GridCodecError {}

pub trait GridRowCodec {
    fn decode(&self, data: &[u8]) -> Result<Vec<GridCell>, GridCodecError>;
    fn encode(&self, cells: &[GridCell]) -> Vec<u8>;
}

// ── ScrollbackRequest / ScrolledState ──────────────────────────────────

#[derive(Clone, Debug)]
pub struct ScrollbackRequest {
    pub first_row: i64,
    pub max_rows: i64,
}

impl ScrollbackRequest {
    #[must_use]
    pub fn range(&self) -> Range<i64> {
        self.first_row..self.first_row.saturating_add(self.max_rows)
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ScrolledState {
    pub offset_lines: i64,
}

impl ScrolledState {
    #[must_use]
    pub fn label(&self) -> String {
        format!("{} lines · Return to live", self.offset_lines)
    }
}

// ── ScrollbackApplyError ───────────────────────────────────────────────

#[derive(Clone, Debug)]
pub enum ScrollbackApplyError {
    NegativeRowCount(i64),
    RowCountMismatch { requested: usize, received: usize },
    Codec(GridCodecError),
    Other(String),
}

impl fmt::Display for ScrollbackApplyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NegativeRowCount(n) => write!(f, "negative row count: {n}"),
            Self::RowCountMismatch {
                requested,
                received,
            } => write!(
                f,
                "row count mismatch: requested {requested}, received {received}"
            ),
            Self::Codec(e) => write!(f, "codec error: {e}"),
            Self::Other(msg) => write!(f, "{msg}"),
        }
    }
}
impl std::error::Error for ScrollbackApplyError {}

// ── ScrollbackViewport ─────────────────────────────────────────────────

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ScrollbackViewport {
    offset: i64,
    live_start_row: i64,
    total_rows: i64,
    geometry_known: bool,
    content_seq: Option<u64>,
    cached_rows: BTreeMap<i64, Vec<GridCell>>,
    in_flight: Option<Range<i64>>,
    queued: Option<Range<i64>>,
}

impl ScrollbackViewport {
    #[must_use]
    pub fn view_offset(&self) -> i64 {
        self.offset
    }

    #[must_use]
    pub fn cached_row(&self, absolute_row: i64) -> Option<&[GridCell]> {
        self.cached_rows.get(&absolute_row).map(Vec::as_slice)
    }

    #[must_use]
    pub fn cached_row_count(&self) -> usize {
        self.cached_rows.len()
    }

    #[must_use]
    pub fn max_offset(&self, visible_rows: usize) -> i64 {
        if self.geometry_known {
            self.total_rows.max(0)
        } else {
            self.offset
                .saturating_add(i64::try_from(visible_rows).unwrap_or(i64::MAX))
                .max(0)
        }
    }

    pub fn set_view_offset(&mut self, offset: i64, visible_rows: usize) -> bool {
        let clamped = offset.clamp(0, self.max_offset(visible_rows));
        if clamped == self.offset {
            return false;
        }
        self.offset = clamped;
        self.queue_missing_window(visible_rows);
        true
    }

    pub fn scroll_by(&mut self, lines: i64, visible_rows: usize) -> bool {
        self.set_view_offset(self.offset + lines, visible_rows)
    }

    pub fn scroll_to_live(&mut self, visible_rows: usize) -> bool {
        self.set_view_offset(0, visible_rows)
    }

    pub fn scroll_to_absolute(
        &mut self,
        absolute_row: i64,
        anchor: f32,
        visible_rows: usize,
    ) -> bool {
        let window_row = (anchor.clamp(0.0, 1.0) * visible_rows as f32).round() as i64;
        self.set_view_offset(
            self.live_start_row
                .saturating_add(window_row)
                .saturating_sub(absolute_row),
            visible_rows,
        )
    }

    #[must_use]
    pub fn absolute_row(&self, window_row: usize) -> i64 {
        self.live_start_row - self.offset + window_row as i64
    }

    #[must_use]
    pub fn window_row_for_absolute(&self, absolute_row: i64) -> Option<i64> {
        let window_row = absolute_row - self.absolute_row(0);
        if window_row >= 0 {
            Some(window_row)
        } else {
            None
        }
    }

    #[must_use]
    pub fn row_at_absolute(&self, buffer: &GridBuffer, absolute_row: i64) -> Vec<GridCell> {
        if let Some(cached) = self.cached_rows.get(&absolute_row) {
            return cached.clone();
        }
        if absolute_row >= self.live_start_row {
            let idx = usize::try_from(absolute_row - self.live_start_row).unwrap_or(usize::MAX);
            normalized_row(buffer.row(idx).unwrap_or_default(), buffer.cols as usize)
        } else {
            normalized_row(
                self.cached_rows
                    .get(&absolute_row)
                    .map(Vec::as_slice)
                    .unwrap_or_default(),
                buffer.cols as usize,
            )
        }
    }

    #[must_use]
    pub fn window_row(&self, buffer: &GridBuffer, window_row: usize) -> Vec<GridCell> {
        let absolute = self.absolute_row(window_row);
        self.row_at_absolute(buffer, absolute)
    }

    #[must_use]
    pub fn compose(&self, buffer: &GridBuffer, visible_rows: usize) -> Vec<Vec<GridCell>> {
        let mut rows = Vec::with_capacity(visible_rows);
        for i in 0..visible_rows {
            rows.push(self.window_row(buffer, i));
        }
        rows
    }

    pub fn begin_fetch(&mut self, visible_rows: usize) -> Option<ScrollbackRequest> {
        self.queue_missing_window(visible_rows);
        if self.in_flight.is_some() {
            return None;
        }
        let range = self.queued.take()?;
        if range.start >= range.end {
            return None;
        }
        self.in_flight = Some(range.clone());
        Some(ScrollbackRequest {
            first_row: range.start,
            max_rows: range.end - range.start,
        })
    }

    pub fn complete_fetch(
        &mut self,
        result: ReadScrollbackCellsResult,
        visible_rows: usize,
    ) -> Result<(), ScrollbackApplyError> {
        self.in_flight = None;
        let declared = usize::try_from(result.row_count)
            .map_err(|_| ScrollbackApplyError::NegativeRowCount(result.row_count))?;
        if declared != result.rows.len() {
            return Err(ScrollbackApplyError::RowCountMismatch {
                requested: declared,
                received: result.rows.len(),
            });
        }
        self.apply_rows_with_geometry(
            result.first_row,
            result.rows,
            result.live_start_row,
            result.total_rows,
            result.content_seq,
            visible_rows,
        );
        Ok(())
    }

    pub fn fail_fetch(&mut self) {
        if let Some(range) = self.in_flight.take() {
            merge_range(&mut self.queued, range);
        }
    }

    pub fn apply_rows(
        &mut self,
        first_row: i64,
        rows: Vec<Vec<GridCell>>,
    ) -> Result<(), ScrollbackApplyError> {
        for (i, row) in rows.into_iter().enumerate() {
            self.cached_rows.insert(first_row + i as i64, row);
        }
        Ok(())
    }

    pub fn apply_geometry(&mut self, live_start_row: i64, total_rows: i64) {
        self.live_start_row = live_start_row;
        self.total_rows = total_rows.max(0);
        self.geometry_known = true;
        self.offset = self.offset.clamp(0, self.max_offset(0));
    }

    pub fn enter_alt_screen(&mut self) -> bool {
        let changed = self.offset != 0 || self.queued.is_some() || self.in_flight.is_some();
        self.offset = 0;
        self.queued = None;
        self.in_flight = None;
        changed
    }

    fn apply_rows_with_geometry(
        &mut self,
        first_row: i64,
        rows: Vec<Vec<GridCell>>,
        live_start_row: i64,
        total_rows: i64,
        content_seq: u64,
        visible_rows: usize,
    ) {
        if self.content_seq != Some(content_seq) {
            self.cached_rows.clear();
            self.content_seq = Some(content_seq);
        }
        for (i, row) in rows.into_iter().enumerate() {
            self.cached_rows.insert(first_row + i as i64, row);
        }
        self.live_start_row = live_start_row;
        self.total_rows = total_rows.max(0);
        self.geometry_known = true;
        self.offset = self.offset.clamp(0, self.max_offset(visible_rows));
        self.cap_cache_near_viewport();
        self.queued = None;
        self.queue_missing_window(visible_rows);
    }

    fn queue_missing_window(&mut self, visible_rows: usize) {
        if self.offset <= 0 || visible_rows == 0 {
            return;
        }
        let visible_rows = i64::try_from(visible_rows).unwrap_or(i64::MAX);
        let top = self.live_start_row.saturating_sub(self.offset);
        let needed_end = self.live_start_row.min(top.saturating_add(visible_rows));
        if top >= needed_end || (top..needed_end).all(|row| self.cached_rows.contains_key(&row)) {
            return;
        }
        let request =
            top.saturating_sub(visible_rows).max(0)..needed_end.saturating_add(visible_rows);
        if self.in_flight.as_ref() == Some(&request) || self.queued.as_ref() == Some(&request) {
            return;
        }
        merge_range(&mut self.queued, request);
    }

    fn cap_cache_near_viewport(&mut self) {
        let anchor = self.live_start_row.saturating_sub(self.offset);
        while self.cached_rows.len() > MAX_SCROLLBACK_CACHE_ROWS {
            let Some((&first, _)) = self.cached_rows.first_key_value() else {
                break;
            };
            let Some((&last, _)) = self.cached_rows.last_key_value() else {
                break;
            };
            if first.abs_diff(anchor) >= last.abs_diff(anchor) {
                self.cached_rows.remove(&first);
            } else {
                self.cached_rows.remove(&last);
            }
        }
    }
}

// ── TerminalModes ──────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, Default)]
pub struct TerminalModes {
    pub alt_screen: bool,
    pub mouse_reporting: bool,
    pub application_cursor: bool,
    pub application_keypad: bool,
    pub bracketed_paste: bool,
}

// ── Wheel / ScrollRouter ───────────────────────────────────────────────

#[derive(Clone, Copy, Debug)]
pub enum WheelDelta {
    Lines(i64),
    Pixels(f64),
}

#[derive(Clone, Copy, Debug)]
pub struct WheelEvent {
    pub delta: WheelDelta,
    pub visible_rows: usize,
    pub phase: ScrollPhase,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ScrollPhase {
    #[default]
    None,
    Began,
    Changed,
    Ended,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WheelRoute {
    Passthrough { lines: i64 },
    Local { lines: i64 },
    Scrollback,
}

#[derive(Clone, Debug, Default)]
pub struct ScrollRouter {
    phase: ScrollPhase,
}

impl ScrollRouter {
    pub fn route(&mut self, modes: TerminalModes, event: WheelEvent) -> Option<WheelRoute> {
        self.phase = event.phase;
        let lines = match event.delta {
            WheelDelta::Lines(lines) if lines != 0 => lines,
            WheelDelta::Pixels(pixels) if pixels != 0.0 => (pixels / 16.0).trunc() as i64,
            _ => return None,
        };
        if modes.alt_screen || modes.mouse_reporting {
            Some(WheelRoute::Passthrough { lines })
        } else if lines == 0 {
            None
        } else {
            Some(WheelRoute::Local { lines })
        }
    }
}

fn normalized_row(source: &[GridCell], cols: usize) -> Vec<GridCell> {
    let mut row = vec![GridCell::BLANK; cols];
    let copied = source.len().min(cols);
    row[..copied].copy_from_slice(&source[..copied]);
    row
}

fn merge_range(target: &mut Option<Range<i64>>, incoming: Range<i64>) {
    if incoming.start >= incoming.end {
        return;
    }
    if let Some(existing) = target {
        existing.start = existing.start.min(incoming.start);
        existing.end = existing.end.max(incoming.end);
    } else {
        *target = Some(incoming);
    }
}
