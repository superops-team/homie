//! Upstream forwarding to a single OpenAI-compatible provider.
//!
//! The upstream key is attached server-side only; the caller never sees it.
//! Streaming (SSE) responses are passed through as raw bytes so the client's
//! SSE framing is preserved.

use std::error::Error;

use axum::{
    body::Body,
    http::{HeaderValue, header},
    response::Response,
};
use futures_util::StreamExt;
use homie_node::config::NodePaths;
use homie_node::credentials::resolve_default_codex_credential;
use reqwest::Client;

type BoxError = Box<dyn Error + Send + Sync>;

#[derive(Clone)]
pub struct Upstream {
    base_url: String,
    api_key: String,
    prefer_node: bool,
    client: Client,
}

/// A fully built client response plus the token counts extracted from the
/// upstream body (zero for streaming responses).
pub struct ForwardResult {
    pub response: Response,
    pub input_tokens: i64,
    pub output_tokens: i64,
}

#[derive(Debug)]
pub struct UpstreamError(pub String);

impl std::fmt::Display for UpstreamError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "upstream error: {}", self.0)
    }
}

impl Error for UpstreamError {}

impl Upstream {
    pub fn new(base_url: String, api_key: String, prefer_node: bool) -> Self {
        Self {
            base_url,
            api_key,
            prefer_node,
            client: Client::new(),
        }
    }

    /// Resolve the `(base_url, api_key)` to use for a request. In `node` mode
    /// the local Homie node credential is preferred, falling back to the
    /// configured static key.
    fn resolve_credential(&self) -> Result<(String, String), UpstreamError> {
        if self.prefer_node
            && let Ok(credential) = resolve_default_codex_credential(&NodePaths::discover())
        {
            return Ok((credential.base_url, credential.token));
        }
        if self.api_key.is_empty() {
            return Err(UpstreamError(
                "no upstream credential available (node resolve failed and no static key)".into(),
            ));
        }
        Ok((self.base_url.clone(), self.api_key.clone()))
    }

    pub async fn forward(&self, path: &str, body: Vec<u8>) -> Result<ForwardResult, UpstreamError> {
        let (base_url, api_key) = self.resolve_credential()?;
        let url = format!("{}{}", base_url, path);
        let resp = self
            .client
            .post(&url)
            .header(header::AUTHORIZATION, format!("Bearer {}", api_key))
            .header(header::CONTENT_TYPE, "application/json")
            .body(body)
            .send()
            .await
            .map_err(|e| UpstreamError(e.to_string()))?;

        let status = resp.status();
        let is_sse = resp
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .map(|ct| ct.contains("text/event-stream"))
            .unwrap_or(false);

        if is_sse {
            let stream = resp
                .bytes_stream()
                .map(|item| item.map_err(|e| -> BoxError { Box::new(e) }));
            let mut response = Response::new(Body::from_stream(stream));
            *response.status_mut() = status;
            response.headers_mut().insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("text/event-stream"),
            );
            Ok(ForwardResult {
                response,
                input_tokens: 0,
                output_tokens: 0,
            })
        } else {
            let bytes = resp
                .bytes()
                .await
                .map_err(|e| UpstreamError(e.to_string()))?;
            let (input_tokens, output_tokens) = extract_usage(&bytes);
            let mut response = Response::new(Body::from(bytes));
            *response.status_mut() = status;
            response.headers_mut().insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/json"),
            );
            Ok(ForwardResult {
                response,
                input_tokens,
                output_tokens,
            })
        }
    }
}

/// Extract `usage.input_tokens` / `usage.output_tokens` from a JSON body.
/// Returns `(0, 0)` when absent or unparseable.
fn extract_usage(body: &[u8]) -> (i64, i64) {
    let Ok(value) = serde_json::from_slice::<serde_json::Value>(body) else {
        return (0, 0);
    };
    let usage = value.get("usage").cloned().unwrap_or_default();
    let input = usage
        .get("input_tokens")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let output = usage
        .get("output_tokens")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    (input, output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_usage_when_present() {
        let body = br#"{"usage":{"input_tokens":10,"output_tokens":5}}"#;
        assert_eq!(extract_usage(body), (10, 5));
    }

    #[test]
    fn usage_absent_is_zero() {
        assert_eq!(extract_usage(br#"{"ok":true}"#), (0, 0));
        assert_eq!(extract_usage(b"not json"), (0, 0));
    }

    #[test]
    fn static_mode_resolves_static_key() {
        let upstream = Upstream::new(
            "https://api.example.com/v1".into(),
            "sk-static".into(),
            false,
        );
        let (base, key) = upstream.resolve_credential().expect("resolve");
        assert_eq!(base, "https://api.example.com/v1");
        assert_eq!(key, "sk-static");
    }

    #[test]
    fn node_mode_falls_back_to_static_key() {
        let dir = tempfile::tempdir().expect("tempdir");
        // SAFETY: tests run in the same process; these set a path that is
        // only read synchronously by `NodePaths::discover` in this test.
        unsafe { std::env::set_var("HOMIE_NODE_HOME", dir.path()) };
        let upstream = Upstream::new(
            "https://api.example.com/v1".into(),
            "sk-fallback".into(),
            true,
        );
        let (base, key) = upstream.resolve_credential().expect("resolve");
        assert_eq!(base, "https://api.example.com/v1");
        assert_eq!(key, "sk-fallback");
    }

    #[test]
    fn node_mode_without_any_credential_errors() {
        let dir = tempfile::tempdir().expect("tempdir");
        // SAFETY: tests run in the same process; these set a path that is
        // only read synchronously by `NodePaths::discover` in this test.
        unsafe { std::env::set_var("HOMIE_NODE_HOME", dir.path()) };
        let upstream = Upstream::new("https://api.example.com/v1".into(), "".into(), true);
        assert!(upstream.resolve_credential().is_err());
    }
}
