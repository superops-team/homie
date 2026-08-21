//! Polls GitHub (via the `gh` CLI) for the state of every PR URL captured as
//! a session artifact: open/merged/closed, draft, review decision,
//! mergeability, CI checks, comment counts, and +/- line stats.
//!
//! Ported from `PullRequestMonitor`. Results land on
//! `SessionRecord.pullRequests`; one shared per-URL cache dedupes PRs that
//! appear in several sessions; fetches per sweep are capped so a screen full
//! of PR links can't turn one sweep into a minute of serial gh calls.
//! Silently inert when `gh` isn't installed.

mod github;
mod monitor;
mod wake;

pub use github::{fetch, parse, parse_threads, pr_coordinates};
pub use monitor::spawn_pr_monitor;
pub use wake::PrMonitorWake;

#[cfg(test)]
pub(crate) use monitor::next_refresh_delay;
#[cfg(test)]
pub(crate) use wake::{PollInterest, RefreshState, initial_sweep_delay, poll_interest};

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{HashMap, HashSet};
    use std::sync::atomic::AtomicBool;
    use std::time::{Duration, Instant};

    use homie_proto::DateMillis;

    #[test]
    fn coordinates_come_out_of_a_pr_url() {
        assert_eq!(
            pr_coordinates("https://github.com/cristicretu/homie/pull/7"),
            Some(("cristicretu".into(), "homie".into(), 7))
        );
        assert_eq!(pr_coordinates("https://github.com/x/pull"), None);
    }

    #[test]
    fn monitor_refreshes_immediately_on_start() {
        assert_eq!(initial_sweep_delay(), Duration::ZERO);
    }

    #[test]
    fn foreground_and_background_cadences_match_visible_pr_ui() {
        let mut state = RefreshState::default();
        assert_eq!(
            state.interval(PollInterest::Foreground),
            Duration::from_secs(60)
        );
        assert_eq!(
            state.interval(PollInterest::Background),
            Duration::from_secs(5 * 60)
        );

        state.record_result(PollInterest::Background, false);
        assert_eq!(
            state.interval(PollInterest::Background),
            Duration::from_secs(10 * 60)
        );
        state.record_result(PollInterest::Background, false);
        state.record_result(PollInterest::Background, false);
        state.record_result(PollInterest::Background, false);
        assert_eq!(
            state.interval(PollInterest::Background),
            Duration::from_secs(30 * 60),
            "background polling caps at thirty minutes"
        );

        state.record_result(PollInterest::Background, true);
        assert_eq!(
            state.interval(PollInterest::Background),
            Duration::from_secs(5 * 60),
            "activity resets the backoff"
        );
    }

    #[test]
    fn attached_prs_become_background_when_the_app_is_inactive() {
        assert_eq!(poll_interest(true, true), PollInterest::Foreground);
        assert_eq!(poll_interest(true, false), PollInterest::Background);
        assert_eq!(poll_interest(false, true), PollInterest::Background);
    }

    #[test]
    fn forced_or_never_fetched_prs_are_due_now() {
        let url = "https://github.com/o/r/pull/1".to_owned();
        let targets = HashMap::from([(url.clone(), PollInterest::Foreground)]);
        assert_eq!(
            next_refresh_delay(&targets, &HashMap::new(), &HashSet::new(), Instant::now()),
            Duration::ZERO
        );

        let refresh = HashMap::from([(
            url.clone(),
            RefreshState {
                last_attempt: Some(Instant::now()),
                ..RefreshState::default()
            },
        )]);
        assert_eq!(
            next_refresh_delay(&targets, &refresh, &HashSet::from([url]), Instant::now(),),
            Duration::ZERO
        );
    }

    #[test]
    fn visibility_wakes_are_delivered_without_waiting_for_the_timer() {
        let wake = PrMonitorWake::default();
        wake.wake_session("s_selected");
        wake.set_foreground_active(true);
        let stop = AtomicBool::new(false);

        let pending = wake.wait(Duration::from_secs(60), &stop);
        assert!(pending.reconcile);
        assert!(pending.foreground);
        assert!(pending.sessions.contains("s_selected"));
    }

    #[test]
    fn deactivation_wakes_reconciliation_without_forcing_a_network_refresh() {
        let wake = PrMonitorWake::default();
        wake.set_foreground_active(false);
        let stop = AtomicBool::new(false);

        let pending = wake.wait(Duration::from_secs(60), &stop);
        assert!(pending.reconcile);
        assert!(!pending.foreground);
        assert!(!wake.foreground_active());
    }

    #[test]
    fn a_gh_view_payload_parses_into_the_wire_status() {
        let payload = serde_json::json!({
            "number": 12,
            "title": "Add the thing",
            "author": {"login": "shawn"},
            "state": "OPEN",
            "isDraft": false,
            "reviewDecision": "",
            "additions": 10, "deletions": 2, "changedFiles": 3,
            "comments": [{"author": {"login": "giga"}, "body": "nice", "createdAt": "2026-08-07T10:00:00Z"}],
            "reviews": [{"author": {"login": "bot"}, "body": "lgtm", "state": "APPROVED", "submittedAt": "2026-08-07T11:00:00Z"}],
            "statusCheckRollup": [
                {"name": "test", "workflowName": "CI", "status": "COMPLETED", "conclusion": "SUCCESS", "detailsUrl": "https://x"},
                {"context": "lint", "state": "FAILURE"},
                {"name": "build", "status": "IN_PROGRESS", "conclusion": ""}
            ],
        });
        let status = parse(
            payload.to_string().as_bytes(),
            "https://github.com/o/r/pull/12",
            DateMillis(0.0),
        )
        .expect("parse");
        assert_eq!(status.number, 12);
        assert_eq!(status.author.as_deref(), Some("shawn"));
        assert_eq!(status.review_decision, None, "empty string means none");
        assert_eq!(
            (
                status.checks_passed,
                status.checks_failed,
                status.checks_pending
            ),
            (1, 1, 1)
        );
        let checks = status.checks.expect("checks");
        assert_eq!(checks[0].name, "CI / test");
        assert_eq!(checks[1].name, "lint");
        let discussion = status.discussion.expect("discussion");
        assert_eq!(discussion.len(), 2);
        assert_eq!(discussion[0].kind, "comment", "sorted by time");
        assert_eq!(discussion[1].state.as_deref(), Some("APPROVED"));
        assert!(
            discussion[0].created_at.expect("date").0 > 1.7e12,
            "the date parser lands in the right epoch decade"
        );
    }

    #[test]
    fn thread_counts_decode_from_graphql() {
        let payload = serde_json::json!({
            "data": {"repository": {"pullRequest": {"reviewThreads": {
                "totalCount": 5,
                "nodes": [{"isResolved": true}, {"isResolved": false}, {"isResolved": true}]
            }}}}
        });
        assert_eq!(parse_threads(payload.to_string().as_bytes()), Some((2, 5)));
    }
}
