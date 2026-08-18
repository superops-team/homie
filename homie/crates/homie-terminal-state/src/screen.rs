use std::sync::mpsc::{self, Receiver, SyncSender};

use alacritty_terminal::event::{Event, EventListener};
use alacritty_terminal::grid::Dimensions;
use alacritty_terminal::index::{Column, Line};
use alacritty_terminal::term::{Config, Term, TermMode};
use alacritty_terminal::vte::ansi::Processor;
use homie_proto::grid::{ChangedRow, GridCell, GridRowCodec, GridUpdate};

use super::wire::{emulator_cell, find, sgr, wire_cell};

/// Plain-text terminal state consumed by the local status detector.
#[derive(Clone, Debug, Default)]
pub struct ScreenSnapshot {
    pub lines: Vec<String>,
    pub osc_title: Option<String>,
    pub osc_progress_state: Option<i64>,
    /// Bumps whenever visible content changes.
    pub content_seq: u64,
}

impl ScreenSnapshot {
    pub fn from_lines<I, S>(lines: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            lines: lines.into_iter().map(Into::into).collect(),
            ..Self::default()
        }
    }
}

/// Scrollback is byte-budgeted: enough history for a client's scrollback view
/// without letting a build log grow daemon memory unboundedly. Divided by the
/// per-line cell cost at construction.
///
/// 4 MiB works out to ~2,180 history rows at 80 columns (~870 at 200) per
/// session. The original 1 MiB kept only 546 rows at 80 columns — shallower
/// than one long compile's output, and users hit the floor scrolling back.
const HISTORY_CELL_BUDGET_BYTES: usize = 4 << 20;

fn history_line_limit(cols: usize) -> usize {
    let bytes_per_line = cols.max(1) * std::mem::size_of::<alacritty_terminal::term::cell::Cell>();
    (HISTORY_CELL_BUDGET_BYTES / bytes_per_line).max(64)
}

/// Fixed screen geometry handed to the emulator.
#[derive(Clone, Copy, Debug)]
struct Geometry {
    cols: usize,
    rows: usize,
}

impl Dimensions for Geometry {
    fn total_lines(&self) -> usize {
        // History beyond the visible screen is not useful for detection: rules
        // read the current screen. Keeping scrollback at zero also bounds the
        // memory a runaway session can cost the daemon.
        self.rows
    }

    fn screen_lines(&self) -> usize {
        self.rows
    }

    fn columns(&self) -> usize {
        self.cols
    }
}

/// Collects the events the emulator emits. Only the title is interesting.
#[derive(Clone)]
struct Collector(SyncSender<Event>);

const EVENT_QUEUE_CAPACITY: usize = 64;

impl EventListener for Collector {
    fn send_event(&self, event: Event) {
        // A full channel means nobody is draining it, which is not worth
        // failing a session over.
        let _ = self.0.try_send(event);
    }
}

pub struct HeadlessScreen {
    term: Term<Collector>,
    parser: Processor,
    events: Receiver<Event>,
    geometry: Geometry,

    title: Option<String>,
    progress_state: Option<i64>,
    progress_value: Option<i64>,

    content_seq: u64,
    last_digest: u64,
    /// Trailing bytes of the previous chunk, so an OSC split across a read
    /// boundary is still recognized.
    progress_carry: Vec<u8>,

    /// Diff baseline for [`grid_update`]: the cells last handed out, so the
    /// next call sends only changed rows.
    ///
    /// [`grid_update`]: HeadlessScreen::grid_update
    last_cells: Vec<GridCell>,
    last_grid_cols: usize,
    last_grid_rows: usize,
}

impl HeadlessScreen {
    pub fn new(cols: usize, rows: usize) -> Self {
        let geometry = Geometry {
            cols: cols.max(1),
            rows: rows.max(1),
        };
        let (sender, events) = mpsc::sync_channel(EVENT_QUEUE_CAPACITY);
        let config = Config {
            scrolling_history: history_line_limit(geometry.cols),
            ..Config::default()
        };
        let term = Term::new(config, &geometry, Collector(sender));
        Self {
            term,
            parser: Processor::new(),
            events,
            geometry,
            title: None,
            progress_state: None,
            progress_value: None,
            content_seq: 0,
            last_digest: 0,
            progress_carry: Vec::new(),
            last_cells: Vec::new(),
            last_grid_cols: 0,
            last_grid_rows: 0,
        }
    }

    /// Feeds raw PTY output into the emulator.
    ///
    /// The whole chunk goes to the parser in one call — vte has a batched
    /// fast path for plain text that byte-at-a-time feeding defeats, and the
    /// difference is multi-x on heavy output like build logs.
    pub fn feed(&mut self, bytes: &[u8]) {
        self.scan_progress(bytes);
        let title_before = self.title.clone();
        self.parser.advance(&mut self.term, bytes);
        self.drain_events();

        // Damage is the cheap gate: when the emulator reports nothing
        // touched, skip fingerprinting entirely. When it does (which includes
        // invisible changes like a cursor toggle), a direct cell hash — no
        // per-line String allocation — decides whether the *content* changed.
        let damaged = match self.term.damage() {
            alacritty_terminal::term::TermDamage::Full => true,
            alacritty_terminal::term::TermDamage::Partial(mut lines) => lines.next().is_some(),
        };
        self.term.reset_damage();
        if damaged || self.title != title_before {
            let digest = self.digest_cells();
            if digest != self.last_digest {
                self.last_digest = digest;
                self.content_seq += 1;
            }
        }
    }

    pub fn resize(&mut self, cols: usize, rows: usize) {
        self.geometry = Geometry {
            cols: cols.max(1),
            rows: rows.max(1),
        };
        self.term.resize(self.geometry);
    }

    pub fn content_seq(&self) -> u64 {
        self.content_seq
    }

    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// True while the child is on the alternate screen (a full-screen program
    /// like an editor or pager).
    pub fn is_alt_screen(&self) -> bool {
        self.term
            .mode()
            .contains(alacritty_terminal::term::TermMode::ALT_SCREEN)
    }

    /// Whether the child has bracketed-paste mode on — submitted prompts are
    /// then framed as a paste so embedded newlines don't submit early.
    pub fn bracketed_paste(&self) -> bool {
        self.term
            .mode()
            .contains(alacritty_terminal::term::TermMode::BRACKETED_PASTE)
    }

    /// The current grid geometry.
    pub fn size(&self) -> (usize, usize) {
        (self.geometry.cols, self.geometry.rows)
    }

    /// Whether the child asked for mouse reporting (any flavor).
    pub fn mouse_reporting(&self) -> bool {
        self.term.mode().intersects(TermMode::MOUSE_MODE)
    }

    /// Cursor (col, row, visible) without touching cell data.
    pub fn cursor(&self) -> (u16, u16, bool) {
        let point = self.term.grid().cursor.point;
        (
            point.column.0 as u16,
            point.line.0.max(0) as u16,
            self.term.mode().contains(TermMode::SHOW_CURSOR),
        )
    }

    /// Builds a `GridUpdate` from the current screen. When `full` is true (or
    /// the geometry changed) every row is included and the diff baseline
    /// resets; otherwise only rows that changed since the last call are
    /// included.
    pub fn grid_update(&mut self, full: bool) -> GridUpdate {
        let cols = self.geometry.cols;
        let rows = self.geometry.rows;
        let geometry_changed = self.last_grid_cols != cols || self.last_grid_rows != rows;
        let force_full = full || geometry_changed;
        if geometry_changed {
            self.last_cells = vec![GridCell::BLANK; cols * rows];
            self.last_grid_cols = cols;
            self.last_grid_rows = rows;
        }

        let grid = self.term.grid();
        let mut changed = Vec::new();
        for y in 0..rows {
            let line = Line(y as i32);
            let base = y * cols;
            let mut row = Vec::with_capacity(cols);
            let mut row_changed = force_full;
            for x in 0..cols {
                let cell = wire_cell(&grid[line][Column(x)]);
                if !row_changed && self.last_cells[base + x] != cell {
                    row_changed = true;
                }
                row.push(cell);
            }
            if !row_changed {
                continue;
            }
            self.last_cells[base..base + cols].copy_from_slice(&row);
            changed.push(ChangedRow::new(y as u16, row));
        }

        let cursor = grid.cursor.point;
        GridUpdate {
            cols: cols as u16,
            rows: rows as u16,
            cursor_col: cursor.column.0 as u16,
            cursor_row: cursor.line.0.max(0) as u16,
            cursor_visible: self.term.mode().contains(TermMode::SHOW_CURSOR),
            is_full_snapshot: force_full,
            changed_rows: changed,
        }
    }

    /// A full-screen snapshot that does NOT disturb the diff baseline. Used
    /// to seed a fresh sink (or repair one that fell behind) without breaking
    /// other sinks' diffs.
    pub fn full_snapshot(&self) -> GridUpdate {
        let cols = self.geometry.cols;
        let rows = self.geometry.rows;
        let grid = self.term.grid();
        let mut all = Vec::with_capacity(rows);
        for y in 0..rows {
            let line = Line(y as i32);
            let mut row = Vec::with_capacity(cols);
            for x in 0..cols {
                row.push(wire_cell(&grid[line][Column(x)]));
            }
            all.push(ChangedRow::new(y as u16, row));
        }
        let cursor = grid.cursor.point;
        GridUpdate {
            cols: cols as u16,
            rows: rows as u16,
            cursor_col: cursor.column.0 as u16,
            cursor_row: cursor.line.0.max(0) as u16,
            cursor_visible: self.term.mode().contains(TermMode::SHOW_CURSOR),
            is_full_snapshot: true,
            changed_rows: all,
        }
    }

    /// Restores a persisted visible grid into a fresh emulator by synthesizing
    /// the byte stream that would have painted it. Each non-blank cell is
    /// cursor-addressed independently, because a wide glyph consumes two
    /// terminal columns while the grid also carries its blank continuation
    /// cell — a naive row string would shift everything after it.
    /// Styled rows above the visible grid, oldest first. Checkpoints persist
    /// these alongside the visible snapshot so adoption does not collapse a
    /// long session to a single scrollback row.
    pub fn history_snapshot(&self) -> Vec<Vec<GridCell>> {
        let grid = self.term.grid();
        let history = grid.history_size();
        let cols = self.geometry.cols;
        let mut rows = Vec::with_capacity(history);
        for index in 0..history {
            let line = Line(index as i32 - history as i32);
            let mut row = Vec::with_capacity(cols);
            for x in 0..cols {
                row.push(wire_cell(&grid[line][Column(x)]));
            }
            rows.push(row);
        }
        rows
    }

    pub fn restore(
        &mut self,
        history: &[Vec<GridCell>],
        update: &GridUpdate,
        alt_screen: bool,
        bracketed_paste: bool,
        mouse_reporting: bool,
    ) -> bool {
        let cols = self.geometry.cols;
        let rows = self.geometry.rows;
        if !update.is_full_snapshot
            || update.cols as usize != cols
            || update.rows as usize != rows
            || update.changed_rows.len() != rows
            || history.len() > history_line_limit(cols)
            || history.iter().any(|row| row.len() != cols)
        {
            return false;
        }

        // Allocate scrollback in the emulator, then replace those rows with
        // the persisted cells. The visible grid is painted below; CSI 2 J
        // clears only that viewport and deliberately leaves history intact.
        if !history.is_empty() {
            let grid = self.term.grid_mut();
            grid.scroll_up(&(Line(0)..Line(rows as i32)), history.len());
            let restored_history = grid.history_size();
            if restored_history != history.len() {
                return false;
            }
            for (index, row) in history.iter().enumerate() {
                let line = Line(index as i32 - restored_history as i32);
                for (x, cell) in row.iter().enumerate() {
                    grid[line][Column(x)] = emulator_cell(*cell);
                }
            }
        }

        let mut bytes = Vec::with_capacity(cols * rows * 2);
        if alt_screen {
            bytes.extend_from_slice(b"\x1b[?1049h");
        }
        bytes.extend_from_slice(b"\x1b[H\x1b[2J");

        let mut sorted: Vec<&ChangedRow> = update.changed_rows.iter().collect();
        sorted.sort_by_key(|row| row.y);
        for row in sorted {
            if row.y as usize >= rows || row.cells.len() != cols {
                return false;
            }
            let mut previous: Option<&GridCell> = None;
            for (x, cell) in row.cells.iter().enumerate() {
                // The initial clear already produced true blank cells;
                // leaving them untouched keeps sparse checkpoints cheap.
                if *cell == GridCell::BLANK {
                    continue;
                }
                bytes.extend_from_slice(format!("\x1b[{};{}H", row.y + 1, x + 1).as_bytes());
                if previous.is_none_or(|prev| {
                    prev.fg != cell.fg || prev.bg != cell.bg || prev.style != cell.style
                }) {
                    bytes.extend_from_slice(sgr(cell).as_bytes());
                }
                match char::from_u32(cell.scalar) {
                    Some(glyph) if cell.scalar != 0 => {
                        let mut buffer = [0u8; 4];
                        bytes.extend_from_slice(glyph.encode_utf8(&mut buffer).as_bytes());
                    }
                    _ => bytes.push(b' '),
                }
                previous = Some(cell);
            }
        }

        bytes.extend_from_slice(b"\x1b[0m");
        if bracketed_paste {
            bytes.extend_from_slice(b"\x1b[?2004h");
        }
        if mouse_reporting {
            bytes.extend_from_slice(b"\x1b[?1000h\x1b[?1006h");
        }
        bytes.extend_from_slice(if update.cursor_visible {
            b"\x1b[?25h"
        } else {
            b"\x1b[?25l"
        });
        bytes.extend_from_slice(
            format!(
                "\x1b[{};{}H",
                (update.cursor_row as usize + 1).min(rows),
                (update.cursor_col as usize + 1).min(cols)
            )
            .as_bytes(),
        );
        self.feed(&bytes);
        // Force the next grid_update to be a full frame: the diff baseline
        // predates the restore.
        self.last_grid_cols = 0;
        self.last_grid_rows = 0;
        true
    }

    /// Encodes a wheel event for the child, when it asked for mouse
    /// reporting. Returns the bytes to write to the PTY — empty means the
    /// child doesn't care and the client should scroll its own scrollback.
    pub fn mouse_wheel(&self, up: bool, lines: usize, col: usize, row: usize) -> Vec<u8> {
        if !self.mouse_reporting() || lines == 0 {
            return Vec::new();
        }
        let x = col.min(self.geometry.cols.saturating_sub(1));
        let y = row.min(self.geometry.rows.saturating_sub(1));
        let button = if up { 64 } else { 65 }; // X11 wheel buttons 4/5, wheel-flagged
        let mut out = Vec::new();
        for _ in 0..lines {
            if self.term.mode().contains(TermMode::SGR_MOUSE) {
                out.extend_from_slice(format!("\x1b[<{button};{};{}M", x + 1, y + 1).as_bytes());
            } else {
                // Legacy X10 encoding: 32 + button, 32 + 1-based coordinate.
                out.push(0x1b);
                out.extend_from_slice(b"[M");
                out.push(32 + button as u8);
                out.push((32 + x + 1).min(255) as u8);
                out.push((32 + y + 1).min(255) as u8);
            }
        }
        out
    }

    /// Scrollback plus the visible screen as plain text, for search.
    ///
    /// Row indices are relative to the oldest line this emulator still
    /// retains. They slide once the history budget evicts, so a client caching
    /// deep scrollback across heavy output may refetch; the visible region and
    /// recent history are exact.
    pub fn scrollback(&self) -> homie_proto::ReadScrollbackResult {
        let grid = self.term.grid();
        let history = grid.history_size();
        let rows = self.geometry.rows;
        let cols = self.geometry.cols;
        let total = history + rows;
        let mut lines = Vec::with_capacity(total);
        for index in 0..total {
            let line = Line(index as i32 - history as i32);
            let mut text = String::with_capacity(cols);
            for x in 0..cols {
                let c = grid[line][Column(x)].c;
                text.push(if c < ' ' && c != '\t' { ' ' } else { c });
            }
            lines.push(text.trim_end().to_string());
        }
        homie_proto::ReadScrollbackResult {
            lines,
            first_row: 0,
            visible_start_row: history as i64,
            cols: cols as i64,
            rows: rows as i64,
            content_seq: self.content_seq,
            is_alt_screen: self.is_alt_screen(),
        }
    }

    /// A window of scrollback rows as encoded cells, clamped to what exists.
    pub fn scrollback_cells(
        &self,
        first_row: i64,
        max_rows: i64,
    ) -> homie_proto::ReadScrollbackCellsResult {
        let grid = self.term.grid();
        let history = grid.history_size();
        let cols = self.geometry.cols;
        let total = history + self.geometry.rows;

        let start = first_row.max(0).min(total as i64) as usize;
        let end = (start + max_rows.max(0) as usize).min(total);
        let mut rows = Vec::with_capacity(end.saturating_sub(start));
        for index in start..end {
            let line = Line(index as i32 - history as i32);
            let mut row = Vec::with_capacity(cols);
            for x in 0..cols {
                row.push(wire_cell(&grid[line][Column(x)]));
            }
            rows.push(row);
        }
        homie_proto::ReadScrollbackCellsResult {
            payload: GridRowCodec::encode_rows(&rows).unwrap_or_default(),
            first_row: start as i64,
            row_count: rows.len() as i64,
            total_rows: total as i64,
            live_start_row: history as i64,
            cols: cols as i64,
            content_seq: self.content_seq,
        }
    }

    /// The visible grid as plain text, trailing blank lines removed.
    pub fn lines(&self) -> Vec<String> {
        let grid = self.term.grid();
        let mut lines: Vec<String> = Vec::with_capacity(self.geometry.rows);
        for row in 0..self.geometry.rows {
            let line = Line(row as i32);
            let mut text = String::with_capacity(self.geometry.cols);
            for column in 0..self.geometry.cols {
                text.push(grid[line][Column(column)].c);
            }
            lines.push(text.trim_end().to_string());
        }
        while lines.last().is_some_and(|line| line.trim().is_empty()) {
            lines.pop();
        }
        lines
    }

    /// A snapshot for the detection engine.
    pub fn snapshot(&self) -> ScreenSnapshot {
        ScreenSnapshot {
            lines: self.lines(),
            osc_title: self.title.clone(),
            osc_progress_state: self.progress_state,
            content_seq: self.content_seq,
        }
    }

    pub fn progress(&self) -> Option<(i64, i64)> {
        Some((self.progress_state?, self.progress_value.unwrap_or(0)))
    }

    fn drain_events(&mut self) {
        while let Ok(event) = self.events.try_recv() {
            if let Event::Title(title) = event {
                self.title = Some(title);
            } else if matches!(event, Event::ResetTitle) {
                self.title = None;
            }
        }
    }

    // (cell mapping lives at module level; see `wire_cell`)

    /// Content fingerprint hashed straight off the grid cells, so
    /// `content_seq` only advances when the visible screen actually changed.
    /// Detection uses that to skip re-evaluating a frame it has already
    /// judged.
    fn digest_cells(&self) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        let grid = self.term.grid();
        for row in 0..self.geometry.rows {
            let line = Line(row as i32);
            for column in 0..self.geometry.cols {
                grid[line][Column(column)].c.hash(&mut hasher);
            }
        }
        self.title.hash(&mut hasher);
        hasher.finish()
    }

    /// Extracts `ESC ] 9 ; 4 ; state ; value` progress reports.
    ///
    /// Agents use this to say "I am working, 40% through"; the emulator has no
    /// concept of it and would silently drop the sequence.
    fn scan_progress(&mut self, bytes: &[u8]) {
        const PREFIX: &[u8] = b"\x1b]9;4;";
        let mut haystack = std::mem::take(&mut self.progress_carry);
        haystack.extend_from_slice(bytes);

        let mut search_from = 0;
        while let Some(found) = find(&haystack[search_from..], PREFIX) {
            let start = search_from + found + PREFIX.len();
            // Terminated by BEL or ST (ESC \).
            let Some(end) = haystack[start..]
                .iter()
                .position(|&b| b == 0x07 || b == 0x1b)
                .map(|offset| start + offset)
            else {
                // Truncated: keep it for the next chunk.
                break;
            };
            let payload = String::from_utf8_lossy(&haystack[start..end]);
            let mut parts = payload.split(';');
            self.progress_state = parts.next().and_then(|value| value.trim().parse().ok());
            self.progress_value = parts.next().and_then(|value| value.trim().parse().ok());
            search_from = end;
        }

        // Keep a tail long enough to rejoin a sequence split across reads.
        let keep = haystack.len().saturating_sub(64);
        self.progress_carry = haystack[keep..].to_vec();
    }
}
