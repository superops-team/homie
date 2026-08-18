//! Per-virtual-key usage recording.
//!
//! Token counts are best-effort: parsed from the upstream response `usage`
//! object when present, otherwise zero. They are estimates, never authoritative
//! billing (see `homie-usage` for the shared pricing estimate).

use std::time::{SystemTime, UNIX_EPOCH};

use crate::db::Db;

#[derive(Clone)]
pub struct UsageStore {
    db: Db,
}

impl UsageStore {
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    pub fn record(
        &self,
        key_id: &str,
        model: &str,
        input_tokens: i64,
        output_tokens: i64,
    ) -> rusqlite::Result<()> {
        let conn = self.db.conn();
        conn.execute(
            "INSERT INTO gateway_usage (key_id, model, occurred_at, input_tokens, output_tokens)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![key_id, model, now_seconds(), input_tokens, output_tokens],
        )?;
        Ok(())
    }

    /// Sum of `input_tokens + output_tokens` for a key since `since` (inclusive),
    /// used by the daily quota check.
    pub fn sum_tokens_since(&self, key_id: &str, since: i64) -> rusqlite::Result<i64> {
        let conn = self.db.conn();
        conn.query_row(
            "SELECT COALESCE(SUM(input_tokens + output_tokens), 0)
             FROM gateway_usage WHERE key_id = ?1 AND occurred_at >= ?2",
            rusqlite::params![key_id, since],
            |row| row.get(0),
        )
    }
}

fn now_seconds() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| i64::try_from(d.as_secs()).unwrap_or(i64::MAX))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_usage_per_key() {
        let db = Db::open_in_memory().expect("db");
        let store = UsageStore::new(db.clone());
        store.record("k1", "gpt-5", 10, 5).expect("record");

        let conn = db.conn();
        let (model, input, output): (String, i64, i64) = conn
            .query_row(
                "SELECT model, input_tokens, output_tokens FROM gateway_usage WHERE key_id = 'k1'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .expect("row");
        assert_eq!(model, "gpt-5");
        assert_eq!(input, 10);
        assert_eq!(output, 5);
    }
}
