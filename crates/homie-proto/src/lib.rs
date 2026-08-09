pub mod grid;
pub mod model;
pub mod paths;
pub mod stream;
pub mod transport;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::fmt;
use uuid::Uuid;

macro_rules! id_type {
    ($name:ident) => {
        #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            #[must_use]
            pub fn new() -> Self {
                Self(Uuid::now_v7().to_string())
            }

            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl From<String> for $name {
            fn from(value: String) -> Self {
                Self(value)
            }
        }

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self(value.to_string())
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(&self.0)
            }
        }
    };
}

id_type!(ProviderId);
id_type!(LlmProfileId);
id_type!(RuntimeId);
id_type!(AgentProfileId);
id_type!(PermissionProfileId);
id_type!(SessionId);
id_type!(VirtualKeyId);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(transparent)]
pub struct RequestId(u64);

impl From<u64> for RequestId {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl RequestId {
    #[must_use]
    pub fn as_u64(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorEnvelope {
    pub code: String,
    pub message: String,
    pub retryable: bool,
    #[serde(default, skip_serializing_if = "Map::is_empty")]
    pub details: Map<String, Value>,
}

impl ErrorEnvelope {
    #[must_use]
    pub fn new(code: impl Into<String>, message: impl Into<String>, retryable: bool) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            retryable,
            details: Map::new(),
        }
    }

    #[must_use]
    pub fn with_detail(mut self, key: impl Into<String>, value: impl Serialize) -> Self {
        let value = serde_json::to_value(value).unwrap_or(Value::Null);
        self.details.insert(key.into(), value);
        self
    }
}

pub struct Method;

impl Method {
    pub const HELLO: &'static str = "hello";
    pub const STATE_SNAPSHOT: &'static str = "state.snapshot";
    pub const DAEMON_PREPARE_SHUTDOWN: &'static str = "daemon.prepare_shutdown";
    pub const DAEMON_SHUTDOWN: &'static str = "daemon.shutdown";
    pub const CLIENT_SET_ACTIVE: &'static str = "client.set_active";
    pub const GOVERNOR_CONFIGURE: &'static str = "governor.configure";
    pub const AGENT_READINESS: &'static str = "agent.readiness";
    pub const SESSION_SPAWN: &'static str = "session.spawn";
    pub const SESSION_LIST: &'static str = "session.list";
    pub const SESSION_SNAPSHOT: &'static str = "session.snapshot";
    pub const SESSION_STATUS: &'static str = "session.status";
    pub const SESSION_ARTIFACTS: &'static str = "session.artifacts";
    pub const SESSION_PORTS: &'static str = "session.ports";
    pub const SESSION_SET_PARENT: &'static str = "session.set_parent";
    pub const SESSION_LIST_CHILDREN: &'static str = "session.list_children";
    pub const SESSION_PARENT: &'static str = "session.parent";
    pub const SESSION_KILL: &'static str = "session.kill";
    pub const SESSION_REMOVE: &'static str = "session.remove";
    pub const SESSION_RENAME: &'static str = "session.rename";
    pub const SESSION_RESUME: &'static str = "session.resume";
    pub const SESSION_ATTACH: &'static str = "session.attach";
    pub const SESSION_SEND_TEXT: &'static str = "session.send_text";
    pub const SESSION_RESIZE: &'static str = "session.resize";
    pub const SESSION_SET_OWNER: &'static str = "session.set_owner";
    pub const SESSION_READ_SCREEN: &'static str = "session.read_screen";
    pub const SESSION_READ_SCROLLBACK: &'static str = "session.read_scrollback";
    pub const SESSION_READ_SCROLLBACK_CELLS: &'static str = "session.read_scrollback_cells";
    pub const SESSION_READ_DIFF: &'static str = "session.read_diff";
    pub const SESSION_MARK_SEEN: &'static str = "session.mark_seen";
    pub const SESSION_HIBERNATE: &'static str = "session.hibernate";
    pub const SESSION_WAKE: &'static str = "session.wake";
    pub const SESSION_ARCHIVE: &'static str = "session.archive";
    pub const SESSION_UNARCHIVE: &'static str = "session.unarchive";
    pub const SESSION_REOPEN_LAST: &'static str = "session.reopen_last";
    pub const SESSION_HISTORY: &'static str = "session.history";
    pub const SESSION_RESUME_FROM_HISTORY: &'static str = "session.resume_from_history";
    pub const WORKTREE_CREATE: &'static str = "worktree.create";
    pub const WORKTREE_LIST: &'static str = "worktree.list";
    pub const WORKTREE_REMOVE: &'static str = "worktree.remove";
    pub const WORKTREE_OVERVIEW: &'static str = "worktree.overview";
    pub const PROJECT_ADD: &'static str = "project.add";
    pub const HOST_SYNC_PREFS: &'static str = "host.sync_prefs";
    pub const HOST_LOCATE_REPO: &'static str = "host.locate_repo";
    pub const EVENTS_SUBSCRIBE: &'static str = "events.subscribe";
    pub const EVENTS_WAIT: &'static str = "events.wait";
    pub const HOOK_REPORT: &'static str = "hook.report";
    pub const TEST_RUN: &'static str = "test.run";
    pub const BROWSER_ACT: &'static str = "browser.act";
    pub const LLM_VIRTUAL_KEY_ISSUE: &'static str = "llm.virtual_key.issue";
    pub const LLM_VIRTUAL_KEY_REVOKE: &'static str = "llm.virtual_key.revoke";
    pub const LLM_PROXY_STATUS: &'static str = "llm.proxy.status";
    pub const AGENT_PROFILE_CREATE: &'static str = "agent.profile.create";
    pub const AGENT_PROFILE_UPDATE: &'static str = "agent.profile.update";
    pub const AGENT_PROFILE_LIST: &'static str = "agent.profile.list";
    pub const AGENT_PROFILE_SET_DEFAULT: &'static str = "agent.profile.set_default";
    pub const SKILLS_LIST: &'static str = "skills.list";
    pub const MCP_SERVER_LIST: &'static str = "mcp.server.list";
    pub const PERMISSION_PROFILE_LIST: &'static str = "permission.profile.list";
    pub const CONTEXT_SESSION_SUMMARY: &'static str = "context.session.summary";
    pub const TASK_LIST: &'static str = "task.list";
    pub const TASK_CREATE: &'static str = "task.create";
    pub const TASK_UPDATE: &'static str = "task.update";
    pub const MEMORY_SEARCH: &'static str = "memory.search";
    pub const MEMORY_WRITE_CANDIDATE: &'static str = "memory.write_candidate";

    pub const ALL: &'static [&'static str] = &[
        Self::HELLO,
        Self::STATE_SNAPSHOT,
        Self::DAEMON_PREPARE_SHUTDOWN,
        Self::DAEMON_SHUTDOWN,
        Self::CLIENT_SET_ACTIVE,
        Self::GOVERNOR_CONFIGURE,
        Self::AGENT_READINESS,
        Self::SESSION_SPAWN,
        Self::SESSION_LIST,
        Self::SESSION_SNAPSHOT,
        Self::SESSION_STATUS,
        Self::SESSION_ARTIFACTS,
        Self::SESSION_PORTS,
        Self::SESSION_SET_PARENT,
        Self::SESSION_LIST_CHILDREN,
        Self::SESSION_PARENT,
        Self::SESSION_KILL,
        Self::SESSION_REMOVE,
        Self::SESSION_RENAME,
        Self::SESSION_RESUME,
        Self::SESSION_ATTACH,
        Self::SESSION_SEND_TEXT,
        Self::SESSION_RESIZE,
        Self::SESSION_SET_OWNER,
        Self::SESSION_READ_SCREEN,
        Self::SESSION_READ_SCROLLBACK,
        Self::SESSION_READ_SCROLLBACK_CELLS,
        Self::SESSION_READ_DIFF,
        Self::SESSION_MARK_SEEN,
        Self::SESSION_HIBERNATE,
        Self::SESSION_WAKE,
        Self::SESSION_ARCHIVE,
        Self::SESSION_UNARCHIVE,
        Self::SESSION_REOPEN_LAST,
        Self::SESSION_HISTORY,
        Self::SESSION_RESUME_FROM_HISTORY,
        Self::WORKTREE_CREATE,
        Self::WORKTREE_LIST,
        Self::WORKTREE_REMOVE,
        Self::WORKTREE_OVERVIEW,
        Self::PROJECT_ADD,
        Self::HOST_SYNC_PREFS,
        Self::HOST_LOCATE_REPO,
        Self::EVENTS_SUBSCRIBE,
        Self::EVENTS_WAIT,
        Self::HOOK_REPORT,
        Self::TEST_RUN,
        Self::BROWSER_ACT,
        Self::LLM_VIRTUAL_KEY_ISSUE,
        Self::LLM_VIRTUAL_KEY_REVOKE,
        Self::LLM_PROXY_STATUS,
        Self::AGENT_PROFILE_CREATE,
        Self::AGENT_PROFILE_UPDATE,
        Self::AGENT_PROFILE_LIST,
        Self::AGENT_PROFILE_SET_DEFAULT,
        Self::SKILLS_LIST,
        Self::MCP_SERVER_LIST,
        Self::PERMISSION_PROFILE_LIST,
        Self::CONTEXT_SESSION_SUMMARY,
        Self::TASK_LIST,
        Self::TASK_CREATE,
        Self::TASK_UPDATE,
        Self::MEMORY_SEARCH,
        Self::MEMORY_WRITE_CANDIDATE,
    ];
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionSpawnRequest {
    pub cwd: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_session_id: Option<SessionId>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionAttachRequest {
    pub session_id: SessionId,
    #[serde(default)]
    pub output_offset: u64,
    #[serde(default = "default_max_output_bytes")]
    pub max_bytes: usize,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionSendTextRequest {
    pub session_id: SessionId,
    pub text: String,
    pub submit: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionResizeRequest {
    pub session_id: SessionId,
    pub cols: u16,
    pub rows: u16,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionKillRequest {
    pub session_id: SessionId,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionHistoryRequest {
    pub claude_root: String,
    pub codex_root: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tracked: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionResumeFromHistoryRequest {
    pub agent_kind: String,
    pub external_id: String,
    pub cwd: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HostLocateRepoParams {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,
    #[serde(rename = "originURL", default, skip_serializing_if = "Option::is_none")]
    pub origin_url: Option<String>,
    #[serde(rename = "sessionID", default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<SessionId>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct HostLocateRepoResult {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(rename = "originURL", default, skip_serializing_if = "Option::is_none")]
    pub origin_url: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct HostSyncPrefsParams {
    pub host: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PrefsSyncToolReport {
    pub tool: String,
    pub ok: bool,
    #[serde(default)]
    pub synced: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct HostSyncPrefsResult {
    pub tools: Vec<PrefsSyncToolReport>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HostNodeConfig {
    pub endpoint: String,
    pub token_file: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HostEntry {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub ssh: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_cwd: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node: Option<HostNodeConfig>,
}

impl HostEntry {
    #[must_use]
    pub fn display_name(&self) -> &str {
        self.name.as_deref().unwrap_or(&self.id)
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct HostsConfig {
    #[serde(default)]
    pub hosts: Vec<HostEntry>,
}

impl HostsConfig {
    #[must_use]
    pub fn host(&self, id: &str) -> Option<&HostEntry> {
        self.hosts.iter().find(|host| host.id == id)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteConfig {
    pub port: u16,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bind_host: Option<String>,
    pub token: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub forward_any_port: Option<bool>,
}

pub const NODE_PROTOCOL_VERSION: u32 = 1;

pub struct NodeMethod;

impl NodeMethod {
    pub const HELLO: &'static str = "node.hello";
    pub const STATUS: &'static str = "node.status";
    pub const USAGE_RECORD: &'static str = "usage.record";
    pub const USAGE_QUERY: &'static str = "usage.query";
    pub const USAGE_REFRESH: &'static str = "usage.refresh";
}

pub struct NodeCapability;

impl NodeCapability {
    pub const ACCOUNTS: &'static str = "accounts.v1";
    pub const CODEX_APP_SERVER: &'static str = "codex.app-server.v1";
    pub const CLAUDE_SUPERVISOR: &'static str = "claude.supervisor.v1";
    pub const FLEET_USAGE: &'static str = "usage-ledger.v1";
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ProviderKind {
    Claude,
    Codex,
}

impl ProviderKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Claude => "claude",
            Self::Codex => "codex",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeHelloParams {
    pub proto: u32,
    pub build: String,
    pub token: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_node_id: Option<String>,
}

impl NodeHelloParams {
    #[must_use]
    pub fn new(build: impl Into<String>, token: impl Into<String>) -> Self {
        Self {
            proto: NODE_PROTOCOL_VERSION,
            build: build.into(),
            token: token.into(),
            client_node_id: None,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeHelloResult {
    pub proto: u32,
    pub control_proto: u32,
    pub build: String,
    pub node_id: String,
    pub display_name: String,
    pub os: String,
    pub arch: String,
    pub capabilities: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeStatusResult {
    pub node: NodeHelloResult,
    pub started_at: i64,
    pub accounts: usize,
    pub active_logins: usize,
    pub pending_moves: usize,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountProfile {
    pub id: String,
    pub provider: ProviderKind,
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub organization: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum InstallationStatus {
    Missing,
    SignedOut,
    Authenticating,
    Ready,
    Error,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountInstallation {
    pub profile_id: String,
    pub provider: ProviderKind,
    pub node_id: String,
    pub status: InstallationStatus,
    pub config_home: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub identity: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plan: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checked_at: Option<i64>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountCatalogResult {
    pub profiles: Vec<AccountProfile>,
    pub installations: Vec<AccountInstallation>,
    #[serde(default)]
    pub defaults: std::collections::BTreeMap<ProviderKind, String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountUpsertParams {
    pub id: String,
    pub provider: ProviderKind,
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub organization: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountProfileParams {
    pub profile_id: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountSetDefaultParams {
    pub provider: ProviderKind,
    pub profile_id: String,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum LoginMode {
    #[default]
    DeviceCode,
    Browser,
    Interactive,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountLoginStartParams {
    pub profile_id: String,
    #[serde(default)]
    pub mode: LoginMode,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginChallenge {
    pub login_id: String,
    pub profile_id: String,
    pub kind: LoginMode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verification_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_code: Option<String>,
    #[serde(default)]
    pub output: String,
    pub complete: bool,
    pub success: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginSessionParams {
    pub login_id: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginInputParams {
    pub login_id: String,
    pub text: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderCallParams {
    pub profile_id: String,
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderCallResult {
    pub provider: ProviderKind,
    pub method: String,
    pub result: Value,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum UsageValueKind {
    SubscriptionQuota,
    EstimatedApiEquivalent,
    AuthoritativeBilled,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum UsageSource {
    AppServer,
    Otel,
    Transcript,
    Manual,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageEvent {
    pub id: String,
    pub occurred_at: i64,
    pub provider: ProviderKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default)]
    pub input_tokens: i64,
    #[serde(default)]
    pub output_tokens: i64,
    #[serde(default)]
    pub cache_read_tokens: i64,
    #[serde(default)]
    pub cache_write_tokens: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub estimated_usd: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub billed_usd: Option<f64>,
    pub value_kind: UsageValueKind,
    pub source: UsageSource,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageQueryParams {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<ProviderKind>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageTotals {
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_read_tokens: i64,
    pub cache_write_tokens: i64,
    pub estimated_usd: f64,
    pub billed_usd: f64,
    pub events: i64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageQueryResult {
    pub node_id: String,
    pub totals: UsageTotals,
    pub by_provider: std::collections::BTreeMap<ProviderKind, UsageTotals>,
    pub authoritative_billing_available: bool,
    pub updated_at: i64,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TransferMode {
    Move,
    Fork,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckpointPrepareParams {
    pub session_id: String,
    pub provider: ProviderKind,
    pub profile_id: String,
    pub workspace_root: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_session_id: Option<String>,
    pub mode: TransferMode,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckpointFile {
    pub path: String,
    pub digest: String,
    pub size: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unix_mode: Option<u32>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckpointManifest {
    pub version: u32,
    pub checkpoint_id: String,
    pub source_node_id: String,
    pub session_id: String,
    pub provider: ProviderKind,
    pub profile_id: String,
    pub workspace_root: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_session_id: Option<String>,
    pub mode: TransferMode,
    pub created_at: i64,
    pub files: Vec<CheckpointFile>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_state: Option<CheckpointFile>,
    #[serde(default)]
    pub excluded: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CheckpointManifestParams {
    pub manifest: CheckpointManifest,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckpointIdParams {
    pub checkpoint_id: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct BlobHasParams {
    pub digests: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct BlobHasResult {
    pub missing: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BlobReadParams {
    pub digest: String,
    #[serde(default)]
    pub offset: u64,
    #[serde(default)]
    pub max_bytes: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BlobChunk {
    pub digest: String,
    pub offset: u64,
    pub hex: String,
    pub eof: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BlobPutParams {
    pub digest: String,
    pub offset: u64,
    pub hex: String,
    pub eof: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckpointStageResult {
    pub checkpoint_id: String,
    pub quarantine_path: String,
    pub ready: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum MovePhase {
    Prepared,
    Transferring,
    Staged,
    TargetReady,
    Committed,
    Aborted,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveCommitParams {
    pub checkpoint_id: String,
    pub target_node_id: String,
    pub lease_id: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveAbortParams {
    pub checkpoint_id: String,
    pub reason: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveRecord {
    pub checkpoint_id: String,
    pub session_id: String,
    pub source_node_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_node_id: Option<String>,
    pub phase: MovePhase,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lease_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    pub updated_at: i64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorktreeInfo {
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    pub is_bare: bool,
    pub is_detached: bool,
    pub is_prunable: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorktreeCreateRequest {
    pub repo_path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorktreeListRequest {
    pub repo_path: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorktreeRemoveRequest {
    pub repo_path: String,
    pub worktree_path: String,
    #[serde(default)]
    pub force: bool,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SessionDiffBase {
    #[default]
    DefaultBranch,
    Head,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionReadDiffRequest {
    #[serde(rename = "sessionID")]
    pub session_id: SessionId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base: Option<SessionDiffBase>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionReadDiffResult {
    #[serde(with = "base64_bytes")]
    pub patch: Vec<u8>,
    pub repo_root: String,
    pub truncated: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_ref: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EventsSubscribeRequest {
    #[serde(default)]
    pub after_seq: u64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub event_filter: Vec<String>,
}

mod base64_bytes {
    use base64::Engine as _;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(value: &[u8], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&base64::engine::general_purpose::STANDARD.encode(value))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let encoded = String::deserialize(deserializer)?;
        base64::engine::general_purpose::STANDARD
            .decode(encoded)
            .map_err(serde::de::Error::custom)
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EventsWaitRequest {
    #[serde(default)]
    pub after_seq: u64,
    #[serde(default = "default_wait_timeout_ms")]
    pub timeout_ms: u64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub event_filter: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EventCursor {
    pub next_seq: u64,
}

fn default_max_output_bytes() -> usize {
    8192
}

fn default_wait_timeout_ms() -> u64 {
    30_000
}

pub struct EventName;

impl EventName {
    pub const RUNTIME_READY: &'static str = "runtime.ready";
    pub const RUNTIME_UNHEALTHY: &'static str = "runtime.unhealthy";
    pub const SESSION_CREATED: &'static str = "session.created";
    pub const SESSION_SPAWNED: &'static str = "session.spawned";
    pub const SESSION_UPDATED: &'static str = "session.updated";
    pub const SESSION_RESOURCES: &'static str = "session.resources";
    pub const SESSION_STATUS: &'static str = "session.status";
    pub const SESSION_NEEDS_INPUT: &'static str = "session.needs_input";
    pub const SESSION_OUTPUT: &'static str = "session.output";
    pub const SESSION_ARTIFACT: &'static str = "session.artifact";
    pub const SESSION_ARCHIVED: &'static str = "session.archived";
    pub const SESSION_REMOVED: &'static str = "session.removed";
    pub const PROJECT_UPDATED: &'static str = "project.updated";
    pub const WORKTREE_CREATED: &'static str = "worktree.created";
    pub const WORKTREE_REMOVED: &'static str = "worktree.removed";
    pub const LLM_REQUEST_STARTED: &'static str = "llm.request.started";
    pub const LLM_REQUEST_COMPLETED: &'static str = "llm.request.completed";
    pub const LLM_REQUEST_FAILED: &'static str = "llm.request.failed";
    pub const TOOL_CALL_STARTED: &'static str = "tool.call.started";
    pub const TOOL_CALL_COMPLETED: &'static str = "tool.call.completed";
    pub const TOOL_CALL_FAILED: &'static str = "tool.call.failed";
    pub const METRICS_WRITE_FAILED: &'static str = "metrics.write_failed";
    pub const CONTEXT_UPDATED: &'static str = "context.updated";
    pub const EVENTS_DROPPED: &'static str = "events.dropped";

    pub const ALL: &'static [&'static str] = &[
        Self::RUNTIME_READY,
        Self::RUNTIME_UNHEALTHY,
        Self::SESSION_CREATED,
        Self::SESSION_SPAWNED,
        Self::SESSION_UPDATED,
        Self::SESSION_RESOURCES,
        Self::SESSION_STATUS,
        Self::SESSION_NEEDS_INPUT,
        Self::SESSION_OUTPUT,
        Self::SESSION_ARTIFACT,
        Self::SESSION_ARCHIVED,
        Self::SESSION_REMOVED,
        Self::PROJECT_UPDATED,
        Self::WORKTREE_CREATED,
        Self::WORKTREE_REMOVED,
        Self::LLM_REQUEST_STARTED,
        Self::LLM_REQUEST_COMPLETED,
        Self::LLM_REQUEST_FAILED,
        Self::TOOL_CALL_STARTED,
        Self::TOOL_CALL_COMPLETED,
        Self::TOOL_CALL_FAILED,
        Self::METRICS_WRITE_FAILED,
        Self::CONTEXT_UPDATED,
        Self::EVENTS_DROPPED,
    ];
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SessionStatus {
    Created,
    Starting,
    Running,
    NeedsInput,
    Idle,
    Hibernated,
    Archived,
    Exited,
    Unknown(String),
}

impl Serialize for SessionStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(match self {
            Self::Created => "created",
            Self::Starting => "starting",
            Self::Running => "running",
            Self::NeedsInput => "needs_input",
            Self::Idle => "idle",
            Self::Hibernated => "hibernated",
            Self::Archived => "archived",
            Self::Exited => "exited",
            Self::Unknown(value) => value,
        })
    }
}

impl<'de> Deserialize<'de> for SessionStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Ok(match value.as_str() {
            "created" => Self::Created,
            "starting" => Self::Starting,
            "running" => Self::Running,
            "needs_input" => Self::NeedsInput,
            "idle" => Self::Idle,
            "hibernated" => Self::Hibernated,
            "archived" => Self::Archived,
            "exited" => Self::Exited,
            _ => Self::Unknown(value),
        })
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ControlMessage {
    Request {
        #[serde(rename = "requestId")]
        request_id: RequestId,
        method: String,
        #[serde(default)]
        params: Value,
    },
    Response {
        #[serde(rename = "requestId")]
        request_id: RequestId,
        ok: bool,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        result: Option<Value>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        error: Option<ErrorEnvelope>,
    },
    Event {
        event: String,
        seq: u64,
        #[serde(default)]
        params: Value,
    },
}

impl ControlMessage {
    #[must_use]
    pub fn request(request_id: RequestId, method: impl Into<String>, params: Value) -> Self {
        Self::Request {
            request_id,
            method: method.into(),
            params,
        }
    }

    #[must_use]
    pub fn success(request_id: RequestId, result: Value) -> Self {
        Self::Response {
            request_id,
            ok: true,
            result: Some(result),
            error: None,
        }
    }

    #[must_use]
    pub fn failure(request_id: RequestId, error: ErrorEnvelope) -> Self {
        Self::Response {
            request_id,
            ok: false,
            result: None,
            error: Some(error),
        }
    }

    #[must_use]
    pub fn event(event: impl Into<String>, seq: u64, params: Value) -> Self {
        Self::Event {
            event: event.into(),
            seq,
            params,
        }
    }
}

// ---------------------------------------------------------------------------
// Detection types — risk classification and needs-input detail
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskHint {
    Neutral,
    FileWrite,
    Network,
    Destructive,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NeedsInputKind {
    Approval,
    Question,
    Error,
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NeedsInputSource {
    Hook,
    ScreenScrape,
    User,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NeedsInputDetail {
    pub kind: NeedsInputKind,
    pub source: NeedsInputSource,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,
    pub summary: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt_excerpt: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<String>>,
    pub risk_hint: RiskHint,
    #[serde(default)]
    pub occurred_at: i64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExitReason {
    Normal,
    Signal,
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExitInfo {
    pub code: Option<i32>,
    pub signal: Option<i32>,
    pub reason: ExitReason,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id_round_trips_as_text() {
        let id = ProviderId::from("provider_1");
        let json = serde_json::to_string(&id).expect("serialize");
        assert_eq!(json, "\"provider_1\"");
        let decoded: ProviderId = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded.as_str(), "provider_1");
    }
}
