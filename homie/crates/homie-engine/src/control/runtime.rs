//! Control-channel runtime lifecycle.
//!
//! UnixListener binding, remote Holder restore, idle/ack shutdown, and the
//! per-connection guards (active connection counter, event subscription) that
//! keep the daemon alive and clean up after disconnected clients. These live
//! apart from the wire codec and the method handlers because they change for
//! lifecycle reasons, not protocol ones.

use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use homie_proto::{ControlError, JsonValue};
use serde_json::json;

use super::ControlServer;
use super::wire::{encode, poisoned};

impl ControlServer {
    /// Re-adopts remote Holder sessions in the background.
    ///
    /// Every binding costs at least one SSH round trip, and each carries a
    /// two-minute timeout. Doing that before `bind()` meant the control socket
    /// did not exist until the last host answered: one reachable-but-hung host
    /// kept the whole app disconnected, and because the executor forces
    /// `SSH_ASKPASS_REQUIRE`, a host needing a passphrase could raise a modal
    /// from a daemon with no UI behind it. Local sessions are served
    /// immediately now, and remote ones join as they are verified.
    pub fn spawn_remote_restore(self: &Arc<Self>) {
        if self.remote_bindings.is_none() {
            return;
        }
        let manager = self.remote.clone();
        let server = Arc::clone(self);
        if let Err(error) = std::thread::Builder::new()
            .name("homie-remote-restore".into())
            .spawn(move || {
                // Before adoption, not after: adoption prunes bindings for
                // sessions it finds dead, and a pruned binding is
                // indistinguishable from a record that never had one. Running
                // first is what keeps the legacy test — "has a host and no
                // binding" — from swallowing this launch's own casualties.
                server.retire_legacy_remote_sessions();
                let Some(manager) = manager else {
                    return;
                };
                let adopted = server.restore_remote_bindings(&manager);
                if !adopted.is_empty() {
                    eprintln!(
                        "homie-engine: adopted {} remote Holder session(s): {adopted:?}",
                        adopted.len()
                    );
                }
            })
        {
            eprintln!("homie-engine: could not start remote session restore: {error}");
        }
    }

    /// One-shot upgrade path for sessions the deleted `ssh -t` + tmux transport
    /// created. See [`crate::legacy_remote`] for what it does, what it refuses
    /// to do, and why this is not a tmux fallback.
    ///
    /// Deliberately independent of `with_remote`: a build with no Helper
    /// artifact still has the user's old records and still owes them a working
    /// Resume button and a cleaned-up host.
    fn retire_legacy_remote_sessions(&self) {
        let plan = crate::legacy_remote::Plan {
            registry: &self.registry,
            bindings: self.remote_bindings.as_ref(),
            hosts: &homie_proto::HostsConfig::load(self.hosts_file()),
            marker_path: self.legacy_remote_marker(),
        };
        let outcome =
            crate::legacy_remote::retire_legacy_remote_sessions(&plan, &crate::hosts::run_shell);
        if let Some(summary) = outcome.summary() {
            eprintln!("{summary}");
        }
        // These records have no live session, so the registry watcher — which
        // only diffs live ones — will never announce the rewrite. Without this
        // the sidebar keeps showing them as running until the next relaunch.
        if !outcome.migrated.is_empty()
            && let Ok(registry) = self.registry.lock()
        {
            for id in &outcome.migrated {
                self.publish_updated(&registry, id);
            }
        }
    }

    /// Beside the socket, next to `remote-bindings` — one file, deletable the
    /// day this migration is retired.
    fn legacy_remote_marker(&self) -> PathBuf {
        self.socket_path
            .parent()
            .map(|parent| parent.join("legacy-remote-migration.json"))
            .unwrap_or_else(|| PathBuf::from("legacy-remote-migration.json"))
    }

    fn restore_remote_bindings(
        &self,
        manager: &Arc<crate::remote::manager::RemoteManager>,
    ) -> Vec<String> {
        let Some(store) = &self.remote_bindings else {
            return Vec::new();
        };
        let Ok(bindings) = store.load_all() else {
            return Vec::new();
        };
        let hosts = homie_proto::HostsConfig::load(self.hosts_file());
        let mut registry = match self.registry.lock() {
            Ok(registry) => registry,
            Err(_) => return Vec::new(),
        };
        let records = registry
            .records()
            .into_iter()
            .map(|record| (record.id.0.clone(), record))
            .collect::<std::collections::HashMap<_, _>>();
        let mut adopted = Vec::new();
        for binding in bindings {
            let Some(record) = records.get(&binding.session_id) else {
                continue;
            };
            if record.host.as_deref() != Some(&binding.host_id) {
                continue;
            }
            let Some(host) = hosts.host(&binding.host_id) else {
                continue;
            };
            let Ok(helper) =
                manager.existing_helper(host, &binding.helper_build_id, binding.protocol)
            else {
                continue;
            };
            let selector = homie_proto::remote_pty::SessionSelector {
                session_id: binding.session_id.clone(),
                session_token: binding.session_token.clone(),
                expected_incarnation: Some(binding.session_incarnation.clone()),
            };
            let Ok(inspection) = manager.inspect(&helper, &selector) else {
                continue;
            };
            if matches!(record.status, homie_proto::SessionStatus::Exited(_))
                || matches!(
                    inspection.process_state,
                    homie_proto::remote_pty::RemoteProcessState::Exited { .. }
                )
            {
                let _ = manager.kill(&helper, &selector);
                let _ = store.remove(&binding.session_id);
                continue;
            }
            if !matches!(
                inspection.process_state,
                homie_proto::remote_pty::RemoteProcessState::Running { .. }
            ) {
                continue;
            }
            let manifest_id = record.kind.id().to_string();
            let spec = crate::session::SessionSpec {
                id: binding.session_id.clone(),
                pty: crate::pty::PtySpec::new(Vec::new(), &record.cwd)
                    .size(inspection.cols, inspection.rows),
                manifest_id: manifest_id.clone(),
                authority: crate::session::authority_for(&manifest_id, &registry.engine()),
                logs_dir: self.logs_dir.clone(),
                holder: None,
                remote: None,
                defer_launch: false,
            };
            let remote = crate::session::RemoteAdoptSpec {
                manager: Arc::clone(manager),
                helper,
                token: binding.session_token,
                incarnation: binding.session_incarnation,
                binding_store: store.clone(),
                output_offset: binding.last_output_offset,
            };
            if registry.adopt_remote(spec, remote).is_ok() {
                adopted.push(binding.session_id);
            }
        }
        adopted
    }

    /// Binds the socket, owner-only.
    ///
    /// The socket carries a user's terminal contents and can spawn processes as
    /// them, so the permissions are part of the security model, not a detail.
    /// A stale socket file from a dead daemon is replaced; a *live* one is not,
    /// which is what stops two engines fighting over the same endpoint.
    pub fn bind(&self) -> std::io::Result<UnixListener> {
        if self.socket_path.exists() {
            if UnixStream::connect(&self.socket_path).is_ok() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::AddrInUse,
                    format!(
                        "something is already serving {}",
                        self.socket_path.display()
                    ),
                ));
            }
            std::fs::remove_file(&self.socket_path)?;
        }
        if let Some(parent) = self.socket_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let listener = UnixListener::bind(&self.socket_path)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&self.socket_path, std::fs::Permissions::from_mode(0o600))?;
        }
        Ok(listener)
    }
}

impl ControlServer {
    pub(super) fn daemon_prepare_shutdown(&self) -> Result<JsonValue, ControlError> {
        let mut registry = self.registry.lock().map_err(poisoned)?;
        let _ = registry.persist();
        Ok(json!({}))
    }

    /// Releases the detached Engine after the desktop App goes away, but only
    /// when doing so cannot strand a live Agent or interrupt another client.
    /// The delayed recheck happens after the acknowledgement has flushed and
    /// the requesting connection has had time to close.
    pub(super) fn daemon_shutdown_if_idle(&self) -> Result<JsonValue, ControlError> {
        let live_sessions = {
            let mut registry = self.registry.lock().map_err(poisoned)?;
            let live_sessions = registry.live_count();
            if live_sessions == 0 {
                let _ = registry.persist();
            }
            live_sessions
        };
        let connections = self.active_connections.load(Ordering::Acquire);
        let refusal = idle_shutdown_refusal(live_sessions, connections);
        if let Some(reason) = refusal {
            return encode(&homie_proto::DaemonShutdownIfIdleResult {
                will_exit: false,
                reason: Some(reason.to_owned()),
            });
        }

        let registry = Arc::clone(&self.registry);
        let active_connections = Arc::clone(&self.active_connections);
        let remote = self.remote.clone();
        let holder = self.holder.clone();
        let browser = self.browser.get().cloned();
        let socket_path = self.socket_path.clone();
        std::thread::spawn(move || {
            // The control response must reach the App before its client shuts
            // down. Wait up to one second for precisely that connection to
            // disappear; any new/other client cancels the exit.
            for _ in 0..20 {
                std::thread::sleep(Duration::from_millis(50));
                if active_connections.load(Ordering::Acquire) == 0 {
                    let still_idle = registry
                        .lock()
                        .is_ok_and(|registry| registry.live_count() == 0);
                    if still_idle {
                        if let Some(remote) = remote {
                            remote.close_control_masters();
                        }
                        if let Some(holder) = holder {
                            let paths = crate::holder::HolderManagerPaths::new(&holder.holders_dir);
                            let _ = crate::holder::HolderManagerClient::new(paths.socket())
                                .shutdown_if_idle();
                        }
                        if let Some(browser) = browser {
                            browser.shutdown();
                        }
                        let _ = std::fs::remove_file(socket_path);
                        std::process::exit(0);
                    }
                    return;
                }
            }
        });
        encode(&homie_proto::DaemonShutdownIfIdleResult {
            will_exit: true,
            reason: None,
        })
    }

    /// Ack first, then exit: the response has to flush before the process
    /// dies, so the client sees a clean reply followed by a socket drop and
    /// relaunches the fresh binary.
    pub(super) fn daemon_shutdown(&self) -> Result<JsonValue, ControlError> {
        {
            let mut registry = self.registry.lock().map_err(poisoned)?;
            let _ = registry.persist();
        }
        let browser = self.browser.get().cloned();
        let socket_path = self.socket_path.clone();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(200));
            if let Some(browser) = browser {
                browser.shutdown();
            }
            let _ = std::fs::remove_file(socket_path);
            std::process::exit(0);
        });
        Ok(json!({}))
    }
}

impl Drop for ControlServer {
    fn drop(&mut self) {
        // Leaving the socket file behind would make the next start think a
        // daemon is already running.
        let _ = std::fs::remove_file(&self.socket_path);
    }
}

/// A connection's live event subscription: stopping it ends the forwarder,
/// whose stream-drop unsubscribes from the bus.
pub(super) struct SubscriptionHandle {
    pub(super) stop: Arc<std::sync::atomic::AtomicBool>,
    _thread: std::thread::JoinHandle<()>,
}

impl SubscriptionHandle {
    /// Builds a handle that stops its forwarder thread on drop.
    pub(super) fn new(
        stop: Arc<std::sync::atomic::AtomicBool>,
        thread: std::thread::JoinHandle<()>,
    ) -> Self {
        Self {
            stop,
            _thread: thread,
        }
    }
}

impl Drop for SubscriptionHandle {
    fn drop(&mut self) {
        // Dropping a JoinHandle detaches rather than cancels its thread. Make
        // the subscription's 250 ms receive timeout a real upper bound on
        // cleanup instead of leaking one polling thread per reconnect.
        self.stop.store(true, std::sync::atomic::Ordering::Release);
    }
}

pub(super) struct ActiveConnectionGuard {
    connections: Arc<AtomicUsize>,
}

impl ActiveConnectionGuard {
    pub(super) fn new(connections: Arc<AtomicUsize>) -> Self {
        connections.fetch_add(1, Ordering::AcqRel);
        Self { connections }
    }
}

impl Drop for ActiveConnectionGuard {
    fn drop(&mut self) {
        self.connections.fetch_sub(1, Ordering::AcqRel);
    }
}

fn idle_shutdown_refusal(live_sessions: usize, connections: usize) -> Option<&'static str> {
    if live_sessions != 0 {
        Some("live sessions still require the Engine")
    } else if connections == 0 {
        Some("request is not associated with a live control connection")
    } else if connections > 1 {
        Some("another control client still requires the Engine")
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::{SubscriptionHandle, idle_shutdown_refusal};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::time::Duration;

    #[test]
    fn idle_shutdown_requires_exactly_the_requesting_client_and_no_session() {
        assert_eq!(
            idle_shutdown_refusal(1, 1),
            Some("live sessions still require the Engine")
        );
        assert_eq!(
            idle_shutdown_refusal(0, 0),
            Some("request is not associated with a live control connection")
        );
        assert_eq!(
            idle_shutdown_refusal(0, 2),
            Some("another control client still requires the Engine")
        );
        assert_eq!(idle_shutdown_refusal(0, 1), None);
    }

    #[test]
    fn dropping_an_event_subscription_stops_its_detached_thread() {
        let stop = Arc::new(AtomicBool::new(false));
        let finished = Arc::new(AtomicBool::new(false));
        let worker_stop = Arc::clone(&stop);
        let worker_finished = Arc::clone(&finished);
        let thread = std::thread::spawn(move || {
            while !worker_stop.load(Ordering::Acquire) {
                std::thread::yield_now();
            }
            worker_finished.store(true, Ordering::Release);
        });
        drop(SubscriptionHandle::new(stop, thread));
        for _ in 0..100 {
            if finished.load(Ordering::Acquire) {
                return;
            }
            std::thread::sleep(Duration::from_millis(1));
        }
        panic!("subscription worker did not observe Drop cancellation");
    }
}
