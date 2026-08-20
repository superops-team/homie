use super::*;

impl TerminalPane {
    pub(super) fn handle_pane_event(
        &mut self,
        event: PaneEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match event {
            PaneEvent::InteractiveInput => self.repaint_pacer.prioritize_interactive_damage(),
            PaneEvent::AttachmentState(id, state) => {
                if let Some(resident) = self.residents.get_mut(&id) {
                    resident.attachment_state = state;
                }
                if self.selected_id().as_ref() == Some(&id) {
                    cx.notify();
                }
            }
            PaneEvent::Chunk(id, TerminalChunk::Grid(update)) => {
                if let Some(hold) = self.reflow_holds.get_mut(&id) {
                    if hold.park(update) {
                        self.release_reflow_hold(&id, window, cx);
                    }
                    return;
                }
                self.apply_grid_updates(id, [update], window, cx);
            }
            PaneEvent::Chunk(
                id,
                TerminalChunk::Modes {
                    alt_screen,
                    mouse_reporting,
                },
            ) => {
                if let Some(resident) = self.residents.get_mut(&id) {
                    resident.element.set_modes(alt_screen, mouse_reporting);
                }
                if self.selected_id().as_ref() == Some(&id) {
                    cx.notify();
                }
            }
            PaneEvent::Chunk(_, TerminalChunk::Pong) => {}
            PaneEvent::FindSnapshot(id, request, snapshot) => {
                let visible = self.selected_id().as_ref() == Some(&id);
                if let Some(resident) = self.residents.get_mut(&id)
                    && let Some(find) = resident.find.as_mut()
                    && resident
                        .element
                        .apply_find_snapshot(find, &request, snapshot)
                {
                    resident.element.sync_find_highlights(find);
                    if visible {
                        cx.notify();
                    }
                }
            }
            PaneEvent::ScrollbackCells(id, result, visible_rows) => {
                if let Some(resident) = self.residents.get_mut(&id) {
                    let _ = resident
                        .element
                        .complete_scrollback_fetch(result, visible_rows);
                }
                self.pump_scrollback_fetch(&id, visible_rows);
                if self.selected_id().as_ref() == Some(&id) {
                    cx.notify();
                }
            }
            PaneEvent::ScrollbackFailed(id) => {
                if let Some(resident) = self.residents.get_mut(&id) {
                    resident.element.fail_scrollback_fetch();
                }
                if self.selected_id().as_ref() == Some(&id) {
                    cx.notify();
                }
            }
            PaneEvent::ClipboardUploadFinished(id, result) => match result {
                Ok(remote_path) => {
                    if let Some(resident) = self.residents.get(&id) {
                        resident.attachment.input(paste(&remote_path, false));
                    }
                }
                Err(error) => eprintln!("homie: clipboard image upload failed: {error}"),
            },
        }
    }

    /// Applies grid frames to a resident and repaints if what landed is worth a
    /// frame. Takes a batch because a held reflow releases its parked frames
    /// together: applying them one by one would paint each intermediate.
    fn apply_grid_updates(
        &mut self,
        id: SessionId,
        updates: impl IntoIterator<Item = GridUpdate>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let now = self.started_at.elapsed();
        let selected = self.selected_id();
        let mut schedule_find = false;
        let mut changed = false;
        let mut applied = false;
        if let Some(resident) = self.residents.get_mut(&id) {
            for update in updates {
                applied = true;
                changed |= resident.element.apply_damage(update).changed;
            }
            if applied && let Some(find) = resident.find.as_mut() {
                schedule_find = find.on_output(now);
            }
        }
        if !applied {
            return;
        }
        let repaint = terminal_damage_should_repaint(
            window.is_window_active(),
            selected.as_ref(),
            &id,
            changed,
        );
        if schedule_find {
            self.schedule_find(id, Duration::from_millis(100), window, cx);
        }
        if repaint {
            self.request_terminal_repaint(window, cx);
        }
    }

    /// Holds a session's grid still until its column change has fully
    /// round-tripped. A hold already in flight is extended rather than
    /// released, so a second change landing mid-hold covers its own reflow too;
    /// its frames carry over, because a daemon that never answers the second
    /// resize (a hibernated tree, a session the phone owns) would otherwise
    /// leave the pane painting whatever was on screen before the first one.
    pub(super) fn hold_reflow(
        &mut self,
        id: SessionId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let held = id.clone();
        let release = cx.spawn_in(window, async move |this, cx| {
            cx.background_executor().timer(REFLOW_HOLD).await;
            let _ = this.update_in(cx, |this, window, cx| {
                this.release_reflow_hold(&held, window, cx);
            });
        });
        let parked = self
            .reflow_holds
            .remove(&id)
            .map_or_else(Vec::new, |hold| hold.parked);
        self.reflow_holds.insert(
            id,
            ReflowHold {
                parked,
                saw_snapshot: false,
                _release: release,
            },
        );
    }

    /// Ends a hold and paints everything it parked as a single frame.
    fn release_reflow_hold(&mut self, id: &SessionId, window: &mut Window, cx: &mut Context<Self>) {
        let Some(hold) = self.reflow_holds.remove(id) else {
            return;
        };
        self.apply_grid_updates(id.clone(), hold.parked, window, cx);
    }

    fn request_terminal_repaint(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        match self.repaint_pacer.on_damage(self.started_at.elapsed()) {
            RepaintAction::RepaintNow => cx.notify(),
            RepaintAction::Schedule(delay) => {
                cx.spawn_in(window, async move |this, cx| {
                    cx.background_executor().timer(delay).await;
                    let _ = this.update_in(cx, |this, _window, cx| {
                        if this.repaint_pacer.on_timer(this.started_at.elapsed()) {
                            cx.notify();
                        }
                    });
                })
                .detach();
            }
            RepaintAction::None => {}
        }
    }
}
