use super::*;
use std::process::Command;

fn write(path: &Path, contents: impl AsRef<[u8]>) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, contents).unwrap();
}

fn workspace() -> tempfile::TempDir {
    let temporary = tempfile::tempdir().unwrap();
    fs::create_dir(temporary.path().join(".git")).unwrap();
    temporary
}

#[test]
fn resolves_common_terminal_reference_formats() {
    let workspace = workspace();
    let source = workspace.path().join("src/main.rs");
    write(&source, "fn main() {}\n");
    fs::create_dir_all(workspace.path().join("nested")).unwrap();
    let intelligence = CodeIntelligence::for_session(workspace.path().join("nested")).unwrap();

    let relative = intelligence
        .resolve_reference("  --> [src/main.rs:12:3],")
        .unwrap();
    assert_eq!(relative.relative_path, Path::new("src/main.rs"));
    assert_eq!(
        relative.target,
        Some(SourceTarget {
            line: 12,
            column: 3
        })
    );

    let absolute = intelligence
        .resolve_reference(&format!("{}:7", source.display()))
        .unwrap();
    assert_eq!(absolute.target, Some(SourceTarget { line: 7, column: 1 }));

    let uri = intelligence
        .resolve_reference(&format!("file://{}#L4C2", source.display()))
        .unwrap();
    assert_eq!(uri.target, Some(SourceTarget { line: 4, column: 2 }));

    let stack = intelligence
        .resolve_reference(&format!("at render ({}(9,5))", source.display()))
        .unwrap();
    assert_eq!(stack.target, Some(SourceTarget { line: 9, column: 5 }));
}

#[test]
fn decodes_file_uris_and_rejects_traversal_and_symlink_escapes() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("workspace");
    fs::create_dir_all(root.join(".git")).unwrap();
    write(&root.join("space name.rs"), "fn safe() {}\n");
    write(&parent.path().join("secret.rs"), "secret\n");
    let intelligence = CodeIntelligence::for_session(&root).unwrap();

    let encoded = root.join("space%20name.rs");
    let resolved = intelligence
        .resolve_reference(&format!("file://{}#L1", encoded.display()))
        .unwrap();
    assert_eq!(resolved.relative_path, Path::new("space name.rs"));

    assert!(matches!(
        intelligence.resolve_reference("../secret.rs:1"),
        Err(CodeIntelligenceError::OutsideWorkspace { .. })
    ));

    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(parent.path().join("secret.rs"), root.join("escape.rs"))
            .unwrap();
        assert!(matches!(
            intelligence.open_reference("escape.rs"),
            Err(CodeIntelligenceError::OutsideWorkspace { .. })
        ));
    }
}

#[test]
fn builds_a_git_aware_index_and_ignores_dependency_and_build_directories() {
    let workspace = tempfile::tempdir().unwrap();
    let status = Command::new("git")
        .args(["init", "-q"])
        .current_dir(workspace.path())
        .status()
        .unwrap();
    assert!(status.success());
    write(&workspace.path().join(".gitignore"), "ignored.rs\n");
    write(&workspace.path().join("src/main.rs"), "fn main() {}\n");
    write(
        &workspace.path().join("src/lib.rs"),
        "pub struct Library;\n",
    );
    write(&workspace.path().join("ignored.rs"), "ignored\n");
    write(&workspace.path().join("vendor/crate.rs"), "vendor\n");
    write(&workspace.path().join("build/output.rs"), "build\n");
    write(
        &workspace.path().join("node_modules/pkg/index.js"),
        "package\n",
    );

    let intelligence = CodeIntelligence::for_session(workspace.path()).unwrap();
    let hits = intelligence.search("", 100);
    let paths: Vec<_> = hits
        .iter()
        .map(|hit| hit.relative_path.to_string_lossy())
        .collect();
    assert!(paths.iter().any(|path| path == "src/main.rs"));
    assert!(paths.iter().any(|path| path == "src/lib.rs"));
    assert!(!paths.iter().any(|path| path == "ignored.rs"));
    assert!(!paths.iter().any(|path| path.starts_with("vendor/")));
    assert!(!paths.iter().any(|path| path.starts_with("build/")));
    assert!(!paths.iter().any(|path| path.starts_with("node_modules/")));
}

#[test]
fn ranks_exact_symbols_and_file_names_above_loose_matches() {
    let workspace = workspace();
    write(
        &workspace.path().join("src/code_intelligence.rs"),
        "pub struct CodeIntelligence;\nfn render_viewer() {}\n",
    );
    write(&workspace.path().join("docs/codebook.md"), "# Codebook\n");
    let intelligence = CodeIntelligence::for_session(workspace.path()).unwrap();

    let symbols = intelligence.search("CodeIntelligence", 10);
    assert_eq!(symbols[0].kind, SearchHitKind::Symbol);
    assert_eq!(symbols[0].line, Some(1));
    assert_eq!(
        symbols[0].relative_path,
        Path::new("src/code_intelligence.rs")
    );

    let files = intelligence.search("code intel", 10);
    assert_eq!(files[0].kind, SearchHitKind::File);
    assert_eq!(
        files[0].relative_path,
        Path::new("src/code_intelligence.rs")
    );

    let function = intelligence.search("render_viewer", 10);
    assert_eq!(function[0].kind, SearchHitKind::Symbol);
    assert_eq!(function[0].line, Some(2));
}

#[test]
fn rejects_binary_invalid_utf8_and_oversize_files() {
    let workspace = workspace();
    write(&workspace.path().join("binary.dat"), [b'a', 0, b'b']);
    write(&workspace.path().join("invalid.txt"), [0xff, 0xfe]);
    write(
        &workspace.path().join("large.txt"),
        vec![b'x'; MAX_SOURCE_BYTES as usize + 1],
    );
    let intelligence = CodeIntelligence::for_session(workspace.path()).unwrap();

    assert!(matches!(
        intelligence.open_reference("binary.dat"),
        Err(CodeIntelligenceError::BinaryFile { .. })
    ));
    assert!(matches!(
        intelligence.open_reference("invalid.txt"),
        Err(CodeIntelligenceError::NotUtf8 { .. })
    ));
    assert!(matches!(
        intelligence.open_reference("large.txt"),
        Err(CodeIntelligenceError::TooLarge { .. })
    ));
}

#[test]
fn source_snapshot_has_byte_ranges_and_clamped_character_targets() {
    let workspace = workspace();
    write(&workspace.path().join("source.rs"), "one\r\nhéllo\nlast\n");
    let intelligence = CodeIntelligence::for_session(workspace.path()).unwrap();

    let snapshot = intelligence.open_reference("source.rs:2:99").unwrap();
    assert_eq!(snapshot.lines.len(), 4);
    assert_eq!(snapshot.lines[0].range, 0..3);
    assert_eq!(&snapshot.text[snapshot.lines[1].range.clone()], "héllo");
    assert_eq!(snapshot.target, Some(SourceTarget { line: 2, column: 6 }));

    let clamped = intelligence.open_reference("source.rs:999:999").unwrap();
    assert_eq!(clamped.target, Some(SourceTarget { line: 4, column: 1 }));
}

#[test]
fn no_reference_and_directories_have_specific_errors() {
    let workspace = workspace();
    fs::create_dir_all(workspace.path().join("src")).unwrap();
    let intelligence = CodeIntelligence::for_session(workspace.path()).unwrap();
    assert!(matches!(
        intelligence.resolve_reference("ordinary terminal output"),
        Err(CodeIntelligenceError::NoFileReference { .. })
    ));
    assert!(matches!(
        intelligence.open_reference("./src"),
        Err(CodeIntelligenceError::NotAFile { .. })
    ));
}
