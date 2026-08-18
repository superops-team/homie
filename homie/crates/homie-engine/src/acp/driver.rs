//! `AcpDriver`: a real `AgentDriverControl` implementation that drives an ACP
//! agent server (for example `codex-acp`) through an [`AcpClient`].
//!
//! This is the first *real* provider driver built on top of
//! `typed-agent-driver-capabilities`. Capabilities are supplied from the
//! `initialize` handshake rather than guessed from the agent id.

use std::sync::Arc;

use homie_proto::DriverCapabilities;
use serde_json::json;

use super::host::{AcpClient, AcpError};
use super::protocol::{METHOD_RESPOND_PERMISSION, METHOD_SESSION_PROMPT, METHOD_SESSION_STOP};
use crate::driver::{AgentDriverControl, DriverError, DriverResult, ModelOption};

fn acp_to_driver(e: AcpError) -> DriverError {
    let code = match e {
        AcpError::Io(_) => "acp_io",
        AcpError::Protocol(_) => "acp_protocol",
        AcpError::Rpc { .. } => "acp_rpc",
        AcpError::Eof => "acp_eof",
    };
    DriverError {
        code,
        message: e.to_string(),
    }
}

/// A live ACP-backed agent driver.
pub struct AcpDriver {
    client: Arc<dyn AcpClient>,
    session_id: String,
    capabilities: DriverCapabilities,
}

impl AcpDriver {
    pub fn new(
        client: Arc<dyn AcpClient>,
        session_id: impl Into<String>,
        capabilities: DriverCapabilities,
    ) -> Self {
        Self {
            client,
            session_id: session_id.into(),
            capabilities,
        }
    }

    pub fn session_id(&self) -> &str {
        &self.session_id
    }
}

impl AgentDriverControl for AcpDriver {
    fn capabilities(&self) -> DriverCapabilities {
        self.capabilities.clone()
    }

    fn cancel_turn(&self) -> DriverResult<()> {
        self.client
            .request(METHOD_SESSION_STOP, json!({ "sessionId": self.session_id }))
            .map(|_| ())
            .map_err(acp_to_driver)
    }

    fn steer_message(&self, text: &str) -> DriverResult<()> {
        self.client
            .request(
                METHOD_SESSION_PROMPT,
                json!({
                    "sessionId": self.session_id,
                    "prompt": [{ "type": "text", "text": text }]
                }),
            )
            .map(|_| ())
            .map_err(acp_to_driver)
    }

    fn respond_permission(&self, request_id: &str, option_id: &str) -> DriverResult<()> {
        // The always-allow/always-deny intent is carried by `option_id` itself
        // ("allow_always"/"deny_always"), so the agent can decide whether to
        // remember it. Homie-side per-kind memory lives in `ApprovalMemory`,
        // which the UI layer drives where the permission kind is known.
        self.client
            .request(
                METHOD_RESPOND_PERMISSION,
                json!({
                    "sessionId": self.session_id,
                    "requestId": request_id,
                    "optionId": option_id
                }),
            )
            .map(|_| ())
            .map_err(acp_to_driver)
    }

    fn model_options(&self) -> DriverResult<Vec<ModelOption>> {
        // Model discovery via `available_commands_update` is a follow-up slice;
        // this driver does not fabricate a model list before it is negotiated.
        Err(DriverError::unsupported("model_options"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::super::protocol::JsonRpcNotification;

    /// Records every request so tests can assert the driver maps each control
    /// action to the expected ACP method and params.
    struct RecordingClient {
        calls: Mutex<Vec<(String, Value)>>,
        counter: AtomicUsize,
    }

    impl RecordingClient {
        fn new() -> Self {
            Self {
                calls: Mutex::new(Vec::new()),
                counter: AtomicUsize::new(0),
            }
        }
        fn calls(&self) -> Vec<(String, Value)> {
            self.calls.lock().unwrap().clone()
        }
    }

    impl AcpClient for RecordingClient {
        fn request(&self, method: &str, params: Value) -> Result<Value, AcpError> {
            self.calls.lock().unwrap().push((method.to_owned(), params));
            Ok(json!({ "n": self.counter.fetch_add(1, Ordering::SeqCst) }))
        }
        fn try_recv_notification(&self) -> Option<JsonRpcNotification> {
            None
        }
    }

    fn caps() -> DriverCapabilities {
        DriverCapabilities {
            prompt: true,
            cancel_turn: true,
            steer_message: true,
            respond_permission: true,
            ..DriverCapabilities::default()
        }
    }

    #[test]
    fn capabilities_are_supplied_not_guessed() {
        let driver = AcpDriver::new(Arc::new(RecordingClient::new()), "sess-1", caps());
        let c = driver.capabilities();
        assert!(c.prompt && c.cancel_turn && c.steer_message && c.respond_permission);
        assert!(!c.rollback && !c.fork);
        assert_eq!(driver.session_id(), "sess-1");
    }

    #[test]
    fn cancel_maps_to_session_stop() {
        let client = Arc::new(RecordingClient::new());
        let driver = AcpDriver::new(client.clone(), "sess-1", caps());
        driver.cancel_turn().expect("cancel");
        let calls = client.calls();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].0, METHOD_SESSION_STOP);
        assert_eq!(calls[0].1["sessionId"], "sess-1");
    }

    #[test]
    fn steer_maps_to_session_prompt_with_text_block() {
        let client = Arc::new(RecordingClient::new());
        let driver = AcpDriver::new(client.clone(), "sess-1", caps());
        driver.steer_message("keep going").expect("steer");
        let calls = client.calls();
        assert_eq!(calls[0].0, METHOD_SESSION_PROMPT);
        assert_eq!(calls[0].1["sessionId"], "sess-1");
        assert_eq!(calls[0].1["prompt"][0]["type"], "text");
        assert_eq!(calls[0].1["prompt"][0]["text"], "keep going");
    }

    #[test]
    fn respond_permission_forwards_request_and_option_ids() {
        let client = Arc::new(RecordingClient::new());
        let driver = AcpDriver::new(client.clone(), "sess-1", caps());
        driver
            .respond_permission("perm-42", "allow_always")
            .expect("respond");
        let calls = client.calls();
        assert_eq!(calls[0].0, METHOD_RESPOND_PERMISSION);
        assert_eq!(calls[0].1["requestId"], "perm-42");
        assert_eq!(calls[0].1["optionId"], "allow_always");
    }

    #[test]
    fn model_options_is_unsupported_in_first_slice() {
        let driver = AcpDriver::new(Arc::new(RecordingClient::new()), "sess-1", caps());
        assert_eq!(driver.model_options().unwrap_err().code, "unsupported");
    }
}
