//! ACP (Agent Client Protocol) host harness.
//!
//! Homie acts as the ACP *host*: it spawns an ACP-compliant agent server (for
//! example `codex-acp`) as a stdio subprocess and speaks JSON-RPC 2.0 to it.
//! This module owns the wire protocol, framing, host loop, approval semantics
//! and the `AgentDriverControl` implementation. It is deliberately std-only to
//! match the engine's synchronous design.

pub mod approval;
pub mod driver;
pub mod frame;
pub mod host;
pub mod protocol;

pub use approval::{ApprovalMemory, PermissionDecision};
pub use driver::AcpDriver;
pub use host::{AcpClient, AcpError, AcpHost, AcpStream};
