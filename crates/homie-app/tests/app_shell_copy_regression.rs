#[test]
fn app_shell_does_not_show_implementation_roadmap_copy() {
    let source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/main.rs"),
    )
    .expect("read app source");

    for forbidden in [
        "Next implementation slices",
        "PTY-backed execution is the next runtime slice",
        "Virtual-key proxy and usage metrics are staged",
        "Preview shell keeps actions read-only",
        "live preview",
    ] {
        assert!(
            !source.contains(forbidden),
            "app shell still contains roadmap placeholder copy: {forbidden}"
        );
    }
}
