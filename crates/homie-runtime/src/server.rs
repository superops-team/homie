use std::sync::Arc;

use thiserror::Error;
use tokio::net::UnixListener;
use tokio::sync::{Semaphore, watch};
use tokio::task::JoinSet;

use crate::connection::{ControlHandler, StreamHandler, serve_connection};

const MAX_ACTIVE_CONNECTIONS: usize = 64;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ServerConfig {
    pub expected_peer_uid: u32,
}

impl ServerConfig {
    #[must_use]
    pub fn current_user() -> Self {
        Self {
            expected_peer_uid: Self::current_process_uid(),
        }
    }

    #[must_use]
    pub fn current_process_uid() -> u32 {
        // SAFETY: geteuid has no preconditions and does not dereference pointers.
        unsafe { libc::geteuid() }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ServerIdentity {
    pub daemon_build: String,
    pub daemon_version: String,
    pub daemon_pid: u32,
    pub daemon_instance_id: String,
    pub executable_hash: String,
}

#[derive(Clone, Debug)]
pub struct ShutdownHandle {
    sender: watch::Sender<bool>,
}

impl ShutdownHandle {
    fn new() -> Self {
        let (sender, _) = watch::channel(false);
        Self { sender }
    }

    pub fn request_shutdown(&self) {
        self.sender.send_replace(true);
    }

    pub(crate) fn subscribe(&self) -> watch::Receiver<bool> {
        self.sender.subscribe()
    }
}

pub struct RuntimeServer {
    config: ServerConfig,
    identity: ServerIdentity,
    control: Arc<dyn ControlHandler>,
    streams: Arc<dyn StreamHandler>,
    connections: Arc<Semaphore>,
    shutdown: ShutdownHandle,
}

impl RuntimeServer {
    #[must_use]
    pub fn new(
        config: ServerConfig,
        identity: ServerIdentity,
        control: Arc<dyn ControlHandler>,
        streams: Arc<dyn StreamHandler>,
    ) -> Self {
        Self {
            config,
            identity,
            control,
            streams,
            connections: Arc::new(Semaphore::new(MAX_ACTIVE_CONNECTIONS)),
            shutdown: ShutdownHandle::new(),
        }
    }

    #[must_use]
    pub fn shutdown_handle(&self) -> ShutdownHandle {
        self.shutdown.clone()
    }

    pub async fn serve_listener(
        self: Arc<Self>,
        listener: UnixListener,
    ) -> Result<(), ServerError> {
        let mut shutdown = self.shutdown.subscribe();
        let mut connection_tasks = JoinSet::new();
        loop {
            if *shutdown.borrow() {
                break;
            }
            tokio::select! {
                biased;
                changed = shutdown.changed() => {
                    if changed.is_err() || *shutdown.borrow_and_update() {
                        break;
                    }
                }
                accepted = listener.accept() => {
                    let (socket, _) = accepted?;
                    let Ok(permit) = self.connections.clone().try_acquire_owned() else {
                        drop(socket);
                        continue;
                    };
                    let identity = self.identity.clone();
                    let control = self.control.clone();
                    let streams = self.streams.clone();
                    let expected_peer_uid = self.config.expected_peer_uid;
                    let connection_shutdown = self.shutdown.clone();
                    connection_tasks.spawn(async move {
                        let _permit = permit;
                        let _ = serve_connection(
                            socket,
                            expected_peer_uid,
                            identity,
                            control,
                            streams,
                            connection_shutdown,
                        )
                        .await;
                    });
                }
                _ = connection_tasks.join_next(), if !connection_tasks.is_empty() => {}
            }
        }

        drop(listener);
        while connection_tasks.join_next().await.is_some() {}
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum ServerError {
    #[error("runtime server accept failed")]
    Accept(#[from] std::io::Error),
}
