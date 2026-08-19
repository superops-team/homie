use super::*;
/// Follows one remote Holder through any number of short-lived SSH Bridges.
/// The Holder remains the PTY owner; a broken Bridge only advances this
/// reconnect loop. Offsets and grid sequences make every retry idempotent.
pub(crate) fn pump_remote(
    shared: Arc<Shared>,
    engine: Arc<ManifestEngine>,
    client: Arc<RemoteSessionClient>,
    manifest_id: String,
) {
    let mut reconnect_delay = Duration::from_millis(50);
    let mut reconnects = 0_u32;
    while !shared.stop.load(Ordering::SeqCst) && !shared.exited.load(Ordering::SeqCst) {
        let output_offset = shared.remote_output_offset.load(Ordering::SeqCst);
        let grid_sequence = shared
            .remote_grid
            .lock()
            .expect("remote grid")
            .as_ref()
            .and_then(|state| state.mirror.sequence());
        let Ok((generation, mut output)) = client.connect(output_offset, grid_sequence) else {
            reconnects = reconnects.saturating_add(1);
            if reconnects.is_multiple_of(3) && remote_inspection_exited(&shared, &client) {
                break;
            }
            wait_for_remote_retry(&shared, reconnect_delay);
            reconnect_delay = (reconnect_delay * 2).min(Duration::from_secs(2));
            continue;
        };
        reconnect_delay = Duration::from_millis(50);
        let disposition = pump_remote_connection(
            &shared,
            &engine,
            &client,
            generation,
            &mut output,
            &manifest_id,
        );
        client.disconnect(generation);
        match disposition {
            RemoteConnectionDisposition::Continue => continue,
            RemoteConnectionDisposition::Reconnect => {
                reconnects = reconnects.saturating_add(1);
                if reconnects.is_multiple_of(3) && remote_inspection_exited(&shared, &client) {
                    break;
                }
                wait_for_remote_retry(&shared, reconnect_delay);
                reconnect_delay = (reconnect_delay * 2).min(Duration::from_secs(2));
            }
            RemoteConnectionDisposition::Exited | RemoteConnectionDisposition::Stopped => break,
            RemoteConnectionDisposition::Fatal => {
                mark_remote_transport_failed(&shared);
                break;
            }
        }
    }
    let _ = shared.log.lock().expect("log").flush();
}

fn remote_inspection_exited(shared: &Shared, client: &RemoteSessionClient) -> bool {
    let Ok(inspection) = client.inspect() else {
        return false;
    };
    let RemoteProcessState::Exited { code, signal } = inspection.process_state else {
        return false;
    };
    record_remote_exit(shared, ProcessExit { code, signal });
    true
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RemoteConnectionDisposition {
    Continue,
    Reconnect,
    Exited,
    Stopped,
    Fatal,
}

fn pump_remote_connection(
    shared: &Shared,
    engine: &ManifestEngine,
    client: &RemoteSessionClient,
    generation: u64,
    output: &mut std::process::ChildStdout,
    manifest_id: &str,
) -> RemoteConnectionDisposition {
    let mut codec = RemoteCodec::new();
    let mut buffer = [0_u8; 64 << 10];
    let mut replaying = false;
    let mut hello_accepted = false;
    let mut last_tick = SystemTime::now();
    let mut last_eval_seq = 0_u64;
    let mut last_scan_at = None;
    let mut last_scan_seq = 0_u64;
    let fd = output.as_raw_fd();

    loop {
        if shared.stop.load(Ordering::SeqCst) {
            return RemoteConnectionDisposition::Stopped;
        }
        if shared.exited.load(Ordering::SeqCst) {
            return RemoteConnectionDisposition::Exited;
        }
        scan_artifacts_if_due(shared, &mut last_scan_at, &mut last_scan_seq);

        let mut poll_fd = libc::pollfd {
            fd,
            events: libc::POLLIN,
            revents: 0,
        };
        // SAFETY: `poll_fd` points to one initialized pollfd and remains valid
        // for the duration of this call.
        let ready = unsafe { libc::poll(&mut poll_fd, 1, TICK_INTERVAL.as_millis() as i32) };
        if ready < 0 {
            if std::io::Error::last_os_error().kind() == std::io::ErrorKind::Interrupted {
                continue;
            }
            return RemoteConnectionDisposition::Reconnect;
        }

        if ready == 0 {
            let now = SystemTime::now();
            let outcome = shared
                .reducer
                .lock()
                .expect("reducer")
                .reduce(StatusSignal::Tick, now);
            apply(shared, &outcome);
            last_tick = now;
            continue;
        }

        match output.read(&mut buffer) {
            Ok(0) => return RemoteConnectionDisposition::Reconnect,
            Ok(count) => {
                let messages = match codec.feed(&buffer[..count]) {
                    Ok(messages) => messages,
                    Err(_) => return RemoteConnectionDisposition::Fatal,
                };
                for message in messages {
                    let disposition = handle_remote_message(
                        shared,
                        engine,
                        client,
                        generation,
                        manifest_id,
                        &mut last_eval_seq,
                        &mut replaying,
                        &mut hello_accepted,
                        message,
                    );
                    if disposition != RemoteConnectionDisposition::Continue {
                        return disposition;
                    }
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(_) => return RemoteConnectionDisposition::Reconnect,
        }

        if last_tick.elapsed().unwrap_or_default() >= TICK_INTERVAL {
            last_tick = SystemTime::now();
            let outcome = shared
                .reducer
                .lock()
                .expect("reducer")
                .reduce(StatusSignal::Tick, last_tick);
            apply(shared, &outcome);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn handle_remote_message(
    shared: &Shared,
    engine: &ManifestEngine,
    client: &RemoteSessionClient,
    generation: u64,
    manifest_id: &str,
    last_eval_seq: &mut u64,
    replaying: &mut bool,
    hello_accepted: &mut bool,
    message: RemoteMessage,
) -> RemoteConnectionDisposition {
    if !*hello_accepted && !matches!(message, RemoteMessage::HelloAck(_)) {
        return RemoteConnectionDisposition::Fatal;
    }
    match message {
        RemoteMessage::HelloAck(acknowledgement) => {
            if *hello_accepted
                || client.validate_hello(&acknowledgement).is_err()
                || client
                    .accept_hello(generation, acknowledgement.controller_epoch)
                    .is_err()
            {
                return RemoteConnectionDisposition::Fatal;
            }
            if let RemoteProcessState::Exited { code, signal } = acknowledgement.process_state {
                record_remote_exit(shared, ProcessExit { code, signal });
                return RemoteConnectionDisposition::Exited;
            }
            *hello_accepted = true;
            RemoteConnectionDisposition::Continue
        }
        RemoteMessage::Terminal(frame) => match frame.frame_type {
            FrameType::ReplayBegin => {
                *replaying = true;
                RemoteConnectionDisposition::Continue
            }
            FrameType::ReplayEnd => {
                *replaying = false;
                RemoteConnectionDisposition::Continue
            }
            FrameType::Output => {
                let Some((offset, bytes)) = frame.output_payload() else {
                    return RemoteConnectionDisposition::Fatal;
                };
                client.observe_output_offset(offset.saturating_add(bytes.len() as u64));
                apply_remote_output(
                    shared,
                    engine,
                    manifest_id,
                    last_eval_seq,
                    offset,
                    bytes,
                    *replaying,
                );
                RemoteConnectionDisposition::Continue
            }
            _ => RemoteConnectionDisposition::Fatal,
        },
        RemoteMessage::FullSnapshot(snapshot) => {
            if apply_remote_snapshot(shared, engine, manifest_id, last_eval_seq, snapshot).is_err()
            {
                RemoteConnectionDisposition::Fatal
            } else {
                RemoteConnectionDisposition::Continue
            }
        }
        RemoteMessage::GridDelta(delta) => {
            if apply_remote_delta(shared, delta).is_err() {
                // A gap is recoverable: the next Hello always reseeds with a
                // full authoritative snapshot.
                RemoteConnectionDisposition::Reconnect
            } else {
                RemoteConnectionDisposition::Continue
            }
        }
        RemoteMessage::ControlGranted(granted) => {
            if client
                .grant_control(generation, granted.controller_epoch)
                .is_err()
            {
                RemoteConnectionDisposition::Reconnect
            } else {
                RemoteConnectionDisposition::Continue
            }
        }
        RemoteMessage::ControlRevoked(_) => RemoteConnectionDisposition::Reconnect,
        RemoteMessage::ProcessExit(exit) => {
            record_remote_exit(shared, exit);
            RemoteConnectionDisposition::Exited
        }
        RemoteMessage::ScrollbackResponse(response) => {
            client.complete_scrollback(response);
            RemoteConnectionDisposition::Continue
        }
        RemoteMessage::Error(error) if error.fatal => RemoteConnectionDisposition::Fatal,
        RemoteMessage::Error(_) => RemoteConnectionDisposition::Continue,
        _ => RemoteConnectionDisposition::Fatal,
    }
}

fn apply_remote_output(
    shared: &Shared,
    engine: &ManifestEngine,
    manifest_id: &str,
    last_eval_seq: &mut u64,
    offset: u64,
    bytes: &[u8],
    replaying: bool,
) {
    let expected = shared.remote_output_offset.load(Ordering::SeqCst);
    let end = offset.saturating_add(bytes.len() as u64);
    if end <= expected {
        return;
    }
    let skip = expected.saturating_sub(offset).min(bytes.len() as u64) as usize;
    let bytes = &bytes[skip..];
    if bytes.is_empty() {
        return;
    }
    shared.remote_output_offset.store(end, Ordering::SeqCst);
    let _ = shared.log.lock().expect("log").append(bytes);
    let observation = {
        let mut screen = shared.screen.lock().expect("screen");
        screen.feed(bytes);
        evaluate_if_screen_changed(shared, &mut screen, engine, manifest_id, last_eval_seq)
    };
    let now = SystemTime::now();
    let mut reducer = shared.reducer.lock().expect("reducer");
    if !replaying {
        let outcome = reducer.reduce(StatusSignal::PtyOutputActivity, now);
        apply(shared, &outcome);
    }
    if let Some(observation) = observation {
        let outcome = reducer.reduce(StatusSignal::Screen(observation), now);
        drop(reducer);
        apply(shared, &outcome);
    }
}

fn apply_remote_snapshot(
    shared: &Shared,
    engine: &ManifestEngine,
    manifest_id: &str,
    last_eval_seq: &mut u64,
    snapshot: FullSnapshot,
) -> std::io::Result<()> {
    {
        let mut remote = shared.remote_grid.lock().expect("remote grid");
        let remote = remote
            .as_mut()
            .ok_or_else(|| std::io::Error::other("remote grid state is unavailable"))?;
        remote
            .mirror
            .apply_snapshot(
                snapshot.sequence,
                &snapshot.grid,
                snapshot.alt_screen,
                snapshot.bracketed_paste,
                snapshot.mouse_reporting,
            )
            .map_err(std::io::Error::other)?;
        remote.revision = remote.revision.saturating_add(1);
        remote.pending = Some(snapshot.grid.clone());
    }
    shared.grid_wake.notify();
    let observation = {
        let mut screen = shared.screen.lock().expect("screen");
        screen.resize(
            usize::from(snapshot.grid.cols),
            usize::from(snapshot.grid.rows),
        );
        if !screen.restore(
            // A remote Full Snapshot carries only the visible grid; scrollback
            // is fetched on demand through `Scroll`, never replayed here.
            &[],
            &snapshot.grid,
            snapshot.alt_screen,
            snapshot.bracketed_paste,
            snapshot.mouse_reporting,
        ) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "remote terminal snapshot could not be restored",
            ));
        }
        evaluate_if_screen_changed(shared, &mut screen, engine, manifest_id, last_eval_seq)
    };
    if let Some(observation) = observation {
        let outcome = shared
            .reducer
            .lock()
            .expect("reducer")
            .reduce(StatusSignal::Screen(observation), SystemTime::now());
        apply(shared, &outcome);
    }
    Ok(())
}

fn apply_remote_delta(shared: &Shared, delta: GridDelta) -> std::io::Result<()> {
    {
        let mut remote = shared.remote_grid.lock().expect("remote grid");
        let remote = remote
            .as_mut()
            .ok_or_else(|| std::io::Error::other("remote grid state is unavailable"))?;
        remote
            .mirror
            .apply_delta(
                delta.sequence,
                &delta.grid,
                delta.alt_screen,
                delta.bracketed_paste,
                delta.mouse_reporting,
            )
            .map_err(std::io::Error::other)?;
        remote.revision = remote.revision.saturating_add(1);
        remote.pending = if remote.pending.is_some() {
            remote.mirror.full_update()
        } else {
            Some(delta.grid)
        };
    }
    shared.grid_wake.notify();
    Ok(())
}

fn record_remote_exit(shared: &Shared, exit: ProcessExit) {
    let local = match (exit.code, exit.signal) {
        (_, Some(signal)) => Exit::Signal(signal),
        (Some(code), None) => Exit::Code(code),
        (None, None) => Exit::Code(-1),
    };
    *shared.exit.lock().expect("exit") = Some(local);
    let outcome = shared.reducer.lock().expect("reducer").reduce(
        StatusSignal::ProcessExit {
            code: exit.code,
            signal: exit.signal,
        },
        SystemTime::now(),
    );
    apply(shared, &outcome);
    shared.exited.store(true, Ordering::SeqCst);
}

fn mark_remote_transport_failed(shared: &Shared) {
    *shared.exit.lock().expect("exit") = Some(Exit::Code(126));
    let outcome = shared.reducer.lock().expect("reducer").reduce(
        StatusSignal::ProcessExit {
            code: Some(126),
            signal: None,
        },
        SystemTime::now(),
    );
    apply(shared, &outcome);
    shared.exited.store(true, Ordering::SeqCst);
}

fn wait_for_remote_retry(shared: &Shared, duration: Duration) {
    let deadline = Instant::now() + duration;
    while !shared.stop.load(Ordering::SeqCst) && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(20));
    }
}
