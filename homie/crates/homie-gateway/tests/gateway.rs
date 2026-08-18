//! End-to-end gateway slice, exercised directly through the axum router:
//! virtual key → `/v1/responses` / `/v1/messages` → wiremock upstream →
//! per-key usage recorded. No real network: the upstream is a wiremock server
//! bound to loopback, and the SQLite store lives in a temp dir.

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use http_body_util::BodyExt;
use tower::ServiceExt;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use std::collections::BTreeMap;

use homie_gateway::config::{Policy, Quota, RateLimit};
use homie_gateway::db::Db;
use homie_gateway::state::AppState;
use homie_gateway::upstream::Upstream;

const MASTER: &str = "master-key";

/// A running app + its temp-dir-backed SQLite, plus the wiremock upstream URI.
struct Harness {
    db: Db,
    app: axum::Router,
}

fn spawn(
    master_key: Option<String>,
    upstream_base: &str,
    models: BTreeMap<String, String>,
    policy: Option<Policy>,
) -> Harness {
    let dir = tempfile::tempdir().expect("tempdir");
    let db = Db::open(&dir.path().join("gateway.sqlite3")).expect("open db");
    // Keep the tempdir alive for the lifetime of the harness by leaking it:
    // the DB connection stays open on the same file, so removal is safe anyway.
    std::mem::forget(dir);
    let upstream = Upstream::new(upstream_base.to_owned(), "upstream-secret".to_owned());
    let state = AppState::new(db.clone(), upstream, master_key, models, policy);
    let app = homie_gateway::routes::router(state);
    Harness { db, app }
}

impl Harness {
    /// Issue a virtual key through the master-protected admin surface.
    async fn create_key(&self, label: &str) -> (String, String) {
        let resp = self
            .app
            .clone()
            .oneshot(
                Request::post("/admin/keys")
                    .header(header::AUTHORIZATION, format!("Bearer {MASTER}"))
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(
                        serde_json::json!({ "label": label }).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let id = json["id"].as_str().unwrap().to_owned();
        let key = json["key"].as_str().unwrap().to_owned();
        (id, key)
    }

    async fn call(&self, method: &str, uri: &str, key: &str, body: &str) -> (StatusCode, Vec<u8>) {
        let req = match method {
            "POST" => Request::post(uri),
            "GET" => Request::get(uri),
            "DELETE" => Request::delete(uri),
            _ => unreachable!(),
        }
        .header(header::AUTHORIZATION, format!("Bearer {key}"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_owned()))
        .unwrap();
        let resp = self.app.clone().oneshot(req).await.unwrap();
        let status = resp.status();
        let bytes = resp
            .into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes()
            .to_vec();
        (status, bytes)
    }

    fn usage_rows(&self) -> Vec<(String, String, i64, i64)> {
        let conn = self.db.conn();
        let mut stmt = conn
            .prepare(
                "SELECT key_id, model, input_tokens, output_tokens FROM gateway_usage ORDER BY id",
            )
            .unwrap();
        stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
    }
}

/// Mount a JSON upstream at a wiremock path returning a fixed body.
async fn upstream_at(server: &MockServer, route: &str, body: &str) {
    Mock::given(method("POST"))
        .and(path(route))
        .respond_with(ResponseTemplate::new(200).set_body_raw(body, "application/json"))
        .mount(server)
        .await;
}

#[tokio::test]
async fn responses_slice_records_usage_per_key() {
    let server = MockServer::start().await;
    upstream_at(
        &server,
        "/responses",
        r#"{"id":"r1","usage":{"input_tokens":12,"output_tokens":7}}"#,
    )
    .await;
    let harness = spawn(Some(MASTER.into()), &server.uri(), BTreeMap::new(), None);

    let (id, key) = harness.create_key("codex").await;
    let (status, bytes) = harness
        .call(
            "POST",
            "/v1/responses",
            &key,
            r#"{"model":"gpt-5.2-codex","input":"hello"}"#,
        )
        .await;

    assert_eq!(status, StatusCode::OK);
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["id"], "r1");

    let rows = harness.usage_rows();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].0, id);
    assert_eq!(rows[0].1, "gpt-5.2-codex");
    assert_eq!(rows[0].2, 12);
    assert_eq!(rows[0].3, 7);
}

#[tokio::test]
async fn messages_slice_uses_same_virtual_key() {
    let server = MockServer::start().await;
    upstream_at(
        &server,
        "/messages",
        r#"{"id":"m1","usage":{"input_tokens":3,"output_tokens":9}}"#,
    )
    .await;
    let harness = spawn(Some(MASTER.into()), &server.uri(), BTreeMap::new(), None);

    let (id, key) = harness.create_key("claude").await;
    let (status, _) = harness
        .call(
            "POST",
            "/v1/messages",
            &key,
            r#"{"model":"claude-sonnet-4","messages":[]}"#,
        )
        .await;

    assert_eq!(status, StatusCode::OK);
    let rows = harness.usage_rows();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].0, id);
    assert_eq!(rows[0].1, "claude-sonnet-4");
    assert_eq!(rows[0].2, 3);
    assert_eq!(rows[0].3, 9);
}

#[tokio::test]
async fn bad_key_is_rejected_and_never_forwarded() {
    let server = MockServer::start().await;
    // No mock mounted: if the request were forwarded it would 404, but a bad
    // key must be rejected at the auth layer before any upstream call.
    let harness = spawn(Some(MASTER.into()), &server.uri(), BTreeMap::new(), None);

    let (status, _) = harness
        .call("POST", "/v1/responses", "sk-bogus", r#"{"model":"m"}"#)
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert!(harness.usage_rows().is_empty());
}

#[tokio::test]
async fn revoked_key_returns_unauthorized() {
    let server = MockServer::start().await;
    upstream_at(&server, "/responses", r#"{"ok":true}"#).await;
    let harness = spawn(Some(MASTER.into()), &server.uri(), BTreeMap::new(), None);

    let (id, key) = harness.create_key("temp").await;
    // Revoke via admin.
    let (status, _) = harness
        .call("DELETE", &format!("/admin/keys/{id}"), MASTER, "")
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (status, _) = harness
        .call("POST", "/v1/responses", &key, r#"{"model":"m"}"#)
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert!(harness.usage_rows().is_empty());
}

#[tokio::test]
async fn master_key_is_accepted_but_not_usage_recorded() {
    let server = MockServer::start().await;
    upstream_at(
        &server,
        "/responses",
        r#"{"usage":{"input_tokens":1,"output_tokens":1}}"#,
    )
    .await;
    let harness = spawn(Some(MASTER.into()), &server.uri(), BTreeMap::new(), None);

    let (status, _) = harness
        .call("POST", "/v1/responses", MASTER, r#"{"model":"m"}"#)
        .await;
    assert_eq!(status, StatusCode::OK);
    // Master is not a virtual key, so no usage row is written.
    assert!(harness.usage_rows().is_empty());
}

#[tokio::test]
async fn admin_requires_master_key() {
    let server = MockServer::start().await;
    let harness = spawn(None, &server.uri(), BTreeMap::new(), None);

    // No master key configured → admin surface is closed (403).
    let (status, _) = harness.call("GET", "/admin/keys", "whatever", "").await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn virtual_key_cannot_admin() {
    let server = MockServer::start().await;
    let harness = spawn(Some(MASTER.into()), &server.uri(), BTreeMap::new(), None);
    let (_, key) = harness.create_key("codex").await;

    // A virtual key must not reach the admin surface (401).
    let (status, _) = harness.call("GET", "/admin/keys", &key, "").await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn codex_model_is_rewritten_before_forward_and_recorded() {
    let server = MockServer::start().await;
    upstream_at(
        &server,
        "/responses",
        r#"{"id":"r-route","usage":{"input_tokens":5,"output_tokens":2}}"#,
    )
    .await;
    let models = BTreeMap::from([("codex".to_string(), "gpt-5.2-codex".to_string())]);
    let harness = spawn(Some(MASTER.into()), &server.uri(), models, None);

    let (id, key) = harness.create_key("codex").await;
    let (status, _) = harness
        .call(
            "POST",
            "/v1/responses",
            &key,
            r#"{"model":"gpt-5","input":"hello"}"#,
        )
        .await;

    assert_eq!(status, StatusCode::OK);
    let rows = harness.usage_rows();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].0, id);
    // The gateway must record the *rewritten* model, not the client-supplied one.
    assert_eq!(rows[0].1, "gpt-5.2-codex");
}

#[tokio::test]
async fn claude_model_is_rewritten_before_forward_and_recorded() {
    let server = MockServer::start().await;
    upstream_at(
        &server,
        "/messages",
        r#"{"id":"m-route","usage":{"input_tokens":2,"output_tokens":8}}"#,
    )
    .await;
    let models = BTreeMap::from([("claude".to_string(), "claude-sonnet-4".to_string())]);
    let harness = spawn(Some(MASTER.into()), &server.uri(), models, None);

    let (id, key) = harness.create_key("claude").await;
    let (status, _) = harness
        .call(
            "POST",
            "/v1/messages",
            &key,
            r#"{"model":"claude-3","messages":[]}"#,
        )
        .await;

    assert_eq!(status, StatusCode::OK);
    let rows = harness.usage_rows();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].0, id);
    assert_eq!(rows[0].1, "claude-sonnet-4");
}

#[tokio::test]
async fn unconfigured_model_passes_through_unchanged() {
    let server = MockServer::start().await;
    upstream_at(
        &server,
        "/responses",
        r#"{"id":"r-passthrough","usage":{"input_tokens":1,"output_tokens":1}}"#,
    )
    .await;
    let harness = spawn(Some(MASTER.into()), &server.uri(), BTreeMap::new(), None);

    let (_, key) = harness.create_key("codex").await;
    let (status, _) = harness
        .call(
            "POST",
            "/v1/responses",
            &key,
            r#"{"model":"gpt-5","input":"hello"}"#,
        )
        .await;

    assert_eq!(status, StatusCode::OK);
    let rows = harness.usage_rows();
    assert_eq!(rows[0].1, "gpt-5");
}

#[tokio::test]
async fn rate_limit_rejects_excess_requests() {
    let server = MockServer::start().await;
    upstream_at(
        &server,
        "/responses",
        r#"{"id":"r-rl","usage":{"input_tokens":1,"output_tokens":1}}"#,
    )
    .await;
    let policy = Policy {
        rate_limit: Some(RateLimit {
            requests_per_minute: 1,
        }),
        quota: None,
    };
    let harness = spawn(
        Some(MASTER.into()),
        &server.uri(),
        BTreeMap::new(),
        Some(policy),
    );

    let (_, key) = harness.create_key("codex").await;
    let (status, _) = harness
        .call("POST", "/v1/responses", &key, r#"{"model":"gpt-5"}"#)
        .await;
    assert_eq!(status, StatusCode::OK);

    let (status, bytes) = harness
        .call("POST", "/v1/responses", &key, r#"{"model":"gpt-5"}"#)
        .await;
    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
    let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body["error"]["type"], "rate_limit_error");

    // Only the first (allowed) request is forwarded and recorded.
    let rows = harness.usage_rows();
    assert_eq!(rows.len(), 1);
}

#[tokio::test]
async fn quota_rejects_when_daily_limit_exceeded() {
    let server = MockServer::start().await;
    upstream_at(
        &server,
        "/responses",
        r#"{"id":"r-quota","usage":{"input_tokens":5,"output_tokens":5}}"#,
    )
    .await;
    let policy = Policy {
        rate_limit: None,
        quota: Some(Quota {
            daily_token_limit: 10,
        }),
    };
    let harness = spawn(
        Some(MASTER.into()),
        &server.uri(),
        BTreeMap::new(),
        Some(policy),
    );

    let (_, key) = harness.create_key("codex").await;
    // First request records 5 + 5 = 10 tokens.
    let (status, _) = harness
        .call("POST", "/v1/responses", &key, r#"{"model":"gpt-5"}"#)
        .await;
    assert_eq!(status, StatusCode::OK);

    // Second request: cumulative 10 is not below limit 10 → 429.
    let (status, bytes) = harness
        .call("POST", "/v1/responses", &key, r#"{"model":"gpt-5"}"#)
        .await;
    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
    let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body["error"]["type"], "quota_error");

    // Only the first request is recorded in usage.
    assert_eq!(harness.usage_rows().len(), 1);
}

#[tokio::test]
async fn master_key_bypasses_policy() {
    let server = MockServer::start().await;
    upstream_at(
        &server,
        "/responses",
        r#"{"id":"r-master","usage":{"input_tokens":1,"output_tokens":1}}"#,
    )
    .await;
    let policy = Policy {
        rate_limit: Some(RateLimit {
            requests_per_minute: 1,
        }),
        quota: None,
    };
    let harness = spawn(
        Some(MASTER.into()),
        &server.uri(),
        BTreeMap::new(),
        Some(policy),
    );

    // Master key is not a virtual key, so rate limiting must not apply.
    let (status, _) = harness
        .call("POST", "/v1/responses", MASTER, r#"{"model":"m"}"#)
        .await;
    assert_eq!(status, StatusCode::OK);
    let (status, _) = harness
        .call("POST", "/v1/responses", MASTER, r#"{"model":"m"}"#)
        .await;
    assert_eq!(status, StatusCode::OK);
}
