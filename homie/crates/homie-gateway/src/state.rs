//! Shared request state: virtual keys, usage, and the upstream forwarder.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use crate::auth::GatewayApiKeyStore;
use crate::config::Policy;
use crate::db::Db;
use crate::policy::RateLimiter;
use crate::upstream::Upstream;
use crate::usage::UsageStore;

#[derive(Clone)]
pub struct AppState {
    pub keys: GatewayApiKeyStore,
    pub usage: UsageStore,
    pub upstream: Upstream,
    pub master_key: Option<String>,
    pub models: BTreeMap<String, String>,
    pub policy: Option<Policy>,
    pub rate_limiter: Arc<Mutex<RateLimiter>>,
    pub db: Db,
}

impl AppState {
    pub fn new(
        db: Db,
        upstream: Upstream,
        master_key: Option<String>,
        models: BTreeMap<String, String>,
        policy: Option<Policy>,
    ) -> Self {
        Self {
            keys: GatewayApiKeyStore::new(db.clone()),
            usage: UsageStore::new(db.clone()),
            upstream,
            master_key,
            models,
            policy,
            rate_limiter: Arc::new(Mutex::new(RateLimiter::new())),
            db,
        }
    }
}
