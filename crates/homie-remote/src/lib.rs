use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::io::{self, Write as _};
#[cfg(unix)]
use std::os::unix::fs::{OpenOptionsExt as _, PermissionsExt as _};
use std::path::{Path, PathBuf};
use thiserror::Error;

pub use homie_proto::HostLocateRepoResult;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct HostEntry {
    pub id: String,
    pub name: Option<String>,
    pub ssh: String,
    pub default_cwd: Option<String>,
    pub node: Option<HostNodeConfig>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct HostNodeConfig {
    pub endpoint: String,
    pub token_file: String,
    pub node_id: Option<String>,
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum HostConfigError {
    #[error("host name is required")]
    MissingName,
    #[error("ssh destination is required")]
    MissingSsh,
    #[error("ssh destination cannot contain whitespace")]
    InvalidSsh,
    #[error("node endpoint and token file must be configured together")]
    IncompleteNodeConfig,
    #[error("node endpoint must include host and port")]
    InvalidNodeEndpoint,
}

pub fn validate_host(host: &HostEntry) -> Result<(), HostConfigError> {
    if host.name.as_deref().unwrap_or_default().trim().is_empty() {
        return Err(HostConfigError::MissingName);
    }
    if host.ssh.trim().is_empty() {
        return Err(HostConfigError::MissingSsh);
    }
    if host.ssh.chars().any(char::is_whitespace) {
        return Err(HostConfigError::InvalidSsh);
    }
    if let Some(node) = &host.node {
        if node.endpoint.trim().is_empty() || node.token_file.trim().is_empty() {
            return Err(HostConfigError::IncompleteNodeConfig);
        }
        let endpoint = node
            .endpoint
            .strip_prefix("tcp://")
            .unwrap_or(&node.endpoint);
        if endpoint.chars().any(char::is_whitespace) || !endpoint.contains(':') {
            return Err(HostConfigError::InvalidNodeEndpoint);
        }
    }
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HandoffPlan {
    pub files: Vec<String>,
    pub quarantine: bool,
}

impl HandoffPlan {
    pub fn new(files: Vec<String>) -> Self {
        Self {
            files: files
                .into_iter()
                .filter(|path| !is_excluded_from_handoff(path))
                .collect(),
            quarantine: true,
        }
    }
}

pub fn is_excluded_from_handoff(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    lower == ".git"
        || lower.starts_with(".git/")
        || lower.starts_with(".env")
        || lower.contains("credential")
        || lower.contains("provider")
        || lower.contains("id_rsa")
        || lower.contains("id_ed25519")
        || lower.starts_with("target/")
        || lower.starts_with("node_modules/")
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrefsSyncSpec {
    pub name: String,
    pub local_dir: PathBuf,
    pub remote_dir: String,
    pub items: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrefsSyncToolReport {
    pub tool: String,
    pub ok: bool,
    pub synced: Vec<String>,
    pub error: Option<String>,
}

impl PrefsSyncToolReport {
    #[must_use]
    pub fn skipped(tool: impl Into<String>) -> Self {
        Self {
            tool: tool.into(),
            ok: true,
            synced: Vec::new(),
            error: None,
        }
    }
}

#[must_use]
pub fn prefs_sync_specs(home: &Path) -> Vec<PrefsSyncSpec> {
    vec![
        PrefsSyncSpec {
            name: "claude".to_string(),
            local_dir: home.join(".claude"),
            remote_dir: ".claude".to_string(),
            items: [
                "CLAUDE.md",
                "settings.json",
                "keybindings.json",
                "commands",
                "skills",
                "agents",
            ]
            .into_iter()
            .map(str::to_string)
            .collect(),
        },
        PrefsSyncSpec {
            name: "codex".to_string(),
            local_dir: home.join(".codex"),
            remote_dir: ".codex".to_string(),
            items: ["config.toml", "AGENTS.md", "prompts"]
                .into_iter()
                .map(str::to_string)
                .collect(),
        },
    ]
}

#[must_use]
pub fn present_sync_items(spec: &PrefsSyncSpec) -> Vec<String> {
    spec.items
        .iter()
        .filter(|item| spec.local_dir.join(item).exists())
        .cloned()
        .collect()
}

const SSH_OPTIONS: &[&str] = &[
    "-o",
    "BatchMode=yes",
    "-o",
    "ConnectTimeout=10",
    "-o",
    "ServerAliveInterval=15",
    "-o",
    "ServerAliveCountMax=2",
];

#[must_use]
pub fn mkdir_argv(host: &HostEntry, spec: &PrefsSyncSpec) -> Vec<String> {
    let mut argv = vec!["ssh".to_string()];
    argv.extend(SSH_OPTIONS.iter().map(|value| (*value).to_string()));
    argv.push(host.ssh.clone());
    argv.push("--".to_string());
    argv.push(format!("mkdir -p {}", shell_quote(&spec.remote_dir)));
    argv
}

#[must_use]
pub fn rsync_argv(host: &HostEntry, spec: &PrefsSyncSpec, present: &[String]) -> Vec<String> {
    let transport = std::iter::once("ssh".to_string())
        .chain(SSH_OPTIONS.iter().map(|value| (*value).to_string()))
        .collect::<Vec<_>>()
        .join(" ");
    let mut argv = vec![
        "rsync".to_string(),
        "-a".to_string(),
        "--timeout=60".to_string(),
        "-e".to_string(),
        transport,
    ];
    argv.extend(
        present
            .iter()
            .map(|item| spec.local_dir.join(item).display().to_string()),
    );
    argv.push(format!("{}:{}/", host.ssh, spec.remote_dir));
    argv
}

#[must_use]
pub fn rsync_failure_message(exit_code: i32, stderr: &str, host_display_name: &str) -> String {
    if stderr.to_ascii_lowercase().contains("command not found")
        || stderr.to_ascii_lowercase().contains("rsync: not found")
        || exit_code == 127
    {
        return format!(
            "rsync is not installed on {host_display_name} - install it there and retry"
        );
    }
    format!("rsync failed (exit {exit_code}): {}", stderr.trim())
}

#[derive(Debug, Error)]
pub enum LocateRepoError {
    #[error("I/O error while locating repo: {0}")]
    Io(#[from] io::Error),
}

pub fn locate_repo(
    cwd: Option<&Path>,
    origin_url: Option<&str>,
    candidates: &[PathBuf],
) -> Result<HostLocateRepoResult, LocateRepoError> {
    let origin_url = match origin_url.map(str::trim).filter(|value| !value.is_empty()) {
        Some(origin_url) => Some(origin_url.to_string()),
        None => match cwd {
            Some(cwd) => discover_repo_origin(cwd)?,
            None => None,
        },
    };
    let Some(origin_url) = origin_url else {
        return Ok(HostLocateRepoResult::default());
    };

    locate_repo_by_origin(&origin_url, candidates)
}

pub fn locate_repo_by_origin(
    origin_url: &str,
    candidates: &[PathBuf],
) -> Result<HostLocateRepoResult, LocateRepoError> {
    for candidate in candidates {
        if discover_repo_origin(candidate)?.as_deref() == Some(origin_url) {
            return Ok(HostLocateRepoResult {
                path: Some(candidate.display().to_string()),
                origin_url: Some(origin_url.to_string()),
            });
        }
    }
    Ok(HostLocateRepoResult {
        path: None,
        origin_url: Some(origin_url.to_string()),
    })
}

pub fn discover_repo_origin(cwd: &Path) -> Result<Option<String>, LocateRepoError> {
    let Some(git_dir) = resolve_git_dir(cwd)? else {
        return Ok(None);
    };
    let config = match fs::read_to_string(git_dir.join("config")) {
        Ok(config) => config,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
    };
    Ok(parse_origin_url(&config))
}

fn resolve_git_dir(cwd: &Path) -> Result<Option<PathBuf>, io::Error> {
    let mut cursor = Some(cwd);
    while let Some(path) = cursor {
        let dot_git = path.join(".git");
        if dot_git.is_dir() {
            return Ok(Some(dot_git));
        }
        if dot_git.is_file() {
            let gitfile = fs::read_to_string(&dot_git)?;
            if let Some(gitdir) = gitfile.trim().strip_prefix("gitdir:") {
                let gitdir = gitdir.trim();
                let resolved = if Path::new(gitdir).is_absolute() {
                    PathBuf::from(gitdir)
                } else {
                    path.join(gitdir)
                };
                return Ok(Some(resolved));
            }
        }
        cursor = path.parent();
    }
    Ok(None)
}

fn parse_origin_url(config: &str) -> Option<String> {
    let mut in_origin = false;
    for line in config.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            in_origin = trimmed == "[remote \"origin\"]" || trimmed == "[remote 'origin']";
            continue;
        }
        if in_origin {
            let Some((key, value)) = trimmed.split_once('=') else {
                continue;
            };
            if key.trim() == "url" {
                let value = value.trim();
                if !value.is_empty() {
                    return Some(value.to_string());
                }
            }
        }
    }
    None
}

fn shell_quote(value: &str) -> String {
    if value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '/' | '.' | '_' | '-'))
    {
        value.to_string()
    } else {
        format!("'{}'", value.replace('\'', "'\\''"))
    }
}

#[derive(Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteCompanionConfig {
    pub port: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bind_host: Option<String>,
    pub token: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub forward_any_port: Option<bool>,
}

impl fmt::Debug for RemoteCompanionConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RemoteCompanionConfig")
            .field("port", &self.port)
            .field("bind_host", &self.bind_host)
            .field("token", &"[redacted]")
            .field("forward_any_port", &self.forward_any_port)
            .finish()
    }
}

impl RemoteCompanionConfig {
    #[must_use]
    pub fn load(path: impl AsRef<Path>) -> Option<Self> {
        fs::read(path)
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
    }

    pub fn save(&self, path: impl AsRef<Path>) -> io::Result<()> {
        let path = path.as_ref();
        let parent = path.parent().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "remote companion config path has no parent",
            )
        })?;
        fs::create_dir_all(parent)?;
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("remote.json");
        let temporary = parent.join(format!(".{file_name}.tmp-{}", std::process::id()));
        let mut data = serde_json::to_vec_pretty(self).map_err(io::Error::other)?;
        data.push(b'\n');

        let result = (|| {
            #[cfg(unix)]
            {
                let mut file = fs::OpenOptions::new()
                    .create(true)
                    .truncate(true)
                    .write(true)
                    .mode(0o600)
                    .open(&temporary)?;
                file.write_all(&data)?;
                file.sync_all()?;
                fs::set_permissions(&temporary, fs::Permissions::from_mode(0o600))?;
            }
            #[cfg(not(unix))]
            {
                fs::write(&temporary, data)?;
            }
            fs::rename(&temporary, path)?;
            Ok::<(), io::Error>(())
        })();
        if result.is_err() {
            let _ = fs::remove_file(&temporary);
        }
        result
    }

    pub fn remove(path: impl AsRef<Path>) -> io::Result<()> {
        match fs::remove_file(path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error),
        }
    }

    #[must_use]
    pub fn endpoint_label(&self) -> String {
        let host = self.bind_host.as_deref().unwrap_or("loopback only");
        format!("{host}:{}", self.port)
    }

    #[must_use]
    pub fn pairing_url(&self) -> Option<String> {
        let host = self.bind_host.as_deref()?;
        Some(format!("homie://{host}:{}?token={}", self.port, self.token))
    }
}
