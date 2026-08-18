//! Homie local LLM gateway library. The `homie-gateway` binary is a thin shell
//! over this library; integration tests exercise it directly.

pub mod auth;
pub mod config;
pub mod db;
pub mod routes;
pub mod state;
pub mod upstream;
pub mod usage;
