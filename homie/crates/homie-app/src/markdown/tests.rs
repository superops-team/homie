use super::*;

fn paragraph(document: &MarkdownDocument, index: usize) -> &InlineText {
    match &document.blocks[index] {
        MarkdownBlock::Paragraph(content) => content,
        block => panic!("expected paragraph, got {block:?}"),
    }
}

#[test]
fn parses_headings_paragraph_continuations_and_rules_as_blocks() {
    let document = MarkdownDocument::parse(
        "# Review cockpit\n\nFirst line\ncontinues here.\n\n- - -\n\n###### Small ######",
    );

    assert!(!document.truncated);
    assert_eq!(document.blocks.len(), 4);
    assert_eq!(
        document.blocks[0],
        MarkdownBlock::Heading {
            level: 1,
            content: parse_inline("Review cockpit"),
        }
    );
    assert_eq!(
        paragraph(&document, 1).plain_text(),
        "First line continues here."
    );
    assert_eq!(document.blocks[2], MarkdownBlock::ThematicBreak);
    assert_eq!(
        document.blocks[3],
        MarkdownBlock::Heading {
            level: 6,
            content: parse_inline("Small"),
        }
    );
}

#[test]
fn parses_unordered_ordered_and_task_lists_with_continuations() {
    let document = MarkdownDocument::parse(
        "- plain\n- [ ] waiting\n- [X] shipped\n  with details\n\n3. third\n4) fourth",
    );

    let MarkdownBlock::List {
        ordered,
        start,
        items,
    } = &document.blocks[0]
    else {
        panic!("unordered list")
    };
    assert!(!ordered);
    assert_eq!(*start, 1);
    assert_eq!(items.len(), 3);
    assert_eq!(items[0].checked, None);
    assert_eq!(items[1].checked, Some(false));
    assert_eq!(items[2].checked, Some(true));
    assert_eq!(items[2].content.plain_text(), "shipped with details");

    let MarkdownBlock::List {
        ordered,
        start,
        items,
    } = &document.blocks[1]
    else {
        panic!("ordered list")
    };
    assert!(ordered);
    assert_eq!(*start, 3);
    assert_eq!(items.len(), 2);
}

#[test]
fn parses_nested_quotes_into_nested_documents() {
    let document = MarkdownDocument::parse(
        "> ## Reviewer\n> First paragraph\n> continues.\n>\n> > Nested **answer**",
    );
    let MarkdownBlock::Quote(outer) = &document.blocks[0] else {
        panic!("outer quote")
    };
    assert!(matches!(
        outer.first(),
        Some(MarkdownBlock::Heading { level: 2, .. })
    ));
    assert_eq!(
        match &outer[1] {
            MarkdownBlock::Paragraph(text) => text.plain_text(),
            block => panic!("paragraph, got {block:?}"),
        },
        "First paragraph continues."
    );
    assert!(matches!(outer[2], MarkdownBlock::Quote(_)));
}

#[test]
fn fenced_code_preserves_source_and_language_without_inline_parsing() {
    let document = MarkdownDocument::parse(
        "```rust\nlet marker = \"**not bold**\";\nprintln!(\"{marker}\");\n```",
    );
    assert_eq!(
        document.blocks,
        vec![MarkdownBlock::CodeBlock {
            language: Some("rust".to_owned()),
            code: "let marker = \"**not bold**\";\nprintln!(\"{marker}\");".to_owned(),
        }]
    );
}

#[test]
fn parses_inline_emphasis_code_strike_and_nested_styles() {
    let document = MarkdownDocument::parse(
        "Normal **bold and _italic_** plus *soft*, ~~gone~~, and `a **literal**`.",
    );
    let spans = &paragraph(&document, 0).spans;
    assert_eq!(
        paragraph(&document, 0).plain_text(),
        "Normal bold and italic plus soft, gone, and a **literal**."
    );
    assert!(
        spans
            .iter()
            .any(|span| span.text == "bold and " && span.style.bold)
    );
    assert!(
        spans
            .iter()
            .any(|span| { span.text == "italic" && span.style.bold && span.style.italic })
    );
    assert!(
        spans
            .iter()
            .any(|span| span.text == "soft" && span.style.italic)
    );
    assert!(
        spans
            .iter()
            .any(|span| span.text == "gone" && span.style.strikethrough)
    );
    assert!(
        spans
            .iter()
            .any(|span| span.text == "a **literal**" && span.style.code)
    );
}

#[test]
fn keeps_link_labels_but_sanitizes_activation_data() {
    let document = MarkdownDocument::parse(
        "[Web](https://example.com/a) [Local](file:///tmp/a.rs) [Bad](javascript:alert(1)) [Relative](../README.md)",
    );
    let spans = &paragraph(&document, 0).spans;
    assert_eq!(
        paragraph(&document, 0).plain_text(),
        "Web Local Bad Relative"
    );
    assert_eq!(
        spans.iter().find(|span| span.text == "Web").unwrap().link,
        Some("https://example.com/a".to_owned())
    );
    assert_eq!(
        spans.iter().find(|span| span.text == "Local").unwrap().link,
        Some("file:///tmp/a.rs".to_owned())
    );
    assert_eq!(
        spans
            .iter()
            .filter_map(|span| span.link.as_deref())
            .collect::<Vec<_>>(),
        vec!["https://example.com/a", "file:///tmp/a.rs"]
    );

    let unsafe_document =
        MarkdownDocument::parse("[Bad](javascript:alert(1)) [Relative](../README.md)");
    assert_eq!(paragraph(&unsafe_document, 0).plain_text(), "Bad Relative");
    assert!(
        paragraph(&unsafe_document, 0)
            .spans
            .iter()
            .all(|span| span.link.is_none())
    );
}

#[test]
fn parses_angle_and_bare_autolinks_without_swallowing_punctuation() {
    let document = MarkdownDocument::parse(
        "See <https://example.com/docs> and https://example.com/path?q=1. Email <dev@example.com>.",
    );
    let content = paragraph(&document, 0);
    assert_eq!(
        content.plain_text(),
        "See https://example.com/docs and https://example.com/path?q=1. Email dev@example.com."
    );
    let links = content
        .spans
        .iter()
        .filter_map(|span| span.link.as_deref())
        .collect::<Vec<_>>();
    assert_eq!(
        links,
        vec!["https://example.com/docs", "https://example.com/path?q=1"]
    );
}

#[test]
fn raw_html_is_retained_as_text_and_images_become_alt_text() {
    let document = MarkdownDocument::parse(
        "<section class=\"note\">hello</section> ![diagram](https://example.com/x.png)",
    );
    assert_eq!(
        paragraph(&document, 0).plain_text(),
        "<section class=\"note\">hello</section> diagram"
    );
    assert!(
        paragraph(&document, 0)
            .spans
            .iter()
            .all(|span| span.link.is_none())
    );
}

#[test]
fn source_limit_respects_utf8_and_adds_a_visible_notice() {
    let mut source = "a".repeat(MAX_SOURCE_BYTES - 1);
    source.push('🛠');
    source.push_str("tail");
    let document = MarkdownDocument::parse(&source);

    assert!(document.truncated);
    assert_eq!(
        paragraph(&document, document.blocks.len() - 1).plain_text(),
        TRUNCATION_NOTICE
    );
    assert!(document.plain_text().contains(TRUNCATION_NOTICE));
}

#[test]
fn plain_text_keeps_block_list_quote_and_code_structure() {
    let document = MarkdownDocument::parse(
        "# Summary\n\n- [x] Tests\n- Review\n\n> Important\n\n```text\nline one\nline two\n```",
    );
    assert_eq!(
        document.plain_text(),
        "Summary\n\n- [x] Tests\n- Review\n\n> Important\n\nline one\nline two"
    );
}

#[test]
fn malformed_inline_syntax_remains_readable_literal_text() {
    let document =
        MarkdownDocument::parse("Unclosed **bold and [link](missing plus \\*escaped stars\\*.");
    assert_eq!(
        paragraph(&document, 0).plain_text(),
        "Unclosed **bold and [link](missing plus *escaped stars*."
    );
}
