use homie_runtime::{WorktreeOverviewEntry, WorktreeSheet};

#[test]
fn cleanup_requires_stale_clean_merged_non_main_worktree() {
    let mut sheet = WorktreeSheet::new(vec![
        entry("/repo/main", Some("main"), true, false, true),
        entry("/repo/dirty", Some("feature"), true, true, true),
        entry("/repo/unmerged", Some("feature"), true, false, false),
        entry("/repo/stale", Some("feature"), true, false, true),
    ]);

    assert!(sheet.request_cleanup("/repo/main").is_none());
    assert!(sheet.request_cleanup("/repo/dirty").is_none());
    assert!(sheet.request_cleanup("/repo/unmerged").is_none());

    let cleanup = sheet
        .request_cleanup("/repo/stale")
        .expect("stale worktree can be cleaned");
    assert_eq!(cleanup.repo_path, "/repo");
    assert_eq!(cleanup.worktree_path, "/repo/stale");
    assert!(!cleanup.force);
}

fn entry(
    path: &str,
    branch: Option<&str>,
    stale_suggestion: bool,
    dirty: bool,
    merged: bool,
) -> WorktreeOverviewEntry {
    WorktreeOverviewEntry {
        project_root: "/repo".to_string(),
        path: path.to_string(),
        branch: branch.map(str::to_string),
        session_id: None,
        session_status: None,
        dirty,
        merged,
        age_days: 20,
        stale_suggestion,
    }
}
