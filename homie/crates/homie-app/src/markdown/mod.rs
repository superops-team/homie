//! Bounded, presentation-neutral Markdown parsing for native artifact views.
//!
//! The inspector consumes a small owned document model. Markdown syntax,
//! hostile-input limits, and link activation policy stay inside this module;
//! no renderer needs to interpret source text or decide whether a URL is safe.

const MAX_SOURCE_BYTES: usize = 256 * 1024;
const MAX_BLOCKS: usize = 8_192;
const MAX_LIST_ITEMS: usize = 4_096;
const MAX_QUOTE_DEPTH: usize = 24;
const MAX_INLINE_DEPTH: usize = 32;
const TRUNCATION_NOTICE: &str = "[Markdown truncated at 256 KiB]";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MarkdownDocument {
    pub blocks: Vec<MarkdownBlock>,
    pub truncated: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MarkdownBlock {
    Heading {
        level: u8,
        content: InlineText,
    },
    Paragraph(InlineText),
    List {
        ordered: bool,
        start: usize,
        items: Vec<MarkdownListItem>,
    },
    Quote(Vec<MarkdownBlock>),
    CodeBlock {
        language: Option<String>,
        code: String,
    },
    ThematicBreak,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MarkdownListItem {
    pub checked: Option<bool>,
    pub content: InlineText,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct InlineText {
    pub spans: Vec<MarkdownSpan>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MarkdownSpan {
    pub text: String,
    pub style: InlineStyle,
    pub link: Option<String>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct InlineStyle {
    pub bold: bool,
    pub italic: bool,
    pub code: bool,
    pub strikethrough: bool,
}

impl MarkdownDocument {
    #[must_use]
    pub fn parse(source: &str) -> Self {
        let (source, source_truncated) = bounded_source(source);
        let lines = source.lines().collect::<Vec<_>>();
        let parsed = parse_blocks(&lines, 0);
        let truncated = source_truncated || parsed.limited;
        let mut blocks = parsed.blocks;
        if truncated {
            blocks.push(MarkdownBlock::Paragraph(parse_inline(TRUNCATION_NOTICE)));
        }
        Self { blocks, truncated }
    }

    /// A structure-preserving text projection for agent context and copying.
    #[must_use]
    pub fn plain_text(&self) -> String {
        blocks_plain_text(&self.blocks)
    }
}

impl InlineText {
    #[must_use]
    pub fn plain_text(&self) -> String {
        self.spans.iter().map(|span| span.text.as_str()).collect()
    }
}

struct ParsedBlocks {
    blocks: Vec<MarkdownBlock>,
    limited: bool,
}

#[derive(Clone, Copy)]
struct Fence<'a> {
    marker: u8,
    length: usize,
    info: &'a str,
}

#[derive(Clone, Copy)]
struct ListMarker<'a> {
    ordered: bool,
    number: usize,
    content: &'a str,
}

fn bounded_source(source: &str) -> (&str, bool) {
    if source.len() <= MAX_SOURCE_BYTES {
        return (source, false);
    }
    let mut end = MAX_SOURCE_BYTES;
    while !source.is_char_boundary(end) {
        end -= 1;
    }
    (&source[..end], true)
}

mod block;
mod inline;
#[cfg(test)]
mod tests;

use block::parse_blocks;
use inline::parse_inline;

fn blocks_plain_text(blocks: &[MarkdownBlock]) -> String {
    blocks
        .iter()
        .map(block_plain_text)
        .filter(|text| !text.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn block_plain_text(block: &MarkdownBlock) -> String {
    match block {
        MarkdownBlock::Heading { content, .. } | MarkdownBlock::Paragraph(content) => {
            content.plain_text()
        }
        MarkdownBlock::List {
            ordered,
            start,
            items,
        } => items
            .iter()
            .enumerate()
            .map(|(index, item)| {
                let marker = if *ordered {
                    format!("{}. ", start.saturating_add(index))
                } else {
                    "- ".to_owned()
                };
                let task = match item.checked {
                    Some(true) => "[x] ",
                    Some(false) => "[ ] ",
                    None => "",
                };
                format!("{marker}{task}{}", item.content.plain_text())
            })
            .collect::<Vec<_>>()
            .join("\n"),
        MarkdownBlock::Quote(blocks) => blocks_plain_text(blocks)
            .lines()
            .map(|line| {
                if line.is_empty() {
                    ">".to_owned()
                } else {
                    format!("> {line}")
                }
            })
            .collect::<Vec<_>>()
            .join("\n"),
        MarkdownBlock::CodeBlock { code, .. } => code.clone(),
        MarkdownBlock::ThematicBreak => "---".to_owned(),
    }
}
