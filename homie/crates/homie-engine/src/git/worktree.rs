use std::path::Path;
use std::process::Command;

use super::is_repository;
use super::run;

/// One entry from `git worktree list --porcelain`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorktreeInfo {
    pub path: String,
    pub branch: Option<String>,
    pub is_bare: bool,
    pub is_detached: bool,
    pub is_prunable: bool,
}

pub fn list_worktrees(repo: &Path) -> std::io::Result<Vec<WorktreeInfo>> {
    let porcelain = run(&["worktree", "list", "--porcelain"], repo)?;
    Ok(parse_porcelain(&porcelain))
}

/// Parses `git worktree list --porcelain`.
///
/// Blocks are separated by blank lines, but a `worktree` line also starts a new
/// block — trailing output without a final blank line still has to flush.
pub fn parse_porcelain(porcelain: &str) -> Vec<WorktreeInfo> {
    /// One block being accumulated. Replaced wholesale on flush, so flags
    /// cannot leak from one worktree into the next.
    #[derive(Default)]
    struct Block {
        path: Option<String>,
        branch: Option<String>,
        is_bare: bool,
        is_detached: bool,
        is_prunable: bool,
    }

    fn flush(block: &mut Block, results: &mut Vec<WorktreeInfo>) {
        let Some(path) = block.path.take() else {
            return;
        };
        let finished = std::mem::take(block);
        results.push(WorktreeInfo {
            path,
            branch: finished.branch,
            is_bare: finished.is_bare,
            is_detached: finished.is_detached,
            is_prunable: finished.is_prunable,
        });
    }

    let mut results = Vec::new();
    let mut block = Block::default();

    for line in porcelain.split('\n') {
        if line.is_empty() {
            flush(&mut block, &mut results);
        } else if let Some(rest) = line.strip_prefix("worktree ") {
            flush(&mut block, &mut results);
            block.path = Some(rest.to_string());
        } else if let Some(rest) = line.strip_prefix("branch ") {
            block.branch = Some(rest.strip_prefix("refs/heads/").unwrap_or(rest).to_string());
        } else if line == "bare" {
            block.is_bare = true;
        } else if line == "detached" {
            block.is_detached = true;
        } else if line == "prunable" || line.starts_with("prunable ") {
            block.is_prunable = true;
        }
        // `HEAD <sha>` and other keys carry nothing this type models.
    }
    flush(&mut block, &mut results);
    results
}

/// Adjective/noun pairs for generated branch names. Memorable beats unique:
/// these names end up in branch lists and sidebar rows that people read.
pub(crate) const ADJECTIVES: &[&str] = &[
    "brisk", "calm", "deft", "eager", "fleet", "gentle", "hardy", "keen", "lively", "merry",
    "nimble", "plucky", "quiet", "rapid", "steady", "swift", "tidy", "vivid", "witty", "zesty",
];

pub(crate) const NOUNS: &[&str] = &[
    "otter", "heron", "maple", "cedar", "falcon", "willow", "badger", "sparrow", "cypress",
    "marten", "juniper", "raven", "birch", "lynx", "hazel", "osprey", "aspen", "finch", "poplar",
    "wren",
];

/// `homie/<adjective>-<noun>-<4hex>`.
pub fn generated_branch_name() -> String {
    let mut bytes = [0u8; 4];
    getrandom::fill(&mut bytes).expect("the OS random source");
    let adjective = ADJECTIVES[bytes[0] as usize % ADJECTIVES.len()];
    let noun = NOUNS[bytes[1] as usize % NOUNS.len()];
    let hex = u16::from_be_bytes([bytes[2], bytes[3]]);
    format!("homie/{adjective}-{noun}-{hex:04x}")
}

/// Lowercases a branch name and collapses every run of characters outside
/// `[a-z0-9]` into a single dash, so it can be a directory name.
pub fn branch_to_path_slug(branch: &str) -> String {
    let mut slug = String::with_capacity(branch.len());
    let mut last_was_dash = false;
    for character in branch.to_lowercase().chars() {
        if character.is_ascii_lowercase() || character.is_ascii_digit() {
            slug.push(character);
            last_was_dash = false;
        } else if !last_was_dash {
            slug.push('-');
            last_was_dash = true;
        }
    }
    slug.trim_matches('-').to_string()
}

/// Creates a worktree beside the repository, on a new branch.
///
/// The path is `<parent>/<repo-name>-<branch-slug>`, which keeps sibling
/// worktrees visibly related to their repository in a file browser and in
/// shell completion.
pub fn create_worktree(
    repo: &Path,
    branch: Option<&str>,
    base: Option<&str>,
) -> std::io::Result<WorktreeInfo> {
    let branch_name = branch
        .map(str::to_string)
        .unwrap_or_else(generated_branch_name);
    let slug = branch_to_path_slug(&branch_name);
    let parent = repo.parent().unwrap_or(Path::new("."));
    let repo_name = repo
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| "repo".into());
    let worktree_path = parent.join(format!("{repo_name}-{slug}"));
    let worktree_str = worktree_path.to_string_lossy().to_string();

    let mut args = vec!["worktree", "add", "-b", &branch_name, &worktree_str];
    if let Some(base) = base {
        args.push(base);
    }
    run(&args, repo)?;

    // Report the path git will report. It records the *resolved* path, and on
    // macOS the common roots are symlinks — `/tmp` is `/private/tmp`, and a
    // repo can easily sit under one too. Handing back the unresolved path means
    // a caller that stores it cannot later match it against `list_worktrees` or
    // remove it by path.
    let resolved = std::fs::canonicalize(&worktree_path)
        .map(|path| path.to_string_lossy().to_string())
        .unwrap_or(worktree_str);

    Ok(WorktreeInfo {
        path: resolved,
        branch: Some(branch_name),
        is_bare: false,
        is_detached: false,
        is_prunable: false,
    })
}

pub fn remove_worktree(repo: &Path, worktree: &str, force: bool) -> std::io::Result<()> {
    let mut args = vec!["worktree", "remove"];
    if force {
        args.push("--force");
    }
    args.push(worktree);
    run(&args, repo)?;
    Ok(())
}

/// The aggregated staleness view for the worktree overview pane: every
/// worktree of every project, joined with the session (live wins) occupying
/// it, its dirtiness, merged-ness into the default branch, and age — plus the
/// "safe to clean up" suggestion.
///
/// Pure domain logic (git subprocess + staleness join) extracted from the
/// control handler so it can be tested without a daemon socket.
pub fn worktree_overview(
    records: &[homie_proto::SessionRecord],
    mut roots: Vec<String>,
) -> homie_proto::WorktreeOverviewResult {
    roots.sort();

    // Join sessions by worktree path (fallback cwd); a live session wins
    // over an exited one sharing the path.
    let mut session_by_path: std::collections::HashMap<String, &homie_proto::SessionRecord> =
        std::collections::HashMap::new();
    let running = |record: &homie_proto::SessionRecord| {
        !matches!(
            record.status,
            homie_proto::SessionStatus::Exited(_) | homie_proto::SessionStatus::Unknown
        )
    };
    for record in records {
        let path = record
            .worktree_path
            .clone()
            .unwrap_or_else(|| record.cwd.clone());
        match session_by_path.get(&path) {
            Some(existing) if running(existing) || !running(record) => {}
            _ => {
                session_by_path.insert(path, record);
            }
        }
    }

    let run_git = |args: &[&str], dir: &str| -> Option<String> {
        let output = Command::new("git")
            .args(args)
            .current_dir(dir)
            .env("LC_ALL", "C")
            .env("LANG", "C")
            .env("LANGUAGE", "C")
            .env("GIT_TERMINAL_PROMPT", "0")
            .output()
            .ok()?;
        output
            .status
            .success()
            .then(|| String::from_utf8_lossy(&output.stdout).trim().to_string())
    };

    let mut entries = Vec::new();
    let mut seen_paths = std::collections::HashSet::new();
    for root in roots {
        if !is_repository(Path::new(&root)) {
            continue;
        }
        let Ok(worktrees) = list_worktrees(Path::new(&root)) else {
            continue;
        };
        // Repo's default branch: origin/HEAD symbolic ref, else "main".
        let default_branch = run_git(
            &["symbolic-ref", "--short", "refs/remotes/origin/HEAD"],
            &root,
        )
        .and_then(|full| full.rsplit('/').next().map(str::to_string))
        .filter(|short| !short.is_empty())
        .unwrap_or_else(|| "main".into());
        let merged_branches: std::collections::HashSet<String> = run_git(
            &[
                "branch",
                "--merged",
                &default_branch,
                "--format=%(refname:short)",
            ],
            &root,
        )
        .map(|output| output.lines().map(str::to_string).collect())
        .unwrap_or_default();

        for worktree in worktrees {
            if worktree.is_bare || !seen_paths.insert(worktree.path.clone()) {
                continue;
            }
            let is_main = worktree.path == root;
            let dirty = run_git(&["status", "--porcelain"], &worktree.path)
                .is_some_and(|output| !output.is_empty());
            let merged = worktree.branch.as_ref().is_some_and(|branch| {
                branch != &default_branch && merged_branches.contains(branch)
            });
            let age_days = std::fs::metadata(&worktree.path)
                .ok()
                .and_then(|meta| meta.created().or_else(|_| meta.modified()).ok())
                .and_then(|at| at.elapsed().ok())
                .map(|elapsed| (elapsed.as_secs() / 86_400) as i64)
                .unwrap_or(0);
            let record = session_by_path.get(&worktree.path);
            let session_alive = record.is_some_and(|record| running(record));
            entries.push(homie_proto::WorktreeOverviewEntry {
                path: worktree.path.clone(),
                branch: worktree.branch.clone(),
                project_root: root.clone(),
                session_id: record.map(|record| record.id.clone()),
                session_status: record.map(|record| record.status.clone()),
                dirty,
                merged,
                age_days,
                stale_suggestion: !is_main && !session_alive && merged && !dirty && age_days > 7,
            });
        }
    }

    homie_proto::WorktreeOverviewResult { entries }
}
