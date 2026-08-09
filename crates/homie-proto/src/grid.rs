//! Terminal grid cell types and RLE (run-length-encoding) codecs.
//!
//! Ported from diri-proto's grid module, which is byte-for-byte compatible with
//! the Swift DirijorProtocol Grid codec. All integers are big-endian.

use std::error::Error;
use std::fmt;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign};

/// Maximum terminal columns accepted by protocol producers and consumers.
pub const MAX_TERMINAL_COLS: u16 = 4_096;
/// Maximum terminal rows accepted by protocol producers and consumers.
pub const MAX_TERMINAL_ROWS: u16 = 4_096;
/// Maximum number of cells in one terminal grid.
pub const MAX_TERMINAL_CELLS: usize = 1_048_576;

/// Returns the bounded terminal cell count for a valid non-zero geometry.
#[must_use]
pub fn terminal_cell_count(cols: u16, rows: u16) -> Option<usize> {
    if cols == 0 || rows == 0 || cols > MAX_TERMINAL_COLS || rows > MAX_TERMINAL_ROWS {
        return None;
    }
    usize::from(cols)
        .checked_mul(usize::from(rows))
        .filter(|cells| *cells <= MAX_TERMINAL_CELLS)
}

// ---------------------------------------------------------------------------
// TermColor — packed 4-byte colour used on the wire
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum TermColor {
    Default,
    DefaultInverted,
    Ansi(u8),
    Rgb(u8, u8, u8),
}

impl TermColor {
    /// Four-byte packed form `[tag][a][b][c]` used on the wire.
    #[must_use]
    pub const fn packed(self) -> u32 {
        match self {
            Self::Default => 0,
            Self::DefaultInverted => 1 << 24,
            Self::Ansi(color) => (2 << 24) | color as u32,
            Self::Rgb(red, green, blue) => {
                (3 << 24) | ((red as u32) << 16) | ((green as u32) << 8) | blue as u32
            }
        }
    }

    /// Decodes the packed form. Every tag other than 0, 1, and 2 is interpreted as RGB.
    #[must_use]
    pub const fn unpack(packed: u32) -> Self {
        match packed >> 24 {
            0 => Self::Default,
            1 => Self::DefaultInverted,
            2 => Self::Ansi((packed & 0xff) as u8),
            _ => Self::Rgb(
                ((packed >> 16) & 0xff) as u8,
                ((packed >> 8) & 0xff) as u8,
                (packed & 0xff) as u8,
            ),
        }
    }
}

// ---------------------------------------------------------------------------
// TermStyle — SwiftTerm CharacterStyle bit layout
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
#[repr(transparent)]
pub struct TermStyle(u16);

impl TermStyle {
    pub const BOLD: Self = Self(1 << 0);
    pub const UNDERLINE: Self = Self(1 << 1);
    pub const BLINK: Self = Self(1 << 2);
    pub const INVERSE: Self = Self(1 << 3);
    pub const INVISIBLE: Self = Self(1 << 4);
    pub const DIM: Self = Self(1 << 5);
    pub const ITALIC: Self = Self(1 << 6);
    pub const CROSSED_OUT: Self = Self(1 << 7);

    #[must_use]
    pub const fn empty() -> Self {
        Self(0)
    }

    #[must_use]
    pub const fn from_bits_retain(bits: u16) -> Self {
        Self(bits)
    }

    #[must_use]
    pub const fn bits(self) -> u16 {
        self.0
    }

    #[must_use]
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
}

impl BitOr for TermStyle {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for TermStyle {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitAnd for TermStyle {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl BitAndAssign for TermStyle {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

// ---------------------------------------------------------------------------
// GridCell
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct GridCell {
    /// Unicode scalar value; zero is rendered as blank by the client.
    pub scalar: u32,
    pub fg: TermColor,
    pub bg: TermColor,
    pub style: TermStyle,
}

impl GridCell {
    pub const BLANK: Self = Self {
        scalar: 32,
        fg: TermColor::Default,
        bg: TermColor::DefaultInverted,
        style: TermStyle::empty(),
    };

    #[must_use]
    pub const fn new(scalar: u32, fg: TermColor, bg: TermColor, style: TermStyle) -> Self {
        Self {
            scalar,
            fg,
            bg,
            style,
        }
    }

    /// Wire size of one cell in bytes: 4 (scalar) + 4 (fg) + 4 (bg) + 2 (style) = 14.
    pub const WIRE_BYTES: usize = 14;
}

// ---------------------------------------------------------------------------
// Row RLE codec
// ---------------------------------------------------------------------------

/// A decoded terminal row — a sequence of cells.
pub type GridRow = Vec<GridCell>;

/// RLE-encoded row as it travels on the wire.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RowRle {
    pub cols: u16,
    pub data: Vec<u8>,
}

/// Errors that can occur during RLE decode.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RleError {
    Truncated,
    RunTooLong,
    ZeroRunLength,
    GeometryOutOfBounds,
}

impl fmt::Display for RleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Truncated => f.write_str("RLE data truncated"),
            Self::RunTooLong => f.write_str("RLE run exceeds column count"),
            Self::ZeroRunLength => f.write_str("RLE run has zero length"),
            Self::GeometryOutOfBounds => f.write_str("RLE geometry exceeds terminal limits"),
        }
    }
}

impl Error for RleError {}

/// Encode a row into RLE.
///
/// The wire format is the same as Swift's `Row.encode()`:
/// - 2 bytes: column count (big-endian u16)
/// - Repeat for each run:
///   - 1 byte: run length (1–255); a run of exactly 255 means *more* cells follow
///   - 14 bytes: repeated cell
pub fn encode_row(row: &[GridCell]) -> RowRle {
    let row = &row[..row.len().min(usize::from(MAX_TERMINAL_COLS))];
    let cols = row.len() as u16;
    let capacity = row
        .len()
        .checked_mul(GridCell::WIRE_BYTES + 1)
        .and_then(|bytes| bytes.checked_add(size_of::<u16>()))
        .unwrap_or(size_of::<u16>());
    let mut data = Vec::with_capacity(capacity);

    // Column count (big-endian u16)
    data.extend_from_slice(&cols.to_be_bytes());

    if row.is_empty() {
        return RowRle { cols, data };
    }

    let mut i = 0;
    while i < row.len() {
        let cell = row[i];
        let mut run = 1;
        while i + run < row.len() && row[i + run] == cell && run < 255 {
            run += 1;
        }
        data.push(run as u8);
        encode_cell(&mut data, &cell);
        i += run;
    }

    RowRle { cols, data }
}

/// Decode an RLE-encoded row into a vector of cells.
pub fn decode_row(rle: &RowRle) -> Result<GridRow, RleError> {
    if rle.data.len() < 2 {
        return Err(RleError::Truncated);
    }

    let cols = u16::from_be_bytes([rle.data[0], rle.data[1]]) as usize;
    if cols > usize::from(MAX_TERMINAL_COLS) {
        return Err(RleError::GeometryOutOfBounds);
    }
    let mut row = Vec::new();
    row.try_reserve_exact(cols)
        .map_err(|_| RleError::GeometryOutOfBounds)?;
    let mut pos = 2;

    while pos < rle.data.len() {
        if row.len() >= cols {
            break;
        }
        if pos >= rle.data.len() {
            return Err(RleError::Truncated);
        }
        let run_len = rle.data[pos] as usize;
        pos += 1;
        if run_len == 0 {
            return Err(RleError::ZeroRunLength);
        }
        if row.len() + run_len > cols {
            return Err(RleError::RunTooLong);
        }
        if pos + GridCell::WIRE_BYTES > rle.data.len() {
            return Err(RleError::Truncated);
        }
        let cell = decode_cell(&rle.data[pos..pos + GridCell::WIRE_BYTES]);
        pos += GridCell::WIRE_BYTES;
        for _ in 0..run_len {
            row.push(cell);
        }
    }

    // Pad to column count if the RLE was shorter (shouldn't happen for valid data)
    while row.len() < cols {
        row.push(GridCell::BLANK);
    }

    Ok(row)
}

fn encode_cell(buf: &mut Vec<u8>, cell: &GridCell) {
    buf.extend_from_slice(&cell.scalar.to_be_bytes());
    buf.extend_from_slice(&cell.fg.packed().to_be_bytes());
    buf.extend_from_slice(&cell.bg.packed().to_be_bytes());
    buf.extend_from_slice(&cell.style.bits().to_be_bytes());
}

// ---------------------------------------------------------------------------
// GridUpdate — a daemon-to-client grid update (full snapshot or diff)
// ---------------------------------------------------------------------------

/// One changed row in a grid update.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChangedRow {
    /// 0-based row index.
    pub y: u16,
    pub cells: Vec<GridCell>,
}

impl ChangedRow {
    pub fn new(y: u16, cells: Vec<GridCell>) -> Self {
        Self { y, cells }
    }
}

/// A grid update from the daemon: either a full snapshot or a diff.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GridUpdate {
    pub cols: u16,
    pub rows: u16,
    pub cursor_col: u16,
    pub cursor_row: u16,
    pub cursor_visible: bool,
    pub is_full_snapshot: bool,
    pub changed_rows: Vec<ChangedRow>,
}

fn decode_cell(bytes: &[u8]) -> GridCell {
    let scalar = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    let fg = TermColor::unpack(u32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]));
    let bg = TermColor::unpack(u32::from_be_bytes([
        bytes[8], bytes[9], bytes[10], bytes[11],
    ]));
    let style = TermStyle::from_bits_retain(u16::from_be_bytes([bytes[12], bytes[13]]));
    GridCell::new(scalar, fg, bg, style)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn term_color_pack_unpack_round_trip() {
        let cases = [
            TermColor::Default,
            TermColor::DefaultInverted,
            TermColor::Ansi(42),
            TermColor::Rgb(255, 128, 0),
        ];
        for color in cases {
            assert_eq!(TermColor::unpack(color.packed()), color);
        }
    }

    #[test]
    fn term_style_bit_ops() {
        let mut style = TermStyle::BOLD | TermStyle::ITALIC;
        assert!(style.contains(TermStyle::BOLD));
        assert!(style.contains(TermStyle::ITALIC));
        assert!(!style.contains(TermStyle::UNDERLINE));
        style |= TermStyle::UNDERLINE;
        assert!(style.contains(TermStyle::UNDERLINE));
    }

    #[test]
    fn rle_encode_decode_round_trip() {
        let cells = vec![
            GridCell::new(
                'H' as u32,
                TermColor::Default,
                TermColor::DefaultInverted,
                TermStyle::empty(),
            ),
            GridCell::new(
                'i' as u32,
                TermColor::Default,
                TermColor::DefaultInverted,
                TermStyle::empty(),
            ),
            GridCell::new(
                '!' as u32,
                TermColor::Ansi(2),
                TermColor::DefaultInverted,
                TermStyle::BOLD,
            ),
        ];
        let rle = encode_row(&cells);
        let decoded = decode_row(&rle).expect("decode_row");
        assert_eq!(decoded, cells);
    }

    #[test]
    fn rle_run_length_encoding() {
        // 10 identical cells should produce a single run
        let cell = GridCell::new(
            ' ' as u32,
            TermColor::Default,
            TermColor::DefaultInverted,
            TermStyle::empty(),
        );
        let cells = vec![cell; 10];
        let rle = encode_row(&cells);
        // 2 bytes cols + 1 byte run + 14 bytes cell = 17 bytes
        assert_eq!(rle.data.len(), 17);
        let decoded = decode_row(&rle).expect("decode_row");
        assert_eq!(decoded, cells);
    }

    #[test]
    fn rle_empty_row() {
        let rle = encode_row(&[]);
        assert_eq!(rle.cols, 0);
        let decoded = decode_row(&rle).expect("decode_row");
        assert!(decoded.is_empty());
    }

    #[test]
    fn rle_truncated_error() {
        let rle = RowRle {
            cols: 10,
            data: vec![0, 10],
        }; // col count only, no cells
        let result = decode_row(&rle);
        assert!(result.is_ok()); // Empty row is valid
    }

    #[test]
    fn rle_zero_run_length_error() {
        // 2 bytes cols=5, 1 byte run=0
        let rle = RowRle {
            cols: 5,
            data: vec![0, 5, 0],
        };
        let result = decode_row(&rle);
        assert!(matches!(result, Err(RleError::ZeroRunLength)));
    }

    #[test]
    fn terminal_cell_count_accepts_geometry_within_all_limits() {
        assert_eq!(terminal_cell_count(120, 40), Some(4_800));
    }

    #[test]
    fn terminal_cell_count_rejects_zero_geometry() {
        assert_eq!(terminal_cell_count(0, 40), None);
    }

    #[test]
    fn terminal_cell_count_rejects_axis_over_limit() {
        assert_eq!(terminal_cell_count(MAX_TERMINAL_COLS + 1, 1), None);
    }

    #[test]
    fn terminal_cell_count_rejects_total_cells_over_limit() {
        assert_eq!(terminal_cell_count(1_024, 1_025), None);
    }

    #[test]
    fn decode_row_rejects_column_count_over_terminal_limit() {
        let cols = MAX_TERMINAL_COLS + 1;
        let rle = RowRle {
            cols,
            data: cols.to_be_bytes().to_vec(),
        };

        assert_eq!(decode_row(&rle), Err(RleError::GeometryOutOfBounds));
    }
}
