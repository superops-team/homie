use super::*;
impl DeferredLaunch {
    pub(super) fn new() -> Self {
        Self {
            state: Mutex::new(DeferredState {
                pending: None,
                deadline: Instant::now() + LAUNCH_FALLBACK,
                queued_input: Vec::new(),
                launched: false,
                cancelled: false,
            }),
            cond: std::sync::Condvar::new(),
        }
    }

    /// Records a client size while unlaunched, pushing the exec back so the
    /// viewport can settle. False once launched: resize the PTY instead.
    pub(super) fn propose_size(&self, cols: u16, rows: u16) -> bool {
        let mut state = self.state.lock().expect("deferred");
        if state.launched {
            return false;
        }
        state.pending = Some((cols, rows));
        state.deadline = Instant::now() + LAUNCH_DEBOUNCE;
        self.cond.notify_all();
        true
    }

    /// Queues input while unlaunched. False once launched: write through.
    pub(super) fn queue_input(&self, bytes: &[u8]) -> bool {
        let mut state = self.state.lock().expect("deferred");
        if state.launched {
            return false;
        }
        state.queued_input.extend_from_slice(bytes);
        true
    }

    /// Blocks until the debounce window closes and returns the launch size;
    /// `None` when the session was cancelled before ever launching.
    fn wait_for_launch_size(&self, fallback: (u16, u16)) -> Option<(u16, u16)> {
        let mut state = self.state.lock().expect("deferred");
        loop {
            if state.cancelled {
                return None;
            }
            let now = Instant::now();
            if now >= state.deadline {
                return Some(state.pending.unwrap_or(fallback));
            }
            let wait = state.deadline - now;
            state = self.cond.wait_timeout(state, wait).expect("deferred").0;
        }
    }

    /// Marks the launch complete, handing back input queued meanwhile and a
    /// size proposed after `chosen` was taken (to apply as a normal resize).
    /// `None` when a cancel raced the launch: the caller owns the cleanup of
    /// the child it just started.
    fn finish_launch(&self, chosen: (u16, u16)) -> Option<LaunchHandoff> {
        let mut state = self.state.lock().expect("deferred");
        if state.cancelled {
            return None;
        }
        state.launched = true;
        Some(LaunchHandoff {
            queued_input: std::mem::take(&mut state.queued_input),
            late_size: state.pending.filter(|pending| *pending != chosen),
        })
    }

    /// True when cancellation happened before launch — there is no child.
    fn cancel(&self) -> bool {
        let mut state = self.state.lock().expect("deferred");
        if state.launched {
            return false;
        }
        state.cancelled = true;
        self.cond.notify_all();
        true
    }
}

impl RemoteLaunchCleanup {
    fn disarm(&mut self) {
        self.armed = false;
    }
}

impl Drop for RemoteLaunchCleanup {
    fn drop(&mut self) {
        if !self.armed {
            return;
        }
        let _ = self.manager.kill(&self.helper, &self.selector);
        let _ = self.binding_store.remove(&self.selector.session_id);
    }
}

impl Drop for Session {
    /// Dropping a session ends the *watch*; what happens to the child depends
    /// on who owns the PTY.
    ///
    /// Direct: the child has to go, not merely be forgotten — the pump thread
    /// cannot be reclaimed while the terminal has a writer, and a forgotten
    /// child would keep running with nothing watching or reaping it.
    ///
    /// Held: the child is deliberately left running. Surviving the owner is
    /// the holder's whole purpose; a restarted daemon adopts it via
    /// [`Session::adopt`].
    fn drop(&mut self) {
        self.shared.stop.store(true, Ordering::SeqCst);
        // A drop while the exec is still deferred wakes the launcher so the
        // join below is prompt; the child was never spawned.
        if let Some(deferred) = &self.deferred {
            let _ = deferred.cancel();
        }
        if let Transport::Direct(pty) = &self.transport
            && !self.shared.exited.load(Ordering::SeqCst)
            && let Ok(pty) = pty.lock()
        {
            let _ = pty.kill_group(libc::SIGKILL);
        }
        if let Transport::Remote(client) = &self.transport {
            client.close();
        }
        if let Some(pump) = self.pump.take() {
            let _ = pump.join();
        }
    }
}

pub(crate) fn new_shared(spec: &SessionSpec, log: OutputLog) -> Arc<Shared> {
    Arc::new(Shared {
        id: spec.id.clone(),
        status: Mutex::new(SessionStatus::Starting),
        needs_input: Mutex::new(None),
        title: Mutex::new(None),
        prompt_title: Mutex::new(None),
        prompt_input: Mutex::new(PromptInputState::default()),
        log: Mutex::new(log),
        screen: Mutex::new(HeadlessScreen::new(
            spec.pty.cols as usize,
            spec.pty.rows as usize,
        )),
        reducer: Mutex::new(StatusReducer::new(spec.authority, SystemTime::now())),
        exit: Mutex::new(None),
        exited: AtomicBool::new(false),
        stop: AtomicBool::new(false),
        state_version: AtomicU64::new(0),
        last_hot: AtomicU64::new(unix_secs()),
        artifacts: Mutex::new(Vec::new()),
        hibernated: AtomicBool::new(false),
        queued_input: Mutex::new(Vec::new()),
        child_pid: std::sync::atomic::AtomicI32::new(0),
        remote_grid: Mutex::new(None),
        remote_output_offset: AtomicU64::new(0),
        grid_wake: GridWake::new(),
    })
}

/// Waits for a freshly launched holder and returns the exit-marker floor:
/// 250 × 20ms.
///
/// Any stat answer attaches — `alive: false` just means the child already
/// exited, and the pump will find its marker. A child so short-lived that the
/// holder has *already cleaned up* is attached by evidence instead: the log
/// advancing past the pre-spawn tail proves the holder ran and wrote a
/// marker.
pub(crate) fn wait_for_holder(
    client: &HolderClient,
    logs_dir: &Path,
    session_id: &str,
    pre_spawn_tail: u64,
) -> Result<u64, crate::holder::HolderError> {
    for _ in 0..250 {
        if let Ok(stat) = client.stat() {
            return Ok(stat.epoch_offset.unwrap_or(pre_spawn_tail));
        }
        if let Ok(mut log) = OutputLog::reader(logs_dir, session_id) {
            log.refresh_from_disk();
            if log.tail_offset() > pre_spawn_tail {
                return Ok(pre_spawn_tail);
            }
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    Err(crate::holder::HolderError::Launch(
        "holder did not become ready".into(),
    ))
}

pub(crate) fn holder_io_error(error: crate::holder::HolderError) -> std::io::Error {
    std::io::Error::other(error.to_string())
}

/// Records a deferred launch that never produced a child: the session
/// reports exit 127, the spawn-failure convention the app already knows.
pub(crate) fn mark_launch_failed(shared: &Shared) {
    *shared.exit.lock().expect("exit") = Some(Exit::Code(127));
    let outcome = shared.reducer.lock().expect("reducer").reduce(
        StatusSignal::ProcessExit {
            code: Some(127),
            signal: None,
        },
        SystemTime::now(),
    );
    apply(shared, &outcome);
    shared.exited.store(true, Ordering::SeqCst);
}

impl Session {
    /// Spawns the child and starts watching it — through a holder when the
    /// spec carries a [`HolderConfig`], directly otherwise.
    pub fn spawn(spec: SessionSpec, engine: Arc<ManifestEngine>) -> std::io::Result<Self> {
        if spec.remote.is_some() {
            return Self::spawn_remote(spec, engine);
        }
        match spec.holder.clone() {
            Some(holder) if spec.defer_launch => Self::spawn_held_deferred(spec, &holder, engine),
            Some(holder) => Self::spawn_held(spec, &holder, engine),
            None => Self::spawn_direct(spec, engine),
        }
    }

    fn spawn_remote(mut spec: SessionSpec, engine: Arc<ManifestEngine>) -> std::io::Result<Self> {
        if spec.holder.is_some() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "a session cannot use local and remote Holders together",
            ));
        }
        let remote = spec.remote.take().expect("checked");
        remote.launch.validate().map_err(std::io::Error::other)?;
        if remote.launch.session_id != spec.id {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "remote launch session id does not match SessionSpec",
            ));
        }
        let token = remote.launch.session_token.clone();
        let launched = remote.manager.launch(&remote.helper, &remote.launch)?;
        let mut cleanup = RemoteLaunchCleanup {
            manager: Arc::clone(&remote.manager),
            helper: remote.helper.clone(),
            binding_store: remote.binding_store.clone(),
            selector: homie_proto::remote_pty::SessionSelector {
                session_id: spec.id.clone(),
                session_token: token.clone(),
                expected_incarnation: Some(launched.session_incarnation.clone()),
            },
            armed: true,
        };
        if launched.session_id != spec.id {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "remote Helper launched the wrong session",
            ));
        }
        let binding = RemoteBinding {
            session_id: spec.id.clone(),
            host_id: remote.host_id,
            helper_build_id: remote.helper.build_id.clone(),
            protocol: remote.helper.protocol,
            session_token: token.clone(),
            session_incarnation: launched.session_incarnation.clone(),
            last_output_offset: 0,
        };
        remote.binding_store.save(&binding)?;
        let client = Arc::new(RemoteSessionClient::new(
            Arc::clone(&remote.manager),
            remote.helper,
            spec.id.clone(),
            token,
            launched.session_incarnation,
            remote.binding_store,
            0,
        ));
        let log = OutputLog::writer(&spec.logs_dir, &spec.id)?;
        let shared = new_shared(&spec, log);
        *shared.remote_grid.lock().expect("remote grid") = Some(RemoteGridState {
            mirror: GridMirror::new(),
            revision: 0,
            pending: None,
        });

        let pump = {
            let shared = Arc::clone(&shared);
            let engine = Arc::clone(&engine);
            let client = Arc::clone(&client);
            let manifest_id = spec.manifest_id.clone();
            std::thread::Builder::new()
                .name(format!("homie-remote-session-{}", spec.id))
                .spawn(move || pump_remote(shared, engine, client, manifest_id))?
        };

        let session = Self {
            shared,
            transport: Transport::Remote(client),
            pump: Some(pump),
            manifest_id: spec.manifest_id,
            deferred: None,
        };
        cleanup.disarm();
        Ok(session)
    }

    /// Reattaches an Engine restarted after the Holder was launched. The
    /// owner-only binding provides the bearer and exact Helper build.
    pub fn adopt_remote(
        spec: SessionSpec,
        remote: RemoteAdoptSpec,
        engine: Arc<ManifestEngine>,
    ) -> std::io::Result<Self> {
        Self::adopt_remote_with_status(spec, remote, engine, None)
    }

    /// Reattaches a remote Holder while retaining the last canonical status.
    /// The incoming Full Snapshot still updates the reducer; seeding prevents
    /// an Engine/App restart from presenting an already-idle Agent as a fresh
    /// launch during startup grace.
    pub fn adopt_remote_with_status(
        spec: SessionSpec,
        remote: RemoteAdoptSpec,
        engine: Arc<ManifestEngine>,
        initial_status: Option<(SessionStatus, Option<NeedsInputDetail>)>,
    ) -> std::io::Result<Self> {
        let client = Arc::new(RemoteSessionClient::new(
            remote.manager,
            remote.helper,
            spec.id.clone(),
            remote.token,
            remote.incarnation,
            remote.binding_store,
            remote.output_offset,
        ));
        let log = OutputLog::writer(&spec.logs_dir, &spec.id)?;
        let shared = new_shared(&spec, log);
        shared
            .remote_output_offset
            .store(remote.output_offset, Ordering::SeqCst);
        *shared.remote_grid.lock().expect("remote grid") = Some(RemoteGridState {
            mirror: GridMirror::new(),
            revision: 0,
            pending: None,
        });
        if let Some((status, needs_input)) = initial_status {
            *shared.status.lock().expect("status") = status;
            *shared.needs_input.lock().expect("needs input") = needs_input;
        }
        shared
            .reducer
            .lock()
            .expect("reducer")
            .finish_startup_grace(SystemTime::now());
        let pump = {
            let shared = Arc::clone(&shared);
            let engine = Arc::clone(&engine);
            let client = Arc::clone(&client);
            let manifest_id = spec.manifest_id.clone();
            std::thread::Builder::new()
                .name(format!("homie-remote-session-{}", spec.id))
                .spawn(move || pump_remote(shared, engine, client, manifest_id))?
        };
        Ok(Self {
            shared,
            transport: Transport::Remote(client),
            pump: Some(pump),
            manifest_id: spec.manifest_id,
            deferred: None,
        })
    }

    fn spawn_direct(spec: SessionSpec, engine: Arc<ManifestEngine>) -> std::io::Result<Self> {
        let pty = Pty::spawn(&spec.pty)?;
        let log = OutputLog::writer(&spec.logs_dir, &spec.id)?;
        let shared = new_shared(&spec, log);
        shared.child_pid.store(pty.pid() as i32, Ordering::SeqCst);

        let reader = pty.reader()?;
        let pty = Arc::new(Mutex::new(pty));

        let pump = {
            let shared = Arc::clone(&shared);
            let engine = Arc::clone(&engine);
            let pty = Arc::clone(&pty);
            let manifest_id = spec.manifest_id.clone();
            std::thread::Builder::new()
                .name(format!("homie-session-{}", spec.id))
                .spawn(move || pump_loop(shared, engine, pty, reader, manifest_id))?
        };

        Ok(Self {
            shared,
            transport: Transport::Direct(pty),
            pump: Some(pump),
            manifest_id: spec.manifest_id,
            deferred: None,
        })
    }

    /// Spawns through the holder manager, so the child outlives this process.
    fn spawn_held(
        spec: SessionSpec,
        holder: &HolderConfig,
        engine: Arc<ManifestEngine>,
    ) -> std::io::Result<Self> {
        let paths = HolderPaths::new(&holder.holders_dir, &spec.id);
        // Incarnation-boundary fallback for pre-epoch holders: everything
        // already in the log predates the child about to spawn.
        let pre_spawn_tail = {
            let mut log = OutputLog::reader(&spec.logs_dir, &spec.id)?;
            log.refresh_from_disk();
            log.tail_offset()
        };
        let launch = HolderLaunchSpec {
            session_id: spec.id.clone(),
            socket_path: paths.socket().to_string_lossy().into_owned(),
            pid_file_path: paths.pid_file().to_string_lossy().into_owned(),
            log_file_path: spec
                .logs_dir
                .join(format!("{}.bin", spec.id))
                .to_string_lossy()
                .into_owned(),
            argv: spec.pty.argv.clone(),
            cwd: spec.pty.cwd.to_string_lossy().into_owned(),
            environment: spec.pty.env.iter().cloned().collect(),
            cols: spec.pty.cols.max(2),
            rows: spec.pty.rows.max(2),
            disk_capacity: crate::holder::protocol::DEFAULT_DISK_CAPACITY,
        };
        HolderLauncher::launch(&holder.executable, &paths, &launch).map_err(holder_io_error)?;

        let client = HolderClient::new(paths.socket());
        let floor = wait_for_holder(&client, &spec.logs_dir, &spec.id, pre_spawn_tail)
            .map_err(holder_io_error)?;
        Self::attach(spec, client, floor, engine)
    }

    /// Spawns through a holder, but not yet: the exec waits for the first
    /// client size to settle ([`LAUNCH_DEBOUNCE`] after each proposal, at
    /// most [`LAUNCH_FALLBACK`] total without one), so the agent's one-shot
    /// banner renders at the real viewport width. Until then input queues
    /// and the session presents an empty screen at the estimated size.
    fn spawn_held_deferred(
        spec: SessionSpec,
        holder: &HolderConfig,
        engine: Arc<ManifestEngine>,
    ) -> std::io::Result<Self> {
        let paths = HolderPaths::new(&holder.holders_dir, &spec.id);
        let client = HolderClient::new(paths.socket());
        let log = OutputLog::reader(&spec.logs_dir, &spec.id)?;
        let shared = new_shared(&spec, log);
        let deferred = Arc::new(DeferredLaunch::new());

        let pump = {
            let shared = Arc::clone(&shared);
            let engine = Arc::clone(&engine);
            let client = client.clone();
            let deferred = Arc::clone(&deferred);
            let holder = holder.clone();
            let manifest_id = spec.manifest_id.clone();
            let logs_dir = spec.logs_dir.clone();
            let id = spec.id.clone();
            let mut pty = spec.pty.clone();
            std::thread::Builder::new()
                .name(format!("homie-session-{}", spec.id))
                .spawn(move || {
                    let Some((cols, rows)) = deferred.wait_for_launch_size((pty.cols, pty.rows))
                    else {
                        return; // cancelled before ever launching
                    };
                    pty.cols = cols.max(2);
                    pty.rows = rows.max(2);
                    shared
                        .screen
                        .lock()
                        .expect("screen")
                        .resize(pty.cols as usize, pty.rows as usize);

                    let pre_spawn_tail = {
                        let mut log = shared.log.lock().expect("log");
                        log.refresh_from_disk();
                        log.tail_offset()
                    };
                    let launch = HolderLaunchSpec {
                        session_id: id.clone(),
                        socket_path: paths.socket().to_string_lossy().into_owned(),
                        pid_file_path: paths.pid_file().to_string_lossy().into_owned(),
                        log_file_path: logs_dir
                            .join(format!("{id}.bin"))
                            .to_string_lossy()
                            .into_owned(),
                        argv: pty.argv.clone(),
                        cwd: pty.cwd.to_string_lossy().into_owned(),
                        environment: pty.env.iter().cloned().collect(),
                        cols: pty.cols,
                        rows: pty.rows,
                        disk_capacity: crate::holder::protocol::DEFAULT_DISK_CAPACITY,
                    };
                    if HolderLauncher::launch(&holder.executable, &paths, &launch).is_err() {
                        mark_launch_failed(&shared);
                        return;
                    }
                    let Ok(floor) = wait_for_holder(&client, &logs_dir, &id, pre_spawn_tail) else {
                        mark_launch_failed(&shared);
                        return;
                    };
                    if let Ok(stat) = client.stat() {
                        shared.child_pid.store(stat.child_pid, Ordering::SeqCst);
                    }
                    let Some(handoff) = deferred.finish_launch((cols, rows)) else {
                        // A terminate raced the launch and believes there is
                        // no child; there is one now, so it goes with us.
                        let _ = client.kill_tree();
                        return;
                    };
                    if !handoff.queued_input.is_empty() {
                        let _ = client.write(&handoff.queued_input);
                    }
                    if let Some((cols, rows)) = handoff.late_size {
                        // A size proposed while the exec was in flight: apply
                        // as an ordinary resize now that the PTY exists.
                        let _ = client.resize(cols.max(2), rows.max(2));
                        shared
                            .screen
                            .lock()
                            .expect("screen")
                            .resize(cols.max(2) as usize, rows.max(2) as usize);
                    }
                    pump_held(shared, engine, client, floor, manifest_id)
                })?
        };

        Ok(Self {
            shared,
            transport: Transport::Held(client),
            pump: Some(pump),
            manifest_id: spec.manifest_id,
            deferred: Some(deferred),
        })
    }

    /// Reconstitutes a live session owned by a holder a previous daemon
    /// spawned. The holder must already be alive; `stat` is its current view.
    pub fn adopt(
        spec: SessionSpec,
        holder: &HolderConfig,
        stat: &HolderStat,
        engine: Arc<ManifestEngine>,
    ) -> std::io::Result<Self> {
        Self::adopt_with_status(spec, holder, stat, engine, None)
    }

    /// Adopt, seeding the visible status from the persisted record: a fresh
    /// reducer starts at Starting, and without evidence (a hook, a screen
    /// change) an adopted idle Claude would sit "starting" forever — the
    /// restart would rewrite history the record already knows.
    pub fn adopt_with_status(
        spec: SessionSpec,
        holder: &HolderConfig,
        stat: &HolderStat,
        engine: Arc<ManifestEngine>,
        initial_status: Option<(SessionStatus, Option<NeedsInputDetail>)>,
    ) -> std::io::Result<Self> {
        let paths = HolderPaths::new(&holder.holders_dir, &spec.id);
        let client = HolderClient::new(paths.socket());
        // Exit markers below the adopted holder's epoch were written by prior
        // incarnations of this session id — never by this child. Markers at
        // or above it (including one written while no daemon ran) apply.
        let floor = stat.epoch_offset.unwrap_or(0);
        let mut spec = spec;
        if let (Some(cols), Some(rows)) = (stat.cols, stat.rows) {
            spec.pty.cols = cols;
            spec.pty.rows = rows;
        }
        let session = Self::attach(spec, client, floor, engine)?;
        if let Some((status, needs_input)) = initial_status {
            *session.shared.status.lock().expect("status") = status;
            *session.shared.needs_input.lock().expect("needs input") = needs_input;
        }
        Ok(session)
    }

    /// The held-transport core: a read-only log tail drives the screen and
    /// reducer; the holder socket carries input, resize, and kill.
    fn attach(
        spec: SessionSpec,
        client: HolderClient,
        exit_marker_floor: u64,
        engine: Arc<ManifestEngine>,
    ) -> std::io::Result<Self> {
        let log = OutputLog::reader(&spec.logs_dir, &spec.id)?;
        let shared = new_shared(&spec, log);
        if let Ok(stat) = client.stat() {
            shared.child_pid.store(stat.child_pid, Ordering::SeqCst);
        }

        let pump = {
            let shared = Arc::clone(&shared);
            let engine = Arc::clone(&engine);
            let client = client.clone();
            let manifest_id = spec.manifest_id.clone();
            std::thread::Builder::new()
                .name(format!("homie-session-{}", spec.id))
                .spawn(move || pump_held(shared, engine, client, exit_marker_floor, manifest_id))?
        };

        Ok(Self {
            shared,
            transport: Transport::Held(client),
            pump: Some(pump),
            manifest_id: spec.manifest_id,
            deferred: None,
        })
    }

    /// Ends the session, killing the child's whole tree.
    pub fn terminate(&mut self, grace: Duration) -> std::io::Result<Exit> {
        // Killed before the deferred exec: there is no child. Cancel wakes
        // the launcher (which double-checks under the same lock, killing a
        // child it raced into existence), and the session records a kill.
        if let Some(deferred) = &self.deferred
            && deferred.cancel()
        {
            self.shared.stop.store(true, Ordering::SeqCst);
            if let Some(pump) = self.pump.take() {
                let _ = pump.join();
            }
            self.shared.exited.store(true, Ordering::SeqCst);
            return Ok(Exit::Signal(libc::SIGKILL));
        }
        let exit = match &self.transport {
            Transport::Direct(pty) => pty.lock().expect("pty").terminate(grace)?,
            Transport::Held(client) => {
                // The holder escalates TERM → KILL itself; wait for the exit
                // marker to land in the log so the recorded exit is the real
                // one.
                let _ = client.kill_tree();
                let deadline = std::time::Instant::now() + grace + Duration::from_secs(1);
                while std::time::Instant::now() < deadline {
                    if self.shared.exited.load(Ordering::SeqCst) {
                        break;
                    }
                    std::thread::sleep(Duration::from_millis(20));
                }
                self.shared
                    .exit
                    .lock()
                    .expect("exit")
                    .unwrap_or(Exit::Signal(libc::SIGKILL))
            }
            Transport::Remote(client) => {
                if !self.shared.exited.load(Ordering::SeqCst) {
                    let _ = client.signal(libc::SIGTERM);
                    let deadline = std::time::Instant::now() + grace;
                    while std::time::Instant::now() < deadline
                        && !self.shared.exited.load(Ordering::SeqCst)
                    {
                        std::thread::sleep(Duration::from_millis(20));
                    }
                }
                // `kill` also stops the per-session Holder. Do this even when
                // the Agent already exited naturally; an explicit lifecycle
                // termination must not leave an idle remote owner behind.
                if let Err(error) = client.kill()
                    && !self.shared.exited.load(Ordering::SeqCst)
                {
                    return Err(error);
                }
                self.shared
                    .exit
                    .lock()
                    .expect("exit")
                    .unwrap_or(Exit::Signal(libc::SIGKILL))
            }
        };
        self.shared.stop.store(true, Ordering::SeqCst);
        if let Transport::Remote(client) = &self.transport {
            client.close();
        }
        if let Some(pump) = self.pump.take() {
            let _ = pump.join();
        }
        Ok(exit)
    }
}
