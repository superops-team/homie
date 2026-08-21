//! Executing MCP tools against a live registry.
//!
//! The calling agent's own session id arrives in its environment
//! (`HOMIE_SESSION_ID`), which is what lets `whoami` and `list_children`
//! answer questions about *this* session and the ones it spawned.

mod lineage;
mod registry;

pub use registry::RegistryHost;

/// Environment variable carrying the calling session's id.
pub const SESSION_ID_ENV: &str = "HOMIE_SESSION_ID";
