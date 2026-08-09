//! Debounced, capped find over the live grid.
//!
//! Ported from diri-term. Provides case-insensitive search with
//! debounced queries, match capping, and navigation.

use std::time::Duration;

use crate::buffer::GridBuffer;

pub const SEARCH_DEBOUNCE: Duration = Duration::from_millis(200);
pub const OUTPUT_RESCAN_DELAY: Duration = Duration::from_millis(100);
pub const MATCH_CAP: usize = 500;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FindMatch {
    pub absolute_row: i64,
    pub start_col: usize,
    pub end_col_exclusive: usize,
    pub text: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FindSpan {
    pub row: usize,
    pub start_col: usize,
    pub end_col_exclusive: usize,
    pub is_current: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NavigationTarget {
    Live,
}

#[derive(Clone, Debug)]
pub struct SearchRequest {
    pub query: String,
    pub is_rescan: bool,
    pub generation: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FindSnapshot {
    pub lines: Vec<String>,
    pub first_row: i64,
    pub visible_start_row: i64,
    pub cols: i64,
    pub rows: i64,
    pub content_seq: u64,
    pub is_alt_screen: bool,
}

#[derive(Clone, Debug, Default)]
pub struct TerminalFindModel {
    query: String,
    matches: Vec<FindMatch>,
    current_index: usize,
    generation: u64,
    search_due: Option<Duration>,
    rescan_due: Option<Duration>,
}

impl TerminalFindModel {
    #[must_use]
    pub fn query(&self) -> &str {
        &self.query
    }

    #[must_use]
    pub fn matches(&self) -> &[FindMatch] {
        &self.matches
    }

    #[must_use]
    pub const fn current_index(&self) -> usize {
        self.current_index
    }

    pub fn set_query(&mut self, query: impl Into<String>, now: Duration) {
        let query = query.into();
        if query == self.query {
            return;
        }
        self.query = query;
        self.generation = self.generation.wrapping_add(1);
        self.rescan_due = None;
        if self.query.is_empty() {
            self.matches.clear();
            self.current_index = 0;
            self.search_due = None;
        } else {
            self.search_due = Some(now.saturating_add(SEARCH_DEBOUNCE));
        }
    }

    pub fn on_output(&mut self, now: Duration) -> bool {
        if self.query.is_empty() || self.rescan_due.is_some() {
            return false;
        }
        self.rescan_due = Some(now.saturating_add(OUTPUT_RESCAN_DELAY));
        true
    }

    pub fn take_due_search(&mut self, now: Duration) -> Option<(String, bool, u64)> {
        let is_rescan = if self.search_due.is_some_and(|due| due <= now) {
            self.search_due = None;
            false
        } else if self.rescan_due.is_some_and(|due| due <= now) {
            self.rescan_due = None;
            true
        } else {
            return None;
        };
        Some((self.query.clone(), is_rescan, self.generation))
    }

    /// Searches the live grid and updates matches.
    pub fn apply_snapshot(&mut self, query: &str, generation: u64, live: &GridBuffer) -> bool {
        if generation != self.generation || query != self.query {
            return false;
        }
        self.matches = build_matches(&self.query, live);
        if self.matches.is_empty() {
            self.current_index = 0;
        } else {
            self.current_index = self.current_index.min(self.matches.len().saturating_sub(1));
        }
        true
    }

    #[must_use]
    pub fn visible_spans(&self, visible_start_row: i64, visible_rows: usize) -> Vec<FindSpan> {
        self.matches
            .iter()
            .enumerate()
            .filter_map(|(index, item)| {
                let row = item.absolute_row.checked_sub(visible_start_row)?;
                let row = usize::try_from(row).ok()?;
                (row < visible_rows).then_some(FindSpan {
                    row,
                    start_col: item.start_col,
                    end_col_exclusive: item.end_col_exclusive,
                    is_current: index == self.current_index,
                })
            })
            .collect()
    }

    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<NavigationTarget> {
        self.advance(1)
    }

    pub fn previous(&mut self) -> Option<NavigationTarget> {
        self.advance(-1)
    }

    fn advance(&mut self, direction: isize) -> Option<NavigationTarget> {
        if self.matches.is_empty() {
            return None;
        }
        let len = self.matches.len() as isize;
        let idx = self.current_index as isize;
        self.current_index = ((idx + direction).rem_euclid(len)) as usize;
        Some(NavigationTarget::Live)
    }
}

fn build_matches(query: &str, live: &GridBuffer) -> Vec<FindMatch> {
    let needle: Vec<char> = query.chars().collect();
    if needle.is_empty() {
        return Vec::new();
    }
    let mut matches = Vec::new();

    let live_rows = usize::from(live.rows);
    for row in 0..live_rows {
        let Some((line, columns)) = live.row_text_with_columns(row) else {
            continue;
        };
        append_matches(&line, Some(&columns), row as i64, &needle, &mut matches);
        if matches.len() >= MATCH_CAP {
            matches.truncate(MATCH_CAP);
            return matches;
        }
    }
    matches.sort_by_key(|item| (item.absolute_row, item.start_col));
    matches.truncate(MATCH_CAP);
    matches
}

fn append_matches(
    line: &str,
    columns: Option<&[usize]>,
    absolute_row: i64,
    needle: &[char],
    output: &mut Vec<FindMatch>,
) {
    let haystack: Vec<char> = line.chars().collect();
    if haystack.len() < needle.len() {
        return;
    }
    let mut index = 0;
    while index + needle.len() <= haystack.len() && output.len() < MATCH_CAP {
        if chars_equal_ci(&haystack[index..index + needle.len()], needle) {
            let start_col = column_for(index, columns);
            let end_col_exclusive = column_past_end(index + needle.len(), columns);
            output.push(FindMatch {
                absolute_row,
                start_col,
                end_col_exclusive,
                text: line.to_owned(),
            });
            index += needle.len();
        } else {
            index += 1;
        }
    }
}

fn chars_equal_ci(haystack: &[char], needle: &[char]) -> bool {
    haystack
        .iter()
        .zip(needle)
        .all(|(left, right)| left.to_lowercase().eq(right.to_lowercase()))
}

fn column_for(index: usize, columns: Option<&[usize]>) -> usize {
    columns.map_or(index, |columns| {
        columns
            .get(index)
            .copied()
            .or_else(|| columns.last().map(|last| last + 1))
            .unwrap_or(index)
    })
}

fn column_past_end(index: usize, columns: Option<&[usize]>) -> usize {
    column_for(index, columns)
}

#[cfg(test)]
mod tests {
    use homie_proto::grid::{GridCell, TermColor, TermStyle};

    use super::*;

    fn cell(ch: char) -> GridCell {
        GridCell::new(
            u32::from(ch),
            TermColor::Default,
            TermColor::DefaultInverted,
            TermStyle::empty(),
        )
    }

    fn live_buffer(lines: &[&str], cols: usize) -> GridBuffer {
        let mut buffer = GridBuffer::new(cols as u16, lines.len() as u16);
        for (row_index, line) in lines.iter().enumerate() {
            for (col, ch) in line.chars().take(cols).enumerate() {
                buffer.cells[row_index * cols + col] = cell(ch);
            }
        }
        buffer
    }

    #[test]
    fn search_and_rescan_deadlines_are_debounced() {
        let mut model = TerminalFindModel::default();
        model.set_query("needle", Duration::from_millis(10));
        assert!(model.take_due_search(Duration::from_millis(209)).is_none());
        assert_eq!(
            model
                .take_due_search(Duration::from_millis(210))
                .map(|r| r.1),
            Some(false)
        );

        model.on_output(Duration::from_millis(300));
        assert!(model.take_due_search(Duration::from_millis(399)).is_none());
        assert_eq!(
            model
                .take_due_search(Duration::from_millis(400))
                .map(|r| r.1),
            Some(true)
        );
    }

    #[test]
    fn matching_is_case_insensitive_and_non_overlapping() {
        let live = live_buffer(&["BaNaNa", "", ""], 8);
        let mut model = TerminalFindModel::default();
        model.set_query("ana", Duration::ZERO);
        let (query, _, generation) = model.take_due_search(SEARCH_DEBOUNCE).expect("search due");
        model.apply_snapshot(&query, generation, &live);
        assert_eq!(model.matches().len(), 1);
        assert_eq!(
            (
                model.matches()[0].start_col,
                model.matches()[0].end_col_exclusive
            ),
            (1, 4)
        );
    }

    #[test]
    fn match_count_is_capped() {
        let mut buffer = GridBuffer::new(2, 600);
        for i in 0..600 {
            buffer.cells[i * 2] = cell('a');
        }
        let mut model = TerminalFindModel::default();
        model.set_query("a", Duration::ZERO);
        let (query, _, generation) = model.take_due_search(SEARCH_DEBOUNCE).expect("search due");
        model.apply_snapshot(&query, generation, &buffer);
        assert_eq!(model.matches().len(), MATCH_CAP);
    }

    #[test]
    fn navigation_wraps_around() {
        let live = live_buffer(&["a", "a", "a"], 2);
        let mut model = TerminalFindModel::default();
        model.set_query("a", Duration::ZERO);
        let (query, _, generation) = model.take_due_search(SEARCH_DEBOUNCE).expect("search due");
        model.apply_snapshot(&query, generation, &live);
        assert_eq!(model.matches().len(), 3);
        assert_eq!(model.current_index(), 0);
        assert!(model.next().is_some());
        assert_eq!(model.current_index(), 1);
        assert!(model.previous().is_some());
        assert_eq!(model.current_index(), 0);
        assert!(model.previous().is_some());
        assert_eq!(model.current_index(), 2);
    }
}
