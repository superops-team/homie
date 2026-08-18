//! Shared SQLite handle: virtual keys and usage live in one local database.
//!
//! The connection is cheap and local; a `std::sync::Mutex` guards it because
//! `rusqlite::Connection` is `!Sync`. Handlers hold the lock only for the
//! microseconds a single statement takes.

use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard};

use rusqlite::Connection;

#[derive(Clone)]
pub struct Db {
    inner: Arc<Mutex<Connection>>,
}

impl Db {
    pub fn open(path: &Path) -> rusqlite::Result<Self> {
        let conn = Connection::open(path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS gateway_api_keys (
                id          TEXT PRIMARY KEY,
                label       TEXT,
                key_hash    TEXT NOT NULL UNIQUE,
                created_at  INTEGER NOT NULL,
                last_used_at INTEGER
            );
            CREATE TABLE IF NOT EXISTS gateway_usage (
                id            INTEGER PRIMARY KEY AUTOINCREMENT,
                key_id        TEXT NOT NULL,
                model         TEXT NOT NULL,
                occurred_at   INTEGER NOT NULL,
                input_tokens  INTEGER NOT NULL,
                output_tokens INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_usage_key ON gateway_usage(key_id);
            CREATE TABLE IF NOT EXISTS gateway_audit (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                key_id      TEXT NOT NULL,
                event       TEXT NOT NULL,
                occurred_at INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_audit_key ON gateway_audit(key_id);",
        )?;
        Ok(Self {
            inner: Arc::new(Mutex::new(conn)),
        })
    }

    #[cfg(test)]
    pub fn open_in_memory() -> rusqlite::Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch(
            "CREATE TABLE gateway_api_keys (
                id          TEXT PRIMARY KEY,
                label       TEXT,
                key_hash    TEXT NOT NULL UNIQUE,
                created_at  INTEGER NOT NULL,
                last_used_at INTEGER
            );
            CREATE TABLE gateway_usage (
                id            INTEGER PRIMARY KEY AUTOINCREMENT,
                key_id        TEXT NOT NULL,
                model         TEXT NOT NULL,
                occurred_at   INTEGER NOT NULL,
                input_tokens  INTEGER NOT NULL,
                output_tokens INTEGER NOT NULL
            );
            CREATE TABLE gateway_audit (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                key_id      TEXT NOT NULL,
                event       TEXT NOT NULL,
                occurred_at INTEGER NOT NULL
            );",
        )?;
        Ok(Self {
            inner: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn conn(&self) -> MutexGuard<'_, Connection> {
        self.inner.lock().expect("gateway db mutex poisoned")
    }
}
