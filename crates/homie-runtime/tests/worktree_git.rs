use std::fs;
use std::path::Path;
use std::process::Command;

use homie_runtime::{
    WorktreeCreateRequest, WorktreeRemoveRequest, create_worktree, list_git_worktrees,
    parse_worktree_porcelain, remove_worktree,
};
use tempfile::TempDir;

#[test]
fn parses_worktree_porcelain_like_diri() {
    let porcelain = "worktree /repo\nHEAD abc123\nbranch refs/heads/main\n\n\
        worktree /repo/.claude/worktrees/bright-fox\nHEAD def456\n\
        branch refs/heads/worktree-bright-fox\nprunable stale admin dir\n\n\
        worktree /repo/bare\nbare\n\n\
        worktree /repo/detached\nHEAD 123456\ndetached";
    let worktrees = parse_worktree_porcelain(porcelain);

    assert_eq!(worktrees.len(), 4);
    assert_eq!(worktrees[0].path, "/repo");
    assert_eq!(worktrees[0].branch.as_deref(), Some("main"));
    assert_eq!(worktrees[1].branch.as_deref(), Some("worktree-bright-fox"));
    assert!(worktrees[1].is_prunable);
    assert!(worktrees[2].is_bare);
    assert!(worktrees[3].is_detached);
}

#[test]
fn creates_lists_and_removes_real_git_worktree() {
    let temp = TempDir::new().expect("tempdir");
    let repo = temp.path().join("repo");
    init_repo(&repo);

    let created = create_worktree(WorktreeCreateRequest {
        repo_path: repo.clone(),
        branch: Some("feature/demo".to_string()),
        base: Some("HEAD".to_string()),
    })
    .expect("create worktree");
    assert!(Path::new(&created.path).is_dir());
    assert_eq!(created.branch.as_deref(), Some("feature/demo"));
    assert!(created.path.ends_with("repo-feature-demo"));

    let listed = list_git_worktrees(&repo).expect("list worktrees");
    assert!(listed.iter().any(|entry| entry.path == created.path));

    remove_worktree(WorktreeRemoveRequest {
        repo_path: repo.clone(),
        worktree_path: created.path.clone().into(),
        force: true,
    })
    .expect("remove worktree");
    assert!(!Path::new(&created.path).exists());
}

fn init_repo(repo: &Path) {
    fs::create_dir_all(repo).expect("repo dir");
    run_git(repo, ["init", "-q", "-b", "main"]);
    run_git(repo, ["config", "user.email", "test@example.invalid"]);
    run_git(repo, ["config", "user.name", "Homie Test"]);
    fs::write(repo.join("README.md"), "hello\n").expect("readme");
    run_git(repo, ["add", "."]);
    run_git(repo, ["commit", "-q", "-m", "init"]);
}

fn run_git<const N: usize>(cwd: &Path, args: [&str; N]) {
    let output = Command::new("/usr/bin/git")
        .args(args)
        .current_dir(cwd)
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()
        .expect("run git");
    assert!(
        output.status.success(),
        "git failed stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
