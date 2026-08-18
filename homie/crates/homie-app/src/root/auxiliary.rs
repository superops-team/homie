use super::*;

impl RootView {
    pub(crate) fn open_auxiliary_terminal(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if self.preview {
            return false;
        }
        self.sync_auxiliary_terminal(window, cx);
        let selected = self
            .services
            .store
            .store
            .read()
            .expect("session store lock poisoned")
            .selected_session_id()
            .cloned();
        let Some(parent) = selected else {
            return false;
        };
        if self.auxiliary_parent.as_ref() == Some(&parent) && self.auxiliary_terminal.is_some() {
            self.collapsed_auxiliary_parents.insert(parent);
            self.auxiliary_terminal = None;
            self.auxiliary_id = None;
            self.auxiliary_parent = None;
            if let Some(primary) = &self.terminal {
                primary.update(cx, |terminal, cx| terminal.focus(window, cx));
            }
            cx.notify();
            return true;
        }
        if self.collapsed_auxiliary_parents.remove(&parent) {
            self.sync_auxiliary_terminal(window, cx);
            if let Some(terminal) = &self.auxiliary_terminal {
                terminal.update(cx, |terminal, cx| terminal.focus(window, cx));
                return true;
            }
        }
        if self.auxiliary_spawn_parent.as_ref() == Some(&parent) {
            return true;
        }
        let spawned = self
            .services
            .store
            .store
            .write()
            .expect("session store lock poisoned")
            .spawn_auxiliary_terminal(parent.clone());
        if spawned {
            self.auxiliary_spawn_parent = Some(parent);
            cx.notify();
        }
        spawned
    }

    /// Reconciles the UI-owned pane entity with the daemon-owned child shell.
    /// The relationship survives app restarts because it lives in the session
    /// record; the GPUI entity remains disposable rendering state.
    pub(crate) fn sync_auxiliary_terminal(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.preview {
            return;
        }
        let (selected, auxiliary, spawn_failed) = {
            let store = self
                .services
                .store
                .store
                .read()
                .expect("session store lock poisoned");
            let selected = store.selected_session_id().cloned();
            let auxiliary = selected
                .as_ref()
                .and_then(|parent| store.auxiliary_terminal_for(parent));
            (selected, auxiliary, store.last_action_error().is_some())
        };

        if selected
            .as_ref()
            .is_some_and(|parent| self.collapsed_auxiliary_parents.contains(parent))
        {
            // Collapsing a pane is UI-only: keep its daemon shell alive so
            // the next ⌘J restores the same scrollback and process state.
            self.auxiliary_terminal = None;
            self.auxiliary_id = None;
            self.auxiliary_parent = None;
            return;
        }

        if let Some(session) = auxiliary {
            let parent = session
                .parent
                .clone()
                .expect("auxiliary terminal has an owning session");
            if self.auxiliary_id.as_ref() == Some(&session.id)
                && self.auxiliary_parent.as_ref() == Some(&parent)
            {
                self.auxiliary_spawn_parent = None;
                return;
            }

            let runtime = Arc::clone(&self.services.store);
            let tokio = Arc::clone(&self.services.tokio);
            let id = session.id.clone();
            let terminal =
                cx.new(|cx| TerminalPane::new_fixed(runtime, tokio, id.clone(), window, cx));
            if let (Some(navigation), Some(utility_surfaces)) =
                (&self.navigation, &self.utility_surfaces)
            {
                terminal.update(cx, |terminal, _| {
                    terminal.set_shell_entities(navigation.clone(), utility_surfaces.clone());
                });
            }
            let should_focus = self.auxiliary_spawn_parent.as_ref() == Some(&parent);
            self.auxiliary_id = Some(session.id.clone());
            self.auxiliary_parent = Some(parent);
            self.auxiliary_terminal = Some(terminal.clone());
            self.auxiliary_spawn_parent = None;
            if should_focus {
                terminal.update(cx, |terminal, cx| terminal.focus(window, cx));
            }
            cx.notify();
            return;
        }

        let spawn_still_pending = selected
            .as_ref()
            .is_some_and(|selected| self.auxiliary_spawn_parent.as_ref() == Some(selected))
            && !spawn_failed;
        if spawn_still_pending {
            return;
        }
        let had_auxiliary_state = self.auxiliary_terminal.is_some()
            || self.auxiliary_id.is_some()
            || self.auxiliary_parent.is_some()
            || self.auxiliary_spawn_parent.is_some();
        self.auxiliary_terminal = None;
        self.auxiliary_id = None;
        self.auxiliary_parent = None;
        self.auxiliary_spawn_parent = None;
        if had_auxiliary_state {
            cx.notify();
        }
    }
}
