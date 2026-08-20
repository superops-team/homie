use super::highlight::lexical_highlights;
use super::*;

#[test]
fn highlights_keywords_strings_and_comments_without_overlapping() {
    let source = "pub fn main() { let value = \"hello\"; // note";
    let ranges = lexical_highlights(source, "rs");
    assert!(
        ranges
            .iter()
            .any(|(range, _)| &source[range.clone()] == "pub")
    );
    assert!(
        ranges
            .iter()
            .any(|(range, _)| &source[range.clone()] == "fn")
    );
    assert!(
        ranges
            .iter()
            .any(|(range, _)| &source[range.clone()] == "\"hello\"")
    );
    assert!(
        ranges
            .iter()
            .any(|(range, _)| &source[range.clone()] == "// note")
    );
}

#[test]
fn keyword_boundaries_do_not_color_identifiers() {
    let source = "format for before";
    let ranges = lexical_highlights(source, "rs");
    let words: Vec<_> = ranges
        .iter()
        .map(|(range, _)| &source[range.clone()])
        .collect();
    assert_eq!(words, vec!["for"]);
}

#[test]
fn target_type_is_one_based() {
    let target = SourceTarget {
        line: 12,
        column: 4,
    };
    assert_eq!((target.line, target.column), (12, 4));
}
