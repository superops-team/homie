//! Session/artifact/PR → toolbar chip projection.
//!
//! Pure projection: no `Window`/`Context`/`Entity`/render dependency, so it
//! stays unit-testable in isolation.

use homie_proto::{ArtifactKind, PullRequestStatus, SessionArtifact, SessionRecord};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChipTint {
    Red,
    Orange,
    Yellow,
    Green,
    Purple,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PaneChip {
    pub id: String,
    pub label: String,
    pub system_image: &'static str,
    pub open_url: Option<String>,
    pub copy_string: String,
    pub tint: Option<ChipTint>,
    pub help: String,
    pub checks: Option<PullRequestStatus>,
}

impl PaneChip {
    pub fn for_session(session: &SessionRecord) -> Vec<Self> {
        let mut result = Vec::new();
        let artifacts = session.artifacts.as_deref().unwrap_or_default();
        let statuses = session.pull_requests.as_deref().unwrap_or_default();
        let pull_requests = artifacts
            .iter()
            .filter(|artifact| artifact.kind == ArtifactKind::PullRequest)
            .map(|artifact| {
                (
                    artifact,
                    statuses.iter().find(|status| status.url == artifact.url),
                )
            })
            .collect::<Vec<_>>();

        // Primary PR destinations are the highest-value links, so expose all
        // of them before their supporting checks/comments or generic URLs.
        for (artifact, status) in &pull_requests {
            result.push(Self::from_artifact(artifact, *status));
        }
        for (artifact, status) in pull_requests {
            if let Some(status) = status {
                if let Some(checks) = Self::checks_chip(artifact, status) {
                    result.push(checks);
                }
                if let Some(comments) = Self::comments_chip(artifact, status) {
                    result.push(comments);
                }
            }
        }
        for artifact in artifacts
            .iter()
            .filter(|artifact| artifact.kind != ArtifactKind::PullRequest)
        {
            result.push(Self::from_artifact(artifact, None));
        }
        for port in session.listening_ports.as_deref().unwrap_or_default() {
            let url = format!("http://localhost:{}", port.port);
            result.push(Self {
                id: format!("port-{}", port.port),
                label: format!(":{}", port.port),
                system_image: "network",
                open_url: Some(url.clone()),
                copy_string: url.clone(),
                tint: None,
                help: url,
                checks: None,
            });
        }
        result
    }

    fn from_artifact(artifact: &SessionArtifact, pr: Option<&PullRequestStatus>) -> Self {
        match artifact.kind {
            ArtifactKind::PullRequest => {
                let mut label = pr_number(&artifact.url)
                    .map_or_else(|| "PR".to_owned(), |number| format!("PR #{number}"));
                if let Some(pr) = pr
                    && pr.additions + pr.deletions > 0
                {
                    label.push_str(&format!(" +{} −{}", pr.additions, pr.deletions));
                }
                Self {
                    id: format!("art-{}", artifact.url),
                    label,
                    system_image: pr.map_or("arrow.triangle.pull", |pr| match pr.state.as_str() {
                        "MERGED" => "arrow.triangle.merge",
                        "CLOSED" => "xmark.circle",
                        _ => "arrow.triangle.pull",
                    }),
                    open_url: Some(artifact.url.clone()),
                    copy_string: artifact.url.clone(),
                    tint: pr.and_then(pr_tint),
                    help: pr.map_or_else(|| artifact.url.clone(), pr_help),
                    checks: None,
                }
            }
            ArtifactKind::LinearIssue => Self::quiet_artifact(
                artifact,
                linear_key(&artifact.url).unwrap_or_else(|| "Linear".to_owned()),
                "checklist",
            ),
            ArtifactKind::Preview => Self::quiet_artifact(
                artifact,
                url_port(&artifact.url)
                    .map_or_else(|| url_host(&artifact.url), |port| format!(":{port}")),
                "network",
            ),
            ArtifactKind::Link | ArtifactKind::Unknown => {
                Self::quiet_artifact(artifact, url_host(&artifact.url), "link")
            }
        }
    }

    fn quiet_artifact(
        artifact: &SessionArtifact,
        label: String,
        system_image: &'static str,
    ) -> Self {
        Self {
            id: format!("art-{}", artifact.url),
            label,
            system_image,
            open_url: Some(artifact.url.clone()),
            copy_string: artifact.url.clone(),
            tint: None,
            help: artifact.url.clone(),
            checks: None,
        }
    }

    fn checks_chip(artifact: &SessionArtifact, pr: &PullRequestStatus) -> Option<Self> {
        let total = pr.checks_passed + pr.checks_failed + pr.checks_pending;
        if total <= 0 {
            return None;
        }
        let (system_image, tint) = if pr.checks_failed > 0 {
            ("xmark.circle.fill", ChipTint::Red)
        } else if pr.checks_pending > 0 {
            ("clock.fill", ChipTint::Yellow)
        } else {
            ("checkmark.circle.fill", ChipTint::Green)
        };
        let mut states = vec![format!("{} passed", pr.checks_passed)];
        if pr.checks_failed > 0 {
            states.push(format!("{} failed", pr.checks_failed));
        }
        if pr.checks_pending > 0 {
            states.push(format!("{} running", pr.checks_pending));
        }
        Some(Self {
            id: format!("art-{}-checks", artifact.url),
            label: format!("{}/{total}", pr.checks_passed),
            system_image,
            open_url: Some(format!("{}/checks", artifact.url.trim_end_matches('/'))),
            copy_string: artifact.url.clone(),
            tint: Some(tint),
            help: format!("Checks: {}", states.join(" · ")),
            checks: Some(pr.clone()),
        })
    }

    fn comments_chip(artifact: &SessionArtifact, pr: &PullRequestStatus) -> Option<Self> {
        let count = pr.comment_count + pr.review_count;
        let (label, tint) = if let Some(total) = pr.total_threads.filter(|total| *total > 0) {
            let resolved = pr.resolved_threads.unwrap_or(0);
            (
                format!("{resolved}/{total}"),
                Some(if resolved == total {
                    ChipTint::Green
                } else {
                    ChipTint::Orange
                }),
            )
        } else if count > 0 {
            (count.to_string(), None)
        } else {
            return None;
        };
        Some(Self {
            id: format!("art-{}-comments", artifact.url),
            label,
            system_image: "bubble.left",
            open_url: Some(artifact.url.clone()),
            copy_string: artifact.url.clone(),
            tint,
            help: comments_help(pr),
            checks: None,
        })
    }
}

fn pr_number(url: &str) -> Option<String> {
    let parts: Vec<_> = url.split('/').filter(|part| !part.is_empty()).collect();
    if let Some(index) = parts.iter().position(|part| *part == "pull") {
        return parts
            .get(index + 1)
            .map(|part| part.chars().take_while(char::is_ascii_digit).collect())
            .filter(|part: &String| !part.is_empty());
    }
    parts
        .last()
        .filter(|part| part.chars().all(|character| character.is_ascii_digit()))
        .map(|part| (*part).to_owned())
}

fn linear_key(url: &str) -> Option<String> {
    let parts: Vec<_> = url.split('/').collect();
    let index = parts.iter().position(|part| *part == "issue")?;
    parts.get(index + 1).map(|part| (*part).to_owned())
}

fn url_host(url: &str) -> String {
    url.split_once("://")
        .map_or(url, |(_, remainder)| remainder)
        .split('/')
        .next()
        .unwrap_or(url)
        .split(':')
        .next()
        .unwrap_or(url)
        .to_owned()
}

fn url_port(url: &str) -> Option<u16> {
    let authority = url
        .split_once("://")
        .map_or(url, |(_, remainder)| remainder)
        .split('/')
        .next()?;
    authority.rsplit_once(':')?.1.parse().ok()
}

fn pr_tint(pr: &PullRequestStatus) -> Option<ChipTint> {
    if pr.state == "MERGED" {
        return Some(ChipTint::Purple);
    }
    if pr.state == "CLOSED" || pr.mergeable.as_deref() == Some("CONFLICTING") {
        return Some(ChipTint::Red);
    }
    if pr.is_draft {
        return None;
    }
    match pr.review_decision.as_deref() {
        Some("CHANGES_REQUESTED") => Some(ChipTint::Orange),
        Some("REVIEW_REQUIRED") => Some(ChipTint::Yellow),
        Some("APPROVED") => Some(ChipTint::Green),
        _ => None,
    }
}

fn pr_help(pr: &PullRequestStatus) -> String {
    let overall = if pr.state == "MERGED" {
        "merged"
    } else if pr.state == "CLOSED" {
        "closed"
    } else if pr.is_draft {
        "draft"
    } else {
        "open"
    };
    let title = pr.title.as_deref().map_or_else(
        || overall.to_owned(),
        |title| format!("{title} — {overall}"),
    );
    format!(
        "{title} · +{} −{} · {} file{}",
        pr.additions,
        pr.deletions,
        pr.changed_files,
        if pr.changed_files == 1 { "" } else { "s" }
    )
}

fn comments_help(pr: &PullRequestStatus) -> String {
    let mut parts = Vec::new();
    if let Some(total) = pr.total_threads.filter(|total| *total > 0) {
        parts.push(format!(
            "{} of {total} threads resolved",
            pr.resolved_threads.unwrap_or(0)
        ));
    }
    parts.push(format!(
        "{} comment{}",
        pr.comment_count,
        if pr.comment_count == 1 { "" } else { "s" }
    ));
    parts.push(format!(
        "{} review{}",
        pr.review_count,
        if pr.review_count == 1 { "" } else { "s" }
    ));
    parts.join(" · ")
}
