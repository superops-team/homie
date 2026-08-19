//! HTTP routes: a single OpenAI Responses endpoint for Codex (and future
//! OpenAI-compatible agents). Anthropic Messages and the master-key admin
//! surface are gone: the daemon mints virtual keys at spawn and enforces
//! policy/usage on `/v1/responses`.

use axum::{
    Router,
    body::Bytes,
    extract::{DefaultBodyLimit, Extension, State},
    http::StatusCode,
    middleware,
    response::{IntoResponse, Response},
    routing::{get, post},
};

use crate::auth::{Caller, authenticate};
use crate::policy::{DenyReason, QuotaChecker, deny_response, now_seconds, record_audit};
use crate::state::AppState;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/v1/responses", post(handle_responses))
        .layer(DefaultBodyLimit::max(64 * 1024 * 1024))
        .layer(middleware::from_fn_with_state(state.clone(), authenticate))
        .with_state(state)
}

async fn healthz() -> &'static str {
    "ok"
}

async fn handle_responses(
    State(state): State<AppState>,
    Extension(caller): Extension<Caller>,
    body: Bytes,
) -> Response {
    forward_and_record(&state, &caller, "/responses", body).await
}

async fn forward_and_record(
    state: &AppState,
    caller: &Caller,
    path: &str,
    body: Bytes,
) -> Response {
    let routed = route_key(path)
        .map(|key| apply_model_route(&state.models, key, body.as_ref()))
        .unwrap_or_else(|| body.to_vec());
    let model = extract_model(&routed);

    // Enforce policy only for virtual keys (the master key is the trusted admin
    // surface and is not rate-limited or quota-counted).
    if let Caller::VirtualKey(key_id) = caller
        && let Some(deny) = check_policy(state, key_id)
    {
        return deny;
    }

    match state.upstream.forward(path, routed).await {
        Ok(result) => {
            if let Caller::VirtualKey(id) = caller {
                let _ = state
                    .usage
                    .record(id, &model, result.input_tokens, result.output_tokens);
            }
            result.response
        }
        Err(_) => (StatusCode::BAD_GATEWAY, "bad gateway").into_response(),
    }
}

/// Apply rate-limit then quota checks. Returns a `429` response when either
/// denies, recording the denial in `gateway_audit`.
fn check_policy(state: &AppState, key_id: &str) -> Option<Response> {
    let Some(policy) = &state.policy else {
        return None;
    };
    let now = now_seconds();

    if let Some(rate_limit) = &policy.rate_limit {
        let allowed = state
            .rate_limiter
            .lock()
            .expect("rate limiter mutex poisoned")
            .allow(key_id, rate_limit.requests_per_minute, now);
        if !allowed {
            let _ = record_audit(&state.db, key_id, "rate_limited", now);
            return Some(deny_response(DenyReason::RateLimit));
        }
    }

    if let Some(quota) = &policy.quota {
        let checker = QuotaChecker::new(&state.usage);
        match checker.allow(key_id, quota.daily_token_limit, now) {
            Ok(true) => {}
            Ok(false) => {
                let _ = record_audit(&state.db, key_id, "quota_exceeded", now);
                return Some(deny_response(DenyReason::Quota));
            }
            Err(_) => {
                return Some(
                    (StatusCode::INTERNAL_SERVER_ERROR, "quota check failed").into_response(),
                );
            }
        }
    }

    None
}

/// Route key: HTTP path → agent model-map key. Only the OpenAI Responses path
/// remains after Anthropic Messages was removed.
fn route_key(path: &str) -> Option<&'static str> {
    match path {
        "/responses" => Some("codex"),
        _ => None,
    }
}

/// Rewrite the top-level `model` string when a configured mapping exists.
/// Returns the body unchanged when the mapping is absent, the body is not
/// JSON, or the top-level `model` is not a string.
fn apply_model_route(
    models: &std::collections::BTreeMap<String, String>,
    key: &str,
    body: &[u8],
) -> Vec<u8> {
    let Some(target) = models.get(key) else {
        return body.to_vec();
    };
    let Ok(mut value) = serde_json::from_slice::<serde_json::Value>(body) else {
        return body.to_vec();
    };
    if let Some(model) = value.as_object_mut().and_then(|obj| obj.get_mut("model"))
        && model.is_string()
    {
        *model = serde_json::Value::String(target.clone());
    }
    serde_json::to_vec(&value).unwrap_or_else(|_| body.to_vec())
}

fn extract_model(body: &[u8]) -> String {
    serde_json::from_slice::<serde_json::Value>(body)
        .ok()
        .and_then(|value| {
            value
                .get("model")
                .and_then(|m| m.as_str())
                .map(str::to_owned)
        })
        .unwrap_or_else(|| "unknown".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn route_key_maps_only_responses_to_codex() {
        assert_eq!(route_key("/responses"), Some("codex"));
        assert_eq!(route_key("/messages"), None);
        assert_eq!(route_key("/v1/responses"), None);
        assert_eq!(route_key(""), None);
    }

    #[test]
    fn apply_model_route_rewrites_when_configured() {
        let models =
            std::collections::BTreeMap::from([("codex".to_string(), "gpt-5.2-codex".to_string())]);
        let body = br#"{"model":"gpt-5"}"#;
        let rewritten = apply_model_route(&models, "codex", body);
        let v: serde_json::Value = serde_json::from_slice(&rewritten).expect("json");
        assert_eq!(v["model"], "gpt-5.2-codex");
    }

    #[test]
    fn apply_model_route_passes_through_when_key_missing() {
        let models = std::collections::BTreeMap::new();
        let body = br#"{"model":"gpt-5"}"#;
        assert_eq!(apply_model_route(&models, "codex", body), body);
    }

    #[test]
    fn apply_model_route_passes_through_non_json() {
        let models =
            std::collections::BTreeMap::from([("codex".to_string(), "gpt-5.2-codex".to_string())]);
        let body = b"not json";
        assert_eq!(apply_model_route(&models, "codex", body), body);
    }

    #[test]
    fn apply_model_route_passes_through_non_string_model() {
        let models =
            std::collections::BTreeMap::from([("codex".to_string(), "gpt-5.2-codex".to_string())]);
        let body = br#"{"model":123}"#;
        let rewritten = apply_model_route(&models, "codex", body);
        let v: serde_json::Value = serde_json::from_slice(&rewritten).expect("json");
        assert_eq!(v["model"], 123);
    }

    #[test]
    fn extract_model_from_body() {
        assert_eq!(
            extract_model(br#"{"model":"gpt-5.2-codex"}"#),
            "gpt-5.2-codex"
        );
        assert_eq!(extract_model(br#"{"x":1}"#), "unknown");
        assert_eq!(extract_model(b"junk"), "unknown");
    }
}
