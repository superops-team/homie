use std::path::PathBuf;

use super::{DiffFile, DiffHunk, DiffRow, DiffRowKind, DiffSnapshot};

pub(super) fn parse_unified_diff_bytes(patch: &[u8]) -> DiffSnapshot {
    let mut snapshot = DiffSnapshot::default();
    let mut old_line = None;
    let mut new_line = None;
    let mut current_file = None;
    let mut current_hunk = None;
    let mut file_preamble = Vec::new();

    for raw_line in patch.split_inclusive(|byte| *byte == b'\n') {
        let line_bytes = trim_patch_line(raw_line);
        let line = String::from_utf8_lossy(line_bytes);

        if let Some(header) = line.strip_prefix("diff --git ") {
            finish_hunk(&mut snapshot, &mut current_hunk);
            finish_file(&mut snapshot, &mut current_file);

            let path = PathBuf::from(diff_path(header));
            let row_start = snapshot.rows.len();
            snapshot.files += 1;
            snapshot.file_diffs.push(DiffFile {
                path: path.clone(),
                row_range: row_start..row_start,
                additions: 0,
                deletions: 0,
                hunks: Vec::new(),
            });
            current_file = Some(snapshot.file_diffs.len() - 1);
            current_hunk = None;
            file_preamble.clear();
            file_preamble.extend_from_slice(raw_line);
            old_line = None;
            new_line = None;

            push_row(
                &mut snapshot,
                DiffRow {
                    kind: DiffRowKind::File,
                    old_line: None,
                    new_line: None,
                    text: path.to_string_lossy().into_owned(),
                },
            );
            continue;
        }

        if line.starts_with("@@") {
            finish_hunk(&mut snapshot, &mut current_hunk);
            let (old, new) = parse_hunk_start(&line);
            let header = line.into_owned();
            old_line = old;
            new_line = new;
            let row_start = snapshot.rows.len();
            if let Some(file_index) = current_file {
                let mut hunk_patch = file_preamble.clone();
                hunk_patch.extend_from_slice(raw_line);
                snapshot.file_diffs[file_index].hunks.push(DiffHunk {
                    header: header.clone(),
                    row_range: row_start..row_start,
                    old_start: old,
                    new_start: new,
                    additions: 0,
                    deletions: 0,
                    patch: hunk_patch,
                    fingerprint: 0,
                });
                current_hunk = Some((file_index, snapshot.file_diffs[file_index].hunks.len() - 1));
            }
            push_row(
                &mut snapshot,
                DiffRow {
                    kind: DiffRowKind::Hunk,
                    old_line: None,
                    new_line: None,
                    text: header,
                },
            );
            continue;
        }

        if let Some((file_index, hunk_index)) = current_hunk {
            snapshot.file_diffs[file_index].hunks[hunk_index]
                .patch
                .extend_from_slice(raw_line);
        } else if current_file.is_some() {
            // Everything before the first hunk is part of the complete file
            // preamble repeated by every independently applicable hunk.
            file_preamble.extend_from_slice(raw_line);
        }

        let row = if line.starts_with("--- ") || line.starts_with("+++ ") {
            continue;
        } else if let Some(text) = line.strip_prefix('+') {
            let current = new_line;
            new_line = new_line.map(|line| line.saturating_add(1));
            snapshot.additions += 1;
            if let Some(file_index) = current_file {
                snapshot.file_diffs[file_index].additions += 1;
            }
            if let Some((file_index, hunk_index)) = current_hunk {
                snapshot.file_diffs[file_index].hunks[hunk_index].additions += 1;
            }
            DiffRow {
                kind: DiffRowKind::Addition,
                old_line: None,
                new_line: current,
                text: text.to_owned(),
            }
        } else if let Some(text) = line.strip_prefix('-') {
            let current = old_line;
            old_line = old_line.map(|line| line.saturating_add(1));
            snapshot.deletions += 1;
            if let Some(file_index) = current_file {
                snapshot.file_diffs[file_index].deletions += 1;
            }
            if let Some((file_index, hunk_index)) = current_hunk {
                snapshot.file_diffs[file_index].hunks[hunk_index].deletions += 1;
            }
            DiffRow {
                kind: DiffRowKind::Deletion,
                old_line: current,
                new_line: None,
                text: text.to_owned(),
            }
        } else if let Some(text) = line.strip_prefix(' ') {
            let old = old_line;
            let new = new_line;
            old_line = old_line.map(|line| line.saturating_add(1));
            new_line = new_line.map(|line| line.saturating_add(1));
            DiffRow {
                kind: DiffRowKind::Context,
                old_line: old,
                new_line: new,
                text: text.to_owned(),
            }
        } else {
            DiffRow {
                kind: DiffRowKind::Meta,
                old_line: None,
                new_line: None,
                text: line.into_owned(),
            }
        };
        push_row(&mut snapshot, row);
    }

    finish_hunk(&mut snapshot, &mut current_hunk);
    finish_file(&mut snapshot, &mut current_file);
    snapshot
}

fn trim_patch_line(mut line: &[u8]) -> &[u8] {
    if let Some(without_newline) = line.strip_suffix(b"\n") {
        line = without_newline;
    }
    line.strip_suffix(b"\r").unwrap_or(line)
}

fn push_row(snapshot: &mut DiffSnapshot, row: DiffRow) {
    snapshot.max_text_columns = snapshot
        .max_text_columns
        .max(row.text.chars().count().min(500));
    snapshot.rows.push(row);
}

fn finish_hunk(snapshot: &mut DiffSnapshot, current: &mut Option<(usize, usize)>) {
    let Some((file_index, hunk_index)) = current.take() else {
        return;
    };
    let hunk = &mut snapshot.file_diffs[file_index].hunks[hunk_index];
    hunk.row_range.end = snapshot.rows.len();
    hunk.fingerprint = fnv1a64(&hunk.patch);
}

fn finish_file(snapshot: &mut DiffSnapshot, current: &mut Option<usize>) {
    let Some(file_index) = current.take() else {
        return;
    };
    snapshot.file_diffs[file_index].row_range.end = snapshot.rows.len();
}

pub(super) fn fnv1a64(bytes: &[u8]) -> u64 {
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;
    bytes.iter().fold(OFFSET, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(PRIME)
    })
}

fn diff_path(header: &str) -> String {
    header
        .rsplit_once(" b/")
        .map(|(_, path)| path)
        .or_else(|| header.rsplit_once(" \"b/").map(|(_, path)| path))
        .unwrap_or(header)
        .trim_matches('"')
        .to_owned()
}

pub(super) fn parse_hunk_start(header: &str) -> (Option<u32>, Option<u32>) {
    let mut fields = header.split_whitespace();
    let _at = fields.next();
    let old = fields.next().and_then(|field| range_start(field, '-'));
    let new = fields.next().and_then(|field| range_start(field, '+'));
    (old, new)
}

fn range_start(field: &str, prefix: char) -> Option<u32> {
    field.strip_prefix(prefix)?.split(',').next()?.parse().ok()
}
