//! Virtual key issuance, validation, and HTTP authentication.
//!
//! Only a SHA-256 hash of each virtual key is stored; the raw key is returned
//! once at creation and never again. A configured master key is accepted in
//! addition to virtual keys.

use std::time::{SystemTime, UNIX_EPOCH};

use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::db::Db;
use crate::state::AppState;

/// A newly issued virtual key; the raw `key` is shown exactly once.
#[derive(Debug)]
pub struct CreatedGatewayApiKey {
    pub id: String,
    pub label: Option<String>,
    pub key: String,
    pub created_at: i64,
}

/// Metadata for an existing key; never contains the raw key.
#[derive(Clone, Debug, Serialize)]
pub struct ApiKeyRecord {
    pub id: String,
    pub label: Option<String>,
    pub created_at: i64,
    pub last_used_at: Option<i64>,
}

/// Who a request authenticated as.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Caller {
    VirtualKey(String),
    Master,
}

/// Stores virtual keys in SQLite and answers `accept` lookups.
#[derive(Clone)]
pub struct GatewayApiKeyStore {
    db: Db,
}

impl GatewayApiKeyStore {
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    pub fn create(&self, label: Option<String>) -> rusqlite::Result<CreatedGatewayApiKey> {
        let id = random_hex(16);
        let key = format!("sk-{}{}", random_hex(16), random_hex(16));
        let now = now_seconds();
        let hash = hash_key(&key);
        let conn = self.db.conn();
        conn.execute(
            "INSERT INTO gateway_api_keys (id, label, key_hash, created_at, last_used_at)
             VALUES (?1, ?2, ?3, ?4, NULL)",
            rusqlite::params![id, label, hash, now],
        )?;
        Ok(CreatedGatewayApiKey {
            id,
            label,
            key,
            created_at: now,
        })
    }

    pub fn delete(&self, id: &str) -> rusqlite::Result<bool> {
        let conn = self.db.conn();
        let affected = conn.execute("DELETE FROM gateway_api_keys WHERE id = ?1", [id])?;
        Ok(affected > 0)
    }

    pub fn list(&self) -> rusqlite::Result<Vec<ApiKeyRecord>> {
        let conn = self.db.conn();
        let mut stmt = conn.prepare(
            "SELECT id, label, created_at, last_used_at FROM gateway_api_keys ORDER BY created_at",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(ApiKeyRecord {
                id: row.get(0)?,
                label: row.get(1)?,
                created_at: row.get(2)?,
                last_used_at: row.get(3)?,
            })
        })?;
        rows.collect()
    }

    /// Validate a raw key, updating `last_used_at` on a virtual-key hit.
    /// Returns the virtual key id, or `None`.
    pub fn accept(&self, key: &str) -> Option<String> {
        let hash = hash_key(key);
        let conn = self.db.conn();
        let id: Option<String> = conn
            .query_row(
                "SELECT id FROM gateway_api_keys WHERE key_hash = ?1",
                [&hash],
                |row| row.get(0),
            )
            .ok();
        if let Some(id) = &id {
            let _ = conn.execute(
                "UPDATE gateway_api_keys SET last_used_at = ?1 WHERE id = ?2",
                rusqlite::params![now_seconds(), id],
            );
        }
        id
    }
}

/// Authenticate a request against the master key or virtual keys, tagging the
/// request with the resolved [`Caller`] for later usage recording.
pub async fn authenticate(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    let caller = resolve_caller(&state, &request);
    match caller {
        Some(caller) => {
            request.extensions_mut().insert(caller);
            next.run(request).await
        }
        None => unauthorized(),
    }
}

fn resolve_caller<B>(state: &AppState, request: &Request<B>) -> Option<Caller> {
    let key = extract_key(request.headers())?;
    if let Some(master) = &state.master_key
        && constant_time_eq(master.as_bytes(), key.as_bytes())
    {
        return Some(Caller::Master);
    }
    state.keys.accept(&key).map(Caller::VirtualKey)
}

fn extract_key(headers: &HeaderMap) -> Option<String> {
    if let Some(value) = headers.get(header::AUTHORIZATION)
        && let Ok(value) = value.to_str()
        && let Some(token) = value.strip_prefix("Bearer ")
    {
        return Some(token.to_owned());
    }
    headers
        .get("x-api-key")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_owned())
}

fn unauthorized() -> Response {
    (StatusCode::UNAUTHORIZED, "unauthorized").into_response()
}

pub fn random_hex(bytes: usize) -> String {
    let mut buf = vec![0u8; bytes];
    getrandom::fill(&mut buf).expect("random");
    hex_encode(&buf)
}

pub fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        out.push(char::from(HEX[usize::from(b >> 4)]));
        out.push(char::from(HEX[usize::from(b & 0x0f)]));
    }
    out
}

fn hash_key(key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());
    hex_encode(&hasher.finalize())
}

fn now_seconds() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| i64::try_from(d.as_secs()).unwrap_or(i64::MAX))
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    let mut diff = left.len() ^ right.len();
    for i in 0..left.len().max(right.len()) {
        let l = left.get(i).copied().unwrap_or_default();
        let r = right.get(i).copied().unwrap_or_default();
        diff |= usize::from(l ^ r);
    }
    diff == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_then_accept_and_delete() {
        let db = Db::open_in_memory().expect("db");
        let store = GatewayApiKeyStore::new(db);
        let created = store.create(Some("test".into())).expect("create");
        assert!(created.key.starts_with("sk-"));
        assert_eq!(store.accept(&created.key), Some(created.id.clone()));
        assert!(store.delete(&created.id).expect("delete"));
        assert_eq!(store.accept(&created.key), None);
    }

    #[test]
    fn list_never_returns_raw_key() {
        let db = Db::open_in_memory().expect("db");
        let store = GatewayApiKeyStore::new(db);
        let created = store.create(None).expect("create");
        let list = store.list().expect("list");
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, created.id);
        // No field on the record can ever expose the raw key.
        let serialized = format!("{:?}", list[0]);
        assert!(!serialized.contains(&created.key));
    }

    #[test]
    fn accept_updates_last_used() {
        let db = Db::open_in_memory().expect("db");
        let store = GatewayApiKeyStore::new(db);
        let created = store.create(None).expect("create");
        assert!(store.accept(&created.key).is_some());
        let list = store.list().expect("list");
        assert!(list[0].last_used_at.is_some());
    }

    #[test]
    fn random_hex_is_unique_and_sized() {
        let a = random_hex(16);
        let b = random_hex(16);
        assert_ne!(a, b);
        assert_eq!(a.len(), 32);
    }
}
