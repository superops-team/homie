use std::path::{Component, Path, PathBuf};

use super::{SourceLine, SourceTarget};

pub(super) fn lexical_normalize(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::Prefix(_) | Component::RootDir | Component::Normal(_) => {
                normalized.push(component.as_os_str());
            }
        }
    }
    normalized
}

pub(super) fn source_lines(text: &str) -> Vec<SourceLine> {
    let mut result = Vec::new();
    let mut start = 0;
    for (offset, character) in text.char_indices() {
        if character != '\n' {
            continue;
        }
        let mut end = offset;
        if end > start && text.as_bytes()[end - 1] == b'\r' {
            end -= 1;
        }
        result.push(SourceLine {
            number: result.len() + 1,
            range: start..end,
        });
        start = offset + 1;
    }
    result.push(SourceLine {
        number: result.len() + 1,
        range: start..text.len(),
    });
    result
}

pub(super) fn clamp_target(target: SourceTarget, text: &str, lines: &[SourceLine]) -> SourceTarget {
    let line = target.line.clamp(1, lines.len().max(1));
    let line_range = &lines[line - 1].range;
    let characters = text[line_range.clone()].chars().count();
    let column = target.column.clamp(1, characters + 1);
    SourceTarget { line, column }
}

pub(super) fn path_score(query: &str, path: &str) -> Option<u32> {
    let mut score = fuzzy_score(query, path)?;
    let file_name = path.rsplit('/').next().unwrap_or(path);
    if let Some(file_score) = fuzzy_score(query, file_name) {
        score = score.max(file_score.saturating_add(2_000));
    }
    if file_name.eq_ignore_ascii_case(query) {
        score = score.saturating_add(5_000);
    }
    Some(score)
}

pub(super) fn fuzzy_score(query: &str, candidate: &str) -> Option<u32> {
    let candidate_lower = candidate.to_lowercase();
    let mut total = 0u32;
    for token in query.to_lowercase().split_whitespace() {
        let token_score = if let Some(position) = candidate_lower.find(token) {
            let boundary = position == 0
                || candidate_lower[..position]
                    .chars()
                    .next_back()
                    .is_some_and(|character| !character.is_alphanumeric());
            4_000u32
                .saturating_sub(position.min(2_000) as u32)
                .saturating_add(if boundary { 800 } else { 0 })
                .saturating_add((token.chars().count() as u32).saturating_mul(80))
        } else {
            subsequence_score(token, &candidate_lower)?
        };
        total = total.saturating_add(token_score);
    }
    Some(total.saturating_sub(candidate_lower.chars().count().min(1_000) as u32))
}

fn subsequence_score(query: &str, candidate: &str) -> Option<u32> {
    let mut query = query.chars();
    let mut wanted = query.next()?;
    let mut score = 0u32;
    let mut previous_match = None;
    let mut matched = 0usize;

    for (position, character) in candidate.chars().enumerate() {
        if character != wanted {
            continue;
        }
        matched += 1;
        score = score.saturating_add(180);
        if previous_match.is_some_and(|previous| previous + 1 == position) {
            score = score.saturating_add(220);
        }
        if position == 0
            || candidate
                .chars()
                .nth(position.saturating_sub(1))
                .is_some_and(|previous| !previous.is_alphanumeric())
        {
            score = score.saturating_add(300);
        }
        previous_match = Some(position);
        let Some(next) = query.next() else {
            return Some(
                score
                    .saturating_add((matched as u32).saturating_mul(40))
                    .saturating_sub(position.min(1_000) as u32),
            );
        };
        wanted = next;
    }
    None
}

pub(super) fn excerpt(text: &str, max_characters: usize) -> String {
    let mut result: String = text.chars().take(max_characters).collect();
    if text.chars().count() > max_characters {
        result.push('…');
    }
    result
}
