//! GitHub interaction: resolving the `gh` binary, running bounded subprocesses,
//! and decoding the JSON it returns into wire-format pull-request state.
//!
//! Every decode path is input-driven so tests can feed canned JSON without a
//! subprocess.

use std::time::{Duration, Instant};

use homie_proto::{DateMillis, PrCheck, PrDiscussionItem, PullRequestStatus};
use serde_json::Value;

pub(crate) fn resolve_gh() -> Option<String> {
    let path = std::env::var("PATH").ok()?;
    path.split(':')
        .map(|dir| std::path::Path::new(dir).join("gh"))
        .find(|candidate| candidate.is_file())
        .map(|path| path.to_string_lossy().into_owned())
}

/// `gh pr view <url> --json …` plus a GraphQL round trip for review-thread
/// resolution (which `pr view` can't report). None on any failure — the last
/// cached status stays in effect.
pub fn fetch(url: &str, gh: &str, include_threads: bool) -> Option<PullRequestStatus> {
    const FIELDS: &str = "number,title,author,body,baseRefName,headRefName,state,isDraft,\
        reviewDecision,mergeable,mergeStateStatus,additions,deletions,changedFiles,\
        comments,reviews,statusCheckRollup";
    let data = run_gh(
        gh,
        &["pr", "view", url, "--json", FIELDS],
        Duration::from_secs(15),
    )?;
    let mut status = parse(&data, url, now())?;

    if include_threads && let Some((owner, repo, number)) = pr_coordinates(url) {
        let query = "query=query($owner:String!,$name:String!,$number:Int!){\
            repository(owner:$owner,name:$name){pullRequest(number:$number){\
            reviewThreads(first:100){totalCount nodes{isResolved}}}}}";
        if let Some(thread_data) = run_gh(
            gh,
            &[
                "api",
                "graphql",
                "-f",
                query,
                "-f",
                &format!("owner={owner}"),
                "-f",
                &format!("name={repo}"),
                "-F",
                &format!("number={number}"),
            ],
            Duration::from_secs(15),
        ) && let Some((resolved, total)) = parse_threads(&thread_data)
        {
            status.resolved_threads = Some(resolved);
            status.total_threads = Some(total);
        }
    }
    Some(status)
}

/// Runs gh with a watchdog so a hung network call can't wedge the sweep.
fn run_gh(gh: &str, args: &[&str], timeout: Duration) -> Option<Vec<u8>> {
    let mut child = std::process::Command::new(gh)
        .args(args)
        .env("GH_PROMPT_DISABLED", "1")
        .env("GH_NO_UPDATE_NOTIFIER", "1")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .ok()?;
    let deadline = Instant::now() + timeout;
    loop {
        match child.try_wait() {
            Ok(Some(exit)) if exit.success() => break,
            Ok(Some(_)) => return None,
            Ok(None) if Instant::now() >= deadline => {
                let _ = child.kill();
                let _ = child.wait();
                return None;
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(100)),
            Err(_) => return None,
        }
    }
    let mut output = Vec::new();
    use std::io::Read;
    child.stdout.take()?.read_to_end(&mut output).ok()?;
    Some(output)
}

/// `github.com/owner/repo/pull/123` → (owner, repo, 123).
pub fn pr_coordinates(url: &str) -> Option<(String, String, i64)> {
    let parts: Vec<&str> = url.split('/').collect();
    let pull = parts.iter().position(|part| *part == "pull")?;
    if pull < 2 || pull + 1 >= parts.len() {
        return None;
    }
    let number: i64 = parts[pull + 1]
        .chars()
        .take_while(char::is_ascii_digit)
        .collect::<String>()
        .parse()
        .ok()?;
    Some((
        parts[pull - 2].to_string(),
        parts[pull - 1].to_string(),
        number,
    ))
}

/// Decodes the reviewThreads GraphQL response into (resolved, total).
pub fn parse_threads(data: &[u8]) -> Option<(i64, i64)> {
    let value: Value = serde_json::from_slice(data).ok()?;
    let threads = &value["data"]["repository"]["pullRequest"]["reviewThreads"];
    let total = threads["totalCount"].as_i64()?;
    let resolved = threads["nodes"]
        .as_array()?
        .iter()
        .filter(|node| node["isResolved"].as_bool() == Some(true))
        .count() as i64;
    Some((resolved, total))
}

/// Decodes one `gh pr view --json` payload. Input-driven so tests feed
/// canned JSON without a subprocess.
pub fn parse(data: &[u8], url: &str, fetched_at: DateMillis) -> Option<PullRequestStatus> {
    let view: Value = serde_json::from_slice(data).ok()?;
    let number = view["number"].as_i64()?;
    let string = |value: &Value| value.as_str().map(str::to_string);
    let nonempty = |value: &Value| value.as_str().filter(|s| !s.is_empty()).map(str::to_string);

    let checks: Vec<PrCheck> = view["statusCheckRollup"]
        .as_array()
        .map(|rollup| {
            rollup
                .iter()
                .map(|check| {
                    // CheckRun reports conclusion once COMPLETED; StatusContext
                    // only has state. One word decides the bucket; an empty
                    // conclusion means still running.
                    let verdict = ["conclusion", "state", "status"]
                        .iter()
                        .filter_map(|key| check[*key].as_str())
                        .find(|value| !value.is_empty())
                        .unwrap_or("");
                    let result = match verdict {
                        "SUCCESS" | "NEUTRAL" | "SKIPPED" => "pass",
                        "FAILURE" | "ERROR" | "CANCELLED" | "TIMED_OUT" | "ACTION_REQUIRED"
                        | "STARTUP_FAILURE" => "fail",
                        _ => "pending",
                    };
                    let base = check["name"]
                        .as_str()
                        .or_else(|| check["context"].as_str())
                        .unwrap_or("check");
                    let name = match check["workflowName"].as_str() {
                        Some(workflow) => format!("{workflow} / {base}"),
                        None => base.to_string(),
                    };
                    PrCheck {
                        name,
                        result: result.to_string(),
                        detail: (!verdict.is_empty()).then(|| verdict.to_string()),
                        url: string(&check["detailsUrl"]).or_else(|| string(&check["targetUrl"])),
                    }
                })
                .collect()
        })
        .unwrap_or_default();
    let count = |result: &str| checks.iter().filter(|check| check.result == result).count() as i64;

    let discussion_item = |kind: &str, entry: &Value, created_key: &str| PrDiscussionItem {
        kind: kind.to_string(),
        author: entry["author"]["login"]
            .as_str()
            .unwrap_or("ghost")
            .to_string(),
        body: entry["body"].as_str().unwrap_or("").to_string(),
        state: if kind == "review" {
            string(&entry["state"])
        } else {
            None
        },
        created_at: entry[created_key].as_str().and_then(parse_github_date),
        url: string(&entry["url"]),
    };
    let comments: Vec<PrDiscussionItem> = view["comments"]
        .as_array()
        .map(|list| {
            list.iter()
                .map(|comment| discussion_item("comment", comment, "createdAt"))
                .collect()
        })
        .unwrap_or_default();
    let reviews: Vec<PrDiscussionItem> = view["reviews"]
        .as_array()
        .map(|list| {
            list.iter()
                .map(|review| discussion_item("review", review, "submittedAt"))
                .collect()
        })
        .unwrap_or_default();
    let comment_count = comments.len() as i64;
    let review_count = reviews.len() as i64;
    let mut discussion: Vec<PrDiscussionItem> = comments.into_iter().chain(reviews).collect();
    discussion.sort_by(|a, b| {
        let time = |item: &PrDiscussionItem| item.created_at.as_ref().map_or(f64::MIN, |at| at.0);
        time(a)
            .partial_cmp(&time(b))
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    Some(PullRequestStatus {
        url: url.to_string(),
        number,
        title: string(&view["title"]),
        author: string(&view["author"]["login"]),
        body: string(&view["body"]),
        base_ref_name: string(&view["baseRefName"]),
        head_ref_name: string(&view["headRefName"]),
        state: view["state"].as_str().unwrap_or("OPEN").to_string(),
        is_draft: view["isDraft"].as_bool().unwrap_or(false),
        review_decision: nonempty(&view["reviewDecision"]),
        mergeable: string(&view["mergeable"]),
        merge_state_status: string(&view["mergeStateStatus"]),
        additions: view["additions"].as_i64().unwrap_or(0),
        deletions: view["deletions"].as_i64().unwrap_or(0),
        changed_files: view["changedFiles"].as_i64().unwrap_or(0),
        comment_count,
        review_count,
        resolved_threads: None,
        total_threads: None,
        checks_passed: count("pass"),
        checks_failed: count("fail"),
        checks_pending: count("pending"),
        checks: (!checks.is_empty()).then_some(checks),
        discussion: (!discussion.is_empty()).then_some(discussion),
        fetched_at,
    })
}

fn parse_github_date(value: &str) -> Option<DateMillis> {
    // ISO 8601 `2026-08-07T12:34:56Z`; a hand parser avoids a chrono
    // dependency for one field the client only sorts and displays by.
    let bytes = value.as_bytes();
    if bytes.len() < 20 || bytes[4] != b'-' || bytes[7] != b'-' || bytes[10] != b'T' {
        return None;
    }
    let number = |range: std::ops::Range<usize>| -> Option<i64> { value.get(range)?.parse().ok() };
    let (year, month, day) = (number(0..4)?, number(5..7)?, number(8..10)?);
    let (hour, minute, second) = (number(11..13)?, number(14..16)?, number(17..19)?);
    // Days since epoch via the civil-days algorithm.
    let years = if month <= 2 { year - 1 } else { year };
    let era = years.div_euclid(400);
    let year_of_era = years - era * 400;
    let day_of_year = (153 * (if month > 2 { month - 3 } else { month + 9 }) + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    let days = era * 146_097 + day_of_era - 719_468;
    Some(DateMillis(
        ((days * 86_400 + hour * 3_600 + minute * 60 + second) * 1_000) as f64,
    ))
}

fn now() -> DateMillis {
    DateMillis::from(std::time::SystemTime::now())
}
