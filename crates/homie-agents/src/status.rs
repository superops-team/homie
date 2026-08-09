use std::collections::HashSet;
use std::time::{Duration, SystemTime};

use homie_proto::{NeedsInputDetail, NeedsInputKind, NeedsInputSource, RiskHint, SessionStatus};

use crate::detect::manifest::ManifestState;
use crate::detect::redact::redact;
use crate::detect::{ScreenObservation, classify_risk};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Authority {
    HooksPrimary,
    ScreenPrimary,
    ProcessOnly,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ClaudeHook {
    SessionStart,
    UserPromptSubmit,
    PreToolUse,
    PermissionRequest {
        tool_name: Option<String>,
        input_summary: Option<String>,
    },
    Notification {
        notification_type: Option<String>,
        message: Option<String>,
    },
    Stop,
    SubagentStart(String),
    SubagentStop(String),
    SessionEnd,
}

#[derive(Clone, Debug, PartialEq)]
pub enum StatusSignal {
    ClaudeHook {
        hook: ClaudeHook,
        is_subagent: bool,
    },
    CodexTurnComplete,
    Screen(ScreenObservation),
    PtyOutputActivity,
    UserKeystroke,
    ProcessExit {
        code: Option<i32>,
        signal: Option<i32>,
    },
    Tick,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ReducerOutcome {
    pub status_change: Option<SessionStatus>,
    pub needs_input: Option<NeedsInputDetail>,
    pub turn_completed: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct ReducerTiming {
    pub idle_confirmations: u32,
    pub idle_confirm_cap: Duration,
    pub startup_grace: Duration,
    pub blocker_clear_scans: u32,
    pub staleness_timeout: Duration,
}

impl Default for ReducerTiming {
    fn default() -> Self {
        Self {
            idle_confirmations: 3,
            idle_confirm_cap: Duration::from_millis(700),
            startup_grace: Duration::from_secs(3),
            blocker_clear_scans: 2,
            staleness_timeout: Duration::from_secs(60),
        }
    }
}

#[derive(Clone, Debug)]
struct InternalState {
    spawned_at: SystemTime,
    last_signal_at: SystemTime,
    turn_in_flight: bool,
    active_subagents: HashSet<String>,
    idle_candidate_since: Option<SystemTime>,
    idle_confirms: u32,
    idle_strong: bool,
    pending_turn_completed: bool,
    screen_blocker_active: bool,
    blocker_miss_scans: u32,
    screen_belief: Option<ManifestState>,
    last_screen_seq: Option<u64>,
}

impl InternalState {
    fn new(spawned_at: SystemTime) -> Self {
        Self {
            spawned_at,
            last_signal_at: spawned_at,
            turn_in_flight: false,
            active_subagents: HashSet::new(),
            idle_candidate_since: None,
            idle_confirms: 0,
            idle_strong: false,
            pending_turn_completed: false,
            screen_blocker_active: false,
            blocker_miss_scans: 0,
            screen_belief: None,
            last_screen_seq: None,
        }
    }
}

pub struct StatusReducer {
    status: SessionStatus,
    authority: Authority,
    timing: ReducerTiming,
    state: InternalState,
}

impl StatusReducer {
    pub fn new(authority: Authority, spawned_at: SystemTime) -> Self {
        Self {
            status: SessionStatus::Starting,
            authority,
            timing: ReducerTiming::default(),
            state: InternalState::new(spawned_at),
        }
    }

    pub fn with_timing(mut self, timing: ReducerTiming) -> Self {
        self.timing = timing;
        self
    }

    pub fn status(&self) -> &SessionStatus {
        &self.status
    }

    pub fn active_subagents(&self) -> usize {
        self.state.active_subagents.len()
    }

    pub fn reduce(&mut self, signal: StatusSignal, now: SystemTime) -> ReducerOutcome {
        let mut outcome = ReducerOutcome::default();

        if self.status == SessionStatus::Exited {
            return outcome;
        }

        if matches!(signal, StatusSignal::ProcessExit { .. }) {
            self.set_status(SessionStatus::Exited, &mut outcome);
            return outcome;
        }

        if self.authority == Authority::ProcessOnly {
            if matches!(signal, StatusSignal::PtyOutputActivity)
                && self.status == SessionStatus::Starting
            {
                self.state.turn_in_flight = true;
                self.state.last_signal_at = now;
                self.set_status(SessionStatus::Running, &mut outcome);
            }
            return outcome;
        }

        match signal {
            StatusSignal::ProcessExit { .. } => {}
            StatusSignal::PtyOutputActivity | StatusSignal::UserKeystroke => {
                self.state.last_signal_at = now;
            }
            StatusSignal::ClaudeHook { hook, is_subagent } => {
                self.handle_claude_hook(hook, is_subagent, now, &mut outcome);
            }
            StatusSignal::CodexTurnComplete => {
                self.state.last_signal_at = now;
                self.handle_strong_idle(now, &mut outcome);
            }
            StatusSignal::Screen(observation) => self.handle_screen(observation, now, &mut outcome),
            StatusSignal::Tick => self.handle_tick(now, &mut outcome),
        }

        outcome
    }

    fn set_status(&mut self, new: SessionStatus, outcome: &mut ReducerOutcome) {
        if self.status != new {
            self.status = new.clone();
            outcome.status_change = Some(new);
        }
    }

    fn cancel_idle_candidacy(&mut self) {
        self.state.idle_candidate_since = None;
        self.state.idle_confirms = 0;
        self.state.idle_strong = false;
        self.state.pending_turn_completed = false;
    }

    fn go_working(&mut self, now: SystemTime, clear_blocker: bool, outcome: &mut ReducerOutcome) {
        self.cancel_idle_candidacy();
        if clear_blocker {
            self.state.screen_blocker_active = false;
            self.state.blocker_miss_scans = 0;
        }
        self.state.turn_in_flight = true;
        self.state.last_signal_at = now;
        self.set_status(SessionStatus::Running, outcome);
    }

    fn handle_strong_idle(&mut self, now: SystemTime, outcome: &mut ReducerOutcome) {
        if self.status == SessionStatus::Starting {
            self.set_status(SessionStatus::Idle, outcome);
            return;
        }
        if self.status != SessionStatus::Running {
            return;
        }
        self.state.idle_strong = true;
        self.state.pending_turn_completed = self.state.turn_in_flight;
        if self.state.idle_candidate_since.is_none() {
            self.state.idle_candidate_since = Some(now);
        }
        if self.state.screen_belief == Some(ManifestState::Idle) {
            self.state.idle_confirms += 1;
            self.commit_idle(now, outcome);
        }
    }

    fn confirm_idle(&mut self, now: SystemTime, outcome: &mut ReducerOutcome) {
        if self.status != SessionStatus::Running {
            return;
        }
        if self.state.idle_candidate_since.is_none() {
            self.state.idle_candidate_since = Some(now);
            self.state.idle_confirms = 0;
        }
        self.state.idle_confirms += 1;
        let required = if self.state.idle_strong {
            1
        } else {
            self.timing.idle_confirmations
        };
        let elapsed = self
            .state
            .idle_candidate_since
            .and_then(|since| now.duration_since(since).ok())
            .unwrap_or_default();
        if self.state.idle_confirms >= required
            || (elapsed >= self.timing.idle_confirm_cap && self.state.idle_confirms >= 1)
        {
            self.commit_idle(now, outcome);
        }
    }

    fn commit_idle(&mut self, now: SystemTime, outcome: &mut ReducerOutcome) {
        let fire = self.state.pending_turn_completed;
        self.set_status(SessionStatus::Idle, outcome);
        self.state.turn_in_flight = false;
        if fire {
            outcome.turn_completed = true;
        }
        self.cancel_idle_candidacy();
        self.state.last_signal_at = now;
    }

    fn handle_claude_hook(
        &mut self,
        hook: ClaudeHook,
        is_subagent: bool,
        now: SystemTime,
        outcome: &mut ReducerOutcome,
    ) {
        self.state.last_signal_at = now;
        match &hook {
            ClaudeHook::SubagentStart(id) => {
                self.state.active_subagents.insert(id.clone());
                return;
            }
            ClaudeHook::SubagentStop(id) => {
                self.state.active_subagents.remove(id);
                return;
            }
            _ => {}
        }
        if is_subagent {
            return;
        }

        match hook {
            ClaudeHook::SessionStart => {
                if self.status == SessionStatus::Starting {
                    self.set_status(SessionStatus::Idle, outcome);
                }
            }
            ClaudeHook::UserPromptSubmit | ClaudeHook::PreToolUse => {
                self.go_working(now, true, outcome);
            }
            ClaudeHook::PermissionRequest {
                tool_name,
                input_summary,
            } => {
                let detail = permission_detail(tool_name, input_summary, now);
                outcome.needs_input = Some(detail);
                self.cancel_idle_candidacy();
                self.set_status(SessionStatus::NeedsInput, outcome);
            }
            ClaudeHook::Notification {
                notification_type,
                message,
            } => self.handle_notification(notification_type, message, now, outcome),
            ClaudeHook::Stop => self.handle_strong_idle(now, outcome),
            ClaudeHook::SessionEnd | ClaudeHook::SubagentStart(_) | ClaudeHook::SubagentStop(_) => {
            }
        }
    }

    fn handle_notification(
        &mut self,
        notification_type: Option<String>,
        message: Option<String>,
        now: SystemTime,
        outcome: &mut ReducerOutcome,
    ) {
        match notification_type.as_deref() {
            Some("permission_prompt") => {
                outcome.needs_input = Some(notification_detail(
                    NeedsInputKind::Approval,
                    message.unwrap_or_else(|| "Permission required".into()),
                    now,
                ));
                self.cancel_idle_candidacy();
                self.set_status(SessionStatus::NeedsInput, outcome);
            }
            Some("idle_prompt") | Some("agent_needs_input") | Some("elicitation_dialog") => {
                outcome.needs_input = Some(notification_detail(
                    NeedsInputKind::Question,
                    message.unwrap_or_else(|| "Waiting for input".into()),
                    now,
                ));
                self.cancel_idle_candidacy();
                self.set_status(SessionStatus::NeedsInput, outcome);
            }
            Some("agent_completed") => self.handle_strong_idle(now, outcome),
            _ => {}
        }
    }

    fn handle_screen(
        &mut self,
        observation: ScreenObservation,
        now: SystemTime,
        outcome: &mut ReducerOutcome,
    ) {
        self.state.last_signal_at = now;
        if observation.state == ManifestState::Skip {
            return;
        }
        if self.state.last_screen_seq == Some(observation.content_seq) {
            return;
        }
        self.state.last_screen_seq = Some(observation.content_seq);
        self.state.screen_belief = Some(observation.state);

        if observation.state == ManifestState::BlockedPermission
            || observation.state == ManifestState::BlockedQuestion
        {
            self.state.screen_blocker_active = true;
            self.state.blocker_miss_scans = 0;
            outcome.needs_input = Some(screen_detail(&observation, now));
            self.cancel_idle_candidacy();
            self.set_status(SessionStatus::NeedsInput, outcome);
            return;
        }

        if self.state.screen_blocker_active {
            self.state.blocker_miss_scans += 1;
            if self.state.blocker_miss_scans < self.timing.blocker_clear_scans {
                return;
            }
            self.state.screen_blocker_active = false;
            self.state.blocker_miss_scans = 0;
        }

        if self.status == SessionStatus::Starting {
            let elapsed = now
                .duration_since(self.state.spawned_at)
                .unwrap_or_default();
            let definitive = self.authority == Authority::ScreenPrimary
                && observation.state == ManifestState::Working;
            if elapsed < self.timing.startup_grace && !definitive {
                return;
            }
        }

        match observation.state {
            ManifestState::Working => self.go_working(now, false, outcome),
            ManifestState::Idle => {
                if self.status == SessionStatus::Running {
                    self.confirm_idle(now, outcome);
                } else if self.status == SessionStatus::Starting
                    || self.status == SessionStatus::NeedsInput
                {
                    self.set_status(SessionStatus::Idle, outcome);
                }
            }
            ManifestState::BlockedPermission
            | ManifestState::BlockedQuestion
            | ManifestState::Skip => {}
        }
    }

    fn handle_tick(&mut self, now: SystemTime, outcome: &mut ReducerOutcome) {
        if self.status == SessionStatus::Running {
            let quiet = now
                .duration_since(self.state.last_signal_at)
                .unwrap_or_default();
            if quiet > self.timing.staleness_timeout {
                self.set_status(SessionStatus::Unknown("stale".into()), outcome);
                return;
            }
        }
        if self.status == SessionStatus::Running
            && self.state.idle_strong
            && self.state.idle_candidate_since.is_some()
        {
            self.state.idle_confirms += 1;
            self.commit_idle(now, outcome);
        }
    }
}

fn permission_detail(
    tool_name: Option<String>,
    input_summary: Option<String>,
    now: SystemTime,
) -> NeedsInputDetail {
    let tool = tool_name.clone().unwrap_or_default();
    let summary = match tool.as_str() {
        "Bash" => format!(
            "wants to run `{}`",
            input_summary.clone().unwrap_or_default()
        ),
        "Edit" | "Write" | "MultiEdit" | "NotebookEdit" => format!(
            "wants to edit {}",
            input_summary.clone().unwrap_or_else(|| "a file".into())
        ),
        "" => input_summary
            .clone()
            .unwrap_or_else(|| "Permission required".into()),
        other => match &input_summary {
            Some(detail) => format!("wants to use {other}: {detail}"),
            None => format!("wants to use {other}"),
        },
    };
    let risk_source = input_summary.clone().unwrap_or(tool);
    NeedsInputDetail {
        kind: NeedsInputKind::Approval,
        source: NeedsInputSource::Hook,
        tool_name,
        summary: redact(&summary),
        prompt_excerpt: input_summary.as_deref().map(redact),
        options: None,
        risk_hint: proto_risk(classify_risk(&risk_source)),
        occurred_at: unix(now),
    }
}

fn notification_detail(kind: NeedsInputKind, text: String, now: SystemTime) -> NeedsInputDetail {
    NeedsInputDetail {
        kind,
        source: NeedsInputSource::Hook,
        tool_name: None,
        summary: redact(&text),
        prompt_excerpt: None,
        options: None,
        risk_hint: proto_risk(classify_risk(&text)),
        occurred_at: unix(now),
    }
}

fn screen_detail(observation: &ScreenObservation, now: SystemTime) -> NeedsInputDetail {
    let first_line = observation.prompt_excerpt.as_ref().and_then(|excerpt| {
        excerpt
            .split('\n')
            .map(str::trim)
            .find(|line| !line.is_empty())
            .map(str::to_string)
    });
    let summary = first_line.unwrap_or_else(|| "Waiting for input".into());
    let risk_source = observation
        .prompt_excerpt
        .clone()
        .unwrap_or_else(|| summary.clone());
    NeedsInputDetail {
        kind: if observation.state == ManifestState::BlockedQuestion {
            NeedsInputKind::Question
        } else {
            NeedsInputKind::Approval
        },
        source: NeedsInputSource::ScreenScrape,
        tool_name: None,
        summary: redact(&summary),
        prompt_excerpt: observation.prompt_excerpt.clone(),
        options: observation.options.clone(),
        risk_hint: proto_risk(classify_risk(&risk_source)),
        occurred_at: unix(now),
    }
}

fn proto_risk(risk: crate::detect::RiskHint) -> RiskHint {
    match risk {
        crate::detect::RiskHint::Neutral => RiskHint::Neutral,
        crate::detect::RiskHint::FileWrite => RiskHint::FileWrite,
        crate::detect::RiskHint::Network => RiskHint::Network,
        crate::detect::RiskHint::Destructive => RiskHint::Destructive,
    }
}

fn unix(now: SystemTime) -> i64 {
    now.duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}
