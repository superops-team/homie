//! The `streamable-http` transport for the MCP server, embedded in the daemon.
//!
//! Agents connect to `POST /mcp` on a loopback listener instead of spawning a
//! stdio child. The caller's session id arrives in the `X-Homie-Session-Id`
//! header (moved off process environment so a single long-lived endpoint serves
//! every session), and a per-daemon bearer secret gates the whole surface.
//!
//! The JSON-RPC core is the pure [`super::McpServer`]; this module only adds
//! transport: auth, header parsing, and the HTTP listener lifecycle.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use axum::{
    Router,
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
    routing::post,
};
use serde_json::{Value, json};

use super::{McpServer, RegistryHost, tool_definitions_for};
use crate::registry::Registry;
use crate::session::HolderConfig;

/// Environment variable carrying the per-daemon MCP bearer secret into a
/// session; agents read it to set `Authorization: Bearer`.
pub const TOKEN_ENV: &str = "HOMIE_MCP_TOKEN";

/// HTTP header carrying the calling session's id.
pub const SESSION_HEADER: &str = "x-homie-session-id";

/// Preferred loopback port; the daemon falls back to an ephemeral port on
/// conflict so a second daemon or a stray listener never breaks orchestration.
pub const PREFERRED_PORT: u16 = 7941;

/// Runtime facts about the MCP endpoint injected into agents that opt in. The
/// bearer secret is in-memory only: it is never written to disk, logs, or the
/// injection fact file.
#[derive(Clone)]
pub struct McpRuntime {
    pub base_url: String,
    pub token: String,
}

impl std::fmt::Debug for McpRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("McpRuntime")
            .field("base_url", &self.base_url)
            .field("token", &"<redacted>")
            .finish_non_exhaustive()
    }
}

/// Shared state for the MCP HTTP handlers. Cloned into every request.
#[derive(Clone)]
struct McpHttpState {
    token: String,
    registry: Arc<Mutex<Registry>>,
    logs_dir: PathBuf,
    holder: Option<HolderConfig>,
    kinds: Vec<String>,
}

/// Start the MCP `streamable-http` listener on `127.0.0.1`, write the injection
/// fact file, and return the runtime facts for injection. Returns `None` when a
/// listener cannot be bound (the daemon continues serving orchestration without
/// MCP).
pub fn start(
    registry: Arc<Mutex<Registry>>,
    logs_dir: PathBuf,
    holder: Option<HolderConfig>,
    kinds: Vec<String>,
    inject_dir: &Path,
) -> Option<McpRuntime> {
    let token = random_secret();
    let (std_listener, port) = bind_loopback()?;
    let base_url = format!("http://127.0.0.1:{port}");
    let runtime = McpRuntime {
        base_url: base_url.clone(),
        token: token.clone(),
    };

    // The fact file only records the URL; the secret travels in-process.
    let fact = json!({ "url": format!("{base_url}/mcp") });
    let _ = std::fs::write(
        inject_dir.join("mcp-http.json"),
        serde_json::to_vec_pretty(&fact).ok()?,
    );

    let _ = std_listener.set_nonblocking(true);
    let listener = tokio::net::TcpListener::from_std(std_listener).ok()?;
    let state = McpHttpState {
        token,
        registry,
        logs_dir,
        holder,
        kinds,
    };

    std::thread::Builder::new()
        .name("homied-mcp".into())
        .spawn(move || {
            let runtime = match tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
            {
                Ok(runtime) => runtime,
                Err(error) => {
                    eprintln!("homied-rs: MCP runtime init failed: {error}");
                    return;
                }
            };
            runtime.block_on(async move {
                eprintln!("homied-rs: MCP listening on {base_url}");
                let app = router(state);
                let _ = axum::serve(listener, app).await;
            });
        })
        .ok();

    Some(runtime)
}

fn router(state: McpHttpState) -> Router {
    Router::new()
        .route("/mcp", post(handle_mcp))
        .with_state(state)
}

async fn handle_mcp(
    State(state): State<McpHttpState>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let authorized = headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .is_some_and(|token| token == state.token);
    if !authorized {
        return (StatusCode::UNAUTHORIZED, "unauthorized").into_response();
    }

    let caller = headers
        .get(SESSION_HEADER)
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);

    let message: Value = match serde_json::from_slice(&body) {
        Ok(value) => value,
        Err(_) => {
            return json_response(json!({
                "jsonrpc": "2.0",
                "id": Value::Null,
                "error": { "code": -32700, "message": "Parse error" },
            }));
        }
    };

    // Tool calls block (wait_for_agent, wait_for_children, browser), so run the
    // synchronous core off the async worker.
    let result = tokio::task::spawn_blocking(move || {
        let host =
            RegistryHost::new(state.registry.clone(), state.logs_dir.clone()).with_caller(caller);
        let host = match state.holder {
            Some(holder) => host.with_holder(holder),
            None => host,
        };
        let server = McpServer::new(tool_definitions_for(&state.kinds), host);
        server.handle(&message)
    })
    .await;

    match result {
        Ok(Some(response)) => json_response(response),
        // Notifications and client responses produce no reply: 202 Accepted.
        Ok(None) => StatusCode::ACCEPTED.into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "MCP handler task failed").into_response(),
    }
}

fn json_response(value: Value) -> Response {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        serde_json::to_vec(&value).unwrap_or_else(|_| b"{}".to_vec()),
    )
        .into_response()
}

/// Bind the preferred port, then an ephemeral one, on loopback only.
fn bind_loopback() -> Option<(std::net::TcpListener, u16)> {
    for port in [PREFERRED_PORT, 0] {
        if let Ok(listener) = std::net::TcpListener::bind(("127.0.0.1", port)) {
            let port = listener.local_addr().ok()?.port();
            return Some((listener, port));
        }
    }
    None
}

/// A fresh 256-bit bearer secret, hex-encoded. Not derived from anything
/// predictable; held only in memory.
fn random_secret() -> String {
    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes).expect("random");
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
