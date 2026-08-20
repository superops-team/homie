use super::*;

#[test]
fn manifest_ids_have_readable_fallback_labels() {
    assert_eq!(title_case_id("claude-code"), "Claude Code");
    assert_eq!(title_case_id("open_code"), "Open Code");
}
