//! HTTP routes: OpenAI Responses for Codex, Anthropic Messages for Claude Code,
//! and a master-key-protected admin surface for virtual key lifecycle.

use axum::{
    Json, Router,
    body::Bytes,
    extract::{DefaultBodyLimit, Extension, Path, State},
    http::StatusCode,
    middleware,
    response::{IntoResponse, Response},
    routing::{delete, get, post},
};
use serde::Deserialize;

use crate::auth::{Caller, authenticate, require_master};
use crate::state::AppState;

pub fn router(state: AppState) -> Router {
    let admin = Router::new()
        .route("/admin/keys", post(create_key).get(list_keys))
        .route("/admin/keys/{id}", delete(delete_key))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            require_master,
        ));

    let api = Router::new()
        .route("/v1/responses", post(handle_responses))
        .route("/v1/messages", post(handle_messages))
        .layer(DefaultBodyLimit::max(64 * 1024 * 1024))
        .layer(middleware::from_fn_with_state(state.clone(), authenticate));

    Router::new()
        .route("/healthz", get(healthz))
        .merge(api)
        .merge(admin)
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

async fn handle_messages(
    State(state): State<AppState>,
    Extension(caller): Extension<Caller>,
    body: Bytes,
) -> Response {
    forward_and_record(&state, &caller, "/messages", body).await
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

#[derive(Deserialize)]
struct CreateKeyRequest {
    label: Option<String>,
}

async fn create_key(
    State(state): State<AppState>,
    Json(payload): Json<CreateKeyRequest>,
) -> Response {
    match state.keys.create(payload.label) {
        Ok(created) => (
            StatusCode::CREATED,
            Json(serde_json::json!({
                "id": created.id,
                "label": created.label,
                "key": created.key,
                "created_at": created.created_at,
            })),
        )
            .into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "failed to create key").into_response(),
    }
}

async fn list_keys(State(state): State<AppState>) -> Response {
    match state.keys.list() {
        Ok(records) => Json(records).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "failed to list keys").into_response(),
    }
}

async fn delete_key(State(state): State<AppState>, Path(id): Path<String>) -> Response {
    match state.keys.delete(&id) {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => StatusCode::NOT_FOUND.into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "failed to delete key").into_response(),
    }
}

/// Route key: HTTP path → agent model-map key.
fn route_key(path: &str) -> Option<&'static str> {
    match path {
        "/responses" => Some("codex"),
        "/messages" => Some("claude"),
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
    fn route_key_maps_paths_to_agents() {
        assert_eq!(route_key("/responses"), Some("codex"));
        assert_eq!(route_key("/messages"), Some("claude"));
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
