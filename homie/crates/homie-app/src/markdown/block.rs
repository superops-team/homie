use super::inline::parse_inline;
use super::*;

pub(super) fn parse_blocks(lines: &[&str], quote_depth: usize) -> ParsedBlocks {
    let mut blocks = Vec::new();
    let mut index = 0;
    let mut limited = false;

    while index < lines.len() {
        if blocks.len() >= MAX_BLOCKS {
            limited = true;
            break;
        }
        if lines[index].trim().is_empty() {
            index += 1;
            continue;
        }

        if let Some(fence) = fence_start(lines[index]) {
            let mut code_lines = Vec::new();
            index += 1;
            while index < lines.len() && !is_fence_close(lines[index], fence) {
                code_lines.push(lines[index]);
                index += 1;
            }
            if index < lines.len() {
                index += 1;
            }
            blocks.push(MarkdownBlock::CodeBlock {
                language: fence
                    .info
                    .split_whitespace()
                    .next()
                    .filter(|value| !value.is_empty())
                    .map(str::to_owned),
                code: code_lines.join("\n"),
            });
            continue;
        }

        if let Some((level, content)) = heading(lines[index]) {
            blocks.push(MarkdownBlock::Heading {
                level,
                content: parse_inline(content),
            });
            index += 1;
            continue;
        }

        if is_thematic_break(lines[index]) {
            blocks.push(MarkdownBlock::ThematicBreak);
            index += 1;
            continue;
        }

        if quote_content(lines[index]).is_some() {
            if quote_depth >= MAX_QUOTE_DEPTH {
                limited = true;
                break;
            }
            let mut quote_lines = Vec::new();
            while index < lines.len() {
                let Some(content) = quote_content(lines[index]) else {
                    break;
                };
                quote_lines.push(content.to_owned());
                index += 1;
            }
            let quote_refs = quote_lines.iter().map(String::as_str).collect::<Vec<_>>();
            let nested = parse_blocks(&quote_refs, quote_depth + 1);
            limited |= nested.limited;
            blocks.push(MarkdownBlock::Quote(nested.blocks));
            continue;
        }

        if let Some(first) = list_marker(lines[index]) {
            let mut items = Vec::new();
            let ordered = first.ordered;
            let start = first.number;
            while index < lines.len() {
                let Some(marker) = list_marker(lines[index]) else {
                    break;
                };
                if marker.ordered != ordered {
                    break;
                }
                if items.len() >= MAX_LIST_ITEMS {
                    limited = true;
                    index = lines.len();
                    break;
                }

                let (checked, first_line) = task_marker(marker.content);
                let mut content = first_line.trim().to_owned();
                index += 1;
                while index < lines.len() {
                    if lines[index].trim().is_empty() {
                        break;
                    }
                    if list_marker(lines[index]).is_some() || starts_block(lines[index]) {
                        break;
                    }
                    if leading_indent(lines[index]) < 2 {
                        break;
                    }
                    let continuation = lines[index].trim();
                    if !continuation.is_empty() {
                        if !content.is_empty() {
                            content.push(' ');
                        }
                        content.push_str(continuation);
                    }
                    index += 1;
                }
                items.push(MarkdownListItem {
                    checked,
                    content: parse_inline(&content),
                });
            }
            blocks.push(MarkdownBlock::List {
                ordered,
                start,
                items,
            });
            continue;
        }

        let mut paragraph = lines[index].trim().to_owned();
        index += 1;
        while index < lines.len() && !lines[index].trim().is_empty() && !starts_block(lines[index])
        {
            let continuation = lines[index].trim();
            if !continuation.is_empty() {
                if !paragraph.is_empty() {
                    paragraph.push(' ');
                }
                paragraph.push_str(continuation);
            }
            index += 1;
        }
        blocks.push(MarkdownBlock::Paragraph(parse_inline(&paragraph)));
    }

    ParsedBlocks { blocks, limited }
}

fn starts_block(line: &str) -> bool {
    fence_start(line).is_some()
        || heading(line).is_some()
        || is_thematic_break(line)
        || quote_content(line).is_some()
        || list_marker(line).is_some()
}

fn strip_block_indent(line: &str) -> Option<&str> {
    let spaces = line.bytes().take_while(|byte| *byte == b' ').count();
    (spaces <= 3).then_some(&line[spaces..])
}

fn fence_start(line: &str) -> Option<Fence<'_>> {
    let line = strip_block_indent(line)?;
    let marker = *line.as_bytes().first()?;
    if !matches!(marker, b'`' | b'~') {
        return None;
    }
    let length = line.bytes().take_while(|byte| *byte == marker).count();
    if length < 3 {
        return None;
    }
    let info = line[length..].trim();
    if marker == b'`' && info.contains('`') {
        return None;
    }
    Some(Fence {
        marker,
        length,
        info,
    })
}

fn is_fence_close(line: &str, fence: Fence<'_>) -> bool {
    let Some(line) = strip_block_indent(line) else {
        return false;
    };
    let length = line
        .bytes()
        .take_while(|byte| *byte == fence.marker)
        .count();
    length >= fence.length && line[length..].trim().is_empty()
}

fn heading(line: &str) -> Option<(u8, &str)> {
    let line = strip_block_indent(line)?;
    let level = line.bytes().take_while(|byte| *byte == b'#').count();
    if !(1..=6).contains(&level) {
        return None;
    }
    let rest = &line[level..];
    if !rest.is_empty() && !rest.starts_with(char::is_whitespace) {
        return None;
    }
    let mut content = rest.trim();
    let without_hashes = content.trim_end_matches('#');
    if without_hashes.len() != content.len()
        && without_hashes
            .chars()
            .last()
            .is_none_or(char::is_whitespace)
    {
        content = without_hashes.trim_end();
    }
    Some((level as u8, content))
}

fn is_thematic_break(line: &str) -> bool {
    let Some(line) = strip_block_indent(line) else {
        return false;
    };
    let mut marker = None;
    let mut count = 0;
    for character in line.chars() {
        if character.is_whitespace() {
            continue;
        }
        if !matches!(character, '-' | '*' | '_') {
            return false;
        }
        if marker.is_some_and(|current| current != character) {
            return false;
        }
        marker = Some(character);
        count += 1;
    }
    count >= 3
}

fn quote_content(line: &str) -> Option<&str> {
    let line = strip_block_indent(line)?;
    let content = line.strip_prefix('>')?;
    Some(content.strip_prefix(' ').unwrap_or(content))
}

fn list_marker(line: &str) -> Option<ListMarker<'_>> {
    let line = strip_block_indent(line)?;
    let bytes = line.as_bytes();
    match bytes.first().copied()? {
        b'-' | b'+' | b'*' if bytes.get(1).is_some_and(u8::is_ascii_whitespace) => {
            Some(ListMarker {
                ordered: false,
                number: 1,
                content: &line[2..],
            })
        }
        first if first.is_ascii_digit() => {
            let digits = bytes
                .iter()
                .take(9)
                .take_while(|byte| byte.is_ascii_digit())
                .count();
            let delimiter = *bytes.get(digits)?;
            if !matches!(delimiter, b'.' | b')')
                || !bytes.get(digits + 1).is_some_and(u8::is_ascii_whitespace)
            {
                return None;
            }
            let number = line[..digits].parse().ok()?;
            Some(ListMarker {
                ordered: true,
                number,
                content: &line[digits + 2..],
            })
        }
        _ => None,
    }
}

fn task_marker(content: &str) -> (Option<bool>, &str) {
    let bytes = content.as_bytes();
    if bytes.len() >= 3 && bytes[0] == b'[' && bytes[2] == b']' {
        let checked = match bytes[1] {
            b' ' => Some(false),
            b'x' | b'X' => Some(true),
            _ => None,
        };
        if checked.is_some()
            && (bytes.len() == 3 || bytes.get(3).is_some_and(u8::is_ascii_whitespace))
        {
            return (checked, content[3..].trim_start());
        }
    }
    (None, content)
}

fn leading_indent(line: &str) -> usize {
    line.chars()
        .take_while(|character| matches!(character, ' ' | '\t'))
        .map(|character| if character == '\t' { 4 } else { 1 })
        .sum()
}
