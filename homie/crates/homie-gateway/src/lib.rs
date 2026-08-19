//! Homie local LLM gateway library, embedded in the daemon (no standalone
//! binary). Exposes the OpenAI-compatible proxy routes, virtual-key store,
//! usage recording, policy, and upstream forwarding.

pub mod auth;
pub mod config;
pub mod db;
pub mod policy;
pub mod routes;
pub mod state;
pub mod upstream;
pub mod usage;
