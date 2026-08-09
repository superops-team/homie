use serde::{Deserialize, Serialize};

use crate::transport::ClientRole;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum StreamKind {
    #[serde(rename = "events.v1")]
    EventsV1,
    #[serde(rename = "terminal.v1")]
    TerminalV1,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EventStreamOpen {
    #[serde(default)]
    pub after_seq: u64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub event_filter: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalStreamOpen {
    pub session_id: String,
    #[serde(default)]
    pub output_offset: u64,
    pub client_role: ClientRole,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_grid_sequence: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind")]
pub enum StreamOpenRequest {
    #[serde(rename = "events.v1")]
    Events(EventStreamOpen),
    #[serde(rename = "terminal.v1")]
    Terminal(TerminalStreamOpen),
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StreamResetReason {
    EventGap,
    SlowConsumer,
    ResyncRequired,
    ProtocolError,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamReset {
    pub reason: StreamResetReason,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_confirmed_offset: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_seq: Option<u64>,
}
