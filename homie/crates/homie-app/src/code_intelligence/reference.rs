use std::collections::HashSet;
use std::path::{Path, PathBuf};

use super::SourceTarget;

#[derive(Debug)]
pub(super) struct ParsedReference {
    pub(super) path: PathBuf,
    pub(super) target: Option<SourceTarget>,
}

pub(super) fn parse_reference_candidates(raw: &str) -> Vec<ParsedReference> {
    let mut fragments = Vec::new();
    let mut seen = HashSet::new();
    let mut push = |fragment: &str| {
        let fragment = fragment.trim();
        if !fragment.is_empty() && seen.insert(fragment.to_string()) {
            fragments.push(fragment.to_string());
        }
    };

    push(raw);
    for token in raw.split_whitespace() {
        push(token);
    }
    for quote in ['\'', '"', '`'] {
        let mut positions = raw.match_indices(quote).map(|(position, _)| position);
        while let (Some(start), Some(end)) = (positions.next(), positions.next()) {
            if start + quote.len_utf8() < end {
                push(&raw[start + quote.len_utf8()..end]);
            }
        }
    }
    if let Some(start) = raw.find("file://") {
        let uri = &raw[start..];
        let end = uri.find(char::is_whitespace).unwrap_or(uri.len());
        push(&uri[..end]);
    }

    let mut result = Vec::new();
    let mut parsed_seen = HashSet::new();
    for fragment in fragments {
        let cleaned = clean_wrappers(&fragment);
        if let Some(parsed) = parse_reference_fragment(cleaned) {
            let key = (parsed.path.clone(), parsed.target);
            if parsed_seen.insert(key) {
                result.push(parsed);
            }
        }
    }
    result
}

fn clean_wrappers(mut text: &str) -> &str {
    text = text.trim();
    loop {
        text = text.trim_end_matches([',', ';', '!', '?']).trim();
        let bytes = text.as_bytes();
        let wrapped = bytes.len() >= 2
            && matches!(
                (bytes[0], bytes[bytes.len() - 1]),
                (b'(', b')')
                    | (b'[', b']')
                    | (b'{', b'}')
                    | (b'<', b'>')
                    | (b'\'', b'\'')
                    | (b'"', b'"')
                    | (b'`', b'`')
            );
        if wrapped {
            text = text[1..text.len() - 1].trim();
        } else {
            break;
        }
    }
    text.trim_matches(|character: char| {
        matches!(
            character,
            '\'' | '"' | '`' | '[' | ']' | '{' | '}' | '<' | '>'
        )
    })
}

fn parse_reference_fragment(fragment: &str) -> Option<ParsedReference> {
    let fragment = clean_wrappers(fragment);
    if fragment.is_empty() {
        return None;
    }

    if let Some(uri) = fragment.strip_prefix("file://") {
        return parse_file_uri(uri);
    }

    if let Some(parsed) = parse_stack_reference(fragment) {
        return Some(parsed);
    }

    let fragment = fragment.trim_end_matches(['.', ':']);
    let (path, target) = parse_colon_target(fragment);
    looks_like_path(path).then(|| ParsedReference {
        path: PathBuf::from(path),
        target,
    })
}

fn parse_file_uri(uri: &str) -> Option<ParsedReference> {
    let uri = uri.strip_prefix("localhost").unwrap_or(uri);
    let uri = uri.trim_end_matches([',', ';', ')', ']']);
    let (encoded_path, fragment) = uri.split_once('#').unwrap_or((uri, ""));
    let decoded = percent_decode(encoded_path)?;
    let target = parse_line_fragment(fragment);
    let path = PathBuf::from(decoded);
    path.is_absolute()
        .then_some(ParsedReference { path, target })
}

fn parse_line_fragment(fragment: &str) -> Option<SourceTarget> {
    let fragment = fragment.strip_prefix('L')?;
    let (line, column) = fragment
        .split_once(['C', ':'])
        .map_or((fragment, None), |(line, column)| (line, Some(column)));
    Some(SourceTarget {
        line: line.parse().ok()?,
        column: column.and_then(|column| column.parse().ok()).unwrap_or(1),
    })
}

fn parse_stack_reference(fragment: &str) -> Option<ParsedReference> {
    let close = fragment.rfind(')')?;
    if !fragment[close + 1..]
        .trim_matches(|character: char| matches!(character, ',' | ';' | '.'))
        .is_empty()
    {
        return None;
    }
    let open = fragment[..close].rfind('(')?;
    let coordinates = &fragment[open + 1..close];
    let (line, column) = coordinates
        .split_once([',', ':'])
        .map_or((coordinates, None), |(line, column)| (line, Some(column)));
    let line = line.trim().parse().ok()?;
    let column = column
        .and_then(|column| column.trim().parse().ok())
        .unwrap_or(1);
    let path = clean_wrappers(fragment[..open].trim());
    let path = path.split_whitespace().last().unwrap_or(path);
    looks_like_path(path).then(|| ParsedReference {
        path: PathBuf::from(path),
        target: Some(SourceTarget { line, column }),
    })
}

fn parse_colon_target(fragment: &str) -> (&str, Option<SourceTarget>) {
    let Some((before_last, last)) = fragment.rsplit_once(':') else {
        return (fragment, None);
    };
    let Ok(last_number) = last.parse::<usize>() else {
        return (fragment, None);
    };
    if let Some((path, line)) = before_last.rsplit_once(':')
        && let Ok(line) = line.parse::<usize>()
    {
        return (
            path,
            Some(SourceTarget {
                line,
                column: last_number,
            }),
        );
    }
    (
        before_last,
        Some(SourceTarget {
            line: last_number,
            column: 1,
        }),
    )
}

fn looks_like_path(path: &str) -> bool {
    if path.is_empty() {
        return false;
    }
    let path = Path::new(path);
    path.is_absolute()
        || path.components().count() > 1
        || path.extension().is_some()
        || path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| {
                matches!(
                    name,
                    "Cargo.toml" | "Package.swift" | "Makefile" | "Dockerfile" | "CMakeLists.txt"
                )
            })
}

fn percent_decode(encoded: &str) -> Option<String> {
    let bytes = encoded.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            let high = hex_value(*bytes.get(index + 1)?)?;
            let low = hex_value(*bytes.get(index + 2)?)?;
            decoded.push(high * 16 + low);
            index += 3;
        } else {
            decoded.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8(decoded).ok()
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}
