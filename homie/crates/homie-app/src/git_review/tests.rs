use super::status::parse_status;
use super::*;
use crate::diff::{DiffHunk, DiffLayer, load_local_diff};
use std::fs;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_REPO: AtomicU64 = AtomicU64::new(0);

struct TestRepo {
    path: PathBuf,
}

impl TestRepo {
    /// `None` is the intentional, clean skip path when Git is unavailable.
    fn new() -> Option<Self> {
        if !Command::new("git")
            .arg("--version")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|status| status.success())
        {
            eprintln!("skipping Git review test: git is unavailable");
            return None;
        }
        let ordinal = NEXT_REPO.fetch_add(1, Ordering::Relaxed);
        let path =
            std::env::temp_dir().join(format!("homie-git-review-{}-{ordinal}", std::process::id()));
        fs::create_dir(&path).expect("create test repository directory");
        let repo = Self { path };
        repo.git(["init", "--quiet"]);
        repo.git(["symbolic-ref", "HEAD", "refs/heads/main"]);
        repo.git(["config", "user.name", "Homie Test"]);
        repo.git(["config", "user.email", "homie@example.invalid"]);
        Some(repo)
    }

    fn git<const N: usize>(&self, args: [&str; N]) -> Vec<u8> {
        let output = Command::new("git")
            .current_dir(&self.path)
            .args(args)
            .stdin(Stdio::null())
            .env("GIT_TERMINAL_PROMPT", "0")
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .output()
            .expect("run test git");
        assert!(
            output.status.success(),
            "git failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        output.stdout
    }

    fn git_expect_failure<const N: usize>(&self, args: [&str; N]) {
        let output = Command::new("git")
            .current_dir(&self.path)
            .args(args)
            .stdin(Stdio::null())
            .env("GIT_TERMINAL_PROMPT", "0")
            .output()
            .expect("run test git");
        assert!(!output.status.success());
    }

    fn write(&self, path: &str, contents: &str) {
        let path = self.path.join(path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, contents).unwrap();
    }

    fn review(&self) -> GitRepository {
        GitRepository::discover(&self.path).unwrap()
    }

    fn commit_all(&self, message: &str) {
        self.git(["add", "--all"]);
        self.git(["commit", "--quiet", "-m", message]);
    }
}

impl Drop for TestRepo {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

/// Unstaging a staged rename by its destination alone used to leave the
/// source staged as a deletion, so the next commit dropped the file's
/// content. Both index entries must come back together.
#[test]
fn unstaging_a_rename_by_its_destination_restores_the_source() {
    let Some(repo) = TestRepo::new() else { return };
    repo.write("old.txt", "content worth keeping\n");
    repo.commit_all("base");
    repo.git(["mv", "old.txt", "new.txt"]);

    let review = repo.review();
    let staged = review.status().unwrap().staged;
    assert_eq!(staged.len(), 1, "expected one staged rename: {staged:?}");
    assert_eq!(staged[0].kind, ChangeKind::Renamed);

    review
        .unstage_paths(&[PathBuf::from("new.txt")])
        .expect("unstage the rename destination");

    let status = review.status().unwrap();
    assert!(
        status.staged.is_empty(),
        "nothing should remain staged, found {:?}",
        status.staged
    );
    assert!(
        repo.path.join("new.txt").exists(),
        "the renamed file must survive in the worktree"
    );
    // The original content is still reachable from HEAD, which is what a
    // staged deletion would have destroyed on the next commit.
    let head = repo.git(["show", "HEAD:old.txt"]);
    assert_eq!(String::from_utf8_lossy(&head), "content worth keeping\n");
}

#[test]
fn discovers_nested_repository_and_reads_grouped_status() {
    let Some(repo) = TestRepo::new() else { return };
    repo.write("tracked.txt", "base\n");
    repo.commit_all("base");
    repo.write("tracked.txt", "worktree\n");
    repo.write("both.txt", "staged\n");
    repo.git(["add", "both.txt"]);
    repo.write("both.txt", "staged and worktree\n");
    repo.write(":(glob) literal name.txt", "untracked\n");
    fs::create_dir(repo.path.join("nested")).unwrap();

    let review = GitRepository::discover(&repo.path.join("nested")).unwrap();
    assert_eq!(review.root, repo.path.canonicalize().unwrap());
    let status = review.status().unwrap();

    assert_eq!(status.branch.name.as_deref(), Some("main"));
    assert!(status.branch.oid.is_some());
    assert_eq!(status.staged.len(), 1);
    assert_eq!(status.staged[0].path, Path::new("both.txt"));
    assert_eq!(status.staged[0].kind, ChangeKind::Added);
    assert_eq!(status.unstaged.len(), 2);
    assert!(status.unstaged.iter().any(|change| {
        change.path == Path::new("tracked.txt") && change.kind == ChangeKind::Modified
    }));
    assert!(status.unstaged.iter().any(|change| {
        change.path == Path::new("both.txt") && change.kind == ChangeKind::Modified
    }));
    assert_eq!(status.untracked.len(), 1);
    assert_eq!(
        status.untracked[0].path,
        Path::new(":(glob) literal name.txt")
    );
    assert!(status.conflicted.is_empty());
}

#[test]
fn stage_unstage_and_discard_use_literal_paths() {
    let Some(repo) = TestRepo::new() else { return };
    repo.write("base.txt", "base\n");
    repo.commit_all("base");
    let review = repo.review();
    let magic = PathBuf::from(":(glob) [literal].txt");
    repo.write(magic.to_str().unwrap(), "one\n");

    review.stage_paths(std::slice::from_ref(&magic)).unwrap();
    assert!(
        review
            .status()
            .unwrap()
            .staged
            .iter()
            .any(|c| c.path == magic)
    );

    review.unstage_paths(std::slice::from_ref(&magic)).unwrap();
    let status = review.status().unwrap();
    assert!(status.staged.is_empty());
    assert!(status.untracked.iter().any(|c| c.path == magic));

    repo.write("base.txt", "changed\n");
    review
        .discard_unstaged(&[PathBuf::from("base.txt")])
        .unwrap();
    assert_eq!(
        fs::read_to_string(repo.path.join("base.txt")).unwrap(),
        "base\n"
    );
}

#[test]
fn whole_hunk_patches_stage_unstage_and_discard_only_the_selected_hunk() {
    let Some(repo) = TestRepo::new() else { return };
    let base = numbered_lines(30);
    repo.write("review.txt", &base);
    repo.commit_all("base");

    let mut changed = base.lines().map(str::to_owned).collect::<Vec<_>>();
    changed[1] = "changed early".to_owned();
    changed[20] = "changed late".to_owned();
    repo.write("review.txt", &(changed.join("\n") + "\n"));

    let working = load_local_diff(&repo.path, DiffLayer::Working).unwrap();
    let file = working
        .file_diffs
        .iter()
        .find(|file| file.path == Path::new("review.txt"))
        .unwrap();
    assert_eq!(file.hunks.len(), 2);
    let early = hunk_containing(&file.hunks, "changed early");
    repo.review()
        .apply_patch(&early.patch, PatchMutation::Stage)
        .unwrap();

    let cached = String::from_utf8(repo.git(["diff", "--cached", "--", "review.txt"]))
        .expect("utf-8 cached diff");
    let unstaged =
        String::from_utf8(repo.git(["diff", "--", "review.txt"])).expect("utf-8 worktree diff");
    assert!(cached.contains("changed early"));
    assert!(!cached.contains("changed late"));
    assert!(!unstaged.contains("changed early"));
    assert!(unstaged.contains("changed late"));

    let staged = load_local_diff(&repo.path, DiffLayer::Staged).unwrap();
    let staged_hunk = &staged.file_diffs[0].hunks[0];
    repo.review()
        .apply_patch(&staged_hunk.patch, PatchMutation::Unstage)
        .unwrap();
    assert!(repo.git(["diff", "--cached"]).is_empty());

    let working = load_local_diff(&repo.path, DiffLayer::Working).unwrap();
    let file = working
        .file_diffs
        .iter()
        .find(|file| file.path == Path::new("review.txt"))
        .unwrap();
    let early = hunk_containing(&file.hunks, "changed early");
    repo.review()
        .apply_patch(&early.patch, PatchMutation::Discard)
        .unwrap();

    let contents = fs::read_to_string(repo.path.join("review.txt")).unwrap();
    assert!(contents.contains("line 02"));
    assert!(!contents.contains("changed early"));
    assert!(contents.contains("changed late"));
}

#[test]
fn untracked_whole_hunk_patch_can_be_staged() {
    let Some(repo) = TestRepo::new() else { return };
    repo.write("base.txt", "base\n");
    repo.commit_all("base");
    repo.write("new.txt", "first\nsecond\n");

    let working = load_local_diff(&repo.path, DiffLayer::Working).unwrap();
    let file = working
        .file_diffs
        .iter()
        .find(|file| file.path == Path::new("new.txt"))
        .unwrap();
    assert_eq!(file.hunks.len(), 1);
    let review = repo.review();
    assert!(matches!(
        review.apply_patch(&file.hunks[0].patch, PatchMutation::Discard),
        Err(GitReviewError::InvalidPatch { .. })
    ));
    assert_eq!(
        fs::read_to_string(repo.path.join("new.txt")).unwrap(),
        "first\nsecond\n"
    );
    review
        .apply_patch(&file.hunks[0].patch, PatchMutation::Stage)
        .unwrap();

    assert_eq!(
        String::from_utf8(repo.git(["show", ":new.txt"])).unwrap(),
        "first\nsecond\n"
    );
    assert!(
        repo.review()
            .status()
            .unwrap()
            .staged
            .iter()
            .any(|change| change.path == Path::new("new.txt"))
    );
}

#[test]
fn stale_hunk_patch_is_rejected_without_mutating_the_index() {
    let Some(repo) = TestRepo::new() else { return };
    repo.write("stale.txt", "before\n");
    repo.commit_all("base");
    repo.write("stale.txt", "first edit\n");
    let snapshot = load_local_diff(&repo.path, DiffLayer::Working).unwrap();
    let patch = snapshot.file_diffs[0].hunks[0].patch.clone();

    repo.write("stale.txt", "overlapping newer edit\n");
    assert!(matches!(
        repo.review().apply_patch(&patch, PatchMutation::Stage),
        Err(GitReviewError::PatchDoesNotApply {
            mutation: PatchMutation::Stage,
            ..
        })
    ));
    assert!(repo.git(["diff", "--cached"]).is_empty());
}

#[test]
fn patch_input_is_bounded_and_rejects_empty_or_nul_data() {
    let Some(repo) = TestRepo::new() else { return };
    let review = repo.review();
    assert!(matches!(
        review.apply_patch(b" \n\t", PatchMutation::Stage),
        Err(GitReviewError::EmptyPatch)
    ));
    assert!(matches!(
        review.apply_patch(b"diff --git a/a b/a\0", PatchMutation::Stage),
        Err(GitReviewError::InvalidPatch { .. })
    ));
    assert!(matches!(
        review.apply_patch(&vec![b'x'; MAX_PATCH_BYTES + 1], PatchMutation::Stage),
        Err(GitReviewError::PatchTooLarge { .. })
    ));
}

#[test]
fn unstages_on_an_unborn_branch_without_deleting_the_file() {
    let Some(repo) = TestRepo::new() else { return };
    let review = repo.review();
    repo.write("first.txt", "first\n");
    review.stage_paths(&[PathBuf::from("first.txt")]).unwrap();
    review.unstage_paths(&[PathBuf::from("first.txt")]).unwrap();

    assert_eq!(
        fs::read_to_string(repo.path.join("first.txt")).unwrap(),
        "first\n"
    );
    let status = review.status().unwrap();
    assert!(status.staged.is_empty());
    assert_eq!(status.untracked[0].path, Path::new("first.txt"));
}

#[test]
fn commit_validates_message_and_returns_identity() {
    let Some(repo) = TestRepo::new() else { return };
    let review = repo.review();
    repo.write("first.txt", "hello\n");
    review.stage_paths(&[PathBuf::from("first.txt")]).unwrap();

    assert!(matches!(
        review.commit(" \n\t"),
        Err(GitReviewError::EmptyCommitMessage)
    ));
    let commit = review.commit("Review cockpit foundation\n").unwrap();
    assert_eq!(commit.oid.len(), 40);
    assert_eq!(commit.summary, "Review cockpit foundation");
    assert!(review.status().unwrap().staged.is_empty());
}

#[test]
fn conflict_entries_are_kept_out_of_other_groups() {
    let Some(repo) = TestRepo::new() else { return };
    repo.write("conflict.txt", "base\n");
    repo.commit_all("base");
    repo.git(["checkout", "--quiet", "-b", "side"]);
    repo.write("conflict.txt", "side\n");
    repo.commit_all("side");
    repo.git(["checkout", "--quiet", "main"]);
    repo.write("conflict.txt", "main\n");
    repo.commit_all("main");
    repo.git_expect_failure(["merge", "--no-edit", "side"]);

    let status = repo.review().status().unwrap();
    assert!(status.staged.is_empty());
    assert!(status.unstaged.is_empty());
    assert_eq!(status.conflicted.len(), 1);
    assert_eq!(status.conflicted[0].path, Path::new("conflict.txt"));
    assert_eq!(status.conflicted[0].kind, ChangeKind::Unmerged);
}

#[test]
fn rejects_paths_that_can_escape_or_broaden_the_mutation() {
    let Some(repo) = TestRepo::new() else { return };
    let review = repo.review();

    for path in ["../outside", "/absolute", ".", ".git/config", "a/../b"] {
        assert!(matches!(
            review.stage_paths(&[PathBuf::from(path)]),
            Err(GitReviewError::InvalidPath { .. })
        ));
    }
    assert!(matches!(
        review.stage_paths(&[]),
        Err(GitReviewError::EmptySelection)
    ));
}

#[test]
fn parses_branch_tracking_and_rename_records() {
    let bytes = b"# branch.oid abcdef\0# branch.head feature\0# branch.upstream origin/feature\0# branch.ab +3 -2\x002 R. N... 100644 100644 100644 aaaaaaa bbbbbbb R100 new name.txt\0old name.txt\0";
    let status = parse_status(Path::new("/repo"), bytes).unwrap();

    assert_eq!(status.branch.name.as_deref(), Some("feature"));
    assert_eq!(status.branch.upstream.as_deref(), Some("origin/feature"));
    assert_eq!(status.branch.ahead, 3);
    assert_eq!(status.branch.behind, 2);
    assert_eq!(status.staged.len(), 1);
    assert_eq!(status.staged[0].kind, ChangeKind::Renamed);
    assert_eq!(status.staged[0].path, Path::new("new name.txt"));
    assert_eq!(
        status.staged[0].original_path.as_deref(),
        Some(Path::new("old name.txt"))
    );
}

fn numbered_lines(count: usize) -> String {
    (1..=count)
        .map(|line| format!("line {line:02}"))
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}

fn hunk_containing<'a>(hunks: &'a [DiffHunk], needle: &str) -> &'a DiffHunk {
    hunks
        .iter()
        .find(|hunk| String::from_utf8_lossy(&hunk.patch).contains(needle))
        .unwrap_or_else(|| panic!("missing hunk containing {needle:?}"))
}
