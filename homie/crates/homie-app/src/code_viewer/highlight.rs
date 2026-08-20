use super::*;

pub(super) fn source_row(
    snapshot: &SourceSnapshot,
    index: usize,
    targeted: bool,
    extension: &str,
    content_width: f32,
    colors: SemanticColors,
) -> AnyElement {
    let line = &snapshot.lines[index];
    let source = snapshot.text[line.range.clone()]
        .trim_end_matches(['\r', '\n'])
        .to_owned();
    let styled = highlighted_source(source, extension);
    div()
        .id(index)
        .h(px(SOURCE_ROW_HEIGHT))
        .min_w(px(content_width))
        .w_full()
        .flex()
        .items_center()
        .bg(if targeted {
            rgba(0xd977571c)
        } else {
            colors.background
        })
        .when(targeted, |row| {
            row.border_l_2().border_color(rgba(0xd97757ff))
        })
        .child(
            div()
                .w(px(SOURCE_GUTTER_WIDTH))
                .h_full()
                .flex_none()
                .pr(px(10.0))
                .flex()
                .items_center()
                .justify_end()
                .border_r_1()
                .border_color(colors.primary.alpha(0.055))
                .font_family(crate::fonts::mono_family())
                .text_size(px(10.0))
                .text_color(if targeted {
                    rgba(0xd97757ff)
                } else {
                    colors.primary.alpha(0.26)
                })
                .child(line.number.to_string()),
        )
        .child(
            div()
                .h_full()
                .min_w(px(0.0))
                .pl(px(10.0))
                .flex()
                .items_center()
                .font_family(crate::fonts::mono_family())
                .text_size(px(11.5))
                .text_color(rgba(0xd8dee9ff))
                .child(styled),
        )
        .into_any_element()
}

fn highlighted_source(source: String, extension: &str) -> AnyElement {
    let ranges = lexical_highlights(&source, extension);
    if ranges.is_empty() {
        return div().child(source).into_any_element();
    }
    StyledText::new(source)
        .with_highlights(ranges)
        .into_any_element()
}

pub(super) fn lexical_highlights(
    source: &str,
    extension: &str,
) -> Vec<(Range<usize>, HighlightStyle)> {
    let mut ranges = Vec::new();
    let comment_start = match extension {
        "py" | "rb" | "sh" | "bash" | "zsh" | "fish" | "toml" | "yaml" | "yml" => source.find('#'),
        "sql" => source.find("--"),
        _ => source.find("//"),
    };
    let code_end = comment_start.unwrap_or(source.len());
    if let Some(start) = comment_start {
        ranges.push((
            start..source.len(),
            HighlightStyle {
                color: Some(rgba(0x718096ff).into()),
                font_style: Some(gpui::FontStyle::Italic),
                ..HighlightStyle::default()
            },
        ));
    }

    let bytes = source.as_bytes();
    let mut cursor = 0;
    while cursor < code_end {
        let quote = bytes[cursor];
        if quote != b'"' && quote != b'\'' && quote != b'`' {
            cursor += 1;
            continue;
        }
        let start = cursor;
        cursor += 1;
        while cursor < code_end {
            if bytes[cursor] == b'\\' {
                cursor = (cursor + 2).min(code_end);
            } else if bytes[cursor] == quote {
                cursor += 1;
                break;
            } else {
                cursor += 1;
            }
        }
        ranges.push((
            start..cursor,
            HighlightStyle {
                color: Some(rgba(0xd7ba7dff).into()),
                ..HighlightStyle::default()
            },
        ));
    }

    let keywords = match extension {
        "rs" => RUST_KEYWORDS,
        "swift" => SWIFT_KEYWORDS,
        "py" => PYTHON_KEYWORDS,
        "js" | "jsx" | "ts" | "tsx" => JS_KEYWORDS,
        _ => COMMON_KEYWORDS,
    };
    for keyword in keywords {
        for (start, _) in source[..code_end].match_indices(keyword) {
            let end = start + keyword.len();
            let left_ok = start == 0 || !is_ident(source.as_bytes()[start - 1]);
            let right_ok = end == code_end || !is_ident(source.as_bytes()[end]);
            if left_ok && right_ok && !ranges.iter().any(|(range, _)| range.contains(&start)) {
                ranges.push((
                    start..end,
                    HighlightStyle {
                        color: Some(rgba(0xc792eaff).into()),
                        font_weight: Some(FontWeight::MEDIUM),
                        ..HighlightStyle::default()
                    },
                ));
            }
        }
    }
    ranges.sort_by_key(|(range, _)| range.start);
    ranges
}

const fn is_ident(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

const RUST_KEYWORDS: &[&str] = &[
    "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern",
    "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub",
    "ref", "return", "self", "Self", "static", "struct", "super", "trait", "true", "type",
    "unsafe", "use", "where", "while",
];
const SWIFT_KEYWORDS: &[&str] = &[
    "actor",
    "async",
    "await",
    "case",
    "class",
    "defer",
    "else",
    "enum",
    "extension",
    "false",
    "for",
    "func",
    "guard",
    "if",
    "import",
    "in",
    "init",
    "let",
    "nil",
    "protocol",
    "return",
    "self",
    "static",
    "struct",
    "switch",
    "throw",
    "true",
    "try",
    "var",
    "while",
];
const PYTHON_KEYWORDS: &[&str] = &[
    "and", "as", "assert", "async", "await", "break", "class", "continue", "def", "del", "elif",
    "else", "except", "False", "finally", "for", "from", "global", "if", "import", "in", "is",
    "lambda", "None", "not", "or", "pass", "raise", "return", "True", "try", "while", "with",
    "yield",
];
const JS_KEYWORDS: &[&str] = &[
    "async",
    "await",
    "break",
    "case",
    "catch",
    "class",
    "const",
    "continue",
    "default",
    "delete",
    "do",
    "else",
    "export",
    "extends",
    "false",
    "finally",
    "for",
    "from",
    "function",
    "if",
    "import",
    "in",
    "instanceof",
    "let",
    "new",
    "null",
    "of",
    "return",
    "static",
    "super",
    "switch",
    "this",
    "throw",
    "true",
    "try",
    "typeof",
    "undefined",
    "var",
    "while",
    "yield",
];
const COMMON_KEYWORDS: &[&str] = &[
    "class", "const", "else", "enum", "false", "for", "function", "if", "import", "let", "null",
    "return", "static", "struct", "true", "type", "var", "while",
];
