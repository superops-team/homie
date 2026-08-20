use super::*;

/// Running daemon bridge. It owns only tasks and shared channels; UI state remains in `SessionStore`.
pub struct StoreRuntime {
    pub store: Arc<RwLock<SessionStore>>,
    client: Arc<DaemonClient>,
    detach_tx: broadcast::Sender<SessionId>,
    change_tx: broadcast::Sender<()>,
    status_tx: broadcast::Sender<StatusTransition>,
    snapshot_tx: tokio::sync::watch::Sender<StoreSnapshot>,
    action_tx: mpsc::UnboundedSender<SendTextCommand>,
    pub(super) tasks: Mutex<Vec<JoinHandle<()>>>,
}

impl StoreRuntime {
    pub fn start(client: Arc<DaemonClient>, prefs_path: impl Into<PathBuf>) -> io::Result<Self> {
        let (store, effects) = SessionStore::load(prefs_path)?;
        Ok(Self::start_with_store(client, store, effects))
    }

    pub fn start_default(client: Arc<DaemonClient>) -> io::Result<Self> {
        Self::start(client, Prefs::path())
    }

    /// A task-free runtime for deterministic previews. The preview sidebar
    /// owns its fixture store; this bridge exists only to satisfy the shared
    /// application-service interface without connecting to the real daemon.
    pub fn inert() -> Self {
        let (store, _effects) = SessionStore::headless(Prefs::default());
        let store = Arc::new(RwLock::new(store));
        let (detach_tx, _) = broadcast::channel(1);
        let (change_tx, _) = broadcast::channel(1);
        let (status_tx, _) = broadcast::channel(1);
        let snapshot = store
            .read()
            .expect("session store lock poisoned")
            .snapshot();
        let (snapshot_tx, _) = tokio::sync::watch::channel(snapshot);
        let (action_tx, _action_rx) = mpsc::unbounded_channel();
        Self {
            store,
            client: Arc::new(DaemonClient::new()),
            detach_tx,
            change_tx,
            status_tx,
            snapshot_tx,
            action_tx,
            tasks: Mutex::new(Vec::new()),
        }
    }

    fn start_with_store(
        client: Arc<DaemonClient>,
        store: SessionStore,
        effects: mpsc::UnboundedReceiver<StoreEffect>,
    ) -> Self {
        let store = Arc::new(RwLock::new(store));
        let (detach_tx, _) = broadcast::channel(16);
        let (change_tx, _) = broadcast::channel(128);
        let (status_tx, _) = broadcast::channel(32);
        let initial_snapshot = store
            .read()
            .expect("session store lock poisoned")
            .snapshot();
        let (snapshot_tx, _) = tokio::sync::watch::channel(initial_snapshot);
        let (action_tx, mut action_rx) = mpsc::unbounded_channel::<SendTextCommand>();
        let mut tasks = Vec::new();

        let (event_publish_tx, mut event_publish_rx) = mpsc::channel::<StoreEventChange>(128);
        let publish_store = Arc::clone(&store);
        let publish_changes = change_tx.clone();
        let publish_snapshots = snapshot_tx.clone();
        tasks.push(tokio::spawn(async move {
            while let Some(mut change) = event_publish_rx.recv().await {
                // Apply every daemon event immediately, but collapse bursts
                // into one UI/menu publication per display interval. Terminal
                // grid chunks use their own direct path and are unaffected.
                tokio::time::sleep(UI_PUBLISH_INTERVAL).await;
                while let Ok(next) = event_publish_rx.try_recv() {
                    change = change.merge(next);
                }
                let (active, snapshot) = {
                    let store = publish_store.read().expect("session store lock poisoned");
                    (store.app_is_active, store.snapshot())
                };
                // Full model changes still update the menu-bar snapshot while
                // backgrounded. Resource samples are memory-only until the UI
                // is active again, and neither wakes GPUI views in background.
                let (publish_snapshot, notify_views) = event_publication_policy(change, active);
                if publish_snapshot {
                    publish_snapshots.send_replace(snapshot);
                }
                if notify_views {
                    let _ = publish_changes.send(());
                }
            }
        }));

        let mut events = client.events();
        let event_store = Arc::clone(&store);
        tasks.push(tokio::spawn(async move {
            loop {
                match events.recv().await {
                    Ok(event) => {
                        let changed = event_store
                            .write()
                            .expect("session store lock poisoned")
                            .handle_event_change(event);
                        if changed != StoreEventChange::None {
                            let _ = event_publish_tx.try_send(changed);
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
        }));

        let state_client = Arc::clone(&client);
        let state_store = Arc::clone(&store);
        let state_changes = change_tx.clone();
        let state_snapshots = snapshot_tx.clone();
        let mut states = client.connection_state();
        tasks.push(tokio::spawn(async move {
            loop {
                let state = states.borrow_and_update().clone();
                match state {
                    ConnectionState::Connecting => {
                        state_store
                            .write()
                            .expect("session store lock poisoned")
                            .daemon_state = DaemonState::Connecting;
                    }
                    ConnectionState::Disconnected(error) => {
                        state_store
                            .write()
                            .expect("session store lock poisoned")
                            .daemon_state = DaemonState::Unreachable(error);
                    }
                    ConnectionState::Connected(_) => {
                        state_store
                            .write()
                            .expect("session store lock poisoned")
                            .daemon_state = DaemonState::Connected;
                        // The agent catalog first: `hydrate` runs the notification
                        // policy for every arriving session, and that policy reads
                        // descriptors for banner copy and approve keystrokes.
                        // Failure is non-fatal — an old daemon has no descriptors
                        // to give and every reader falls back.
                        if let Ok(agents) = state_client.agent_readiness().await {
                            state_store
                                .write()
                                .expect("session store lock poisoned")
                                .set_agent_catalog(agents);
                        }
                        if let Ok(list) = state_client.sessions().await {
                            let snapshot = {
                                let mut store =
                                    state_store.write().expect("session store lock poisoned");
                                store.hydrate(list);
                                store.snapshot()
                            };
                            state_snapshots.send_replace(snapshot);
                        }
                        let (active, governor) = {
                            let store = state_store.read().expect("session store lock poisoned");
                            (store.app_is_active, store.governor_settings())
                        };
                        let _ = state_client.set_active(active).await;
                        let _ = state_client.configure_governor(governor).await;
                    }
                }
                let _ = state_changes.send(());
                if states.changed().await.is_err() {
                    break;
                }
            }
        }));

        let effect_client = Arc::clone(&client);
        let effect_store = Arc::clone(&store);
        let effect_detach = detach_tx.clone();
        let effect_changes = change_tx.clone();
        let effect_snapshots = snapshot_tx.clone();
        tasks.push(tokio::spawn(run_effects(
            effects,
            effect_client,
            effect_store,
            effect_detach,
            effect_changes,
            effect_snapshots,
            status_tx.clone(),
        )));

        let action_client = Arc::clone(&client);
        let action_status = status_tx.clone();
        tasks.push(tokio::spawn(async move {
            while let Some(command) = action_rx.recv().await {
                if action_client
                    .send_text(&command.session_id, command.text, command.submit)
                    .await
                    .is_err()
                {
                    let _ = action_status.send(reach_failure_transition());
                }
            }
        }));

        client.connect();
        Self {
            store,
            client,
            detach_tx,
            change_tx,
            status_tx,
            snapshot_tx,
            action_tx,
            tasks: Mutex::new(tasks),
        }
    }

    pub fn detachments(&self) -> broadcast::Receiver<SessionId> {
        self.detach_tx.subscribe()
    }

    /// Event-driven invalidation stream for GPUI views. No timer is needed
    /// while daemon/store state is unchanged.
    pub fn changes(&self) -> broadcast::Receiver<()> {
        self.change_tx.subscribe()
    }

    pub fn client(&self) -> &Arc<DaemonClient> {
        &self.client
    }

    pub fn status_events(&self) -> broadcast::Receiver<StatusTransition> {
        self.status_tx.subscribe()
    }

    pub fn snapshots(&self) -> tokio::sync::watch::Receiver<StoreSnapshot> {
        self.snapshot_tx.subscribe()
    }

    pub fn notification_action_sender(&self) -> mpsc::UnboundedSender<SendTextCommand> {
        self.action_tx.clone()
    }

    pub async fn shutdown(&self) {
        self.client.shutdown().await;
        let tasks = std::mem::take(&mut *self.tasks.lock().expect("runtime task lock poisoned"));
        for task in tasks {
            task.abort();
        }
    }
}

impl Drop for StoreRuntime {
    fn drop(&mut self) {
        if let Ok(tasks) = self.tasks.get_mut() {
            for task in tasks.drain(..) {
                task.abort();
            }
        }
    }
}

async fn run_effects(
    mut effects: mpsc::UnboundedReceiver<StoreEffect>,
    client: Arc<DaemonClient>,
    store: Arc<RwLock<SessionStore>>,
    detach_tx: broadcast::Sender<SessionId>,
    change_tx: broadcast::Sender<()>,
    snapshot_tx: tokio::sync::watch::Sender<StoreSnapshot>,
    status_tx: broadcast::Sender<StatusTransition>,
) {
    while let Some(effect) = effects.recv().await {
        let force_snapshot = matches!(
            &effect,
            StoreEffect::SetActive(true) | StoreEffect::PublishSnapshot
        );
        let result: Result<(), ClientError> = match effect {
            StoreEffect::UiChanged | StoreEffect::PublishSnapshot => Ok(()),
            StoreEffect::MarkSeen(id) => client.mark_seen(&id).await,
            StoreEffect::Remove(id) => client.remove(&id).await,
            StoreEffect::Resume { id, automatic } => {
                let result = client.resume(&id).await.map(|_| ());
                if automatic {
                    store
                        .write()
                        .expect("session store lock poisoned")
                        .finish_auto_resume(&id);
                }
                result
            }
            StoreEffect::Archive(id) => client.archive(&id).await,
            StoreEffect::Unarchive(id) => client.unarchive(&id).await,
            StoreEffect::Rename { id, title } => client.rename(&id, title).await,
            StoreEffect::RefreshAgentCatalog => {
                let _ = client.refresh_environment_path().await;
                if let Ok(agents) = client.agent_readiness().await {
                    store
                        .write()
                        .expect("session store lock poisoned")
                        .set_agent_catalog(agents);
                }
                Ok(())
            }
            StoreEffect::Spawn(params) => match client.spawn(params).await {
                Ok(id) => {
                    // The authoritative record still arrives through session.updated.
                    store
                        .write()
                        .expect("session store lock poisoned")
                        .apply_spawn_result(id);
                    Ok(())
                }
                Err(error) => Err(error),
            },
            StoreEffect::SpawnAuxiliary(params) => client.spawn(params).await.map(|_| ()),
            StoreEffect::Migrate { id, target_host } => {
                let destination = {
                    let locked = store.read().expect("session store lock poisoned");
                    target_host
                        .as_deref()
                        .map_or_else(|| "local".to_owned(), |host| locked.host_display_name(host))
                };
                let result = client.migrate(&id, target_host).await;
                let (transition, outcome) = {
                    let mut locked = store.write().expect("session store lock poisoned");
                    locked.finish_migration(&id);
                    match result {
                        Ok(migrated) => {
                            let title = migrated.session.title.clone();
                            let warning = migrated.warning.clone();
                            locked.upsert_session(migrated.session);
                            (
                                migration_transition(&title, &destination, Ok(warning.as_deref())),
                                Ok(()),
                            )
                        }
                        Err(error) => (
                            migration_transition("", &destination, Err(&error.to_string())),
                            Err(error),
                        ),
                    }
                };
                if let Some(transition) = transition {
                    let _ = status_tx.send(transition);
                }
                outcome
            }
            StoreEffect::SyncPrefs { host, host_name } => {
                let result = client.sync_prefs(&host).await;
                store
                    .write()
                    .expect("session store lock poisoned")
                    .finish_prefs_sync(&host);
                let transition = match &result {
                    Ok(report) => prefs_sync_transition(&host_name, Ok(report)),
                    Err(error) => prefs_sync_transition(&host_name, Err(&error.to_string())),
                };
                let _ = status_tx.send(transition);
                result.map(|_| ())
            }
            StoreEffect::LocateRepo {
                key,
                host,
                session_id,
            } => {
                let result = client
                    .locate_repo(homie_proto::HostLocateRepoParams {
                        host,
                        origin_url: None,
                        session_id: Some(session_id),
                    })
                    .await;
                let target = match &result {
                    Ok(found) => match (&found.path, &found.origin_url) {
                        (Some(path), _) => RepoTarget::Resolved(path.clone()),
                        (None, Some(_)) => RepoTarget::NotCloned,
                        (None, None) => RepoTarget::NoOrigin,
                    },
                    // Resolution is best-effort UI sugar: fall back to the
                    // default directory instead of surfacing an error.
                    Err(_) => RepoTarget::NoOrigin,
                };
                store
                    .write()
                    .expect("session store lock poisoned")
                    .set_repo_target(key, target);
                Ok(())
            }
            StoreEffect::ListDirectories {
                request_id,
                host,
                path,
            } => {
                let client = Arc::clone(&client);
                let store = Arc::clone(&store);
                let change_tx = change_tx.clone();
                tokio::spawn(async move {
                    let result = client
                        .list_directories(host, path)
                        .await
                        .map_err(|error| error.to_string());
                    store
                        .write()
                        .expect("session store lock poisoned")
                        .finish_directory_listing(request_id, result);
                    let _ = change_tx.send(());
                });
                Ok(())
            }
            StoreEffect::ReopenLast => match client.reopen_last().await {
                Ok(record) => {
                    let id = record.id.clone();
                    let mut store = store.write().expect("session store lock poisoned");
                    store.upsert_session(record);
                    store.select(id);
                    Ok(())
                }
                Err(error) => Err(error),
            },
            StoreEffect::SetActive(active) => client.set_active(active).await,
            StoreEffect::ConfigureGovernor(settings) => client.configure_governor(settings).await,
            StoreEffect::DetachAttachment(id) => {
                let _ = detach_tx.send(id);
                Ok(())
            }
            StoreEffect::StatusTransition(transition) => {
                let _ = status_tx.send(transition);
                Ok(())
            }
        };
        let mut store = store.write().expect("session store lock poisoned");
        store.last_action_error = result.err().map(|error| error.to_string());
        let active = store.app_is_active;
        let activation_snapshot = force_snapshot.then(|| store.snapshot());
        drop(store);
        if let Some(snapshot) = activation_snapshot {
            snapshot_tx.send_replace(snapshot);
        }
        if active {
            let _ = change_tx.send(());
        }
    }
}
