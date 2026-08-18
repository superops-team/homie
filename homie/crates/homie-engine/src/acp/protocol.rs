//! ACP (Agent Client Protocol) JSON-RPC 2.0 wire types.
//!
//! ACP is a JSON-RPC 2.0 protocol spoken over stdio, newline-delimited. Homie
//! acts as the *host* (client) and spawns an ACP-compliant agent server (for
//! example `codex-acp`) as a subprocess. This module owns the wire DTOs and the
//! method / notification-kind constants. It is pure `serde` — no I/O, no state —
//! so the contract is covered by round-trip and tolerance tests below.

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const JSONRPC_VERSION: &str = "2.0";

// Host -> agent request methods.
pub const METHOD_INITIALIZE: &str = "initialize";
pub const METHOD_SESSION_NEW: &str = "session/new";
pub const METHOD_SESSION_LOAD: &str = "session/load";
pub const METHOD_SESSION_PROMPT: &str = "session/prompt";
pub const METHOD_SESSION_STOP: &str = "session/stop";
pub const METHOD_SESSION_CANCEL: &str = "session/cancel";
pub const METHOD_SESSION_SET_MODE: &str = "session/set_mode";
pub const METHOD_SESSION_SET_MODEL: &str = "session/set_model";

// Host -> agent permission response. This is a Homie-internal method name
// surfaced for the first slice; the exact ACP permission wire shape is pinned
// when a real `codex-acp` server is wired in a follow-up change.
pub const METHOD_RESPOND_PERMISSION: &str = "session/respond_permission";

// Agent -> host notification method.
pub const NOTIFY_SESSION_UPDATE: &str = "session/update";

// `session/update` `sessionUpdate` kind values.
pub const KIND_AGENT_MESSAGE_CHANGED: &str = "agent_message_changed";
pub const KIND_AGENT_THOUGHT_CHANGED: &str = "agent_thought_changed";
pub const KIND_PLAN: &str = "plan";
pub const KIND_TOOL_CALL: &str = "tool_call";
pub const KIND_AVAILABLE_COMMANDS_UPDATE: &str = "available_commands_update";
pub const KIND_CURRENT_MODE_UPDATE: &str = "current_mode_update";
pub const KIND_SESSION_STATUS_UPDATE: &str = "session_status_update";

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: i64,
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RpcError {
    pub code: i64,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<RpcError>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct JsonRpcNotification {
    pub jsonrpc: String,
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

/// A single inbound (agent -> host) frame, classified by its JSON-RPC shape.
///
/// JSON-RPC allows requests, responses and notifications to arrive in any
/// order, so the reader classifies each frame before routing it.
#[derive(Clone, Debug, PartialEq)]
pub enum InboundMessage {
    /// A reply to a host request (`id` + `result`/`error`).
    Response(JsonRpcResponse),
    /// An unsolicited agent push (`method`, no `id`), e.g. `session/update`.
    Notification(JsonRpcNotification),
    /// A server-initiated request (`id` + `method`), e.g. `fs/read_text_file`.
    Request(JsonRpcRequest),
}

/// Parse one inbound JSON-RPC frame and classify it.
///
/// Unknown `sessionUpdate` kinds and unexpected extra fields are tolerated by
/// `serde_json` (fields are ignored) — the contract here is only about the
/// top-level JSON-RPC shape. A malformed frame yields `Err`, never a panic.
pub fn classify_inbound(line: &str) -> Result<InboundMessage, serde_json::Error> {
    let value: Value = serde_json::from_str(line)?;
    let object = value.as_object().ok_or_else(|| {
        serde_json::Error::io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "expected JSON object for inbound ACP frame",
        ))
    })?;

    let has_id = object.contains_key("id");
    let has_method = object.contains_key("method");

    if has_id && has_method {
        let req: JsonRpcRequest = serde_json::from_value(value)?;
        Ok(InboundMessage::Request(req))
    } else if has_id {
        let resp: JsonRpcResponse = serde_json::from_value(value)?;
        Ok(InboundMessage::Response(resp))
    } else {
        let notif: JsonRpcNotification = serde_json::from_value(value)?;
        Ok(InboundMessage::Notification(notif))
    }
}

impl JsonRpcRequest {
    pub fn new(id: i64, method: &str, params: Value) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.into(),
            id,
            method: method.into(),
            params,
        }
    }
}

impl JsonRpcResponse {
    pub fn result(id: i64, result: Value) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.into(),
            id,
            result: Some(result),
            error: None,
        }
    }

    pub fn error(id: i64, code: i64, message: &str) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.into(),
            id,
            result: None,
            error: Some(RpcError {
                code,
                message: message.into(),
                data: None,
            }),
        }
    }
}

impl JsonRpcNotification {
    pub fn new(method: &str, params: Value) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.into(),
            method: method.into(),
            params,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn request_round_trips() {
        let req = JsonRpcRequest::new(1, METHOD_INITIALIZE, json!({"protocolVersion": 1}));
        let encoded = serde_json::to_string(&req).expect("encode");
        let decoded: JsonRpcRequest = serde_json::from_str(&encoded).expect("decode");
        assert_eq!(decoded, req);
        assert_eq!(decoded.id, 1);
        assert_eq!(decoded.method, METHOD_INITIALIZE);
    }

    #[test]
    fn response_result_and_error_round_trip() {
        let ok = JsonRpcResponse::result(7, json!({"protocolVersion": 1}));
        let encoded = serde_json::to_string(&ok).expect("encode");
        let decoded: JsonRpcResponse = serde_json::from_str(&encoded).expect("decode");
        assert_eq!(decoded, ok);
        assert!(decoded.error.is_none());

        let err = JsonRpcResponse::error(8, -32601, "method not found");
        let encoded = serde_json::to_string(&err).expect("encode");
        let decoded: JsonRpcResponse = serde_json::from_str(&encoded).expect("decode");
        assert_eq!(decoded.error.unwrap().code, -32601);
    }

    #[test]
    fn classify_response_notification_and_request() {
        let resp = r#"{"jsonrpc":"2.0","id":3,"result":{"ok":true}}"#;
        assert!(matches!(
            classify_inbound(resp).expect("classify"),
            InboundMessage::Response(_)
        ));

        let notif = r#"{"jsonrpc":"2.0","method":"session/update","params":{"sessionId":"s","sessionUpdate":"agent_message_changed"}}"#;
        assert!(matches!(
            classify_inbound(notif).expect("classify"),
            InboundMessage::Notification(_)
        ));

        let req = r#"{"jsonrpc":"2.0","id":9,"method":"fs/read_text_file","params":{"path":"/x"}}"#;
        assert!(matches!(
            classify_inbound(req).expect("classify"),
            InboundMessage::Request(_)
        ));
    }

    #[test]
    fn unknown_session_update_kind_is_tolerated() {
        // An agent may emit a kind Homie has not enumerated yet; the frame must
        // still parse as a notification rather than erroring.
        let notif = r#"{"jsonrpc":"2.0","method":"session/update","params":{"sessionId":"s","sessionUpdate":"some_future_kind"}}"#;
        let parsed = classify_inbound(notif).expect("classify");
        assert!(matches!(parsed, InboundMessage::Notification(_)));
    }

    #[test]
    fn malformed_frame_is_an_error_not_a_panic() {
        assert!(classify_inbound("not json").is_err());
        assert!(classify_inbound("\"scalar\"").is_err());
    }
}
