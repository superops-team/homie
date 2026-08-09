//! Cell-based terminal grid buffer, key encoding, selection, find, and theme.
//!
//! Ported from diri-term. Provides:
//! - `buffer`: GridBuffer with damage tracking and generation management
//! - `keys`: Platform-independent keyboard and paste encoding for terminal input
//! - `selection`: Scroll-invariant terminal text selection
//! - `find`: Debounced, capped find over live grid
//! - `theme`: Terminal color themes with cell style resolution

pub mod buffer;
pub mod element;
pub mod find;
pub mod keys;
pub mod metrics;
pub mod repaint;
pub mod scrollback;
pub mod selection;
pub mod theme;

pub use buffer::{ApplySummary, CursorState, GridBuffer};
pub use element::{RendererStats, SharedGridBuffer, TerminalElement, TerminalPrepaintState};
pub use find::{
    FindMatch, FindSnapshot, FindSpan, NavigationTarget, SearchRequest, TerminalFindModel,
};
pub use keys::{Key, KeyEvent, Modifiers, NamedKey, TermInputModes, encode_key, paste};
pub use metrics::CellMetrics;
pub use repaint::{RepaintAction, RepaintPacer};
pub use scrollback::{
    ScrollRouter, ScrollbackRequest, ScrollbackViewport, TerminalModes, WheelEvent, WheelRoute,
};
pub use selection::{SelectionPoint, SelectionRange, SelectionSpan, TerminalSelection};
pub use theme::{ResolvedCellStyle, TermTheme, is_default_background};
