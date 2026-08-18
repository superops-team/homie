//! Shared request state: virtual keys, usage, and the upstream forwarder.

use std::collections::BTreeMap;

use crate::auth::GatewayApiKeyStore;
use crate::db::Db;
use crate::upstream::Upstream;
use crate::usage::UsageStore;

#[derive(Clone)]
pub struct AppState {
    pub keys: GatewayApiKeyStore,
    pub usage: UsageStore,
    pub upstream: Upstream,
    pub master_key: Option<String>,
    pub models: BTreeMap<String, String>,
}

impl AppState {
    pub fn new(
        db: Db,
        upstream: Upstream,
        master_key: Option<String>,
        models: BTreeMap<String, String>,
    ) -> Self {
        Self {
            keys: GatewayApiKeyStore::new(db.clone()),
            usage: UsageStore::new(db),
            upstream,
            master_key,
            models,
        }
    }
}
