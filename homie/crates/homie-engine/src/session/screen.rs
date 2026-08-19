use super::*;
impl GridWake {
    pub(super) fn new() -> Self {
        Self {
            inner: Arc::new(GridWakeInner {
                state: Mutex::new(GridWakeState {
                    generation: 0,
                    interactive_budget: 0,
                }),
                changed: Condvar::new(),
            }),
        }
    }

    pub(super) fn notify(&self) {
        let mut state = self.inner.state.lock().expect("grid wake");
        state.generation = state.generation.saturating_add(1);
        self.inner.changed.notify_all();
    }

    pub(super) fn prioritize_interactive_changes(&self) {
        let mut state = self.inner.state.lock().expect("grid wake");
        state.interactive_budget = INTERACTIVE_GRID_BUDGET;
        self.inner.changed.notify_all();
    }

    pub(crate) fn consume_interactive_priority(&self) {
        let mut state = self.inner.state.lock().expect("grid wake");
        state.interactive_budget = state.interactive_budget.saturating_sub(1);
    }

    pub(crate) fn generation(&self) -> u64 {
        self.inner.state.lock().expect("grid wake").generation
    }

    pub(crate) fn wait_for_change(&self, observed: u64, timeout: Duration) -> GridWakeEvent {
        let state = self.inner.state.lock().expect("grid wake");
        if state.generation != observed {
            return grid_wake_event(&state, observed);
        }
        let (state, _) = self
            .inner
            .changed
            .wait_timeout_while(state, timeout, |state| state.generation == observed)
            .expect("grid wake");
        grid_wake_event(&state, observed)
    }

    pub(crate) fn wait_for_priority_or_timeout(
        &self,
        observed: u64,
        timeout: Duration,
    ) -> GridWakeEvent {
        let state = self.inner.state.lock().expect("grid wake");
        let (state, _) = self
            .inner
            .changed
            .wait_timeout_while(state, timeout, |state| {
                state.interactive_budget == 0 || state.generation == observed
            })
            .expect("grid wake");
        grid_wake_event(&state, observed)
    }

    pub(crate) fn same_source(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }
}

fn grid_wake_event(state: &GridWakeState, observed: u64) -> GridWakeEvent {
    GridWakeEvent {
        generation: state.generation,
        interactive: state.interactive_budget > 0 && state.generation != observed,
    }
}

impl Session {
    pub fn view(&self) -> SessionView {
        let prompt_title = self
            .shared
            .prompt_title
            .lock()
            .expect("prompt title")
            .clone();
        let (title, title_source) = if let Some(title) = prompt_title {
            (Some(title), Some(homie_proto::TitleSource::FirstPrompt))
        } else {
            (
                self.shared.title.lock().expect("title").clone(),
                Some(homie_proto::TitleSource::AgentProvided),
            )
        };
        SessionView {
            id: self.shared.id.clone(),
            status: self.shared.status.lock().expect("status").clone(),
            needs_input: self.shared.needs_input.lock().expect("needs input").clone(),
            title,
            title_source,
            tail_offset: self.shared.log.lock().expect("log").tail_offset(),
            exited: self.shared.exited.load(Ordering::SeqCst),
        }
    }

    /// The visible screen, as detection sees it.
    pub fn screen_lines(&self) -> Vec<String> {
        self.shared.screen.lock().expect("screen").lines()
    }

    /// The emulator's current geometry.
    /// URLs the screen has shown, for the artifacts inspector.
    pub fn artifacts(&self) -> Vec<homie_proto::SessionArtifact> {
        self.shared.artifacts.lock().expect("artifacts").clone()
    }

    pub fn screen_size(&self) -> (usize, usize) {
        if let Some(remote) = self
            .shared
            .remote_grid
            .lock()
            .expect("remote grid")
            .as_ref()
            && remote.mirror.sequence().is_some()
        {
            let (cols, rows) = remote.mirror.size();
            return (usize::from(cols), usize::from(rows));
        }
        self.shared.screen.lock().expect("screen").size()
    }

    /// A coherent full snapshot and change-generation baseline for a freshly
    /// attached sink. Sampling the generation on both sides closes the race
    /// where output lands between the seed and pump registration.
    pub(crate) fn attachment_seed(&self) -> AttachmentSeed {
        self.shared.note_hot();
        let wake = self.shared.grid_wake.clone();
        loop {
            let wake_generation = wake.generation();
            let sampled = {
                let remote = self.shared.remote_grid.lock().expect("remote grid");
                remote.as_ref().and_then(|remote| {
                    let grid = remote.mirror.full_update()?;
                    let (cols, rows) = remote.mirror.size();
                    let (cursor_col, cursor_row, cursor_visible) = remote.mirror.cursor();
                    let (alt_screen, _, mouse_reporting) = remote.mirror.modes();
                    Some((
                        grid,
                        (alt_screen, mouse_reporting),
                        GridSignature {
                            content_seq: remote.revision,
                            size: (usize::from(cols), usize::from(rows)),
                            cursor: (cursor_col, cursor_row, cursor_visible),
                            alt_screen,
                            mouse_reporting,
                        },
                    ))
                })
            }
            .unwrap_or_else(|| {
                let screen = self.shared.screen.lock().expect("screen");
                (
                    screen.full_snapshot(),
                    (screen.is_alt_screen(), screen.mouse_reporting()),
                    GridSignature {
                        content_seq: screen.content_seq(),
                        size: screen.size(),
                        cursor: screen.cursor(),
                        alt_screen: screen.is_alt_screen(),
                        mouse_reporting: screen.mouse_reporting(),
                    },
                )
            });
            if wake.generation() == wake_generation {
                return AttachmentSeed {
                    grid: sampled.0,
                    modes: sampled.1,
                    signature: sampled.2,
                    wake,
                    wake_generation,
                };
            }
        }
    }

    /// The next grid diff after a [`GridWake`] notification, if anything
    /// observable changed since `signature`.
    pub fn grid_update_if_changed(
        &self,
        signature: &mut GridSignature,
    ) -> Option<homie_proto::grid::GridUpdate> {
        if let Some(remote) = self
            .shared
            .remote_grid
            .lock()
            .expect("remote grid")
            .as_mut()
            && remote.mirror.sequence().is_some()
        {
            let (cols, rows) = remote.mirror.size();
            let (cursor_col, cursor_row, cursor_visible) = remote.mirror.cursor();
            let (alt_screen, _, mouse_reporting) = remote.mirror.modes();
            let current = GridSignature {
                content_seq: remote.revision,
                size: (usize::from(cols), usize::from(rows)),
                cursor: (cursor_col, cursor_row, cursor_visible),
                alt_screen,
                mouse_reporting,
            };
            if current == *signature {
                return None;
            }
            *signature = current;
            return remote
                .pending
                .take()
                .or_else(|| remote.mirror.full_update());
        }
        let mut screen = self.shared.screen.lock().expect("screen");
        let current = GridSignature {
            content_seq: screen.content_seq(),
            size: screen.size(),
            cursor: screen.cursor(),
            alt_screen: screen.is_alt_screen(),
            mouse_reporting: screen.mouse_reporting(),
        };
        if current == *signature {
            return None;
        }
        *signature = current;
        Some(screen.grid_update(false))
    }

    pub(crate) fn grid_wake(&self) -> GridWake {
        self.shared.grid_wake.clone()
    }

    /// Whether the child has bracketed-paste mode on — the "composer is
    /// alive" tell that gates initial-prompt injection.
    pub fn bracketed_paste(&self) -> bool {
        if let Some(remote) = self
            .shared
            .remote_grid
            .lock()
            .expect("remote grid")
            .as_ref()
            && remote.mirror.sequence().is_some()
        {
            return remote.mirror.modes().1;
        }
        self.shared.screen.lock().expect("screen").bracketed_paste()
    }

    /// Current (alt_screen, mouse_reporting).
    pub fn modes(&self) -> (bool, bool) {
        if let Some(remote) = self
            .shared
            .remote_grid
            .lock()
            .expect("remote grid")
            .as_ref()
            && remote.mirror.sequence().is_some()
        {
            let (alt_screen, _, mouse_reporting) = remote.mirror.modes();
            return (alt_screen, mouse_reporting);
        }
        let screen = self.shared.screen.lock().expect("screen");
        (screen.is_alt_screen(), screen.mouse_reporting())
    }

    /// A wheel event from an attached client: forwarded to the child when it
    /// asked for mouse reporting, otherwise ignored (the client scrolls its
    /// own scrollback).
    pub fn scroll(&self, up: bool, lines: usize, col: usize, row: usize) -> std::io::Result<()> {
        let bytes = self
            .shared
            .screen
            .lock()
            .expect("screen")
            .mouse_wheel(up, lines, col, row);
        if bytes.is_empty() {
            return Ok(());
        }
        // Raw: a wheel is not a keystroke, and must not look like user typing
        // to the status reducer.
        self.write_raw(&bytes)
    }

    pub fn read_scrollback(&self) -> homie_proto::ReadScrollbackResult {
        self.shared.screen.lock().expect("screen").scrollback()
    }

    pub fn read_scrollback_cells(
        &self,
        first_row: i64,
        max_rows: i64,
    ) -> homie_proto::ReadScrollbackCellsResult {
        if let Transport::Remote(client) = &self.transport
            && let Ok(result) = client.read_scrollback_cells(first_row, max_rows)
        {
            return result;
        }
        self.shared
            .screen
            .lock()
            .expect("screen")
            .scrollback_cells(first_row, max_rows)
    }
}

#[cfg(test)]
mod grid_wake_tests {
    use std::time::Duration;

    use super::GridWake;

    #[test]
    fn grid_waiter_sleeps_until_a_real_change_and_coalesces_generations() {
        let wake = GridWake::new();
        let observed = wake.generation();
        let notifier = wake.clone();
        let thread = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(20));
            notifier.notify();
            notifier.notify();
        });

        let changed = wake.wait_for_change(observed, Duration::from_secs(1));
        thread.join().expect("notifier");
        assert!(changed.generation > observed);
        assert!(!changed.interactive);
        assert_eq!(
            wake.wait_for_change(changed.generation, Duration::from_millis(5)),
            changed
        );
    }

    #[test]
    fn interactive_priority_covers_two_grid_changes_then_expires() {
        let wake = GridWake::new();
        let observed = wake.generation();
        wake.prioritize_interactive_changes();

        let unchanged = wake.wait_for_priority_or_timeout(observed, Duration::from_millis(1));
        assert_eq!(unchanged.generation, observed);
        assert!(!unchanged.interactive);

        wake.notify();
        let changed = wake.wait_for_priority_or_timeout(observed, Duration::from_secs(1));
        assert!(changed.generation > observed);
        assert!(changed.interactive);

        wake.consume_interactive_priority();
        wake.notify();
        let trailing = wake.wait_for_change(changed.generation, Duration::from_secs(1));
        assert!(trailing.interactive);

        wake.consume_interactive_priority();
        wake.notify();
        let background = wake.wait_for_change(trailing.generation, Duration::from_secs(1));
        assert!(!background.interactive);
    }
}
