use serde::Deserialize;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PullRequestCheck {
    pub name: String,
    pub result: String,
    pub detail: Option<String>,
    pub url: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PullRequestStatus {
    pub url: String,
    pub number: i64,
    pub title: Option<String>,
    pub state: String,
    pub is_draft: bool,
    pub review_decision: Option<String>,
    pub mergeable: Option<String>,
    pub merge_state_status: Option<String>,
    pub additions: i64,
    pub deletions: i64,
    pub changed_files: i64,
    pub comment_count: i64,
    pub review_count: i64,
    pub resolved_threads: Option<i64>,
    pub total_threads: Option<i64>,
    pub checks_passed: i64,
    pub checks_failed: i64,
    pub checks_pending: i64,
    pub checks: Option<Vec<PullRequestCheck>>,
    pub fetched_at: i64,
}

impl PullRequestStatus {
    #[must_use]
    pub fn overall(&self) -> &'static str {
        if self.state == "MERGED" {
            return "merged";
        }
        if self.state == "CLOSED" {
            return "closed";
        }
        if self.is_draft {
            return "draft";
        }
        if self.mergeable.as_deref() == Some("CONFLICTING") {
            return "conflicts";
        }
        if self.checks_failed > 0 {
            return "checks failing";
        }
        if self.review_decision.as_deref() == Some("CHANGES_REQUESTED") {
            return "changes requested";
        }
        if self.checks_pending > 0 {
            return "checks pending";
        }
        if self.review_decision.as_deref() == Some("REVIEW_REQUIRED") {
            return "needs review";
        }
        if self.merge_state_status.as_deref() == Some("BLOCKED") {
            return "blocked";
        }
        "ready"
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewThreadCounts {
    pub resolved: i64,
    pub total: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PullRequestCoordinates {
    pub owner: String,
    pub repo: String,
    pub number: i64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GhPrView {
    number: i64,
    title: Option<String>,
    state: String,
    #[serde(default)]
    is_draft: bool,
    review_decision: Option<String>,
    mergeable: Option<String>,
    merge_state_status: Option<String>,
    #[serde(default)]
    additions: i64,
    #[serde(default)]
    deletions: i64,
    #[serde(default)]
    changed_files: i64,
    #[serde(default)]
    comments: Vec<serde_json::Value>,
    #[serde(default)]
    reviews: Vec<serde_json::Value>,
    #[serde(default)]
    status_check_rollup: Vec<GhCheck>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GhCheck {
    name: Option<String>,
    context: Option<String>,
    workflow_name: Option<String>,
    status: Option<String>,
    conclusion: Option<String>,
    state: Option<String>,
    details_url: Option<String>,
    target_url: Option<String>,
}

pub fn parse_pr_status(data: &[u8], url: &str, fetched_at: i64) -> Option<PullRequestStatus> {
    let view: GhPrView = serde_json::from_slice(data).ok()?;
    let checks = view
        .status_check_rollup
        .into_iter()
        .map(parse_check)
        .collect::<Vec<_>>();
    let checks_passed = checks.iter().filter(|check| check.result == "pass").count() as i64;
    let checks_failed = checks.iter().filter(|check| check.result == "fail").count() as i64;
    let checks_pending = checks
        .iter()
        .filter(|check| check.result == "pending")
        .count() as i64;
    Some(PullRequestStatus {
        url: url.to_string(),
        number: view.number,
        title: view.title,
        state: view.state,
        is_draft: view.is_draft,
        review_decision: view.review_decision.filter(|value| !value.is_empty()),
        mergeable: view.mergeable,
        merge_state_status: view.merge_state_status,
        additions: view.additions,
        deletions: view.deletions,
        changed_files: view.changed_files,
        comment_count: view.comments.len() as i64,
        review_count: view.reviews.len() as i64,
        resolved_threads: None,
        total_threads: None,
        checks_passed,
        checks_failed,
        checks_pending,
        checks: (!checks.is_empty()).then_some(checks),
        fetched_at,
    })
}

fn parse_check(check: GhCheck) -> PullRequestCheck {
    let verdict = [check.conclusion, check.state, check.status]
        .into_iter()
        .flatten()
        .find(|value| !value.is_empty())
        .unwrap_or_default();
    let result = match verdict.as_str() {
        "SUCCESS" | "NEUTRAL" | "SKIPPED" => "pass",
        "FAILURE" | "ERROR" | "CANCELLED" | "TIMED_OUT" | "ACTION_REQUIRED" | "STARTUP_FAILURE" => {
            "fail"
        }
        _ => "pending",
    };
    let base = check
        .name
        .or(check.context)
        .unwrap_or_else(|| "check".to_string());
    let name = check
        .workflow_name
        .map(|workflow| format!("{workflow} / {base}"))
        .unwrap_or(base);
    PullRequestCheck {
        name,
        result: result.to_string(),
        detail: (!verdict.is_empty()).then_some(verdict),
        url: check.details_url.or(check.target_url),
    }
}

#[derive(Deserialize)]
struct ThreadResponse {
    data: Option<ThreadData>,
}

#[derive(Deserialize)]
struct ThreadData {
    repository: Option<ThreadRepository>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ThreadRepository {
    pull_request: Option<ThreadPullRequest>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ThreadPullRequest {
    review_threads: ThreadCollection,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ThreadCollection {
    total_count: i64,
    nodes: Vec<ThreadNode>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ThreadNode {
    is_resolved: bool,
}

pub fn parse_review_threads(data: &[u8]) -> Option<ReviewThreadCounts> {
    let response: ThreadResponse = serde_json::from_slice(data).ok()?;
    let threads = response.data?.repository?.pull_request?.review_threads;
    Some(ReviewThreadCounts {
        resolved: threads.nodes.iter().filter(|node| node.is_resolved).count() as i64,
        total: threads.total_count,
    })
}

pub fn parse_pr_coordinates(url: &str) -> Option<PullRequestCoordinates> {
    let parts = url.split('/').collect::<Vec<_>>();
    let pull_index = parts.iter().position(|part| *part == "pull")?;
    if pull_index < 2 || pull_index + 1 >= parts.len() {
        return None;
    }
    let number = parts[pull_index + 1]
        .chars()
        .take_while(char::is_ascii_digit)
        .collect::<String>()
        .parse()
        .ok()?;
    Some(PullRequestCoordinates {
        owner: parts[pull_index - 2].to_string(),
        repo: parts[pull_index - 1].to_string(),
        number,
    })
}
