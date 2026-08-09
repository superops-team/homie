use homie_proto::{AgentProfileId, ProviderId, SessionId, VirtualKeyId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use thiserror::Error;
use time::OffsetDateTime;
use uuid::Uuid;

pub const PRICING_ENTRY_COUNT: usize = 15;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ModelPricing {
    pub input: f64,
    pub output: f64,
}

impl ModelPricing {
    #[must_use]
    pub const fn cache_read(self) -> f64 {
        self.input * 0.1
    }

    #[must_use]
    pub const fn cache_write_5m(self) -> f64 {
        self.input * 1.25
    }

    #[must_use]
    pub const fn cache_write_1h(self) -> f64 {
        self.input * 2.0
    }
}

#[must_use]
pub fn match_claude(model: &str) -> Option<ModelPricing> {
    if model.contains("fable") || model.contains("mythos") {
        Some(ModelPricing {
            input: 10.0,
            output: 50.0,
        })
    } else if model.contains("opus-4-1") || model.contains("opus-4-2025") {
        Some(ModelPricing {
            input: 15.0,
            output: 75.0,
        })
    } else if model.contains("opus") {
        Some(ModelPricing {
            input: 5.0,
            output: 25.0,
        })
    } else if model.contains("sonnet") {
        Some(ModelPricing {
            input: 3.0,
            output: 15.0,
        })
    } else if model.contains("haiku-4") {
        Some(ModelPricing {
            input: 1.0,
            output: 5.0,
        })
    } else if model.contains("3-5-haiku") {
        Some(ModelPricing {
            input: 0.8,
            output: 4.0,
        })
    } else if model.contains("haiku") {
        Some(ModelPricing {
            input: 0.25,
            output: 1.25,
        })
    } else {
        None
    }
}

#[must_use]
pub fn match_openai(model: &str) -> Option<ModelPricing> {
    if model.contains("gpt-5.4-mini") {
        Some(ModelPricing {
            input: 0.75,
            output: 4.5,
        })
    } else if model.contains("gpt-5.4") {
        Some(ModelPricing {
            input: 2.5,
            output: 15.0,
        })
    } else if model.contains("gpt-5.5") {
        Some(ModelPricing {
            input: 5.0,
            output: 30.0,
        })
    } else if model.contains("codex-mini") {
        Some(ModelPricing {
            input: 1.5,
            output: 6.0,
        })
    } else if model.contains("codex") {
        Some(ModelPricing {
            input: 1.75,
            output: 14.0,
        })
    } else if model.contains("mini") {
        Some(ModelPricing {
            input: 0.25,
            output: 2.0,
        })
    } else if model.contains("nano") {
        Some(ModelPricing {
            input: 0.05,
            output: 0.4,
        })
    } else if model.contains("gpt-5") {
        Some(ModelPricing {
            input: 1.25,
            output: 10.0,
        })
    } else {
        None
    }
}

#[must_use]
pub fn openai_estimate(
    model: &str,
    input_tokens: i64,
    output_tokens: i64,
    cache_read_tokens: i64,
) -> Option<f64> {
    let pricing = match_openai(model)?;
    Some(
        (input_tokens.max(0) as f64 * pricing.input
            + output_tokens.max(0) as f64 * pricing.output
            + cache_read_tokens.max(0) as f64 * pricing.cache_read())
            / 1_000_000.0,
    )
}

#[must_use]
pub fn claude_estimate(
    model: &str,
    input_tokens: i64,
    output_tokens: i64,
    cache_read_tokens: i64,
    cache_write_5m_tokens: i64,
    cache_write_1h_tokens: i64,
) -> Option<f64> {
    let pricing = match_claude(model)?;
    Some(
        (input_tokens.max(0) as f64 * pricing.input
            + output_tokens.max(0) as f64 * pricing.output
            + cache_read_tokens.max(0) as f64 * pricing.cache_read()
            + cache_write_5m_tokens.max(0) as f64 * pricing.cache_write_5m()
            + cache_write_1h_tokens.max(0) as f64 * pricing.cache_write_1h())
            / 1_000_000.0,
    )
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UsageProviderKind {
    Claude,
    Codex,
}

impl UsageProviderKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Claude => "claude",
            Self::Codex => "codex",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UsageValueKind {
    EstimatedApiEquivalent,
    AuthoritativeBilled,
}

impl UsageValueKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::EstimatedApiEquivalent => "estimated_api_equivalent",
            Self::AuthoritativeBilled => "authoritative_billed",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UsageSourceKind {
    Transcript,
}

impl UsageSourceKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Transcript => "transcript",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TranscriptUsageEvent {
    pub source_event_id: String,
    pub occurred_at: i64,
    pub provider: UsageProviderKind,
    pub profile_id: Option<String>,
    pub session_id: Option<String>,
    pub model: Option<String>,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_read_tokens: i64,
    pub cache_write_tokens: i64,
    pub cache_write_5m_tokens: i64,
    pub cache_write_1h_tokens: i64,
    pub estimated_cost: Option<f64>,
    pub billed_cost: Option<f64>,
    pub value_kind: UsageValueKind,
    pub source: UsageSourceKind,
}

pub fn parse_transcript_usage_events(
    path: &Path,
    provider: UsageProviderKind,
    profile_id: Option<&str>,
) -> std::io::Result<Vec<TranscriptUsageEvent>> {
    let file = File::open(path)?;
    let mut result = Vec::new();
    let mut offset = 0_u64;
    let mut codex_model = None;
    for line in BufReader::new(file).lines() {
        let line = line?;
        let line_offset = offset;
        offset = offset.saturating_add(u64::try_from(line.len()).unwrap_or(u64::MAX) + 1);
        let Ok(value) = serde_json::from_str::<serde_json::Value>(&line) else {
            continue;
        };
        if provider == UsageProviderKind::Codex
            && let Some(model) = value
                .pointer("/payload/model")
                .or_else(|| value.pointer("/payload/current_model"))
                .and_then(serde_json::Value::as_str)
            && matches!(
                value.get("type").and_then(serde_json::Value::as_str),
                Some("session_meta" | "turn_context")
            )
        {
            codex_model = Some(model.to_owned());
            continue;
        }
        let parsed = match provider {
            UsageProviderKind::Claude => parse_claude_transcript_event(&value),
            UsageProviderKind::Codex => {
                parse_codex_transcript_event(&value, codex_model.as_deref())
            }
        };
        let Some(mut event) = parsed else {
            continue;
        };
        event.source_event_id = transcript_event_id(path, line_offset, &value);
        event.profile_id = profile_id.map(str::to_string);
        result.push(event);
    }
    Ok(result)
}

fn parse_claude_transcript_event(value: &serde_json::Value) -> Option<TranscriptUsageEvent> {
    if value.get("type").and_then(serde_json::Value::as_str) != Some("assistant") {
        return None;
    }
    let message = value.get("message")?;
    let usage = message.get("usage")?;
    let occurred_at = event_timestamp(value)?;
    let model = message.get("model").and_then(serde_json::Value::as_str);
    let input = integer(usage.get("input_tokens"));
    let output = integer(usage.get("output_tokens"));
    let cache_read = integer(usage.get("cache_read_input_tokens"));
    let cache_write = integer(usage.get("cache_creation_input_tokens"));
    let cache_write_5m = integer(usage.pointer("/cache_creation/ephemeral_5m_input_tokens"));
    let cache_write_1h = integer(usage.pointer("/cache_creation/ephemeral_1h_input_tokens"));
    Some(TranscriptUsageEvent {
        source_event_id: String::new(),
        occurred_at,
        provider: UsageProviderKind::Claude,
        profile_id: None,
        session_id: value
            .get("sessionId")
            .and_then(serde_json::Value::as_str)
            .map(str::to_string),
        model: model.map(str::to_string),
        input_tokens: input,
        output_tokens: output,
        cache_read_tokens: cache_read,
        cache_write_tokens: cache_write,
        cache_write_5m_tokens: cache_write_5m,
        cache_write_1h_tokens: cache_write_1h,
        estimated_cost: explicit_cost(value).or_else(|| {
            claude_estimate(
                model?,
                input,
                output,
                cache_read,
                cache_write_5m,
                cache_write_1h,
            )
        }),
        billed_cost: None,
        value_kind: UsageValueKind::EstimatedApiEquivalent,
        source: UsageSourceKind::Transcript,
    })
}

fn parse_codex_transcript_event(
    value: &serde_json::Value,
    model: Option<&str>,
) -> Option<TranscriptUsageEvent> {
    if value
        .pointer("/payload/type")
        .and_then(serde_json::Value::as_str)
        != Some("token_count")
    {
        return None;
    }
    let usage = value.pointer("/payload/info/last_token_usage")?;
    let occurred_at = event_timestamp(value)?;
    let input = integer(usage.get("input_tokens"));
    let output = integer(usage.get("output_tokens"));
    let cache_read = integer(usage.get("cached_input_tokens"));
    Some(TranscriptUsageEvent {
        source_event_id: String::new(),
        occurred_at,
        provider: UsageProviderKind::Codex,
        profile_id: None,
        session_id: value
            .pointer("/payload/thread_id")
            .and_then(serde_json::Value::as_str)
            .map(str::to_string),
        model: model.map(str::to_string),
        input_tokens: input,
        output_tokens: output,
        cache_read_tokens: cache_read,
        cache_write_tokens: 0,
        cache_write_5m_tokens: 0,
        cache_write_1h_tokens: 0,
        estimated_cost: explicit_cost(value)
            .or_else(|| openai_estimate(model?, input, output, cache_read)),
        billed_cost: None,
        value_kind: UsageValueKind::EstimatedApiEquivalent,
        source: UsageSourceKind::Transcript,
    })
}

fn transcript_event_id(path: &Path, offset: u64, value: &serde_json::Value) -> String {
    let provider_id = value
        .pointer("/message/id")
        .or_else(|| value.pointer("/payload/id"))
        .and_then(serde_json::Value::as_str);
    let mut bytes = Vec::new();
    bytes.extend_from_slice(path.to_string_lossy().as_bytes());
    bytes.extend_from_slice(&offset.to_le_bytes());
    if let Some(provider_id) = provider_id {
        bytes.extend_from_slice(provider_id.as_bytes());
    }
    format!("transcript:{:016x}", fnv1a_bytes(&bytes))
}

fn event_timestamp(value: &serde_json::Value) -> Option<i64> {
    if let Some(timestamp) = value.get("timestamp").and_then(serde_json::Value::as_i64) {
        return Some(timestamp);
    }
    let timestamp = value.get("timestamp").and_then(serde_json::Value::as_str)?;
    parse_rfc3339_seconds(timestamp)
}

fn parse_rfc3339_seconds(value: &str) -> Option<i64> {
    if value.len() < 20 || &value[4..5] != "-" || &value[7..8] != "-" || &value[10..11] != "T" {
        return None;
    }
    let year = value[0..4].parse::<i64>().ok()?;
    let month = value[5..7].parse::<i64>().ok()?;
    let day = value[8..10].parse::<i64>().ok()?;
    let hour = value[11..13].parse::<i64>().ok()?;
    let minute = value[14..16].parse::<i64>().ok()?;
    let second = value[17..19].parse::<i64>().ok()?;
    let days = days_from_civil(year, month, day);
    let mut result = days * 86_400 + hour * 3_600 + minute * 60 + second;
    let suffix = &value[19..];
    if let Some(position) = suffix.rfind(['+', '-']) {
        let sign = if suffix.as_bytes()[position] == b'+' {
            1
        } else {
            -1
        };
        let timezone = &suffix[position + 1..];
        if timezone.len() >= 5 {
            let hours = timezone[0..2].parse::<i64>().ok()?;
            let minutes = timezone[3..5].parse::<i64>().ok()?;
            result -= sign * (hours * 3_600 + minutes * 60);
        }
    }
    Some(result)
}

fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = year - i64::from(month <= 2);
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let year_of_era = year - era * 400;
    let shifted_month = month + if month > 2 { -3 } else { 9 };
    let day_of_year = (153 * shifted_month + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

fn integer(value: Option<&serde_json::Value>) -> i64 {
    value
        .and_then(serde_json::Value::as_i64)
        .unwrap_or_default()
        .max(0)
}

fn explicit_cost(value: &serde_json::Value) -> Option<f64> {
    value
        .get("costUSD")
        .or_else(|| value.get("cost_usd"))
        .or_else(|| value.pointer("/message/costUSD"))
        .and_then(serde_json::Value::as_f64)
        .filter(|cost| cost.is_finite() && *cost >= 0.0)
}

fn fnv1a_bytes(value: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for &byte in value {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VirtualKeyScope {
    pub session_id: SessionId,
    pub agent_profile_id: AgentProfileId,
    pub provider_id: ProviderId,
    pub allowed_models: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VirtualKeyRequest {
    pub scope: VirtualKeyScope,
    pub expires_at: OffsetDateTime,
}

#[derive(Clone, Eq, PartialEq)]
pub struct IssuedVirtualKey {
    pub key_id: VirtualKeyId,
    pub secret: String,
    pub scope: VirtualKeyScope,
    pub expires_at: OffsetDateTime,
}

impl fmt::Debug for IssuedVirtualKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("IssuedVirtualKey")
            .field("key_id", &self.key_id)
            .field("secret", &"[redacted]")
            .field("scope", &self.scope)
            .field("expires_at", &self.expires_at)
            .finish()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VirtualKeyClaims {
    pub key_id: VirtualKeyId,
    pub scope: VirtualKeyScope,
    pub expires_at: OffsetDateTime,
}

#[derive(Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagedLlmProxyConfig {
    pub base_url: String,
    pub virtual_key: String,
    pub expires_at: OffsetDateTime,
    pub scope: VirtualKeyScope,
}

impl ManagedLlmProxyConfig {
    #[must_use]
    pub fn from_issued_key(base_url: String, issued_key: IssuedVirtualKey) -> Self {
        Self {
            base_url,
            virtual_key: issued_key.secret,
            expires_at: issued_key.expires_at,
            scope: issued_key.scope,
        }
    }
}

impl fmt::Debug for ManagedLlmProxyConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ManagedLlmProxyConfig")
            .field("base_url", &self.base_url)
            .field("virtual_key", &"[redacted]")
            .field("expires_at", &self.expires_at)
            .field("scope", &self.scope)
            .finish()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CredentialDestination {
    RemoteNode,
    McpTool,
    ManagedAgentConfig,
    LogEvent,
}

impl fmt::Display for CredentialDestination {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::RemoteNode => "remote_node",
            Self::McpTool => "mcp_tool",
            Self::ManagedAgentConfig => "managed_agent_config",
            Self::LogEvent => "log_event",
        };
        formatter.write_str(value)
    }
}

#[derive(Clone, Copy, Default)]
pub struct CredentialPropagationPolicy;

impl CredentialPropagationPolicy {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    pub fn ensure_payload_is_secretless(
        &self,
        destination: CredentialDestination,
        payload: impl AsRef<str>,
        raw_provider_key: impl AsRef<str>,
    ) -> Result<(), VirtualKeyError> {
        let raw_provider_key = raw_provider_key.as_ref();
        if !raw_provider_key.is_empty() && payload.as_ref().contains(raw_provider_key) {
            return Err(VirtualKeyError::RawProviderKeyForbidden(destination));
        }
        Ok(())
    }
}

impl fmt::Debug for CredentialPropagationPolicy {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CredentialPropagationPolicy")
            .finish()
    }
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum VirtualKeyError {
    #[error("virtual key not found")]
    NotFound,
    #[error("virtual key expired")]
    Expired,
    #[error("virtual key revoked")]
    Revoked,
    #[error("virtual key scope mismatch")]
    ScopeMismatch,
    #[error("model is not allowed by virtual key scope")]
    ModelNotAllowed,
    #[error("raw provider key is forbidden for {0}")]
    RawProviderKeyForbidden(CredentialDestination),
}

#[derive(Clone, Debug)]
struct StoredVirtualKey {
    key_id: VirtualKeyId,
    scope: VirtualKeyScope,
    expires_at: OffsetDateTime,
    revoked_at: Option<OffsetDateTime>,
}

#[derive(Default)]
pub struct InMemoryVirtualKeyStore {
    keys_by_secret: BTreeMap<String, StoredVirtualKey>,
}

impl InMemoryVirtualKeyStore {
    pub fn issue(&mut self, request: VirtualKeyRequest) -> IssuedVirtualKey {
        let key_id = VirtualKeyId::new();
        let secret = format!("hv_{}", Uuid::now_v7());
        let stored = StoredVirtualKey {
            key_id: key_id.clone(),
            scope: request.scope,
            expires_at: request.expires_at,
            revoked_at: None,
        };
        self.keys_by_secret.insert(secret.clone(), stored.clone());
        IssuedVirtualKey {
            key_id,
            secret,
            scope: stored.scope,
            expires_at: stored.expires_at,
        }
    }

    pub fn revoke(&mut self, key_id: &VirtualKeyId) -> Result<(), VirtualKeyError> {
        let Some((_, key)) = self
            .keys_by_secret
            .iter_mut()
            .find(|(_, key)| key.key_id == *key_id)
        else {
            return Err(VirtualKeyError::NotFound);
        };
        key.revoked_at = Some(OffsetDateTime::now_utc());
        Ok(())
    }

    pub fn validate(
        &self,
        presented: &str,
        scope: &VirtualKeyScope,
        model: &str,
    ) -> Result<VirtualKeyClaims, VirtualKeyError> {
        let key = self
            .keys_by_secret
            .get(presented)
            .ok_or(VirtualKeyError::NotFound)?;
        if key.revoked_at.is_some() {
            return Err(VirtualKeyError::Revoked);
        }
        if key.expires_at <= OffsetDateTime::now_utc() {
            return Err(VirtualKeyError::Expired);
        }
        if &key.scope != scope {
            return Err(VirtualKeyError::ScopeMismatch);
        }
        if !key
            .scope
            .allowed_models
            .iter()
            .any(|allowed| allowed == model)
        {
            return Err(VirtualKeyError::ModelNotAllowed);
        }
        Ok(VirtualKeyClaims {
            key_id: key.key_id.clone(),
            scope: key.scope.clone(),
            expires_at: key.expires_at,
        })
    }
}
