//! Homie observability contracts.

use serde_json::Value;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::path::PathBuf;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SafeFieldError {
    DangerousField { field: String },
    ExpectedObject,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SafeFields {
    values: BTreeMap<String, Value>,
}

impl SafeFields {
    pub fn project(input: &Value) -> Result<Self, SafeFieldError> {
        let object = input.as_object().ok_or(SafeFieldError::ExpectedObject)?;
        let mut values = BTreeMap::new();

        for (field, value) in object {
            if is_dangerous_field(field) {
                return Err(SafeFieldError::DangerousField {
                    field: field.clone(),
                });
            }
            ensure_value_has_no_dangerous_keys(field, value)?;
            if is_allowed_field(field) {
                values.insert(field.clone(), value.clone());
            }
        }

        Ok(Self { values })
    }

    pub fn get(&self, field: &str) -> Option<&Value> {
        self.values.get(field)
    }

    pub fn insert_safe(
        &mut self,
        field: impl Into<String>,
        value: Value,
    ) -> Result<(), SafeFieldError> {
        let field = field.into();
        if is_dangerous_field(&field) {
            return Err(SafeFieldError::DangerousField { field });
        }
        if is_allowed_field(&field) {
            self.values.insert(field, value);
        }
        Ok(())
    }

    pub fn into_inner(self) -> BTreeMap<String, Value> {
        self.values
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum EventName {
    SessionUpdated,
    SessionResources,
    SessionRemoved,
    ProjectUpdated,
    SessionSpawned,
    SessionStatus,
    SessionNeedsInput,
    SessionOutput,
    SessionArtifact,
    SessionArchived,
    WorktreeCreated,
    WorktreeRemoved,
    EventsDropped,
    MetricsWriteFailed,
    VerificationFunctionalCaseExecuted,
}

impl EventName {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SessionUpdated => "session.updated",
            Self::SessionResources => "session.resources",
            Self::SessionRemoved => "session.removed",
            Self::ProjectUpdated => "project.updated",
            Self::SessionSpawned => "session.spawned",
            Self::SessionStatus => "session.status",
            Self::SessionNeedsInput => "session.needs_input",
            Self::SessionOutput => "session.output",
            Self::SessionArtifact => "session.artifact",
            Self::SessionArchived => "session.archived",
            Self::WorktreeCreated => "worktree.created",
            Self::WorktreeRemoved => "worktree.removed",
            Self::EventsDropped => "events.dropped",
            Self::MetricsWriteFailed => "metrics.write_failed",
            Self::VerificationFunctionalCaseExecuted => "verification.functional_case_executed",
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct EventFilter {
    sessions: BTreeSet<String>,
    kinds: BTreeSet<EventName>,
}

impl EventFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_session(mut self, session_id: impl Into<String>) -> Self {
        self.sessions.insert(session_id.into());
        self
    }

    pub fn with_kind(mut self, kind: EventName) -> Self {
        self.kinds.insert(kind);
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SafeEvent {
    name: EventName,
    seq: u64,
    session_id: Option<String>,
    fields: SafeFields,
}

impl SafeEvent {
    pub fn new(name: EventName, seq: u64, session_id: Option<String>, fields: SafeFields) -> Self {
        Self {
            name,
            seq,
            session_id,
            fields,
        }
    }

    pub fn events_dropped(
        dropped: u64,
        from_seq: u64,
        to_seq: u64,
    ) -> Result<Self, SafeFieldError> {
        let fields = SafeFields::project(&serde_json::json!({
            "event.dropped": dropped,
            "event.from_seq": from_seq,
            "event.to_seq": to_seq,
        }))?;
        Ok(Self::new(EventName::EventsDropped, 0, None, fields))
    }

    pub fn name(&self) -> EventName {
        self.name
    }

    pub fn seq(&self) -> u64 {
        self.seq
    }

    pub fn session_id(&self) -> Option<&str> {
        self.session_id.as_deref()
    }

    pub fn fields(&self) -> &SafeFields {
        &self.fields
    }

    pub fn visible_to(&self, filter: &EventFilter) -> bool {
        if self.name == EventName::EventsDropped {
            return true;
        }
        if !filter.kinds.is_empty() && !filter.kinds.contains(&self.name) {
            return false;
        }
        if !filter.sessions.is_empty() {
            let Some(session_id) = &self.session_id else {
                return false;
            };
            return filter.sessions.contains(session_id);
        }
        true
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MetricsWriteFailure {
    pub metrics_kind: String,
    pub metrics_scope: String,
    pub component: String,
    pub operation: String,
    pub safe_error_code: String,
    pub retryable: bool,
    pub occurred_at: i64,
}

impl MetricsWriteFailure {
    pub fn to_event(&self, seq: u64) -> Result<SafeEvent, SafeFieldError> {
        let fields = SafeFields::project(&serde_json::json!({
            "metrics.kind": self.metrics_kind,
            "metrics.scope": self.metrics_scope,
            "component": self.component,
            "operation": self.operation,
            "safe_error_code": self.safe_error_code,
            "retryable": self.retryable,
            "occurred_at": self.occurred_at,
        }))?;
        Ok(SafeEvent::new(
            EventName::MetricsWriteFailed,
            seq,
            None,
            fields,
        ))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UsageValueKind {
    SubscriptionQuota,
    EstimatedApiEquivalent,
    AuthoritativeBilled,
}

impl UsageValueKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SubscriptionQuota => "subscription_quota",
            Self::EstimatedApiEquivalent => "estimated_api_equivalent",
            Self::AuthoritativeBilled => "authoritative_billed",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UsageSource {
    AppServer,
    Otel,
    Transcript,
    Manual,
}

impl UsageSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::AppServer => "app_server",
            Self::Otel => "otel",
            Self::Transcript => "transcript",
            Self::Manual => "manual",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct UsageEvidence {
    pub provider: String,
    pub profile_id: Option<String>,
    pub session_id: Option<String>,
    pub model: Option<String>,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_read_tokens: i64,
    pub cache_write_tokens: i64,
    pub estimated_usd: Option<f64>,
    pub billed_usd: Option<f64>,
    pub value_kind: UsageValueKind,
    pub source: UsageSource,
    pub occurred_at: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UsageEvidenceError {
    NegativeTokens,
    InvalidCost,
    SafeField(SafeFieldError),
}

impl From<SafeFieldError> for UsageEvidenceError {
    fn from(error: SafeFieldError) -> Self {
        Self::SafeField(error)
    }
}

impl UsageEvidence {
    pub fn to_safe_fields(&self) -> Result<SafeFields, UsageEvidenceError> {
        self.validate()?;
        let fields = SafeFields::project(&serde_json::json!({
            "usage.provider": self.provider,
            "usage.profile_id": self.profile_id,
            "usage.session_id": self.session_id,
            "usage.model": self.model,
            "usage.input_tokens": self.input_tokens,
            "usage.output_tokens": self.output_tokens,
            "usage.cache_read_tokens": self.cache_read_tokens,
            "usage.cache_write_tokens": self.cache_write_tokens,
            "usage.estimated_usd": self.estimated_usd,
            "usage.billed_usd": self.billed_usd,
            "usage.value_kind": self.value_kind.as_str(),
            "usage.source": self.source.as_str(),
            "occurred_at": self.occurred_at,
        }))?;
        Ok(fields)
    }

    fn validate(&self) -> Result<(), UsageEvidenceError> {
        if [
            self.input_tokens,
            self.output_tokens,
            self.cache_read_tokens,
            self.cache_write_tokens,
        ]
        .into_iter()
        .any(|tokens| tokens < 0)
        {
            return Err(UsageEvidenceError::NegativeTokens);
        }
        if [self.estimated_usd, self.billed_usd]
            .into_iter()
            .flatten()
            .any(|cost| !cost.is_finite() || cost < 0.0)
        {
            return Err(UsageEvidenceError::InvalidCost);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GateStatus {
    Pass,
    Blocked,
    NotRun,
    Partial,
    Fail,
}

impl GateStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::Blocked => "blocked",
            Self::NotRun => "not_run",
            Self::Partial => "partial",
            Self::Fail => "fail",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CommandEvidence {
    pub command: String,
    pub exit_code: Option<i32>,
    pub status: GateStatus,
    pub output_summary: String,
    pub evidence_path: PathBuf,
    pub fields: SafeFields,
}

impl CommandEvidence {
    pub fn functional_case_event(
        &self,
        case_id: &str,
        seq: u64,
    ) -> Result<SafeEvent, SafeFieldError> {
        let mut fields = SafeFields::project(&serde_json::json!({
            "evidence.command": self.command,
            "evidence.exit_code": self.exit_code,
            "evidence.status": self.status.as_str(),
            "evidence.output_summary": self.output_summary,
            "evidence.path": self.evidence_path.to_string_lossy(),
            "evidence.case_id": case_id,
        }))?;
        for (field, value) in self.fields.clone().into_inner() {
            fields.insert_safe(field, value)?;
        }
        Ok(SafeEvent::new(
            EventName::VerificationFunctionalCaseExecuted,
            seq,
            None,
            fields,
        ))
    }
}

fn is_allowed_field(field: &str) -> bool {
    matches!(
        field,
        "component"
            | "operation"
            | "safe_error_code"
            | "retryable"
            | "occurred_at"
            | "duration_ms"
            | "event.name"
            | "event.seq"
            | "event.kind"
            | "event.from_seq"
            | "event.to_seq"
            | "event.dropped"
            | "session.id"
            | "session.status"
            | "session.from_status"
            | "session.to_status"
            | "session.needs_input_kind"
            | "session.content_seq"
            | "runtime.binary"
            | "runtime.cwd_summary"
            | "runtime.exit_code"
            | "runtime.output_offset"
            | "runtime.cols"
            | "runtime.rows"
            | "metrics.kind"
            | "metrics.scope"
            | "metrics.value"
            | "metrics.unit"
            | "metrics.count"
            | "usage.provider"
            | "usage.profile_id"
            | "usage.session_id"
            | "usage.model"
            | "usage.input_tokens"
            | "usage.output_tokens"
            | "usage.cache_read_tokens"
            | "usage.cache_write_tokens"
            | "usage.estimated_usd"
            | "usage.billed_usd"
            | "usage.value_kind"
            | "usage.source"
            | "usage.first_token_latency_ms"
            | "usage.total_latency_ms"
            | "usage.tool_call_count"
            | "usage.cache_hit_ratio"
            | "evidence.command"
            | "evidence.source"
            | "evidence.exit_code"
            | "evidence.status"
            | "evidence.output_summary"
            | "evidence.path"
            | "evidence.case_id"
            | "agent.kind"
            | "agent.event_type"
            | "agent.is_subagent"
            | "agent.blocker_kind"
            | "agent.risk_level"
    )
}

fn is_dangerous_field(field: &str) -> bool {
    if is_allowed_field(field) {
        return false;
    }
    let normalized = field.to_ascii_lowercase().replace('-', "_");
    let compact = normalized.replace('_', "");
    let dangerous = [
        "authorization",
        "cookie",
        "set_cookie",
        "api_key",
        "provider_key",
        "private_key",
        "token",
        "secret",
        "password",
        "raw_prompt",
        "prompt",
        "raw_request",
        "raw_response",
        "request_body",
        "response_body",
        "headers",
        "tool_args",
        "tool_result",
        "full_tool_args",
        "full_tool_result",
        "env",
        "argv",
        "stdin",
        "stdout",
        "stderr",
    ];
    normalized
        .split(['.', '[', ']'])
        .filter(|segment| !segment.is_empty())
        .any(|segment| dangerous.contains(&segment))
        || [
            "authorization",
            "cookie",
            "setcookie",
            "apikey",
            "providerkey",
            "privatekey",
            "token",
            "secret",
            "password",
            "rawprompt",
            "rawrequest",
            "rawresponse",
            "requestbody",
            "responsebody",
            "headers",
            "toolargs",
            "toolresult",
            "fulltoolargs",
            "fulltoolresult",
        ]
        .into_iter()
        .any(|needle| compact.contains(needle))
}

fn ensure_value_has_no_dangerous_keys(prefix: &str, value: &Value) -> Result<(), SafeFieldError> {
    match value {
        Value::Object(object) => {
            for (key, nested) in object {
                let field = format!("{prefix}.{key}");
                if is_dangerous_field(&field) {
                    return Err(SafeFieldError::DangerousField { field });
                }
                ensure_value_has_no_dangerous_keys(&field, nested)?;
            }
        }
        Value::Array(values) => {
            for (index, nested) in values.iter().enumerate() {
                ensure_value_has_no_dangerous_keys(&format!("{prefix}[{index}]"), nested)?;
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {}
    }
    Ok(())
}
