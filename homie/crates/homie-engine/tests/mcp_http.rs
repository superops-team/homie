//! HTTP transport for the daemon-embedded MCP server.
//!
//! Proves the `POST /mcp` loopback surface: bearer auth (missing/wrong/valid),
//! the core JSON-RPC methods (`initialize`, `tools/list`, `ping`), and that the
//! injection fact file carries only the URL — never the in-memory bearer secret.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use homie_engine::detect::ManifestEngine;
use homie_engine::mcp::http::{self, McpRuntime};
use homie_engine::registry::Registry;
use serde_json::{Value, json};

/// Build a live MCP HTTP endpoint backed by the bundled manifests and a fresh
/// temp dir. Returns the runtime facts plus the temp dir (kept alive for the
/// duration of the test).
fn start_server() -> (McpRuntime, tempfile::TempDir) {
    let manifest_dir = homie_engine::detect::bundled_manifest_dir()
        .canonicalize()
        .expect("manifests");
    let (engine, _warnings) = ManifestEngine::load_dir(&manifest_dir).expect("load manifests");
    let engine = Arc::new(engine);

    let temp = tempfile::tempdir().expect("temp");
    let registry = Registry::new(engine.clone(), temp.path().join("state.json"));
    let registry = Arc::new(Mutex::new(registry));

    let logs_dir = temp.path().join("logs");
    std::fs::create_dir_all(&logs_dir).expect("logs dir");

    let runtime = http::start(
        registry,
        logs_dir,
        None,
        engine.ids().into_iter().map(String::from).collect(),
        temp.path(),
    )
    .expect("mcp start");

    (runtime, temp)
}

async fn send_json(
    client: &reqwest::Client,
    url: &str,
    bearer: Option<&str>,
    body: Value,
) -> reqwest::Response {
    let mut req = client
        .post(url)
        .header(reqwest::header::CONTENT_TYPE, "application/json");
    if let Some(token) = bearer {
        req = req.header(reqwest::header::AUTHORIZATION, format!("Bearer {token}"));
    }
    req.json(&body).send().await.expect("request")
}

#[tokio::test]
async fn auth_and_rpc_surface() {
    let (runtime, temp) = start_server();
    let url = format!("{}/mcp", runtime.base_url);

    // The server thread binds synchronously before `start` returns, but give the
    // tokio accept loop a moment under load.
    let client = reqwest::Client::new();
    let ping_body = json!({ "jsonrpc": "2.0", "id": 0, "method": "ping" });
    let mut last_status = 0;
    for _ in 0..20 {
        let resp = send_json(&client, &url, Some(&runtime.token), ping_body.clone()).await;
        last_status = resp.status().as_u16();
        if last_status == 200 {
            break;
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    assert_eq!(last_status, 200, "MCP server did not come up on {url}");

    // 1. Missing token -> 401.
    let resp = send_json(
        &client,
        &url,
        None,
        json!({ "jsonrpc": "2.0", "id": 1, "method": "ping" }),
    )
    .await;
    assert_eq!(resp.status().as_u16(), 401);

    // 2. Wrong token -> 401.
    let resp = send_json(
        &client,
        &url,
        Some("<wrong-token>"),
        json!({ "jsonrpc": "2.0", "id": 2, "method": "ping" }),
    )
    .await;
    assert_eq!(resp.status().as_u16(), 401);

    // 3. Valid token + initialize -> 200, serverInfo.name == "homie", echoes protocolVersion.
    let resp = send_json(
        &client,
        &url,
        Some(&runtime.token),
        json!({ "jsonrpc": "2.0", "id": 3, "method": "initialize", "params": { "protocolVersion": "2025-06-18" } }),
    )
    .await;
    assert_eq!(resp.status().as_u16(), 200);
    let value: Value = resp.json().await.expect("initialize body");
    assert_eq!(value["result"]["serverInfo"]["name"], "homie");
    assert_eq!(value["result"]["protocolVersion"], "2025-06-18");

    // 4. Valid token + tools/list -> 200, non-empty tools.
    let resp = send_json(
        &client,
        &url,
        Some(&runtime.token),
        json!({ "jsonrpc": "2.0", "id": 4, "method": "tools/list" }),
    )
    .await;
    assert_eq!(resp.status().as_u16(), 200);
    let value: Value = resp.json().await.expect("tools/list body");
    let tools = value["result"]["tools"].as_array().expect("tools array");
    assert!(!tools.is_empty(), "expected at least one MCP tool");

    // 5. Valid token + ping -> 200, result == {}.
    let resp = send_json(
        &client,
        &url,
        Some(&runtime.token),
        json!({ "jsonrpc": "2.0", "id": 5, "method": "ping" }),
    )
    .await;
    assert_eq!(resp.status().as_u16(), 200);
    let value: Value = resp.json().await.expect("ping body");
    assert_eq!(value["result"], json!({}));

    // 6. Fact file carries only the URL; the secret must never be on disk.
    let fact_path = temp.path().join("mcp-http.json");
    let fact_text = std::fs::read_to_string(&fact_path).expect("fact file");
    let fact: Value = serde_json::from_str(&fact_text).expect("fact json");
    assert_eq!(fact["url"], json!(format!("{url}")));
    assert_eq!(
        fact.as_object().expect("fact object").len(),
        1,
        "fact file must hold only url"
    );
    assert!(
        !fact_text.contains(&runtime.token),
        "fact file leaked the in-memory bearer secret"
    );
}
