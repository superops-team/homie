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
    let model = extract_model(&body);
    match state.upstream.forward(path, body.to_vec()).await {
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
    fn extracts_model_from_body() {
        assert_eq!(
            extract_model(br#"{"model":"gpt-5.2-codex"}"#),
            "gpt-5.2-codex"
        );
        assert_eq!(extract_model(br#"{"x":1}"#), "unknown");
        assert_eq!(extract_model(b"junk"), "unknown");
    }
}
