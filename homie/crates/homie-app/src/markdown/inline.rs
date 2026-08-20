use super::*;

pub(super) fn parse_inline(source: &str) -> InlineText {
    let mut spans = Vec::new();
    parse_inline_into(source, InlineStyle::default(), None, 0, &mut spans);
    InlineText { spans }
}

fn parse_inline_into(
    source: &str,
    style: InlineStyle,
    inherited_link: Option<&str>,
    depth: usize,
    spans: &mut Vec<MarkdownSpan>,
) {
    if source.is_empty() {
        return;
    }
    if depth >= MAX_INLINE_DEPTH {
        push_span(spans, source, style, inherited_link);
        return;
    }

    let mut index = 0;
    while index < source.len() {
        let rest = &source[index..];

        if let Some(escaped) = escaped_character(rest) {
            push_span(spans, &escaped.to_string(), style, inherited_link);
            index += 1 + escaped.len_utf8();
            continue;
        }

        if rest.starts_with('`') {
            let ticks = rest.bytes().take_while(|byte| *byte == b'`').count();
            let delimiter = "`".repeat(ticks);
            if let Some(close) = find_unescaped(&rest[ticks..], &delimiter) {
                let mut code = &rest[ticks..ticks + close];
                if code.starts_with(' ')
                    && code.ends_with(' ')
                    && code.len() > 2
                    && !code.trim().is_empty()
                {
                    code = &code[1..code.len() - 1];
                }
                let mut code_style = style;
                code_style.code = true;
                push_span(spans, code, code_style, inherited_link);
                index += ticks + close + ticks;
                continue;
            }
        }

        if rest.starts_with("![")
            && let Some((label, _destination, consumed)) = parse_link(&rest[1..])
        {
            // Images are deliberately presentation-neutral and never fetched;
            // retain their useful alt text as ordinary inline content.
            parse_inline_into(label, style, inherited_link, depth + 1, spans);
            index += 1 + consumed;
            continue;
        }

        if rest.starts_with('[')
            && let Some((label, destination, consumed)) = parse_link(rest)
        {
            let link = sanitize_link(destination);
            parse_inline_into(
                label,
                style,
                link.as_deref().or(inherited_link),
                depth + 1,
                spans,
            );
            index += consumed;
            continue;
        }

        if rest.starts_with('<')
            && let Some(close) = rest[1..].find('>')
        {
            let candidate = &rest[1..1 + close];
            if looks_like_autolink(candidate) {
                let link = sanitize_link(candidate);
                push_span(spans, candidate, style, link.as_deref().or(inherited_link));
                index += close + 2;
                continue;
            }
        }

        if let Some((delimiter, update)) = style_delimiter(rest) {
            let after_open = &rest[delimiter.len()..];
            if let Some(close) = find_style_close(after_open, delimiter) {
                let inner = &after_open[..close];
                if !inner.is_empty() {
                    let mut nested = style;
                    update(&mut nested);
                    parse_inline_into(inner, nested, inherited_link, depth + 1, spans);
                    index += delimiter.len() + close + delimiter.len();
                    continue;
                }
            }
        }

        if inherited_link.is_none()
            && (index == 0
                || source[..index]
                    .chars()
                    .next_back()
                    .is_none_or(|character| !character.is_alphanumeric() && character != '_'))
            && let Some((url, consumed)) = bare_url(rest)
        {
            let link = sanitize_link(url);
            push_span(spans, url, style, link.as_deref());
            index += consumed;
            continue;
        }

        let character = rest.chars().next().expect("non-empty remainder");
        push_span(spans, &character.to_string(), style, inherited_link);
        index += character.len_utf8();
    }
}

fn escaped_character(source: &str) -> Option<char> {
    let mut characters = source.chars();
    if characters.next()? != '\\' {
        return None;
    }
    let escaped = characters.next()?;
    escaped.is_ascii_punctuation().then_some(escaped)
}

type StyleUpdate = fn(&mut InlineStyle);

fn style_delimiter(source: &str) -> Option<(&'static str, StyleUpdate)> {
    fn bold(style: &mut InlineStyle) {
        style.bold = true;
    }
    fn italic(style: &mut InlineStyle) {
        style.italic = true;
    }
    fn strike(style: &mut InlineStyle) {
        style.strikethrough = true;
    }

    if source.starts_with("**") {
        Some(("**", bold))
    } else if source.starts_with("__") {
        Some(("__", bold))
    } else if source.starts_with("~~") {
        Some(("~~", strike))
    } else if source.starts_with('*') {
        Some(("*", italic))
    } else if source.starts_with('_')
        && source[1..]
            .chars()
            .next()
            .is_some_and(|character| !character.is_whitespace())
    {
        Some(("_", italic))
    } else {
        None
    }
}

fn find_style_close(source: &str, delimiter: &str) -> Option<usize> {
    let mut offset = 0;
    while let Some(relative) = source[offset..].find(delimiter) {
        let candidate = offset + relative;
        if candidate > 0
            && !is_escaped(source, candidate)
            && source[..candidate]
                .chars()
                .next_back()
                .is_some_and(|character| !character.is_whitespace())
        {
            return Some(candidate);
        }
        offset = candidate + delimiter.len();
        if offset >= source.len() {
            return None;
        }
    }
    None
}

fn find_unescaped(source: &str, needle: &str) -> Option<usize> {
    let mut offset = 0;
    while let Some(relative) = source[offset..].find(needle) {
        let candidate = offset + relative;
        if !is_escaped(source, candidate) {
            return Some(candidate);
        }
        offset = candidate + needle.len();
    }
    None
}

fn is_escaped(source: &str, index: usize) -> bool {
    source.as_bytes()[..index]
        .iter()
        .rev()
        .take_while(|byte| **byte == b'\\')
        .count()
        % 2
        == 1
}

fn parse_link(source: &str) -> Option<(&str, &str, usize)> {
    if !source.starts_with('[') {
        return None;
    }
    let label_end = matching_bracket(source, 1)?;
    let destination_open = label_end + 1;
    if source.as_bytes().get(destination_open) != Some(&b'(') {
        return None;
    }
    let destination_end = matching_parenthesis(source, destination_open + 1)?;
    let label = &source[1..label_end];
    let raw_destination = source[destination_open + 1..destination_end].trim();
    let destination = link_destination(raw_destination);
    Some((label, destination, destination_end + 1))
}

fn matching_bracket(source: &str, mut index: usize) -> Option<usize> {
    let mut nested = 0;
    while index < source.len() {
        let character = source[index..].chars().next()?;
        if !is_escaped(source, index) {
            match character {
                '[' => nested += 1,
                ']' if nested == 0 => return Some(index),
                ']' => nested -= 1,
                _ => {}
            }
        }
        index += character.len_utf8();
    }
    None
}

fn matching_parenthesis(source: &str, mut index: usize) -> Option<usize> {
    let mut nested = 0;
    let mut angle_destination = false;
    let mut quoted = None;
    while index < source.len() {
        let character = source[index..].chars().next()?;
        if is_escaped(source, index) {
            index += character.len_utf8();
            continue;
        }
        if let Some(quote) = quoted {
            if character == quote {
                quoted = None;
            }
        } else {
            match character {
                '<' if nested == 0 => angle_destination = true,
                '>' if angle_destination => angle_destination = false,
                '"' | '\'' if !angle_destination => quoted = Some(character),
                '(' if !angle_destination => nested += 1,
                ')' if !angle_destination && nested == 0 => return Some(index),
                ')' if !angle_destination => nested -= 1,
                _ => {}
            }
        }
        index += character.len_utf8();
    }
    None
}

fn link_destination(source: &str) -> &str {
    if let Some(inner) = source
        .strip_prefix('<')
        .and_then(|value| value.split_once('>'))
    {
        return inner.0.trim();
    }
    let end = source
        .char_indices()
        .find(|(index, character)| character.is_whitespace() && !is_escaped(source, *index))
        .map_or(source.len(), |(index, _)| index);
    source[..end].trim()
}

fn looks_like_autolink(candidate: &str) -> bool {
    !candidate.contains(char::is_whitespace)
        && (candidate.contains("://")
            || (candidate.contains('@') && candidate.rsplit_once('.').is_some()))
}

fn sanitize_link(destination: &str) -> Option<String> {
    let destination = destination.trim();
    if destination.is_empty()
        || destination
            .chars()
            .any(|character| character.is_control() || character.is_whitespace())
    {
        return None;
    }
    let (scheme, remainder) = destination.split_once(':')?;
    if !matches_ignore_ascii_case(scheme, &["http", "https", "file"])
        || !remainder.starts_with("//")
    {
        return None;
    }
    Some(destination.to_owned())
}

fn matches_ignore_ascii_case(value: &str, candidates: &[&str]) -> bool {
    candidates
        .iter()
        .any(|candidate| value.eq_ignore_ascii_case(candidate))
}

fn bare_url(source: &str) -> Option<(&str, usize)> {
    if !["http://", "https://", "file://"].iter().any(|prefix| {
        source
            .get(..prefix.len())
            .is_some_and(|start| start.eq_ignore_ascii_case(prefix))
    }) {
        return None;
    }
    let mut end = source.len();
    for (index, character) in source.char_indices() {
        if character.is_whitespace() || matches!(character, '<' | '>' | '"' | '\'' | '`') {
            end = index;
            break;
        }
    }
    let candidate = &source[..end];
    let usable = trim_url_punctuation(candidate);
    if usable.is_empty() {
        return None;
    }
    Some((usable, usable.len()))
}

fn trim_url_punctuation(mut candidate: &str) -> &str {
    candidate = candidate.trim_end_matches(['.', ',', ';', ':', '!', '?']);
    for (open, close) in [('(', ')'), ('[', ']'), ('{', '}')] {
        while candidate.ends_with(close)
            && candidate
                .chars()
                .filter(|character| *character == close)
                .count()
                > candidate
                    .chars()
                    .filter(|character| *character == open)
                    .count()
        {
            candidate = &candidate[..candidate.len() - close.len_utf8()];
        }
    }
    candidate
}

fn push_span(spans: &mut Vec<MarkdownSpan>, text: &str, style: InlineStyle, link: Option<&str>) {
    if text.is_empty() {
        return;
    }
    if let Some(last) = spans.last_mut()
        && last.style == style
        && last.link.as_deref() == link
    {
        last.text.push_str(text);
        return;
    }
    spans.push(MarkdownSpan {
        text: text.to_owned(),
        style,
        link: link.map(str::to_owned),
    });
}
