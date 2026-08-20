use super::*;

impl RootView {
    pub(crate) fn spawn(&self, agent: Option<DefaultAgent>) -> bool {
        if self.preview {
            return false;
        }
        let mut store = self
            .services
            .store
            .store
            .write()
            .expect("session store lock poisoned");
        match agent {
            Some(agent) => {
                let host = store.default_spawn_host();
                store.spawn_kind(
                    agent.kind(),
                    SpawnOptions {
                        host,
                        ..SpawnOptions::default()
                    },
                );
            }
            None => store.spawn_shell(SpawnOptions::default()),
        }
        true
    }

    pub(crate) fn spawn_default(&self) -> bool {
        if self.preview {
            return false;
        }
        self.services
            .store
            .store
            .write()
            .expect("session store lock poisoned")
            .spawn_default(SpawnOptions::default());
        true
    }

    pub(crate) fn arrow_surface_visible(&self) -> bool {
        let store = self
            .services
            .store
            .store
            .read()
            .expect("session store lock poisoned");
        store.switcher_state().is_visible() || store.overview_state().is_visible()
    }

    pub(crate) fn close_selected_session(
        &mut self,
        _: &CloseSession,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self
            .auxiliary_terminal
            .as_ref()
            .is_some_and(|terminal| terminal.read(cx).is_focused(window))
            && let Some(id) = self.auxiliary_id.clone()
        {
            self.services
                .store
                .store
                .write()
                .expect("session store lock poisoned")
                .remove_sessions(vec![id]);
            if let Some(terminal) = &self.terminal {
                terminal.update(cx, |terminal, cx| terminal.focus(window, cx));
            }
            return;
        }
        let closed = self
            .sidebar
            .update(cx, |sidebar, cx| sidebar.close_selected_now(cx));
        if !closed {
            cx.propagate();
        }
    }

    pub(crate) fn reopen_last_session(
        &mut self,
        _: &ReopenSession,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.sidebar
            .update(cx, |sidebar, cx| sidebar.reopen_last(cx));
    }
}
