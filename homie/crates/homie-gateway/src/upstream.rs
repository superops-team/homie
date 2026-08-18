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
use reqwest::Client;

type BoxError = Box<dyn Error + Send + Sync>;

#[derive(Clone)]
pub struct Upstream {
    base_url: String,
    api_key: String,
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
    pub fn new(base_url: String, api_key: String) -> Self {
        Self {
            base_url,
            api_key,
            client: Client::new(),
        }
    }

    pub async fn forward(&self, path: &str, body: Vec<u8>) -> Result<ForwardResult, UpstreamError> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .client
            .post(&url)
            .header(header::AUTHORIZATION, format!("Bearer {}", self.api_key))
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
}
