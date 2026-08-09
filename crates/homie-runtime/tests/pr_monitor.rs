use homie_runtime::pr_monitor::{
    PullRequestStatus, parse_pr_coordinates, parse_pr_status, parse_review_threads,
};

const PR_URL: &str = "https://github.com/acme/widgets/pull/123";

#[test]
fn parses_github_pr_payload_and_check_rollup() {
    let json = r#"
    {
      "number": 123,
      "title": "Fix scroll jitter",
      "state": "OPEN",
      "isDraft": false,
      "reviewDecision": "APPROVED",
      "mergeable": "MERGEABLE",
      "mergeStateStatus": "CLEAN",
      "additions": 45,
      "deletions": 12,
      "changedFiles": 3,
      "comments": [{"body": "nice"}, {"body": "ship it"}],
      "reviews": [{"state": "APPROVED"}],
      "statusCheckRollup": [
        {"__typename": "CheckRun", "name": "build", "workflowName": "CI",
         "status": "COMPLETED", "conclusion": "SUCCESS",
         "detailsUrl": "https://github.com/acme/widgets/runs/1"},
        {"__typename": "CheckRun", "name": "test", "status": "IN_PROGRESS", "conclusion": ""},
        {"__typename": "StatusContext", "context": "vercel", "state": "FAILURE"}
      ]
    }
    "#;

    let status = parse_pr_status(json.as_bytes(), PR_URL, 10).expect("parse pr");

    assert_eq!(status.url, PR_URL);
    assert_eq!(status.number, 123);
    assert_eq!(status.title.as_deref(), Some("Fix scroll jitter"));
    assert_eq!(status.state, "OPEN");
    assert!(!status.is_draft);
    assert_eq!(status.review_decision.as_deref(), Some("APPROVED"));
    assert_eq!(status.mergeable.as_deref(), Some("MERGEABLE"));
    assert_eq!(status.additions, 45);
    assert_eq!(status.deletions, 12);
    assert_eq!(status.changed_files, 3);
    assert_eq!(status.comment_count, 2);
    assert_eq!(status.review_count, 1);
    assert_eq!(status.checks_passed, 1);
    assert_eq!(status.checks_failed, 1);
    assert_eq!(status.checks_pending, 1);
    assert_eq!(status.fetched_at, 10);
    assert_eq!(status.overall(), "checks failing");

    let checks = status.checks.as_ref().expect("checks");
    assert_eq!(checks.len(), 3);
    assert_eq!(checks[0].name, "CI / build");
    assert_eq!(checks[0].result, "pass");
    assert_eq!(
        checks[0].url.as_deref(),
        Some("https://github.com/acme/widgets/runs/1")
    );
    assert_eq!(checks[1].name, "test");
    assert_eq!(checks[1].result, "pending");
    assert_eq!(checks[1].detail.as_deref(), Some("IN_PROGRESS"));
    assert_eq!(checks[2].name, "vercel");
    assert_eq!(checks[2].result, "fail");
}

#[test]
fn parses_minimal_payload_and_rejects_garbage() {
    let status = parse_pr_status(
        br#"{"number":7,"state":"MERGED","reviewDecision":""}"#,
        PR_URL,
        20,
    )
    .expect("minimal payload");

    assert_eq!(status.state, "MERGED");
    assert_eq!(status.review_decision, None);
    assert_eq!(status.comment_count, 0);
    assert_eq!(status.checks_passed, 0);
    assert_eq!(status.overall(), "merged");

    assert!(parse_pr_status(b"not json", PR_URL, 0).is_none());
    assert!(parse_pr_status(b"{}", PR_URL, 0).is_none());
}

#[test]
fn overall_rollup_matches_diri_order() {
    assert_eq!(
        status("MERGED", false, None, None, None, 3, 0).overall(),
        "merged"
    );
    assert_eq!(
        status("CLOSED", false, None, None, None, 0, 0).overall(),
        "closed"
    );
    assert_eq!(
        status("OPEN", true, None, None, None, 0, 0).overall(),
        "draft"
    );
    assert_eq!(
        status("OPEN", false, None, Some("CONFLICTING"), None, 0, 0).overall(),
        "conflicts"
    );
    assert_eq!(
        status("OPEN", false, None, None, None, 2, 0).overall(),
        "checks failing"
    );
    assert_eq!(
        status("OPEN", false, Some("CHANGES_REQUESTED"), None, None, 0, 0).overall(),
        "changes requested"
    );
    assert_eq!(
        status("OPEN", false, None, None, None, 0, 1).overall(),
        "checks pending"
    );
    assert_eq!(
        status("OPEN", false, Some("REVIEW_REQUIRED"), None, None, 0, 0).overall(),
        "needs review"
    );
    assert_eq!(
        status("OPEN", false, None, None, Some("BLOCKED"), 0, 0).overall(),
        "blocked"
    );
    assert_eq!(
        status(
            "OPEN",
            false,
            Some("APPROVED"),
            Some("MERGEABLE"),
            None,
            0,
            0
        )
        .overall(),
        "ready"
    );
}

#[test]
fn parses_review_threads_and_pr_coordinates() {
    let json = r#"
    {"data":{"repository":{"pullRequest":{"reviewThreads":{
      "totalCount":5,
      "nodes":[
        {"isResolved":true},{"isResolved":true},{"isResolved":true},
        {"isResolved":false},{"isResolved":false}
      ]}}}}}
    "#;

    let threads = parse_review_threads(json.as_bytes()).expect("threads");
    assert_eq!(threads.resolved, 3);
    assert_eq!(threads.total, 5);
    assert!(parse_review_threads(b"{}").is_none());

    let coords =
        parse_pr_coordinates("https://github.com/anaralabs/anara/pull/4570").expect("coords");
    assert_eq!(coords.owner, "anaralabs");
    assert_eq!(coords.repo, "anara");
    assert_eq!(coords.number, 4570);
    assert!(parse_pr_coordinates("https://github.com/anaralabs/anara").is_none());
}

fn status(
    state: &str,
    is_draft: bool,
    review_decision: Option<&str>,
    mergeable: Option<&str>,
    merge_state_status: Option<&str>,
    failed: i64,
    pending: i64,
) -> PullRequestStatus {
    PullRequestStatus {
        url: PR_URL.to_string(),
        number: 1,
        title: None,
        state: state.to_string(),
        is_draft,
        review_decision: review_decision.map(str::to_string),
        mergeable: mergeable.map(str::to_string),
        merge_state_status: merge_state_status.map(str::to_string),
        additions: 0,
        deletions: 0,
        changed_files: 0,
        comment_count: 0,
        review_count: 0,
        resolved_threads: None,
        total_threads: None,
        checks_passed: 0,
        checks_failed: failed,
        checks_pending: pending,
        checks: None,
        fetched_at: 0,
    }
}
