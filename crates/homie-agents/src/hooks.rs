use homie_proto::{NeedsInputDetail, NeedsInputKind, NeedsInputSource, RiskHint};
use serde::Serialize;
use serde_json::Value;

use crate::detect::redact::redact;
use crate::detect::{RiskHint as DetectRiskHint, classify_risk};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum HookEvent {
    SessionStart,
    UserPromptSubmit {
        is_subagent: bool,
    },
    PreToolUse {
        is_subagent: bool,
    },
    PermissionRequest {
        tool_name: Option<String>,
        summary: Option<String>,
        is_subagent: bool,
    },
    Notification {
        notification_type: Option<String>,
        message: Option<String>,
        is_subagent: bool,
    },
    Stop {
        is_subagent: bool,
    },
    SubagentStart(String),
    SubagentStop(String),
    SessionEnd,
    Unknown(String),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum NotifyEvent {
    CodexTurnComplete,
    Unknown(String),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParsedHook {
    pub event: HookEvent,
    pub session_id: Option<String>,
    pub transcript_path: Option<String>,
    pub first_prompt_title: Option<String>,
    pub needs_input: Option<NeedsInputDetail>,
    pub safe_summary: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParsedNotify {
    pub event: NotifyEvent,
    pub session_id: Option<String>,
    pub first_prompt_title: Option<String>,
    pub safe_summary: String,
}

pub fn parse_claude_hook(event: &str, payload: &Value) -> Option<ParsedHook> {
    let is_subagent = string(payload, "agent_id").is_some();
    let session_id = string(payload, "session_id");
    let transcript_path = string(payload, "transcript_path");
    let mut first_prompt_title = None;
    let mut needs_input = None;

    let parsed = match event {
        "SessionStart" => HookEvent::SessionStart,
        "UserPromptSubmit" => {
            if !is_subagent {
                let prompt = string(payload, "prompt_text").or_else(|| string(payload, "prompt"));
                first_prompt_title = prompt.as_deref().map(title_from_prompt);
            }
            HookEvent::UserPromptSubmit { is_subagent }
        }
        "PreToolUse" => HookEvent::PreToolUse { is_subagent },
        "PermissionRequest" => {
            let (tool_name, summary) = tool_summary(payload);
            if !is_subagent {
                needs_input = Some(permission_detail(tool_name.clone(), summary.clone()));
            }
            HookEvent::PermissionRequest {
                tool_name,
                summary,
                is_subagent,
            }
        }
        "Notification" => {
            let notification_type = string(payload, "notification_type");
            let message = string(payload, "message");
            if !is_subagent && let Some(kind) = needs_input_kind(notification_type.as_deref()) {
                let text = message
                    .clone()
                    .unwrap_or_else(|| "Agent needs your input".to_string());
                needs_input = Some(notification_detail(kind, &text));
            }
            HookEvent::Notification {
                notification_type,
                message: message.map(|text| redact(&text)),
                is_subagent,
            }
        }
        "Stop" => HookEvent::Stop { is_subagent },
        "SubagentStart" => {
            HookEvent::SubagentStart(string(payload, "agent_id").unwrap_or_else(|| "?".into()))
        }
        "SubagentStop" => {
            HookEvent::SubagentStop(string(payload, "agent_id").unwrap_or_else(|| "?".into()))
        }
        "SessionEnd" => HookEvent::SessionEnd,
        other => HookEvent::Unknown(other.to_string()),
    };

    Some(ParsedHook {
        event: parsed,
        session_id,
        transcript_path,
        first_prompt_title,
        needs_input,
        safe_summary: safe_payload_summary(payload),
    })
}

pub fn parse_codex_notify(payload: &Value) -> Option<ParsedNotify> {
    let event = match string(payload, "type").as_deref() {
        Some("agent-turn-complete") => NotifyEvent::CodexTurnComplete,
        Some(other) => NotifyEvent::Unknown(other.to_string()),
        None => return None,
    };
    let first_prompt_title = payload
        .get("input-messages")
        .and_then(Value::as_array)
        .and_then(|messages| messages.first())
        .and_then(Value::as_str)
        .map(title_from_prompt);
    Some(ParsedNotify {
        event,
        session_id: string(payload, "thread-id"),
        first_prompt_title,
        safe_summary: safe_payload_summary(payload),
    })
}

pub fn tool_summary(payload: &Value) -> (Option<String>, Option<String>) {
    let tool = string(payload, "tool_name");
    let Some(input) = payload.get("tool_input").and_then(Value::as_object) else {
        return (tool, None);
    };
    let summary = match tool.as_deref() {
        Some("Bash") => input.get("command").and_then(Value::as_str),
        Some("Edit") | Some("Write") | Some("Read") | Some("NotebookEdit") => {
            input.get("file_path").and_then(Value::as_str)
        }
        Some("WebFetch") | Some("WebSearch") => input
            .get("url")
            .and_then(Value::as_str)
            .or_else(|| input.get("query").and_then(Value::as_str)),
        _ => input.values().find_map(Value::as_str),
    };
    (
        tool,
        summary.map(|text| redact(&text.chars().take(120).collect::<String>())),
    )
}

fn permission_detail(tool_name: Option<String>, input_summary: Option<String>) -> NeedsInputDetail {
    let summary = match tool_name.as_deref() {
        Some("Bash") => format!("wants to run `{}`", input_summary.as_deref().unwrap_or("")),
        Some("Edit") | Some("Write") => {
            format!("wants to edit {}", short_path(input_summary.as_deref()))
        }
        Some("WebFetch") => format!(
            "wants to fetch {}",
            input_summary.as_deref().unwrap_or("a URL")
        ),
        Some(name) if name.starts_with("mcp__") => {
            format!("wants to use {}", name[5..].replace("__", ":"))
        }
        Some(name) => input_summary.as_deref().map_or_else(
            || format!("wants to use {name}"),
            |detail| format!("wants to use {name}: {detail}"),
        ),
        None => input_summary.as_deref().map_or_else(
            || "needs permission".to_string(),
            |detail| format!("needs permission: {detail}"),
        ),
    };
    let risk_hint = proto_risk(classify_risk(input_summary.as_deref().unwrap_or_default()));
    NeedsInputDetail {
        kind: NeedsInputKind::Approval,
        source: NeedsInputSource::Hook,
        tool_name,
        summary: redact(&summary),
        prompt_excerpt: input_summary,
        options: None,
        risk_hint,
        occurred_at: 0,
    }
}

fn notification_detail(kind: NeedsInputKind, text: &str) -> NeedsInputDetail {
    NeedsInputDetail {
        kind,
        source: NeedsInputSource::Hook,
        tool_name: None,
        summary: redact(text),
        prompt_excerpt: None,
        options: None,
        risk_hint: proto_risk(classify_risk(text)),
        occurred_at: 0,
    }
}

fn needs_input_kind(notification_type: Option<&str>) -> Option<NeedsInputKind> {
    match notification_type {
        Some("permission_prompt") => Some(NeedsInputKind::Approval),
        Some("idle_prompt") | Some("agent_needs_input") | Some("elicitation_dialog") => {
            Some(NeedsInputKind::Question)
        }
        _ => None,
    }
}

fn title_from_prompt(prompt: &str) -> String {
    let first_line = prompt
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or("");
    redact(first_line)
        .chars()
        .take(60)
        .collect::<String>()
        .trim()
        .to_string()
}

fn short_path(path: Option<&str>) -> String {
    let Some(path) = path else {
        return "a file".into();
    };
    let parts = path
        .split('/')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    let tail = parts.len().saturating_sub(2);
    parts[tail..].join("/")
}

fn string(value: &Value, key: &str) -> Option<String> {
    value.get(key).and_then(Value::as_str).map(str::to_string)
}

fn safe_payload_summary(value: &Value) -> String {
    redact(&redact_json(value).to_string())
}

fn redact_json(value: &Value) -> Value {
    match value {
        Value::Object(map) => Value::Object(
            map.iter()
                .map(|(key, value)| {
                    if is_secret_key(key) {
                        (key.clone(), Value::String("•••".into()))
                    } else {
                        (key.clone(), redact_json(value))
                    }
                })
                .collect(),
        ),
        Value::Array(values) => Value::Array(values.iter().map(redact_json).collect()),
        Value::String(text) => Value::String(redact(text)),
        other => other.clone(),
    }
}

fn is_secret_key(key: &str) -> bool {
    let key = key.to_ascii_lowercase();
    key.contains("token")
        || key.contains("secret")
        || key.contains("password")
        || key.contains("authorization")
        || key.contains("cookie")
        || key.contains("api_key")
        || key.contains("apikey")
}

fn proto_risk(risk: DetectRiskHint) -> RiskHint {
    match risk {
        DetectRiskHint::Neutral => RiskHint::Neutral,
        DetectRiskHint::FileWrite => RiskHint::FileWrite,
        DetectRiskHint::Network => RiskHint::Network,
        DetectRiskHint::Destructive => RiskHint::Destructive,
    }
}
