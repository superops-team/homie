use super::*;
/// Rescans the visible screen for artifact URLs every ~2s, only when the
/// content actually changed and only when it plausibly contains a URL —
/// most screens never pay more than a substring check.
pub(crate) fn scan_artifacts_if_due(
    shared: &Shared,
    last_scan_at: &mut Option<std::time::Instant>,
    last_scan_seq: &mut u64,
) {
    if last_scan_at.is_some_and(|at| at.elapsed() < Duration::from_secs(2)) {
        return;
    }
    *last_scan_at = Some(std::time::Instant::now());
    let (seq, text) = {
        let screen = shared.screen.lock().expect("screen");
        let seq = screen.content_seq();
        if seq == *last_scan_seq {
            return;
        }
        (seq, screen.lines().join("\n"))
    };
    *last_scan_seq = seq;
    if !(text.contains("http") || text.contains("github.com") || text.contains("linear.app")) {
        return;
    }
    let now = homie_proto::DateMillis::from(SystemTime::now());
    let mut artifacts = shared.artifacts.lock().expect("artifacts");
    *artifacts = crate::artifacts::scan(&text, &artifacts, now);
}

/// The read/evaluate/reduce loop.
///
/// Waits on the terminal with a timeout rather than blocking in `read`. Two
/// reasons, both of which a blocking read got wrong: the debounce timers must
/// keep advancing while the child is *quiet* — that is exactly when staleness
/// and idle confirmation matter — and a blocking read cannot be interrupted, so
/// stopping a session would hang until the child happened to say something.
pub(crate) fn pump_loop(
    shared: Arc<Shared>,
    engine: Arc<ManifestEngine>,
    pty: Arc<Mutex<Pty>>,
    mut reader: crate::pty::PtyStream,
    manifest_id: String,
) {
    // 64 KiB, matching the held pump: every read may trigger an evaluation,
    // so a small buffer multiplies per-chunk costs on burst output.
    let mut buffer = [0u8; 64 << 10];
    let mut last_tick = SystemTime::now();
    let mut last_eval_seq = 0u64;
    let mut last_scan_at = None;
    let mut last_scan_seq = 0u64;
    let fd = reader.as_raw_fd();

    loop {
        if shared.stop.load(Ordering::SeqCst) {
            break;
        }
        scan_artifacts_if_due(&shared, &mut last_scan_at, &mut last_scan_seq);

        // Wait for output, but never longer than a tick. Output interrupts the
        // wait immediately, so the idle tick only slows reducer timers — which
        // are no-ops outside Working anyway.
        let mut poll_fd = libc::pollfd {
            fd,
            events: libc::POLLIN,
            revents: 0,
        };
        // SAFETY: one initialized pollfd, a millisecond timeout.
        let ready = unsafe { libc::poll(&mut poll_fd, 1, shared.quiet_tick().as_millis() as i32) };
        if ready < 0 {
            let error = std::io::Error::last_os_error();
            if error.kind() == std::io::ErrorKind::Interrupted {
                continue;
            }
            break;
        }
        let hung_up = poll_fd.revents & (libc::POLLHUP | libc::POLLERR) != 0;
        let readable = poll_fd.revents & libc::POLLIN != 0;

        let read_result = if readable || hung_up {
            reader.read(&mut buffer)
        } else {
            Ok(usize::MAX) // nothing to read; fall through to the tick
        };

        match read_result {
            Ok(usize::MAX) => {}
            Ok(0) => break, // the child closed the terminal
            Ok(n) => {
                let chunk = &buffer[..n];
                {
                    let mut log = shared.log.lock().expect("log");
                    // A failed disk write must not stop the session: the child
                    // is still running and its status still matters.
                    let _ = log.append(chunk);
                }
                let observation = {
                    let mut screen = shared.screen.lock().expect("screen");
                    screen.feed(chunk);
                    evaluate_if_screen_changed(
                        &shared,
                        &mut screen,
                        &engine,
                        &manifest_id,
                        &mut last_eval_seq,
                    )
                };
                shared.grid_wake.notify();

                let now = SystemTime::now();
                let mut reducer = shared.reducer.lock().expect("reducer");
                let outcome = reducer.reduce(StatusSignal::PtyOutputActivity, now);
                apply(&shared, &outcome);
                if let Some(observation) = observation {
                    let outcome = reducer.reduce(StatusSignal::Screen(observation), now);
                    drop(reducer);
                    apply(&shared, &outcome);
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(_) => break,
        }

        // Ticks drive the debounce timers even when the child is quiet.
        if last_tick.elapsed().unwrap_or_default() >= TICK_INTERVAL {
            last_tick = SystemTime::now();
            let outcome = shared
                .reducer
                .lock()
                .expect("reducer")
                .reduce(StatusSignal::Tick, last_tick);
            apply(&shared, &outcome);
        }
    }

    // The stream ended: reap the child and record how it died.
    let exit = pty.lock().expect("pty").wait().ok();
    *shared.exit.lock().expect("exit") = exit;
    let (code, signal) = match exit {
        Some(Exit::Code(code)) => (Some(code), None),
        Some(Exit::Signal(signal)) => (None, Some(signal)),
        None => (None, None),
    };
    let outcome = shared.reducer.lock().expect("reducer").reduce(
        StatusSignal::ProcessExit { code, signal },
        SystemTime::now(),
    );
    apply(&shared, &outcome);
    shared.exited.store(true, Ordering::SeqCst);
    let _ = shared.log.lock().expect("log").flush();
}

/// Runs manifest detection only when the visible screen actually changed.
///
/// `feed` is called per PTY chunk, but the reducer discards observations whose
/// `content_seq` it has already judged — previously *after* paying for a full
/// snapshot, two region clones, and the regex walk. `content_seq` also covers
/// the title (an OSC title change bumps it), so the title store rides the same
/// gate and only allocates when it moved.
pub(crate) fn evaluate_if_screen_changed(
    shared: &Shared,
    screen: &mut HeadlessScreen,
    engine: &ManifestEngine,
    manifest_id: &str,
    last_eval_seq: &mut u64,
) -> Option<crate::detect::ScreenObservation> {
    let seq = screen.content_seq();
    if seq == *last_eval_seq {
        return None;
    }
    *last_eval_seq = seq;
    {
        let title = screen.title();
        let mut stored = shared.title.lock().expect("title");
        if stored.as_deref() != title {
            *stored = title.map(str::to_string);
            drop(stored);
            shared.bump_state_version();
        }
    }
    engine.evaluate(&screen.snapshot(), manifest_id)
}

/// The held-transport pump: tails the holder-owned output log.
///
/// The holder writes the log; this loop replays a bounded tail, then follows
/// new bytes — stripping exit markers before the emulator sees them, and
/// honoring only markers at or beyond `exit_marker_floor` (bytes below it
/// belong to prior incarnations of the session id). A holder that dies
/// *without* a marker is caught by a periodic liveness probe.
pub(crate) fn pump_held(
    shared: Arc<Shared>,
    engine: Arc<ManifestEngine>,
    client: HolderClient,
    exit_marker_floor: u64,
    manifest_id: String,
) {
    let replay_budget = replay_budget();
    let (checkpoint_path, mut offset, mut watcher, mut marker_buffer) = {
        let mut log = shared.log.lock().expect("log");
        log.refresh_from_disk();
        let checkpoint_path = crate::checkpoint::ScreenCheckpoint::path_for_log(log.path());
        let watcher = log_watch::LogWatcher::new(log.path());
        let tail = log.tail_offset();
        // A fresh-enough checkpoint seeds the emulator from a few KiB and
        // replay resumes at its offset. "Fresh enough" preserves the hard
        // startup-work bound: the remaining tail must fit the same budget a
        // cold replay would use, even if a checkpoint went stale during a
        // sustained output flood. Anything unusable is a cache miss.
        let restored = crate::checkpoint::ScreenCheckpoint::load(&checkpoint_path)
            .filter(|checkpoint| {
                checkpoint.log_offset <= tail
                    && tail - checkpoint.log_offset <= replay_budget as u64
            })
            .filter(|checkpoint| {
                shared.screen.lock().expect("screen").restore(
                    &checkpoint.history,
                    &checkpoint.grid,
                    checkpoint.alt_screen,
                    checkpoint.bracketed_paste,
                    checkpoint.mouse_reporting,
                )
            });
        match restored {
            Some(checkpoint) => (
                checkpoint_path,
                checkpoint.log_offset,
                watcher,
                checkpoint.marker_buffer,
            ),
            None => (
                checkpoint_path,
                log.preferred_replay_start(replay_budget),
                watcher,
                Vec::new(),
            ),
        }
    };
    // Adoption can restore a checkpoint concurrently with a freshly attached
    // App. One event is cheap and guarantees a seed that raced the restore is
    // corrected without bringing back periodic grid polling.
    shared.grid_wake.notify();
    let mut last_checkpoint_key: Option<CheckpointKey> = None;
    let mut checkpoint_dirty_at: Option<Instant> = None;
    let mut last_liveness = Instant::now();
    let mut last_eval_seq = 0u64;
    let mut last_scan_at = None;
    let mut last_scan_seq = 0u64;
    let mut exit_status: Option<HolderExitStatus> = None;
    // Until the tail is first caught up, bytes are history, not activity:
    // they must render, but not flip a quiet adopted session to Working.
    let mut replaying = true;

    while !shared.stop.load(Ordering::SeqCst) && exit_status.is_none() {
        scan_artifacts_if_due(&shared, &mut last_scan_at, &mut last_scan_seq);
        let (start, chunk) = {
            let mut log = shared.log.lock().expect("log");
            log.refresh_from_disk();
            log.read(offset, 64 << 10)
        };

        if chunk.is_empty() {
            if replaying {
                replaying = false;
                // The replay tail is drained: checkpoint immediately, as the
                // reference implementation does right after `replayExistingLog`.
                if checkpoint_dirty_at.take().is_some() {
                    persist_checkpoint(
                        &shared,
                        &checkpoint_path,
                        offset,
                        &marker_buffer,
                        &mut last_checkpoint_key,
                    );
                }
            } else if checkpoint_dirty_at.is_some_and(|at| at.elapsed() >= CHECKPOINT_SETTLE) {
                checkpoint_dirty_at = None;
                persist_checkpoint(
                    &shared,
                    &checkpoint_path,
                    offset,
                    &marker_buffer,
                    &mut last_checkpoint_key,
                );
            }
            // Quiet: block on the log watcher, which wakes the instant the
            // holder appends — the tick interval is only the ceiling for
            // reducer timers and the liveness probe. Attached or Working
            // sessions keep the fast ceiling; idle background ones stretch it.
            let log_replaced = match watcher.as_mut() {
                Some(watcher) => watcher.wait(shared.quiet_tick()),
                None => {
                    std::thread::sleep(shared.quiet_tick());
                    false
                }
            };
            if log_replaced {
                // The watcher's descriptor followed the retired inode through
                // rotation. Make the cached payload reader reopen the path as
                // well, matching the reference implementation's logDidChange(rearm:).
                shared.log.lock().expect("log").invalidate_read_handle();
            }
            let outcome = shared
                .reducer
                .lock()
                .expect("reducer")
                .reduce(StatusSignal::Tick, SystemTime::now());
            apply(&shared, &outcome);

            if last_liveness.elapsed() >= LIVENESS_INTERVAL {
                last_liveness = Instant::now();
                if !client.is_alive() {
                    // One last look for a marker that raced the probe.
                    let (_, tail) = {
                        let mut log = shared.log.lock().expect("log");
                        log.refresh_from_disk();
                        log.read(offset, 64 << 10)
                    };
                    if tail.is_empty() {
                        // Markerless death: the child is gone but how is
                        // unknowable.
                        break;
                    }
                }
            }
            continue;
        }

        // A rotation can move the readable floor past us; resynchronize.
        if start > offset && !marker_buffer.is_empty() {
            marker_buffer.clear();
        }
        offset = start + chunk.len() as u64;
        last_liveness = Instant::now();

        // The floor is an incarnation boundary, so no marker straddles it:
        // markers wholly below are stripped but their statuses ignored.
        let honored_from = exit_marker_floor
            .saturating_sub(start)
            .min(chunk.len() as u64) as usize;
        let mut output = Vec::new();
        if honored_from > 0 {
            marker_buffer.extend_from_slice(&chunk[..honored_from]);
            let (replayed, _stale_exit) = HolderExitMarker::drain(&mut marker_buffer);
            output.extend_from_slice(&replayed);
            if start + honored_from as u64 >= exit_marker_floor {
                marker_buffer.clear(); // an unfinished stale marker ends here
            }
        }
        if honored_from < chunk.len() {
            marker_buffer.extend_from_slice(&chunk[honored_from..]);
            let (live, exit) = HolderExitMarker::drain(&mut marker_buffer);
            output.extend_from_slice(&live);
            if exit.is_some() {
                exit_status = exit;
            }
        }

        if !output.is_empty() {
            checkpoint_dirty_at = Some(Instant::now());
            let observation = {
                let mut screen = shared.screen.lock().expect("screen");
                screen.feed(&output);
                evaluate_if_screen_changed(
                    &shared,
                    &mut screen,
                    &engine,
                    &manifest_id,
                    &mut last_eval_seq,
                )
            };
            shared.grid_wake.notify();
            let now = SystemTime::now();
            let mut reducer = shared.reducer.lock().expect("reducer");
            if !replaying {
                let outcome = reducer.reduce(StatusSignal::PtyOutputActivity, now);
                apply(&shared, &outcome);
            }
            if let Some(observation) = observation {
                let outcome = reducer.reduce(StatusSignal::Screen(observation), now);
                drop(reducer);
                apply(&shared, &outcome);
            }
        }
    }

    // Detaching or exiting: capture the final screen, so the next daemon
    // seeds from a checkpoint instead of pushing a raw tail through a fresh
    // emulator — the reference implementation's teardown persist.
    if checkpoint_dirty_at.is_some() {
        persist_checkpoint(
            &shared,
            &checkpoint_path,
            offset,
            &marker_buffer,
            &mut last_checkpoint_key,
        );
    }

    if shared.stop.load(Ordering::SeqCst) && exit_status.is_none() {
        return; // detaching, not exiting: the held child lives on
    }

    let exit = exit_status.map(|status| match (status.code, status.signal) {
        (_, Some(signal)) => Exit::Signal(signal),
        (code, None) => Exit::Code(code.unwrap_or(-1)),
    });
    *shared.exit.lock().expect("exit") = exit;
    let (code, signal) = match exit {
        Some(Exit::Code(code)) => (Some(code), None),
        Some(Exit::Signal(signal)) => (None, Some(signal)),
        None => (None, None),
    };
    let outcome = shared.reducer.lock().expect("reducer").reduce(
        StatusSignal::ProcessExit { code, signal },
        SystemTime::now(),
    );
    apply(&shared, &outcome);
    shared.exited.store(true, Ordering::SeqCst);
}

/// Everything a checkpoint's content is a function of, mirroring the Swift
/// `CheckpointKey`: grid and cursor state derive from fed log bytes (tracked
/// by the offset and the screen's `content_seq`), so equal keys mean a
/// byte-identical checkpoint that need not be rewritten.
#[derive(Clone, Copy, PartialEq)]
struct CheckpointKey {
    offset: u64,
    content_seq: u64,
    marker_bytes: usize,
    alt_screen: bool,
    bracketed_paste: bool,
    mouse_reporting: bool,
}

/// Writes the current screen as a durable checkpoint, skipping the write when
/// nothing observable changed since the last one.
fn persist_checkpoint(
    shared: &Shared,
    path: &Path,
    offset: u64,
    marker_buffer: &[u8],
    last_key: &mut Option<CheckpointKey>,
) {
    let (history, grid, alt_screen, bracketed_paste, mouse_reporting, content_seq) = {
        let screen = shared.screen.lock().expect("screen");
        (
            screen.history_snapshot(),
            screen.full_snapshot(),
            screen.is_alt_screen(),
            screen.bracketed_paste(),
            screen.mouse_reporting(),
            screen.content_seq(),
        )
    };
    let key = CheckpointKey {
        offset,
        content_seq,
        marker_bytes: marker_buffer.len(),
        alt_screen,
        bracketed_paste,
        mouse_reporting,
    };
    if *last_key == Some(key) {
        return;
    }
    let checkpoint = crate::checkpoint::ScreenCheckpoint {
        log_offset: offset,
        history,
        grid,
        marker_buffer: marker_buffer.to_vec(),
        alt_screen,
        bracketed_paste,
        mouse_reporting,
    };
    // A failed write must not stop the session; the checkpoint is a cache.
    if checkpoint.write_atomically(path).is_ok() {
        *last_key = Some(key);
    }
}

/// Wakes the held pump the moment the holder appends to the log, instead of
/// sleep-polling between reads. The reference implementation used a DispatchSource for
/// exactly this; without it every byte of held-session output arrives up to a
/// quiet-tick late, which reads as ~10fps scrolling in a TUI.
#[cfg(target_os = "macos")]
mod log_watch {
    use std::path::{Path, PathBuf};
    use std::time::Duration;

    pub struct LogWatcher {
        kq: i32,
        fd: i32,
        path: PathBuf,
    }

    impl LogWatcher {
        pub fn new(path: &Path) -> Option<Self> {
            // SAFETY: plain kqueue creation; failure is handled.
            let kq = unsafe { libc::kqueue() };
            if kq < 0 {
                return None;
            }
            let mut watcher = Self {
                kq,
                fd: -1,
                path: path.to_path_buf(),
            };
            watcher.arm();
            Some(watcher)
        }

        fn arm(&mut self) {
            if self.fd >= 0 {
                // SAFETY: closing a descriptor this struct owns.
                unsafe { libc::close(self.fd) };
                self.fd = -1;
            }
            let Ok(cpath) = std::ffi::CString::new(self.path.as_os_str().as_encoded_bytes()) else {
                return;
            };
            // SAFETY: O_EVTONLY opens for watching without inhibiting unmount.
            let fd = unsafe { libc::open(cpath.as_ptr(), libc::O_EVTONLY) };
            if fd < 0 {
                return; // not created yet: wait() degrades to a plain sleep
            }
            self.fd = fd;
            let event = libc::kevent {
                ident: fd as usize,
                filter: libc::EVFILT_VNODE,
                flags: libc::EV_ADD | libc::EV_CLEAR,
                fflags: libc::NOTE_WRITE
                    | libc::NOTE_EXTEND
                    | libc::NOTE_DELETE
                    | libc::NOTE_RENAME,
                data: 0,
                udata: std::ptr::null_mut(),
            };
            // SAFETY: registering one initialized event; no output requested.
            unsafe {
                libc::kevent(
                    self.kq,
                    &event,
                    1,
                    std::ptr::null_mut(),
                    0,
                    std::ptr::null(),
                )
            };
        }

        /// Blocks until the log changes or `timeout` passes. EV_CLEAR keeps
        /// writes that land between waits queued, so wakeups are never lost.
        /// Returns true when rotation replaced the watched file, so the
        /// caller can invalidate any other descriptors for the old inode.
        pub fn wait(&mut self, timeout: Duration) -> bool {
            if self.fd < 0 {
                self.arm();
                if self.fd < 0 {
                    std::thread::sleep(timeout);
                    return false;
                }
            }
            let spec = libc::timespec {
                tv_sec: timeout.as_secs() as libc::time_t,
                tv_nsec: libc::c_long::from(timeout.subsec_nanos()),
            };
            // SAFETY: zeroed kevent output slot, valid timeout.
            let mut out = unsafe { std::mem::zeroed::<libc::kevent>() };
            let woke = unsafe { libc::kevent(self.kq, std::ptr::null(), 0, &mut out, 1, &spec) };
            if woke > 0 && out.fflags & (libc::NOTE_DELETE | libc::NOTE_RENAME) != 0 {
                // Rotation replaced the file: track the new incarnation.
                self.arm();
                return true;
            }
            false
        }
    }

    impl Drop for LogWatcher {
        fn drop(&mut self) {
            if self.fd >= 0 {
                // SAFETY: descriptors this struct owns.
                unsafe { libc::close(self.fd) };
            }
            unsafe { libc::close(self.kq) };
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::log::OutputLog;

        #[test]
        fn replacement_tells_the_log_reader_to_reopen() {
            let root = tempfile::tempdir().expect("temp dir");
            let mut writer = OutputLog::open(root.path(), "s", 1 << 20, 64, false).expect("writer");
            writer.append(&[b'a'; 32]).expect("initial append");
            let mut reader = OutputLog::reader(root.path(), "s").expect("reader");
            let mut watcher = LogWatcher::new(reader.path()).expect("watcher");

            writer.append(&[b'b'; 40]).expect("rotating append");
            writer.append(b"after").expect("post-rotation append");
            writer.flush().expect("flush");

            assert!(
                watcher.wait(Duration::from_secs(1)),
                "rename/delete notification identifies the replacement"
            );
            reader.invalidate_read_handle();
            assert!(reader.refresh_from_disk());
            assert_eq!(reader.tail_offset(), 77);
            let (_, data) = reader.read(72, 16);
            assert_eq!(data, b"after");
        }
    }
}

/// Platform gap, named: non-macOS builds sleep-poll at the tick interval.
/// Linux wants an inotify equivalent here.
#[cfg(not(target_os = "macos"))]
mod log_watch {
    use std::path::Path;
    use std::time::Duration;

    pub struct LogWatcher;

    impl LogWatcher {
        pub fn new(_path: &Path) -> Option<Self> {
            None
        }

        pub fn wait(&mut self, timeout: Duration) -> bool {
            std::thread::sleep(timeout);
            false
        }
    }
}
