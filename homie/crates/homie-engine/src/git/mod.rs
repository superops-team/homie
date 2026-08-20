//! Git facts a session needs: which branch it is on, and whether it is in a
//! linked worktree.
//!
//! Branch reading parses `.git/HEAD` directly rather than shelling out. It is
//! polled for a live label in the sidebar, and a `git` subprocess per session
//! per second is a cost worth not paying. Worktree operations do shell out —
//! they are rare, and reimplementing `git worktree` would be reckless.
//!
//! Ported from the previous Git helper.

use std::path::Path;
use std::process::Command;

mod diff;
mod repo;
mod worktree;

pub use diff::{working_diff, working_diff_remote};
pub use repo::{branch, is_linked_worktree, is_repository, repository_root};
pub use worktree::{
    WorktreeInfo, branch_to_path_slug, create_worktree, generated_branch_name, list_worktrees,
    parse_porcelain, remove_worktree, worktree_overview,
};

#[cfg(test)]
pub(crate) use diff::{WORKING_DIFF_SCRIPT, parse_working_diff};
#[cfg(test)]
pub(crate) use worktree::{ADJECTIVES, NOUNS};

/// Runs a git command, bounded and without inheriting a terminal.
///
/// The hardening is the same as the test helpers': a git that can prompt is a
/// git that can hang forever, and ambient config from the host has no business
/// affecting what the daemon sees.
pub(crate) fn run(args: &[&str], cwd: &Path) -> std::io::Result<String> {
    let output = Command::new("/usr/bin/git")
        .args(args)
        .current_dir(cwd)
        .stdin(std::process::Stdio::null())
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("HOME", cwd)
        .env("LC_ALL", "C")
        .env("LANG", "C")
        .env("LANGUAGE", "C")
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_OPTIONAL_LOCKS", "0")
        .output()?;
    if !output.status.success() {
        return Err(std::io::Error::other(format!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remote_diff_parser_ignores_login_shell_noise_before_its_marker() {
        let mut output = b"welcome from shell rc\nHOMIE_GIT_V1\0/srv/app\0HEAD\0".to_vec();
        output.extend_from_slice(b"diff --git a/a b/a\n");
        let result = parse_working_diff(output).expect("parse");
        assert_eq!(result.repo_root, "/srv/app");
        assert_eq!(result.base_ref.as_deref(), Some("HEAD"));
        assert!(result.patch.starts_with(b"diff --git"));
    }

    #[test]
    fn a_generated_branch_name_is_readable_and_namespaced() {
        let name = generated_branch_name();
        assert!(name.starts_with("homie/"), "got {name}");
        let tail = &name["homie/".len()..];
        let parts: Vec<&str> = tail.split('-').collect();
        assert_eq!(parts.len(), 3, "adjective-noun-hex: {name}");
        assert_eq!(parts[2].len(), 4, "four hex digits: {name}");
        assert!(ADJECTIVES.contains(&parts[0]));
        assert!(NOUNS.contains(&parts[1]));
    }

    #[test]
    fn generated_names_differ() {
        // Not a uniqueness guarantee, just a check that randomness is wired up.
        let names: std::collections::HashSet<String> =
            (0..16).map(|_| generated_branch_name()).collect();
        assert!(names.len() > 1, "every generated name was identical");
    }

    #[test]
    fn a_branch_becomes_a_usable_directory_slug() {
        assert_eq!(
            branch_to_path_slug("homie/swift-wren-1a2b"),
            "homie-swift-wren-1a2b"
        );
        assert_eq!(
            branch_to_path_slug("feature/JIRA-123_fix"),
            "feature-jira-123-fix"
        );
        assert_eq!(
            branch_to_path_slug("--weird//name--"),
            "weird-name",
            "leading and trailing dashes are trimmed and runs collapse"
        );
    }

    /// Exercises create/list/remove against real git in a temp repo.
    #[test]
    fn worktrees_can_be_created_listed_and_removed() {
        let temp = tempfile::tempdir().expect("temp");
        let repo = temp.path().join("repo");
        std::fs::create_dir_all(&repo).expect("mkdir");

        // A minimal repo with one commit, identity passed per-command so no
        // global config is needed.
        let git = |args: &[&str]| {
            let status = std::process::Command::new("/usr/bin/git")
                .args(args)
                .current_dir(&repo)
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .env_clear()
                .env("PATH", "/usr/bin:/bin")
                .env("HOME", temp.path())
                .env("GIT_TERMINAL_PROMPT", "0")
                .env("GIT_CONFIG_NOSYSTEM", "1")
                .status()
                .expect("run git");
            assert!(status.success(), "git {args:?} failed");
        };
        git(&["init", "--quiet", "--initial-branch=main"]);
        std::fs::write(repo.join("f.txt"), b"x").expect("write");
        git(&["add", "."]);
        git(&[
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.invalid",
            "commit",
            "--quiet",
            "-m",
            "init",
        ]);

        let created = create_worktree(&repo, None, None).expect("create");
        assert!(
            created.path.contains("repo-homie-"),
            "the worktree sits beside its repo: {}",
            created.path
        );
        assert_eq!(
            created.path,
            std::fs::canonicalize(&created.path)
                .expect("exists")
                .to_string_lossy(),
            "the reported path is the resolved one git also reports"
        );
        assert!(Path::new(&created.path).is_dir());
        assert!(
            is_linked_worktree(Path::new(&created.path)),
            "a created worktree is a linked one"
        );
        assert_eq!(
            branch(Path::new(&created.path)).as_deref(),
            created.branch.as_deref(),
            "it is on the branch it reported"
        );

        let listed = list_worktrees(&repo).expect("list");
        assert_eq!(listed.len(), 2, "the main checkout plus the new worktree");
        assert!(listed.iter().any(|entry| entry.path == created.path));

        remove_worktree(&repo, &created.path, true).expect("remove");
        assert_eq!(list_worktrees(&repo).expect("list").len(), 1);
    }

    #[test]
    fn porcelain_parsing_handles_a_main_checkout_and_a_linked_worktree() {
        let porcelain = "worktree /repo\nHEAD abc123\nbranch refs/heads/main\n\
             \n\
             worktree /repo/.claude/worktrees/bright-fox\nHEAD def456\n\
             branch refs/heads/worktree-bright-fox\n\n";
        let worktrees = parse_porcelain(porcelain);

        assert_eq!(worktrees.len(), 2);
        assert_eq!(worktrees[0].path, "/repo");
        assert_eq!(worktrees[0].branch.as_deref(), Some("main"));
        assert_eq!(
            worktrees[1].branch.as_deref(),
            Some("worktree-bright-fox"),
            "the refs/heads/ prefix is stripped"
        );
    }

    #[test]
    fn a_final_block_without_a_trailing_blank_line_is_not_dropped() {
        let worktrees = parse_porcelain("worktree /repo\nbranch refs/heads/main");
        assert_eq!(worktrees.len(), 1, "the last block must still flush");
        assert_eq!(worktrees[0].branch.as_deref(), Some("main"));
    }

    #[test]
    fn bare_detached_and_prunable_are_recognized() {
        let porcelain = "worktree /repo\nbare\n\n\
             worktree /repo/wt\nHEAD abc\ndetached\nprunable gitdir file points to non-existent\n";
        let worktrees = parse_porcelain(porcelain);

        assert!(worktrees[0].is_bare);
        assert!(worktrees[1].is_detached);
        assert!(
            worktrees[1].is_prunable,
            "prunable carries a reason after it"
        );
        assert!(
            !worktrees[0].is_detached,
            "flags do not leak between blocks"
        );
    }

    #[test]
    fn head_parsing_reads_a_branch_a_detached_sha_and_nothing() {
        let temp = tempfile::tempdir().expect("temp");
        let repo = temp.path().join("repo");
        let git = repo.join(".git");
        std::fs::create_dir_all(&git).expect("mkdir");

        std::fs::write(git.join("HEAD"), "ref: refs/heads/feature/login\n").expect("write");
        assert_eq!(branch(&repo).as_deref(), Some("feature/login"));

        std::fs::write(git.join("HEAD"), "9f8e7d6c5b4a3210\n").expect("write");
        assert_eq!(
            branch(&repo).as_deref(),
            Some("9f8e7d6c"),
            "a detached HEAD shows a short sha"
        );

        assert_eq!(branch(temp.path()).as_deref(), None, "not a repository");
    }

    #[test]
    fn a_branch_is_found_from_a_subdirectory() {
        let temp = tempfile::tempdir().expect("temp");
        let repo = temp.path().join("repo");
        let nested = repo.join("src/deep/inner");
        std::fs::create_dir_all(&nested).expect("mkdir");
        std::fs::create_dir_all(repo.join(".git")).expect("mkdir");
        std::fs::write(repo.join(".git/HEAD"), "ref: refs/heads/main\n").expect("write");

        assert_eq!(branch(&nested).as_deref(), Some("main"));
    }

    #[test]
    fn a_linked_worktree_is_told_apart_from_the_main_checkout() {
        let temp = tempfile::tempdir().expect("temp");
        let main = temp.path().join("repo");
        std::fs::create_dir_all(main.join(".git")).expect("mkdir");
        assert!(!is_linked_worktree(&main), ".git is a directory here");

        let linked = temp.path().join("wt");
        std::fs::create_dir_all(&linked).expect("mkdir");
        std::fs::write(
            linked.join(".git"),
            format!("gitdir: {}/.git/worktrees/wt\n", main.display()),
        )
        .expect("write");
        assert!(is_linked_worktree(&linked), ".git is a file here");
    }

    #[test]
    fn a_worktrees_head_is_followed_through_the_indirection() {
        let temp = tempfile::tempdir().expect("temp");
        let main = temp.path().join("repo");
        let worktree_git = main.join(".git/worktrees/wt");
        std::fs::create_dir_all(&worktree_git).expect("mkdir");
        std::fs::write(worktree_git.join("HEAD"), "ref: refs/heads/side-branch\n").expect("write");

        let linked = temp.path().join("wt");
        std::fs::create_dir_all(&linked).expect("mkdir");
        std::fs::write(
            linked.join(".git"),
            format!("gitdir: {}\n", worktree_git.display()),
        )
        .expect("write");

        assert_eq!(
            branch(&linked).as_deref(),
            Some("side-branch"),
            "a linked worktree has its own HEAD"
        );
    }

    #[test]
    fn remote_working_diff_pins_machine_readable_locale() {
        assert!(WORKING_DIFF_SCRIPT.contains("export LC_ALL=C LANG=C LANGUAGE=C"));
        assert!(WORKING_DIFF_SCRIPT.contains("export GIT_TERMINAL_PROMPT=0"));
    }
}
