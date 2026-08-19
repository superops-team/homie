use super::*;
impl Session {
    /// Reads recorded output by absolute stream offset, for attach and replay.
    pub fn read_output(&self, from_offset: u64, max_bytes: usize) -> (u64, Vec<u8>) {
        self.shared
            .log
            .lock()
            .expect("log")
            .read(from_offset, max_bytes)
    }

    /// The child's pid (0 before it is known), for tree enumeration.
    pub fn child_pid(&self) -> i32 {
        self.shared.child_pid.load(Ordering::SeqCst)
    }

    /// Marks the session hibernated (input queues) or awake. On wake, the
    /// queued input flushes in order — right after the caller's SIGCONT, as
    /// the reference implementation's wake() did.
    pub fn set_hibernated(&self, hibernated: bool) -> std::io::Result<()> {
        self.shared.hibernated.store(hibernated, Ordering::SeqCst);
        if hibernated {
            return Ok(());
        }
        let queued = std::mem::take(&mut *self.shared.queued_input.lock().expect("queued input"));
        if queued.is_empty() {
            return Ok(());
        }
        self.write_raw(&queued)
    }

    pub fn is_hibernated(&self) -> bool {
        self.shared.hibernated.load(Ordering::SeqCst)
    }

    /// Signals the whole child tree. For held sessions the holder walks the
    /// tree with pid-identity checks; a direct session signals its group.
    /// Returns the (pid, start-time) samples the holder observed, when held.
    pub fn signal_tree(&self, signal: i32) -> std::io::Result<Vec<(i32, i64)>> {
        match &self.transport {
            Transport::Direct(pty) => {
                pty.lock().expect("pty").kill_group(signal)?;
                Ok(Vec::new())
            }
            Transport::Held(client) => Ok(client
                .signal(signal)
                .map_err(holder_io_error)?
                .into_iter()
                .map(|sample| (sample.pid, sample.start_sec))
                .collect()),
            Transport::Remote(client) => {
                client.signal(signal)?;
                Ok(Vec::new())
            }
        }
    }

    pub(super) fn write_raw(&self, bytes: &[u8]) -> std::io::Result<()> {
        // Before the deferred exec there is no PTY: queue for the launch
        // flush, exactly like the reference implementation's `queuedLaunchInput`.
        if let Some(deferred) = &self.deferred
            && deferred.queue_input(bytes)
        {
            return Ok(());
        }
        if self.shared.hibernated.load(Ordering::SeqCst) {
            self.shared
                .queued_input
                .lock()
                .expect("queued input")
                .extend_from_slice(bytes);
            return Ok(());
        }
        match &self.transport {
            Transport::Direct(pty) => {
                use std::io::Write;
                let mut writer = pty.lock().expect("pty").writer()?;
                writer.write_all(bytes)?;
                writer.flush()
            }
            Transport::Held(client) => client.write(bytes).map_err(holder_io_error),
            Transport::Remote(client) => client.write(bytes),
        }
    }

    /// Sends text the way a user would.
    ///
    /// Non-submitting input goes through raw — pickers and permission dialogs
    /// read the literal keypress. A submitted prompt is framed as a bracketed
    /// paste when the child has that mode on (so embedded newlines don't
    /// submit the composer early), and the Enter is a SEPARATE write after a
    /// short settle — never riding the same buffer, where a truncated paste
    /// also loses or misfires it. Ported from `AgentSession.sendText`.
    pub fn send_text(&self, text: &str, submit: bool) -> std::io::Result<()> {
        if !submit {
            return self.write_input(text.as_bytes());
        }
        self.paste_text(text)?;
        std::thread::sleep(Duration::from_millis(30));
        self.submit_input()
    }

    /// Types `text` into the composer WITHOUT submitting it, framed as a
    /// bracketed paste when the child has that mode on. Separated from
    /// [`Self::send_text`] so a caller that cannot see the composer — the
    /// initial-prompt injector — can watch the text echo back before it
    /// commits to an Enter it can never take back.
    ///
    /// Titling happens here rather than at submit, so a prompt the injector
    /// types names its session the same way one the user types does. It is
    /// idempotent, which matters because the injector may retype.
    pub fn paste_text(&self, text: &str) -> std::io::Result<()> {
        self.capture_prompt_title(text);
        let framed = if self.bracketed_paste() {
            format!("\x1b[200~{text}\x1b[201~")
        } else {
            text.to_owned()
        };
        self.write_input(framed.as_bytes())
    }

    /// The Enter that submits whatever is in the composer.
    pub fn submit_input(&self) -> std::io::Result<()> {
        self.write_input(b"\r")
    }

    /// Kill-line (⌃U): what every one of these TUIs uses to empty its
    /// composer. Sent before a retyped prompt so a half-landed first attempt
    /// cannot concatenate with the second.
    pub fn clear_input_line(&self) -> std::io::Result<()> {
        self.write_input(b"\x15")
    }

    /// Sends bytes to the child, as if typed.
    pub fn write_input(&self, bytes: &[u8]) -> std::io::Result<()> {
        // Input means someone is interacting: keep the pump on its fast tick
        // so the echo renders promptly.
        self.shared.note_hot();
        if !bytes.is_empty() {
            // The next grid changes are likely a trailing echo already in
            // flight and the TUI's response. Let the attachment pump interrupt
            // its background coalescing wait instead of making typed input
            // cross a 16 ms frame boundary before the host can render it.
            self.shared.grid_wake.prioritize_interactive_changes();
        }
        self.observe_prompt_input(bytes);
        // Typed before the deferred exec: queue for the launch flush, and
        // still count as a keystroke for the reducer.
        if let Some(deferred) = &self.deferred
            && deferred.queue_input(bytes)
        {
            self.feed_signal(StatusSignal::UserKeystroke);
            return Ok(());
        }
        if self.shared.hibernated.load(Ordering::SeqCst) {
            // Never write into a stopped tree's PTY (nobody drains the slave;
            // the buffer fills and writes wedge) — queue for the wake flush.
            self.shared
                .queued_input
                .lock()
                .expect("queued input")
                .extend_from_slice(bytes);
            self.feed_signal(StatusSignal::UserKeystroke);
            return Ok(());
        }
        match &self.transport {
            Transport::Direct(pty) => {
                use std::io::Write;
                let mut writer = pty.lock().expect("pty").writer()?;
                writer.write_all(bytes)?;
                writer.flush()?;
            }
            Transport::Held(client) => client.write(bytes).map_err(holder_io_error)?,
            Transport::Remote(client) => client.write(bytes)?,
        }
        self.feed_signal(StatusSignal::UserKeystroke);
        Ok(())
    }

    pub fn resize(&self, cols: u16, rows: u16) -> std::io::Result<()> {
        // Before the deferred exec, the FIRST client size decides the launch
        // geometry — record it and push the exec back so the viewport can
        // settle; the emulator is resized at launch, not per proposal.
        if let Some(deferred) = &self.deferred
            && deferred.propose_size(cols, rows)
        {
            return Ok(());
        }
        match &self.transport {
            Transport::Direct(pty) => pty.lock().expect("pty").resize(cols, rows)?,
            Transport::Held(client) => client.resize(cols, rows).map_err(holder_io_error)?,
            Transport::Remote(client) => client.resize(cols, rows)?,
        }
        self.shared
            .screen
            .lock()
            .expect("screen")
            .resize(cols as usize, rows as usize);
        self.shared.grid_wake.notify();
        Ok(())
    }
}
