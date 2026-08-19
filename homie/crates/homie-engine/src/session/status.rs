use super::*;
impl PromptInputState {
    fn observe(&mut self, bytes: &[u8]) -> Option<String> {
        if matches!(bytes, b"\r" | b"\n") {
            let prompt = std::mem::take(&mut self.draft);
            return (!prompt.trim().is_empty()).then_some(prompt);
        }
        if bytes == [0x7f] || bytes == [0x08] {
            self.draft.pop();
            return None;
        }
        if bytes == [0x15] {
            self.draft.clear();
            return None;
        }
        if bytes == [0x17] {
            while self.draft.ends_with(char::is_whitespace) {
                self.draft.pop();
            }
            while self
                .draft
                .chars()
                .last()
                .is_some_and(|c| !c.is_whitespace())
            {
                self.draft.pop();
            }
            return None;
        }

        let bytes = bytes
            .strip_prefix(b"\x1b[200~")
            .and_then(|bytes| bytes.strip_suffix(b"\x1b[201~"))
            .unwrap_or(bytes);
        if bytes.iter().any(|byte| *byte == 0x1b || *byte < 0x09)
            || bytes.iter().any(|byte| (0x0e..0x20).contains(byte))
        {
            return None;
        }
        if let Ok(text) = std::str::from_utf8(bytes) {
            self.draft.push_str(text);
        }
        None
    }
}

/// Applies a reducer outcome to the shared state, bumping the state version
/// only when something observable actually changed — that version is what the
/// registry watcher polls instead of deep-diffing records.
pub(crate) fn apply(shared: &Shared, outcome: &ReducerOutcome) {
    let mut changed = false;
    if let Some(status) = &outcome.status_change {
        {
            let mut current = shared.status.lock().expect("status");
            if *current != *status {
                *current = status.clone();
                changed = true;
            }
        }
        if matches!(status, SessionStatus::Exited(_)) {
            shared.exited.store(true, Ordering::SeqCst);
        }
    }
    if let Some(detail) = &outcome.needs_input {
        let mut current = shared.needs_input.lock().expect("needs input");
        if current.as_ref() != Some(detail) {
            *current = Some(detail.clone());
            changed = true;
        }
    }
    // Leaving a needs-input state clears the pending detail, so the UI does not
    // keep showing a prompt that has been answered.
    if matches!(
        outcome.status_change,
        Some(SessionStatus::Working) | Some(SessionStatus::Idle)
    ) {
        let mut current = shared.needs_input.lock().expect("needs input");
        if current.is_some() {
            *current = None;
            changed = true;
        }
    }
    if changed {
        shared.bump_state_version();
    }
}

impl Session {
    /// Monotonic counter that moves exactly when status, needs-input, or
    /// title change. Poll this before paying for [`Self::view`].
    pub fn state_version(&self) -> u64 {
        self.shared.state_version.load(Ordering::SeqCst)
    }

    pub fn status(&self) -> SessionStatus {
        self.shared.status.lock().expect("status").clone()
    }

    pub(super) fn observe_prompt_input(&self, bytes: &[u8]) {
        if self.manifest_id == "shell"
            || self
                .shared
                .prompt_title
                .lock()
                .expect("prompt title")
                .is_some()
        {
            return;
        }
        if !matches!(
            *self.shared.status.lock().expect("status"),
            SessionStatus::Idle
        ) {
            if matches!(bytes, b"\r" | b"\n") {
                self.shared
                    .prompt_input
                    .lock()
                    .expect("prompt input")
                    .draft
                    .clear();
            }
            return;
        }
        let prompt = self
            .shared
            .prompt_input
            .lock()
            .expect("prompt input")
            .observe(bytes);
        if let Some(prompt) = prompt {
            self.capture_prompt_title(&prompt);
        }
    }

    pub(super) fn capture_prompt_title(&self, prompt: &str) {
        if self.manifest_id == "shell" {
            return;
        }
        let title = crate::hooks::title_from_prompt(prompt);
        if title.is_empty() {
            return;
        }
        let mut current = self.shared.prompt_title.lock().expect("prompt title");
        if current.is_none() {
            *current = Some(title);
            drop(current);
            self.shared.bump_state_version();
        }
    }

    /// Feeds an out-of-band signal — a hook callback, a notify — into the
    /// reducer.
    pub fn feed_signal(&self, signal: StatusSignal) -> ReducerOutcome {
        let outcome = self
            .shared
            .reducer
            .lock()
            .expect("reducer")
            .reduce(signal, SystemTime::now());
        apply(&self.shared, &outcome);
        outcome
    }

    pub fn claude_hook(&self, hook: ClaudeHook, is_subagent: bool) -> ReducerOutcome {
        self.feed_signal(StatusSignal::ClaudeHook { hook, is_subagent })
    }
}

#[cfg(test)]
mod prompt_title_tests {
    use super::PromptInputState;

    #[test]
    fn committed_utf8_prompt_becomes_a_title_candidate() {
        let mut input = PromptInputState::default();
        assert!(input.observe("修".as_bytes()).is_none());
        assert!(input.observe("复 remote attach".as_bytes()).is_none());
        assert_eq!(input.observe(b"\r").as_deref(), Some("修复 remote attach"));
    }

    #[test]
    fn bracketed_paste_and_edits_are_normalized_before_submit() {
        let mut input = PromptInputState::default();
        input.observe(b"wrong");
        input.observe(&[0x15]);
        input.observe(b"\x1b[200~repair remote titles\x1b[201~");
        input.observe(&[0x7f]);
        input.observe(b"e");
        assert_eq!(
            input.observe(b"\r").as_deref(),
            Some("repair remote titlee")
        );
    }
}
