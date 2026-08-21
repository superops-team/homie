//! Wake invalidation and refresh cadence for the PR monitor.
//!
//! The control server signals focus/selection changes and the governor signals
//! newly discovered artifacts. This module owns the condition-variable wake
//! bookkeeping plus the foreground/background refresh backoff state.

use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

/// Match the GitHub Pull Requests extension: active PR UI refreshes once a
/// minute, while background state starts at five minutes and backs off.
const FOREGROUND_REFRESH_INTERVAL: Duration = Duration::from_secs(60);
const BACKGROUND_REFRESH_INTERVAL: Duration = Duration::from_secs(5 * 60);
const MAX_BACKGROUND_REFRESH_INTERVAL: Duration = Duration::from_secs(30 * 60);
const STOP_CHECK_INTERVAL: Duration = Duration::from_secs(1);

pub(crate) fn initial_sweep_delay() -> Duration {
    Duration::ZERO
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum PollInterest {
    Background,
    Foreground,
}

pub(crate) fn poll_interest(attached: bool, foreground_active: bool) -> PollInterest {
    if attached && foreground_active {
        PollInterest::Foreground
    } else {
        PollInterest::Background
    }
}

#[derive(Debug)]
pub(crate) struct RefreshState {
    pub(crate) last_attempt: Option<Instant>,
    pub(crate) background_interval: Duration,
}

impl Default for RefreshState {
    fn default() -> Self {
        Self {
            last_attempt: None,
            background_interval: BACKGROUND_REFRESH_INTERVAL,
        }
    }
}

impl RefreshState {
    pub(crate) fn interval(&self, interest: PollInterest) -> Duration {
        match interest {
            PollInterest::Foreground => FOREGROUND_REFRESH_INTERVAL,
            PollInterest::Background => self.background_interval,
        }
    }

    pub(crate) fn record_result(&mut self, interest: PollInterest, changed: bool) {
        if changed {
            self.background_interval = BACKGROUND_REFRESH_INTERVAL;
        } else if interest == PollInterest::Background {
            self.background_interval = std::cmp::min(
                MAX_BACKGROUND_REFRESH_INTERVAL,
                self.background_interval.saturating_mul(2),
            );
        }
    }
}

#[derive(Default)]
pub(crate) struct PendingWake {
    pub(crate) reconcile: bool,
    pub(crate) foreground: bool,
    pub(crate) sessions: HashSet<String>,
}

impl PendingWake {
    pub(crate) fn is_empty(&self) -> bool {
        !self.reconcile && !self.foreground && self.sessions.is_empty()
    }
}

struct WakeInner {
    pending: Mutex<PendingWake>,
    ready: Condvar,
    foreground_active: AtomicBool,
}

/// Event-driven invalidation for the PR monitor. The control server signals
/// focus/selection and the governor signals newly discovered artifacts.
#[derive(Clone)]
pub struct PrMonitorWake {
    inner: Arc<WakeInner>,
}

impl Default for PrMonitorWake {
    fn default() -> Self {
        Self {
            inner: Arc::new(WakeInner {
                pending: Mutex::new(PendingWake::default()),
                ready: Condvar::new(),
                // The desktop store starts active and only emits a transition
                // when that changes, so the daemon must share that default.
                foreground_active: AtomicBool::new(true),
            }),
        }
    }
}

impl PrMonitorWake {
    pub fn wake_session(&self, session_id: impl Into<String>) {
        let mut pending = self.inner.pending.lock().expect("PR monitor wake");
        pending.sessions.insert(session_id.into());
        drop(pending);
        self.inner.ready.notify_one();
    }

    /// A foreground transition refreshes only attached sessions, not every
    /// record that happened to be viewed in the recent window.
    pub fn set_foreground_active(&self, active: bool) {
        self.inner.foreground_active.store(active, Ordering::SeqCst);
        let mut pending = self.inner.pending.lock().expect("PR monitor wake");
        pending.reconcile = true;
        pending.foreground |= active;
        drop(pending);
        self.inner.ready.notify_one();
    }

    pub(crate) fn foreground_active(&self) -> bool {
        self.inner.foreground_active.load(Ordering::SeqCst)
    }

    pub(crate) fn wait(&self, timeout: Duration, stop: &AtomicBool) -> PendingWake {
        if timeout.is_zero() {
            return PendingWake::default();
        }
        let deadline = Instant::now() + timeout;
        let mut pending = self.inner.pending.lock().expect("PR monitor wake");
        loop {
            if !pending.is_empty() || stop.load(Ordering::SeqCst) {
                return std::mem::take(&mut *pending);
            }
            let Some(remaining) = deadline.checked_duration_since(Instant::now()) else {
                return PendingWake::default();
            };
            let wait_for = remaining.min(STOP_CHECK_INTERVAL);
            let (next, wait) = self
                .inner
                .ready
                .wait_timeout(pending, wait_for)
                .expect("PR monitor wake");
            pending = next;
            if wait.timed_out() && remaining <= STOP_CHECK_INTERVAL {
                return PendingWake::default();
            }
        }
    }
}
