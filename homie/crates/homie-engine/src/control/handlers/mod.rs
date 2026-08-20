//! Control-channel method handlers.
//!
//! The per-method business logic (handshake, session spawn/list/resume, host and
//! worktree operations, hook reporting, governance, browser calls). These stay as
//! `impl ControlServer` methods so they can reach the private fields, but they live
//! apart from the transport layer (serve/handle_line/dispatch) and the wire codec
//! because they change for protocol/business reasons, not framing ones.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use homie_proto::{ControlError, JsonValue, WIRE_VERSION};
use serde_json::{Value, json};

use crate::registry::Registry;

use super::ControlServer;
use super::codec::{history_entry_to_wire, worktree_to_wire};
use super::inject::prepare_agent_input;
use super::wire::{
    decode, encode, io_control_error, migrate_control_error, poisoned, resolve_on_path,
};
use super::{BUILD, next_session_id, process_executable_hash};

mod agent;
mod governor;
mod handshake;
mod host;
mod migrate;
mod resume;
mod session;
mod spawn;
mod worktree;

pub(crate) fn new_record(id: &str, kind: &str, cwd: &str) -> homie_proto::SessionRecord {
    use homie_proto::{AgentKind, DateMillis, Resumability, SessionId, TitleSource};
    let now: DateMillis = std::time::SystemTime::now().into();
    homie_proto::SessionRecord {
        id: SessionId(id.to_string()),
        kind: AgentKind::new(kind),
        cwd: cwd.to_string(),
        project_id: crate::registry::session_project_id(cwd, None),
        worktree_path: None,
        git_branch: None,
        title: kind.to_string(),
        title_source: TitleSource::Placeholder,
        agent_session_id: None,
        transcript_path: None,
        status: homie_proto::SessionStatus::Starting,
        needs_input: None,
        resumability: Resumability::Live,
        parent: None,
        created_at: now,
        updated_at: now,
        last_turn_completed_at: None,
        last_seen_at: None,
        pinned: false,
        archived_at: None,
        host: None,
        remote_persistence: None,
        hibernation: None,
        memory_bytes: None,
        artifacts: None,
        pull_requests: None,
        listening_ports: None,
        foreground_agent: None,
    }
}
