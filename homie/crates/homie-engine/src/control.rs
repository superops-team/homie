//! The control channel: newline-delimited JSON over a Unix socket.
//!
//! This is the daemon's front door — what the app, the CLI and the MCP shim all
//! talk to. The wire format is not ours to choose: `homie-client` already speaks
//! it to the reference implementation, so a Rust engine has to be indistinguishable on the
//! socket or every existing client breaks.
//!
//! What is implemented here is the core of that surface — handshake, list,
//! spawn, input, resize, read, kill. The rest of the method table (worktrees,
//! history, migration, hosts) is not yet ported; unknown methods return a
//! `not_found` control error, which is what an older daemon does for a method
//! it does not know, rather than dropping the connection.

use std::io::{BufRead, BufReader, Read};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicUsize;
use std::sync::{Arc, Mutex};

use homie_proto::control::MAX_CONTROL_LINE_BYTES;
use homie_proto::{ControlError, ControlMessage, JsonValue, Method};
use serde_json::json;
use sha2::{Digest, Sha256};

use crate::registry::Registry;
mod codec;
mod handlers;
mod inject;
mod runtime;
mod wire;
pub(crate) use handlers::new_record;
use runtime::{ActiveConnectionGuard, SubscriptionHandle};
use wire::{decode, encode, poisoned, write_message};

/// Identifies this engine in the handshake, so a client can tell which
/// implementation it reached.
pub const BUILD: &str = concat!("homie-engine-", env!("CARGO_PKG_VERSION"));

pub struct ControlServer {
    registry: Arc<Mutex<Registry>>,
    socket_path: PathBuf,
    logs_dir: PathBuf,
    holder: Option<crate::session::HolderConfig>,
    remote: Option<Arc<crate::remote::manager::RemoteManager>>,
    remote_bindings: Option<crate::remote::binding::RemoteBindingStore>,
    events: crate::events::EventBus,
    attach: crate::attach::AttachHub,
    pr_monitor_wake: crate::pr_monitor::PrMonitorWake,
    injection: Option<InjectionConfig>,
    governor: std::sync::Arc<Mutex<crate::governor::GovernorConfig>>,
    browser: std::sync::OnceLock<crate::browser::BrowserPool>,
    active_connections: Arc<AtomicUsize>,
}

/// Where injection files live and which CLI they point at. Present, spawns
/// become hook-driven and get the homie MCP tools.
#[derive(Clone, Debug)]
pub struct InjectionConfig {
    pub inject_dir: PathBuf,
    pub cli_path: PathBuf,
    /// When set, agents that opt into `codexGateway` route their LLM traffic
    /// through the local gateway; the daemon mints a per-session virtual key
    /// at spawn via this issuer.
    pub gateway: Option<crate::inject::GatewayIssuer>,
}

impl ControlServer {
    pub fn new(registry: Arc<Mutex<Registry>>, socket_path: impl Into<PathBuf>) -> Self {
        // Capture the bytes this process actually started from before an app
        // updater can replace the bundle path underneath the live daemon.
        let _ = process_executable_hash();
        let socket_path = socket_path.into();
        let logs_dir = socket_path
            .parent()
            .map(|parent| parent.join("logs"))
            .unwrap_or_else(|| PathBuf::from("logs"));
        let remote_bindings = socket_path.parent().and_then(|parent| {
            crate::remote::binding::RemoteBindingStore::new(parent.join("remote-bindings")).ok()
        });
        Self {
            registry,
            socket_path,
            logs_dir,
            holder: None,
            remote: None,
            remote_bindings,
            events: crate::events::EventBus::new(),
            attach: crate::attach::AttachHub::new(),
            pr_monitor_wake: crate::pr_monitor::PrMonitorWake::default(),
            injection: None,
            governor: std::sync::Arc::new(Mutex::new(crate::governor::GovernorConfig::default())),
            browser: std::sync::OnceLock::new(),
            active_connections: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Enables spawn-time hook/MCP injection: writes the shim files (like the
    /// reference implementation does at startup) and applies each manifest's mechanisms
    /// to future spawns.
    pub fn with_injection(mut self, config: InjectionConfig) -> Self {
        let _ = crate::inject::write_claude_hooks_file(&config.inject_dir);
        let _ = crate::inject::write_claude_mcp_file(&config.inject_dir, &config.cli_path);
        self.injection = Some(config);
        self
    }

    /// The bus this server publishes to — the daemon shares it with the
    /// registry watcher (see [`crate::events::spawn_registry_watcher`]).
    pub fn events(&self) -> crate::events::EventBus {
        self.events.clone()
    }

    /// The attach hub, for the resource governor's attached-session checks.
    pub fn attach_hub(&self) -> crate::attach::AttachHub {
        self.attach.clone()
    }

    /// Event-driven invalidation shared by selection/focus, artifact
    /// discovery, and the background PR monitor.
    pub fn pr_monitor_wake(&self) -> crate::pr_monitor::PrMonitorWake {
        self.pr_monitor_wake.clone()
    }

    /// The governor tunables `governor.configure` updates in place.
    pub fn governor_config(&self) -> std::sync::Arc<Mutex<crate::governor::GovernorConfig>> {
        std::sync::Arc::clone(&self.governor)
    }

    /// Where session output logs are written. Defaults to `logs/` beside the
    /// socket, matching the reference implementation's layout.
    pub fn with_logs_dir(mut self, logs_dir: impl Into<PathBuf>) -> Self {
        self.logs_dir = logs_dir.into();
        self
    }

    /// Spawn sessions through holders, so they survive this process. This is
    /// how the daemon runs; tests and embedded callers may stay direct.
    pub fn with_holder(mut self, holder: crate::session::HolderConfig) -> Self {
        self.holder = Some(holder);
        self
    }

    /// Enables the SSH-bootstrapped remote Holder transport. The local app
    /// still talks only to this Engine; it never executes SSH itself.
    pub fn with_remote(mut self, manager: Arc<crate::remote::manager::RemoteManager>) -> Self {
        self.remote = Some(manager);
        self
    }

    /// Serves one connection to completion.
    ///
    /// The FIRST line decides what this connection is: an [`AttachRequest`]
    /// makes it a binary session data channel, anything else is control
    /// NDJSON — the same sniff the Swift `ConnectionHub` does, so one socket
    /// path serves both.
    ///
    /// The write half is shared: after `events.subscribe`, a forwarder thread
    /// pushes event frames onto the same socket while this loop keeps
    /// answering requests — one connection carries both, as the reference implementation's
    /// does.
    pub fn serve(&self, stream: UnixStream) -> std::io::Result<()> {
        let _connection = ActiveConnectionGuard::new(Arc::clone(&self.active_connections));
        let mut reader = BufReader::new(stream.try_clone()?);
        let writer = Arc::new(Mutex::new(stream));
        let mut subscription: Option<SubscriptionHandle> = None;

        let mut first = true;
        loop {
            let mut line = Vec::new();
            let read = reader.read_until(b'\n', &mut line)?;
            if read == 0 {
                return Ok(());
            }
            if line.last() == Some(&b'\n') {
                line.pop();
            }
            if line.is_empty() {
                continue;
            }
            if first {
                first = false;
                if let Ok(attach) = serde_json::from_slice::<homie_proto::AttachRequest>(&line) {
                    // Attaching means this session is visible. Reconcile the
                    // actual process first: an adopted holder can be stopped
                    // even when stale persisted metadata says it is awake.
                    // This cold-boundary SIGCONT is harmless for a running
                    // tree and keeps process-tree work off the keystroke path.
                    // Recording visibility before waking the PR monitor keeps
                    // its immediate pass seeing a foreground/recent session
                    // even if registration has not completed yet.
                    if let Ok(mut registry) = self.registry.lock() {
                        let _ = registry.ensure_session_awake(&attach.attach.0);
                        let _ = registry.mark_seen(&attach.attach.0);
                        let _ = registry.persist();
                        self.publish_updated(&registry, &attach.attach.0);
                    }
                    self.pr_monitor_wake.wake_session(attach.attach.0.clone());
                    // Bytes the line reader buffered past the attach line are
                    // already binary frames; hand them over.
                    let buffered = reader.buffer().to_vec();
                    self.attach.serve(
                        &self.registry,
                        &attach.attach.0,
                        reader.into_inner(),
                        buffered,
                        writer,
                    );
                    return Ok(());
                }
            }
            if line.len() > MAX_CONTROL_LINE_BYTES {
                // A client that sends an oversized frame is out of contract;
                // answering would mean buffering unbounded input.
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "control line exceeded the protocol maximum",
                ));
            }
            let Some(response) = self.handle_line(&line, &writer, &mut subscription) else {
                continue;
            };
            write_message(&writer, &response)?;
        }
    }

    fn handle_line(
        &self,
        line: &[u8],
        writer: &Arc<Mutex<UnixStream>>,
        subscription: &mut Option<SubscriptionHandle>,
    ) -> Option<ControlMessage> {
        let message: ControlMessage = match serde_json::from_slice(line) {
            Ok(message) => message,
            Err(error) => {
                // Malformed input gets an error with id 0 rather than silence:
                // a client waiting on a reply should learn it will not come.
                return Some(ControlMessage::Response {
                    id: 0,
                    result: Err(ControlError::bad_request(format!(
                        "could not parse control message: {error}"
                    ))),
                });
            }
        };

        match message {
            ControlMessage::Request { id, method, params }
                if method == Method::EVENTS_SUBSCRIBE =>
            {
                Some(ControlMessage::Response {
                    id,
                    result: self.events_subscribe(params, writer, subscription),
                })
            }
            ControlMessage::Request { id, method, params } => Some(ControlMessage::Response {
                id,
                result: self.dispatch(&method, params),
            }),
            // Responses and events are the daemon's to send, not receive.
            ControlMessage::Response { .. } | ControlMessage::Event { .. } => None,
        }
    }

    /// Turns this connection into an event sink: a forwarder thread streams
    /// matching events as they publish, replaying from `sinceSeq` first.
    /// Re-subscribing replaces the previous subscription, as in Swift.
    fn events_subscribe(
        &self,
        params: Option<JsonValue>,
        writer: &Arc<Mutex<UnixStream>>,
        subscription: &mut Option<SubscriptionHandle>,
    ) -> Result<JsonValue, ControlError> {
        let p: homie_proto::EventsSubscribeParams = decode(params).unwrap_or_default();
        if let Some(previous) = subscription.take() {
            previous
                .stop
                .store(true, std::sync::atomic::Ordering::SeqCst);
        }
        let stream = self.events.subscribe(
            p.since_seq,
            crate::events::Filter::new(
                p.sessions
                    .map(|sessions| sessions.into_iter().map(|id| id.0).collect()),
                p.kinds,
            ),
        );
        let stop = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let handle = {
            let stop = Arc::clone(&stop);
            let writer = Arc::clone(writer);
            std::thread::Builder::new()
                .name("homie-control-events".into())
                .spawn(move || {
                    while !stop.load(std::sync::atomic::Ordering::SeqCst) {
                        let Some(event) = stream.recv(std::time::Duration::from_millis(250)) else {
                            continue;
                        };
                        let frame = ControlMessage::Event {
                            name: event.name,
                            seq: event.seq,
                            params: event.params,
                        };
                        if write_message(&writer, &frame).is_err() {
                            break; // peer is gone; dropping the stream unsubscribes
                        }
                    }
                })
                .map_err(|error| ControlError::internal(error.to_string()))?
        };
        *subscription = Some(SubscriptionHandle::new(stop, handle));
        Ok(json!({ "subscribed": true }))
    }

    /// One-shot long poll for a session reaching one of the `until` statuses.
    fn events_wait(&self, params: Option<JsonValue>) -> Result<JsonValue, ControlError> {
        let p: homie_proto::EventsWaitParams = decode(params)?;
        if p.until.is_empty() {
            return Err(ControlError::bad_request(
                "events.wait needs `until` statuses",
            ));
        }
        let deadline = std::time::Instant::now()
            + std::time::Duration::from_millis(p.timeout_ms.clamp(0, 600_000) as u64);

        // Subscribe before the pre-check, so a transition landing between the
        // two is buffered rather than lost.
        let stream = self.events.subscribe(
            None,
            crate::events::Filter::new(
                Some(vec![p.session_id.0.clone()]),
                Some(vec![homie_proto::EventName::SESSION_UPDATED.to_string()]),
            ),
        );

        let current = |registry: &Registry| -> Option<homie_proto::SessionRecord> {
            registry
                .records()
                .into_iter()
                .find(|record| record.id.0 == p.session_id.0)
        };
        let matches = |record: &homie_proto::SessionRecord| {
            p.until
                .iter()
                .any(|target| crate::events::satisfies_wait_target(&record.status, target))
        };

        let mut latest = {
            let registry = self.registry.lock().map_err(poisoned)?;
            current(&registry).ok_or_else(|| ControlError::not_found(p.session_id.0.clone()))?
        };
        loop {
            if matches(&latest) {
                return encode(&homie_proto::EventsWaitResult {
                    session: latest,
                    timed_out: false,
                });
            }
            let Some(remaining) = deadline.checked_duration_since(std::time::Instant::now()) else {
                return encode(&homie_proto::EventsWaitResult {
                    session: latest,
                    timed_out: true,
                });
            };
            if stream.recv(remaining).is_some() {
                let registry = self.registry.lock().map_err(poisoned)?;
                if let Some(record) = current(&registry) {
                    latest = record;
                }
            }
        }
    }

    fn dispatch(&self, method: &str, params: Option<JsonValue>) -> Result<JsonValue, ControlError> {
        match method {
            Method::HELLO => self.hello(params),
            Method::SESSION_SPAWN => self.session_spawn(params),
            Method::SESSION_LIST | Method::STATE_SNAPSHOT => self.session_list(),
            Method::SESSION_CAPABILITIES => self.session_capabilities(params),
            Method::SESSION_SEND_TEXT => self.session_send_text(params),
            Method::SESSION_RESIZE => self.session_resize(params),
            Method::SESSION_READ_SCREEN => self.session_read_screen(params),
            Method::SESSION_READ_SCROLLBACK => self.session_read_scrollback(params),
            Method::SESSION_READ_SCROLLBACK_CELLS => self.session_read_scrollback_cells(params),
            Method::SESSION_KILL => self.session_kill(params),
            Method::SESSION_REMOVE => self.session_remove(params),
            Method::SESSION_RENAME => self.session_rename(params),
            Method::SESSION_MARK_SEEN => self.session_mark_seen(params),
            Method::SESSION_ARCHIVE => self.session_archive(params),
            Method::SESSION_UNARCHIVE => self.session_unarchive(params),
            Method::SESSION_HISTORY => self.session_history(),
            Method::WORKTREE_CREATE => self.worktree_create(params),
            Method::WORKTREE_LIST => self.worktree_list(params),
            Method::WORKTREE_REMOVE => self.worktree_remove(params),
            Method::WORKTREE_OVERVIEW => self.worktree_overview(),
            Method::TEST_RUN => self.browser_call("run", params),
            "browser.act" => self.browser_call("browser", params),
            Method::EVENTS_WAIT => self.events_wait(params),
            Method::HOST_SYNC_PREFS => self.host_sync_prefs(params),
            Method::HOST_INITIALIZE => self.host_initialize(params),
            Method::HOST_LIST_DIRECTORIES => self.host_list_directories(params),
            Method::SESSION_MIGRATE => self.session_migrate(params),
            Method::HOST_LOCATE_REPO => self.host_locate_repo(params),
            Method::HOOK_REPORT => self.hook_report(params),
            Method::SESSION_RESUME => self.session_resume(params),
            Method::SESSION_RESUME_FROM_HISTORY => self.session_resume_from_history(params),
            Method::SESSION_REOPEN_LAST => self.session_reopen_last(),
            Method::AGENT_READINESS => self.agent_readiness(),
            Method::ENVIRONMENT_REFRESH_PATH => self.environment_refresh_path(),
            Method::PROJECT_ADD => self.project_add(params),
            Method::SESSION_READ_DIFF => self.session_read_diff(params),
            Method::SESSION_HIBERNATE => self.session_hibernate(params),
            Method::SESSION_WAKE => self.session_wake(params),
            Method::DAEMON_PREPARE_SHUTDOWN => self.daemon_prepare_shutdown(),
            Method::DAEMON_SHUTDOWN_IF_IDLE => self.daemon_shutdown_if_idle(),
            Method::DAEMON_SHUTDOWN => self.daemon_shutdown(),
            Method::GOVERNOR_CONFIGURE => self.governor_configure(params),
            Method::CLIENT_SET_ACTIVE => self.client_set_active(params),
            other => Err(ControlError::not_found(format!(
                "method {other:?} is not implemented by this engine yet"
            ))),
        }
    }

    pub fn socket_path(&self) -> &Path {
        &self.socket_path
    }
}

/// Content identity of the running Engine. It is computed once, then reused by
/// every heartbeat so version coordination has no steady-state hashing cost.
fn process_executable_hash() -> Option<&'static str> {
    static HASH: std::sync::OnceLock<Option<String>> = std::sync::OnceLock::new();
    HASH.get_or_init(|| {
        let executable = std::env::current_exe().ok()?;
        let mut file = std::fs::File::open(executable).ok()?;
        let mut digest = Sha256::new();
        let mut buffer = [0_u8; 64 * 1024];
        loop {
            let read = file.read(&mut buffer).ok()?;
            if read == 0 {
                break;
            }
            digest.update(&buffer[..read]);
        }
        Some(
            digest
                .finalize()
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>(),
        )
    })
    .as_deref()
}

/// A session id in the daemon's format: `s_` plus twelve hex digits.
pub(crate) fn next_session_id() -> String {
    let mut bytes = [0u8; 6];
    getrandom::fill(&mut bytes).expect("the OS random source");
    let hex: String = bytes.iter().map(|byte| format!("{byte:02x}")).collect();
    format!("s_{hex}")
}

#[cfg(test)]
mod tests;
