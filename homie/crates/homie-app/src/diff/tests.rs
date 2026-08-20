use super::load::git_command;
use super::parse::{fnv1a64, parse_hunk_start};
use super::*;
use std::fs;
use std::process::Command;

#[test]
fn parses_files_hunks_counts_and_line_numbers() {
    let patch = "diff --git a/src/main.rs b/src/main.rs\nindex 111..222 100644\n--- a/src/main.rs\n+++ b/src/main.rs\n@@ -10,3 +10,4 @@ fn main() {\n same\n-old\n+new\n+extra\n";
    let snapshot = parse_unified_diff(patch);

    assert_eq!(snapshot.files, 1);
    assert_eq!(snapshot.additions, 2);
    assert_eq!(snapshot.deletions, 1);
    assert_eq!(snapshot.rows[0].kind, DiffRowKind::File);
    assert_eq!(snapshot.rows[0].text, "src/main.rs");
    assert_eq!(snapshot.rows[4].old_line, Some(11));
    assert_eq!(snapshot.rows[4].new_line, None);
    assert_eq!(snapshot.rows[5].old_line, None);
    assert_eq!(snapshot.rows[5].new_line, Some(11));
    assert_eq!(snapshot.rows[6].new_line, Some(12));

    let file = &snapshot.file_diffs[0];
    assert_eq!(file.path, Path::new("src/main.rs"));
    assert_eq!(file.row_range, 0..7);
    assert_eq!(file.additions, 2);
    assert_eq!(file.deletions, 1);
    assert_eq!(file.hunks.len(), 1);
    let hunk = &file.hunks[0];
    assert_eq!(hunk.header, "@@ -10,3 +10,4 @@ fn main() {");
    assert_eq!(hunk.row_range, 2..7);
    assert_eq!(hunk.old_start, Some(10));
    assert_eq!(hunk.new_start, Some(10));
    assert_eq!(hunk.additions, 2);
    assert_eq!(hunk.deletions, 1);
    assert_eq!(hunk.patch, patch.as_bytes());
    assert_eq!(hunk.fingerprint, fnv1a64(patch.as_bytes()));
    assert_ne!(hunk.fingerprint, 0);
}

#[test]
fn parses_single_line_hunk_ranges() {
    assert_eq!(parse_hunk_start("@@ -4 +8 @@"), (Some(4), Some(8)));
}

#[test]
fn empty_patch_is_an_empty_snapshot() {
    assert_eq!(parse_unified_diff(""), DiffSnapshot::default());
}

#[test]
fn daemon_diff_uses_the_local_parser_and_marks_truncation() {
    let snapshot = snapshot_from_read_diff(SessionReadDiffResult {
        patch: b"diff --git a/a.txt b/a.txt\n@@ -1 +1 @@\n-old\n+new\n".to_vec(),
        repo_root: "/srv/app".to_owned(),
        truncated: true,
        base_ref: Some("origin/main".to_owned()),
    });

    assert_eq!(snapshot.repo_root, PathBuf::from("/srv/app"));
    assert_eq!(snapshot.files, 1);
    assert_eq!(snapshot.additions, 1);
    assert_eq!(snapshot.deletions, 1);
    assert!(snapshot.truncated);
    assert_eq!(snapshot.base_ref.as_deref(), Some("origin/main"));
    assert_eq!(
        snapshot.rows.last().unwrap().text,
        "Diff truncated by the daemon"
    );
}

#[test]
fn internal_git_commands_pin_the_machine_readable_locale() {
    let command = git_command(Path::new("/tmp"));
    let environment = command
        .get_envs()
        .filter_map(|(key, value)| Some((key.to_str()?, value?.to_str()?)))
        .collect::<std::collections::HashMap<_, _>>();
    assert_eq!(environment.get("LC_ALL"), Some(&"C"));
    assert_eq!(environment.get("LANG"), Some(&"C"));
}

#[test]
fn loads_tracked_and_untracked_worktree_changes() {
    let directory = tempfile::tempdir().expect("temporary repository");
    let root = directory.path();
    run(root, &["init", "--quiet"]);
    fs::write(root.join("tracked.txt"), "before\n").unwrap();
    run(root, &["add", "tracked.txt"]);
    run(
        root,
        &[
            "-c",
            "user.name=homie tests",
            "-c",
            "user.email=homie@example.invalid",
            "commit",
            "--quiet",
            "-m",
            "fixture",
        ],
    );
    fs::write(root.join("tracked.txt"), "after\n").unwrap();
    fs::write(root.join("untracked.txt"), "new\n").unwrap();

    let snapshot =
        load_worktree_diff_against(root, SessionDiffBase::DefaultBranch).expect("worktree diff");

    assert_eq!(snapshot.repo_root, root.canonicalize().unwrap());
    assert_eq!(snapshot.files, 2);
    assert_eq!(snapshot.additions, 2);
    assert_eq!(snapshot.deletions, 1);
    assert!(
        snapshot
            .rows
            .iter()
            .any(|row| row.kind == DiffRowKind::File && row.text == "untracked.txt")
    );
}

#[test]
fn local_layers_separate_index_from_worktree_and_untracked_content() {
    let directory = tempfile::tempdir().expect("temporary repository");
    let root = directory.path();
    run(root, &["init", "--quiet"]);
    fs::write(root.join("staged.txt"), "staged base\n").unwrap();
    fs::write(root.join("working.txt"), "working base\n").unwrap();
    run(root, &["add", "--all"]);
    run(
        root,
        &[
            "-c",
            "user.name=homie tests",
            "-c",
            "user.email=homie@example.invalid",
            "commit",
            "--quiet",
            "-m",
            "fixture",
        ],
    );

    fs::write(root.join("staged.txt"), "staged change\n").unwrap();
    run(root, &["add", "staged.txt"]);
    fs::write(root.join("working.txt"), "working change\n").unwrap();
    fs::write(root.join("untracked.txt"), "new\n").unwrap();

    let staged = load_local_diff(root, DiffLayer::Staged).expect("staged lane");
    assert_eq!(staged.layer, DiffLayer::Staged);
    assert_eq!(staged.files, 1);
    assert_eq!(staged.file_diffs[0].path, Path::new("staged.txt"));

    let working = load_local_diff(root, DiffLayer::Working).expect("working lane");
    assert_eq!(working.layer, DiffLayer::Working);
    assert_eq!(working.files, 2);
    assert!(
        working
            .file_diffs
            .iter()
            .any(|file| file.path == Path::new("working.txt"))
    );
    assert!(
        working
            .file_diffs
            .iter()
            .any(|file| file.path == Path::new("untracked.txt"))
    );

    let branch = load_local_diff(root, DiffLayer::Branch).expect("branch lane");
    assert_eq!(branch.layer, DiffLayer::Branch);
    assert_eq!(branch.files, 3);
}

#[test]
fn loads_committed_branch_changes_against_main() {
    let directory = tempfile::tempdir().expect("temporary repository");
    let root = directory.path();
    run(root, &["init", "--quiet", "--initial-branch=main"]);
    fs::write(root.join("tracked.txt"), "on main\n").unwrap();
    run(root, &["add", "tracked.txt"]);
    run(
        root,
        &[
            "-c",
            "user.name=homie tests",
            "-c",
            "user.email=homie@example.invalid",
            "commit",
            "--quiet",
            "-m",
            "main fixture",
        ],
    );
    run(root, &["checkout", "--quiet", "-b", "feature"]);
    fs::write(root.join("tracked.txt"), "on feature\n").unwrap();
    run(root, &["add", "tracked.txt"]);
    run(
        root,
        &[
            "-c",
            "user.name=homie tests",
            "-c",
            "user.email=homie@example.invalid",
            "commit",
            "--quiet",
            "-m",
            "feature change",
        ],
    );

    let snapshot =
        load_worktree_diff_against(root, SessionDiffBase::DefaultBranch).expect("branch diff");

    assert_eq!(snapshot.files, 1);
    assert_eq!(snapshot.additions, 1);
    assert_eq!(snapshot.deletions, 1);
    assert_eq!(snapshot.base_ref.as_deref(), Some("main"));
}

#[test]
fn head_comparison_excludes_committed_branch_changes() {
    let directory = tempfile::tempdir().expect("temporary repository");
    let root = directory.path();
    run(root, &["init", "--quiet", "--initial-branch=main"]);
    fs::write(root.join("tracked.txt"), "on main\n").unwrap();
    run(root, &["add", "tracked.txt"]);
    run(
        root,
        &[
            "-c",
            "user.name=homie tests",
            "-c",
            "user.email=homie@example.invalid",
            "commit",
            "--quiet",
            "-m",
            "main fixture",
        ],
    );
    run(root, &["checkout", "--quiet", "-b", "feature"]);
    fs::write(root.join("tracked.txt"), "committed feature\n").unwrap();
    run(root, &["add", "tracked.txt"]);
    run(
        root,
        &[
            "-c",
            "user.name=homie tests",
            "-c",
            "user.email=homie@example.invalid",
            "commit",
            "--quiet",
            "-m",
            "feature change",
        ],
    );

    let clean = load_worktree_diff_against(root, SessionDiffBase::Head).expect("head diff");
    assert_eq!(clean.files, 0);
    assert_eq!(clean.base_ref.as_deref(), Some("HEAD"));

    fs::write(root.join("tracked.txt"), "working change\n").unwrap();
    let dirty = load_worktree_diff_against(root, SessionDiffBase::Head).expect("head diff");
    assert_eq!(dirty.files, 1);
    assert_eq!(dirty.additions, 1);
    assert_eq!(dirty.deletions, 1);
}

fn run(cwd: &Path, arguments: &[&str]) {
    let output = Command::new("git")
        .current_dir(cwd)
        .args(arguments)
        .output()
        .expect("git command");
    assert!(
        output.status.success(),
        "git {arguments:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
