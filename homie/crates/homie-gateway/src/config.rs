//! Local gateway configuration: a single JSON file shared with the Swift CLI.
//!
//! The canonical file is `homie.local.json` (JSON, not TOML) so the Swift CLI
//! and this Rust binary read and write the same bytes. It is git-ignored.

use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::io;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};

use serde::Deserialize;

/// The runtime-facing configuration assembled from the shared file.
#[derive(Clone, Debug)]
pub struct GatewayConfig {
    pub listen: SocketAddr,
    pub base_url: String,
    pub api_key: String,
    pub master_key: Option<String>,
    /// Per-agent upstream model map (`codex` / `claude` → model name). Optional.
    pub models: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FileConfig {
    gateway: GatewaySection,
    upstream: UpstreamSection,
    /// Per-agent upstream model map (`codex` / `claude` → model name).
    #[serde(default)]
    models: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GatewaySection {
    #[serde(default = "default_listen")]
    listen: String,
    #[serde(default)]
    master_key: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpstreamSection {
    base_url: String,
    api_key: String,
}

fn default_listen() -> String {
    "127.0.0.1:7338".to_owned()
}

/// Directory holding `homie.local.json` and `gateway.sqlite3`.
pub fn config_dir() -> PathBuf {
    if let Some(dir) = env::var_os("HOMIE_CONFIG_DIR").filter(|s| !s.is_empty()) {
        return PathBuf::from(dir);
    }
    home_dir()
        .map(|h| h.join(".config").join("homie"))
        .unwrap_or_else(env::temp_dir)
}

pub fn config_path() -> PathBuf {
    if let Some(path) = env::var_os("HOMIE_CONFIG").filter(|s| !s.is_empty()) {
        return PathBuf::from(path);
    }
    config_dir().join("homie.local.json")
}

pub fn db_path() -> PathBuf {
    config_path()
        .parent()
        .map(|p| p.join("gateway.sqlite3"))
        .unwrap_or_else(|| config_dir().join("gateway.sqlite3"))
}

fn home_dir() -> Option<PathBuf> {
    env::var_os("HOME").map(PathBuf::from)
}

impl GatewayConfig {
    /// Load and validate the shared config. Hard-fails (no silent fallback) if
    /// the upstream base URL or API key is missing.
    pub fn load(path: &Path) -> io::Result<Self> {
        let bytes = fs::read(path)?;
        let file: FileConfig = serde_json::from_slice(&bytes).map_err(io::Error::other)?;
        Self::from_file(file)
    }

    fn from_file(file: FileConfig) -> io::Result<Self> {
        let base_url = file.upstream.base_url.trim().trim_end_matches('/');
        if base_url.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "upstream.baseUrl is empty",
            ));
        }
        if file.upstream.api_key.trim().is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "upstream.apiKey is empty",
            ));
        }
        let listen: SocketAddr = file
            .gateway
            .listen
            .parse()
            .map_err(|e| io::Error::other(format!("invalid listen address: {e}")))?;
        if !listen.ip().is_loopback() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "listen address must be loopback",
            ));
        }
        if listen.ip().is_loopback() && file.gateway.master_key.is_none() {
            eprintln!(
                "warning: no master key configured; loopback-only bind is permitted but unauthenticated"
            );
        }
        Ok(Self {
            listen,
            base_url: base_url.to_owned(),
            api_key: file.upstream.api_key,
            master_key: file.gateway.master_key,
            models: file.models,
        })
    }
}

/// Read only the gateway listen address for the `inject` preview, defaulting
/// to the loopback default when the file is absent, missing the field, or
/// unparsable. The preview must work before upstream credentials are set.
pub fn listen_or_default() -> SocketAddr {
    #[derive(Deserialize)]
    struct ListenOnly {
        #[serde(default)]
        gateway: GatewayListenOnly,
    }
    #[derive(Deserialize, Default)]
    struct GatewayListenOnly {
        #[serde(default = "default_listen")]
        listen: String,
    }
    let fallback = || default_listen().parse().expect("default listen parses");
    let Ok(bytes) = fs::read(config_path()) else {
        return fallback();
    };
    serde_json::from_slice::<ListenOnly>(&bytes)
        .ok()
        .and_then(|l| l.gateway.listen.parse().ok())
        .unwrap_or_else(fallback)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_missing_upstream_credentials() {
        let file = FileConfig {
            gateway: GatewaySection {
                listen: default_listen(),
                master_key: None,
            },
            upstream: UpstreamSection {
                base_url: "".into(),
                api_key: "".into(),
            },
            models: BTreeMap::new(),
        };
        assert!(GatewayConfig::from_file(file).is_err());
    }

    #[test]
    fn rejects_non_loopback_listen() {
        let file = FileConfig {
            gateway: GatewaySection {
                listen: "0.0.0.0:7338".into(),
                master_key: Some("m".into()),
            },
            upstream: UpstreamSection {
                base_url: "https://api.example.com/v1".into(),
                api_key: "sk-x".into(),
            },
            models: BTreeMap::new(),
        };
        assert!(GatewayConfig::from_file(file).is_err());
    }

    #[test]
    fn retains_models_map() {
        let file = FileConfig {
            gateway: GatewaySection {
                listen: default_listen(),
                master_key: Some("m".into()),
            },
            upstream: UpstreamSection {
                base_url: "https://api.example.com/v1".into(),
                api_key: "sk-x".into(),
            },
            models: BTreeMap::from([
                ("codex".to_string(), "gpt-5.2-codex".to_string()),
                ("claude".to_string(), "claude-sonnet-4-5".to_string()),
            ]),
        };
        let cfg = GatewayConfig::from_file(file).expect("valid");
        assert_eq!(cfg.models["codex"], "gpt-5.2-codex");
        assert_eq!(cfg.models["claude"], "claude-sonnet-4-5");
    }

    #[test]
    fn trims_trailing_slash_from_base_url() {
        let file = FileConfig {
            gateway: GatewaySection {
                listen: default_listen(),
                master_key: Some("m".into()),
            },
            upstream: UpstreamSection {
                base_url: "https://api.example.com/v1/".into(),
                api_key: "sk-x".into(),
            },
            models: BTreeMap::new(),
        };
        let cfg = GatewayConfig::from_file(file).expect("valid");
        assert_eq!(cfg.base_url, "https://api.example.com/v1");
    }
}
