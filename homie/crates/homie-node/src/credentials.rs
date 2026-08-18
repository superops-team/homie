//! Upstream credential resolution for the local gateway.
//!
//! Phase 1 resolves only the Codex API-key mode: the `OPENAI_API_KEY` field in
//! the profile-scoped `auth.json` that the Codex CLI writes under `CODEX_HOME`.
//! Claude OAuth and Codex ChatGPT-login token refresh are Phase 2.

use std::fs;
use std::path::Path;

use homie_proto::ProviderKind;
use serde::Deserialize;

use crate::accounts::AccountStore;
use crate::config::NodePaths;
use crate::error::{NodeError, NodeResult};

/// The resolved credential kind. Phase 1 only supports the Codex API-key mode.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CredentialKind {
    CodexApiKey,
}

/// A short-lived upstream credential resolved from a provider's local auth file.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedCredential {
    pub kind: CredentialKind,
    pub base_url: String,
    pub token: String,
}

/// Codex `auth.json` — only the API-key field is read in Phase 1.
#[derive(Deserialize)]
struct CodexAuth {
    #[serde(rename = "OPENAI_API_KEY", default)]
    api_key: Option<String>,
}

const CODEX_API_BASE_URL: &str = "https://api.openai.com/v1";

/// Resolve the upstream credential for the default Codex account (Phase 1:
/// API-key mode). Falls back to the first Codex account when no default is set.
pub fn resolve_default_codex_credential(paths: &NodePaths) -> NodeResult<ResolvedCredential> {
    let store = AccountStore::load(paths.clone(), "gateway")?;
    let profile_id = store
        .catalog()
        .defaults
        .get(&ProviderKind::Codex)
        .cloned()
        .or_else(|| {
            store
                .profiles()
                .iter()
                .find(|profile| profile.provider == ProviderKind::Codex)
                .map(|profile| profile.id.clone())
        })
        .ok_or_else(|| NodeError::NotFound("no Codex account".into()))?;
    resolve_codex_api_key(paths, &profile_id)
}

/// Resolve a specific Codex profile's API key from `accounts/codex/<id>/auth.json`.
pub fn resolve_codex_api_key(
    paths: &NodePaths,
    profile_id: &str,
) -> NodeResult<ResolvedCredential> {
    let auth_path = paths
        .accounts_root
        .join(ProviderKind::Codex.as_str())
        .join(profile_id)
        .join("auth.json");
    let token = read_codex_api_key(&auth_path)?;
    Ok(ResolvedCredential {
        kind: CredentialKind::CodexApiKey,
        base_url: CODEX_API_BASE_URL.to_owned(),
        token,
    })
}

fn read_codex_api_key(path: &Path) -> NodeResult<String> {
    let bytes = fs::read(path)
        .map_err(|_| NodeError::NotFound(format!("Codex auth file: {}", path.display())))?;
    let auth: CodexAuth = serde_json::from_slice(&bytes)
        .map_err(|_| NodeError::NotFound("Codex auth file is not valid JSON".into()))?;
    auth.api_key
        .filter(|key| !key.trim().is_empty())
        .ok_or_else(|| {
            NodeError::NotFound(
                "Codex account has no API key (ChatGPT-login mode is not supported in Phase 1)"
                    .into(),
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn write_auth(root: &std::path::Path, profile_id: &str, content: &str) {
        let dir = root.join("accounts").join("codex").join(profile_id);
        fs::create_dir_all(&dir).expect("create auth dir");
        fs::write(dir.join("auth.json"), content).expect("write auth.json");
    }

    #[test]
    fn resolves_codex_api_key() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().join("node");
        write_auth(
            &root,
            "work",
            r#"{"OPENAI_API_KEY":"sk-test-123","tokens":{}}"#,
        );
        let paths = NodePaths::for_root(root);
        let cred = resolve_codex_api_key(&paths, "work").expect("resolve");
        assert_eq!(cred.kind, CredentialKind::CodexApiKey);
        assert_eq!(cred.base_url, "https://api.openai.com/v1");
        assert_eq!(cred.token, "sk-test-123");
    }

    #[test]
    fn missing_auth_file_is_not_found() {
        let dir = tempfile::tempdir().expect("tempdir");
        let paths = NodePaths::for_root(dir.path().join("node"));
        let err = resolve_codex_api_key(&paths, "work").expect_err("missing");
        assert!(matches!(err, NodeError::NotFound(_)));
    }

    #[test]
    fn chatgpt_login_without_api_key_is_not_found() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().join("node");
        write_auth(
            &root,
            "work",
            r#"{"tokens":{"access_token":"at","refresh_token":"rt"}}"#,
        );
        let paths = NodePaths::for_root(root);
        let err = resolve_codex_api_key(&paths, "work").expect_err("no api key");
        assert!(matches!(err, NodeError::NotFound(_)));
    }

    #[test]
    fn invalid_json_is_not_found() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().join("node");
        write_auth(&root, "work", "not json");
        let paths = NodePaths::for_root(root);
        let err = resolve_codex_api_key(&paths, "work").expect_err("bad json");
        assert!(matches!(err, NodeError::NotFound(_)));
    }

    #[test]
    fn resolve_default_prefers_default_then_first_profile() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().join("node");
        write_auth(&root, "work", r#"{"OPENAI_API_KEY":"sk-work"}"#);
        write_auth(&root, "personal", r#"{"OPENAI_API_KEY":"sk-personal"}"#);
        fs::create_dir_all(&root).expect("node root");
        fs::write(
            root.join("accounts.json"),
            r#"{
              "profiles": [
                {"id":"work","provider":"codex","label":"work","tags":[],"createdAt":0,"updatedAt":0},
                {"id":"personal","provider":"codex","label":"personal","tags":[],"createdAt":0,"updatedAt":0}
              ]
            }"#,
        )
        .expect("write accounts.json");
        let paths = NodePaths::for_root(root.clone());
        let cred = resolve_default_codex_credential(&paths).expect("resolve default");
        assert_eq!(cred.token, "sk-work");

        fs::write(
            paths.accounts.clone(),
            r#"{
              "profiles": [
                {"id":"work","provider":"codex","label":"work","tags":[],"createdAt":0,"updatedAt":0},
                {"id":"personal","provider":"codex","label":"personal","tags":[],"createdAt":0,"updatedAt":0}
              ],
              "defaults": {"codex":"personal"}
            }"#,
        )
        .expect("write accounts.json with default");
        let cred = resolve_default_codex_credential(&paths).expect("resolve default");
        assert_eq!(cred.token, "sk-personal");
    }

    #[test]
    fn resolve_default_without_codex_account_is_not_found() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().join("node");
        fs::create_dir_all(&root).expect("node root");
        fs::write(root.join("accounts.json"), r#"{"profiles":[]}"#).expect("accounts");
        let paths = NodePaths::for_root(root);
        let err = resolve_default_codex_credential(&paths).expect_err("no codex");
        assert!(matches!(err, NodeError::NotFound(_)));
    }
}
