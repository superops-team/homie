//! The PR monitor loop: decides which URLs are worth polling, caps each
//! sweep's subprocess budget, applies results to session records, and returns
//! the delay until the next sweep.

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use homie_proto::{ArtifactKind, PullRequestStatus};

use crate::attach::AttachHub;
use crate::events::EventBus;
use crate::registry::Registry;

use super::github::{fetch, resolve_gh};
use super::wake::{
    PendingWake, PollInterest, PrMonitorWake, RefreshState, initial_sweep_delay, poll_interest,
};

/// Review-thread resolution costs a separate GraphQL subprocess; refresh it
/// much less often than the main PR state.
const THREAD_REFRESH_TTL: Duration = Duration::from_secs(1800);
/// Bound each batch, but immediately drain any remaining due work rather than
/// making it wait for another global sweep.
const MAX_FETCHES_PER_BATCH: usize = 2;
/// Recently-seen window: records viewed within this qualify for polling even
/// when no client is attached right now.
const RECENTLY_SEEN: Duration = Duration::from_secs(600);
/// A missed wake cannot strand work forever; this is a local reconciliation
/// only and does not imply a GitHub request when nothing is due.
const IDLE_RECONCILE_INTERVAL: Duration = Duration::from_secs(30 * 60);

pub fn spawn_pr_monitor(
    registry: Arc<Mutex<Registry>>,
    events: EventBus,
    attach: AttachHub,
    wake: PrMonitorWake,
    stop: Arc<AtomicBool>,
) -> std::thread::JoinHandle<()> {
    std::thread::Builder::new()
        .name("homie-pr-monitor".into())
        .spawn(move || {
            let Some(gh) = resolve_gh() else {
                eprintln!("homied-rs: pull-request monitor idle: gh not on PATH");
                return;
            };
            let mut cache: HashMap<String, PullRequestStatus> = HashMap::new();
            let mut refresh: HashMap<String, RefreshState> = HashMap::new();
            let mut last_thread_attempt: HashMap<String, Instant> = HashMap::new();
            let mut forced_urls = HashSet::new();
            let mut delay = initial_sweep_delay();
            loop {
                let pending = wake.wait(delay, &stop);
                if stop.load(Ordering::SeqCst) {
                    return;
                }
                delay = sweep(
                    &registry,
                    &events,
                    &attach,
                    &gh,
                    &mut cache,
                    &mut refresh,
                    &mut last_thread_attempt,
                    &mut forced_urls,
                    pending,
                    wake.foreground_active(),
                );
            }
        })
        .expect("spawn pr monitor")
}

#[allow(clippy::too_many_arguments)]
fn sweep(
    registry: &Arc<Mutex<Registry>>,
    events: &EventBus,
    attach: &AttachHub,
    gh: &str,
    cache: &mut HashMap<String, PullRequestStatus>,
    refresh: &mut HashMap<String, RefreshState>,
    last_thread_attempt: &mut HashMap<String, Instant>,
    forced_urls: &mut HashSet<String>,
    pending: PendingWake,
    foreground_active: bool,
) -> Duration {
    // PR URLs worth polling: sessions currently attached, or viewed recently.
    // Merely being a live restored process is not evidence anyone is looking
    // at its PR pill.
    let records = {
        let Ok(guard) = registry.lock() else {
            return IDLE_RECONCILE_INTERVAL;
        };
        guard.records()
    };
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as f64;
    let mut wanted: Vec<(String, Vec<String>)> = Vec::new();
    let mut targets: HashMap<String, PollInterest> = HashMap::new();
    for record in &records {
        let recently_seen = record
            .last_seen_at
            .as_ref()
            .is_some_and(|seen| now_ms - seen.0 < RECENTLY_SEEN.as_millis() as f64);
        let attached = attach.has_sinks(&record.id.0);
        if !(attached || recently_seen) {
            continue;
        }
        let urls: Vec<String> = record
            .artifacts
            .as_deref()
            .unwrap_or_default()
            .iter()
            .filter(|artifact| artifact.kind == ArtifactKind::PullRequest)
            .map(|artifact| artifact.url.clone())
            .collect();
        if !urls.is_empty() {
            let interest = poll_interest(attached, foreground_active);
            for url in &urls {
                targets
                    .entry(url.clone())
                    .and_modify(|current| *current = (*current).max(interest))
                    .or_insert(interest);
            }
            if pending.sessions.contains(&record.id.0) || (pending.foreground && attached) {
                forced_urls.extend(urls.iter().cloned());
            }
            if let Some(statuses) = &record.pull_requests {
                for status in statuses {
                    let replace = cache
                        .get(&status.url)
                        .is_none_or(|cached| cached.fetched_at < status.fetched_at);
                    if replace {
                        cache.insert(status.url.clone(), status.clone());
                    }
                }
            }
            wanted.push((record.id.0.clone(), urls));
        }
    }
    if wanted.is_empty() {
        forced_urls.clear();
        return IDLE_RECONCILE_INTERVAL;
    }
    forced_urls.retain(|url| targets.contains_key(url));

    // Foreground first, then never-attempted/oldest. The batch cap protects
    // serial gh subprocesses; remaining due URLs make the returned delay zero
    // and are drained immediately by the next iteration.
    let now = Instant::now();
    let mut due: Vec<(String, PollInterest, Option<Instant>)> = targets
        .iter()
        .filter_map(|(url, interest)| {
            let state = refresh.entry(url.clone()).or_default();
            let forced = forced_urls.contains(url);
            let is_due = forced
                || state.last_attempt.is_none_or(|at| {
                    now.saturating_duration_since(at) >= state.interval(*interest)
                });
            is_due.then_some((url.clone(), *interest, state.last_attempt))
        })
        .collect();
    due.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.2.cmp(&b.2)));
    for (url, interest, _) in due.into_iter().take(MAX_FETCHES_PER_BATCH) {
        forced_urls.remove(&url);
        refresh.entry(url.clone()).or_default().last_attempt = Some(Instant::now());
        let refresh_threads = last_thread_attempt
            .get(&url)
            .is_none_or(|at| at.elapsed() >= THREAD_REFRESH_TTL);
        if refresh_threads {
            last_thread_attempt.insert(url.clone(), Instant::now());
        }
        let mut changed = false;
        if let Some(mut status) = fetch(&url, gh, refresh_threads) {
            if !refresh_threads && let Some(previous) = cache.get(&url) {
                status.resolved_threads = previous.resolved_threads;
                status.total_threads = previous.total_threads;
            }
            changed = cache
                .get(&url)
                .is_none_or(|previous| !status_materially_same(previous, &status));
            cache.insert(url.clone(), status);
        }
        refresh
            .entry(url)
            .or_default()
            .record_result(interest, changed);
    }

    for (id, urls) in wanted {
        let statuses: Vec<PullRequestStatus> = urls
            .iter()
            .filter_map(|url| cache.get(url).cloned())
            .collect();
        let record = {
            let Ok(mut guard) = registry.lock() else {
                return IDLE_RECONCILE_INTERVAL;
            };
            let changed = guard.apply_pull_request_statuses(&id, statuses);
            if changed {
                let _ = guard.persist();
                guard.records().into_iter().find(|record| record.id.0 == id)
            } else {
                None
            }
        };
        if let Some(record) = record {
            events.publish_encoded(homie_proto::EventName::SESSION_UPDATED, &record, Some(&id));
        }
    }

    next_refresh_delay(&targets, refresh, forced_urls, Instant::now())
}

pub(crate) fn next_refresh_delay(
    targets: &HashMap<String, PollInterest>,
    refresh: &HashMap<String, RefreshState>,
    forced_urls: &HashSet<String>,
    now: Instant,
) -> Duration {
    targets
        .iter()
        .map(|(url, interest)| {
            if forced_urls.contains(url) {
                return Duration::ZERO;
            }
            let Some(state) = refresh.get(url) else {
                return Duration::ZERO;
            };
            let Some(last_attempt) = state.last_attempt else {
                return Duration::ZERO;
            };
            state
                .interval(*interest)
                .saturating_sub(now.saturating_duration_since(last_attempt))
        })
        .min()
        .unwrap_or(IDLE_RECONCILE_INTERVAL)
}

fn status_materially_same(a: &PullRequestStatus, b: &PullRequestStatus) -> bool {
    let mut b_pinned = b.clone();
    b_pinned.fetched_at = a.fetched_at;
    *a == b_pinned
}
