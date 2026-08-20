use super::*;

impl RootView {
    pub(crate) fn colors(&self) -> SemanticColors {
        let store = self
            .services
            .store
            .store
            .read()
            .expect("session store lock poisoned");
        crate::app_theme::colors(&store.preferences().terminal_theme)
    }

    pub(crate) fn on_key_down(
        &mut self,
        event: &KeyDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.launcher.read(cx).is_open() {
            let reopen = event.keystroke.modifiers.platform
                && event.keystroke.key == "n"
                && !event.keystroke.modifiers.shift;
            self.launcher.update(cx, |launcher, cx| {
                launcher.handle_key_down(event, _window, cx);
            });
            if !reopen {
                cx.stop_propagation();
            }
            return;
        }
        if let Some(surfaces) = &self.utility_surfaces
            && surfaces.read(cx).is_open()
        {
            let global_overlay_shortcut = event.keystroke.modifiers.platform
                && matches!(event.keystroke.key.as_str(), "h" | "," | "k" | "p");
            if !global_overlay_shortcut {
                surfaces.update(cx, |surfaces, cx| {
                    surfaces.key_down(event, _window, cx);
                });
                cx.stop_propagation();
                return;
            }
        }
        if let Some(surfaces) = &self.session_surfaces {
            surfaces.update(cx, |surfaces, cx| {
                surfaces.handle_key_down(event, _window, cx);
            });
        }
        if !event.keystroke.modifiers.platform {
            return;
        }
        match event.keystroke.key.as_str() {
            "k" => {
                if let Some(navigation) = &self.navigation {
                    navigation.update(cx, |navigation, cx| {
                        navigation.toggle_command_palette(&ToggleCommandPalette, _window, cx);
                    });
                }
            }
            "p" => {
                if let Some(navigation) = &self.navigation {
                    navigation.update(cx, |navigation, cx| {
                        navigation.toggle_quick_open(&ToggleQuickOpen, _window, cx);
                    });
                }
            }
            "h" if event.keystroke.modifiers.shift => {
                if let Some(surfaces) = &self.utility_surfaces {
                    surfaces.update(cx, |surfaces, cx| surfaces.toggle_history(cx));
                }
            }
            "," => {
                if let Some(surfaces) = &self.utility_surfaces {
                    surfaces.update(cx, |surfaces, cx| surfaces.open_settings(cx));
                }
            }
            "b" => self.sidebar.update(cx, |sidebar, cx| sidebar.toggle(cx)),
            "d" if event.keystroke.modifiers.shift => self.toggle_inspector(cx),
            key @ ("t" | "n") => match new_session_shortcut(key, event.keystroke.modifiers) {
                Some(NewSessionShortcut::Default) => {
                    if !self.spawn_default() {
                        return;
                    }
                }
                Some(NewSessionShortcut::Shell) => {
                    if !self.spawn(None) {
                        return;
                    }
                }
                Some(NewSessionShortcut::Codex) => {
                    if !self.spawn(Some(DefaultAgent::Codex)) {
                        return;
                    }
                }
                None => return,
            },
            // ⌥⌘W: worktrees overview. ⌘⇧W archives the selected session;
            // plain ⌘W is bound globally to CloseSession.
            "w" if event.keystroke.modifiers.alt => {
                if let Some(surfaces) = &self.utility_surfaces {
                    surfaces.update(cx, |surfaces, cx| surfaces.open_worktrees(cx));
                } else {
                    return;
                }
            }
            "w" if event.keystroke.modifiers.shift => {
                let archived = self
                    .sidebar
                    .update(cx, |sidebar, cx| sidebar.archive_selected(cx));
                if !archived {
                    return;
                }
            }
            "r" if !event.keystroke.modifiers.shift => {
                let renaming = self
                    .sidebar
                    .update(cx, |sidebar, cx| sidebar.rename_selected(_window, cx));
                if !renaming {
                    return;
                }
            }
            // ⇧⌘J retains the attention-navigation command that previously
            // occupied plain ⌘J.
            "j" if event.keystroke.modifiers.shift => {
                let selected = self
                    .sidebar
                    .update(cx, |sidebar, cx| sidebar.select_next_needing_input(cx));
                if !selected {
                    return;
                }
            }
            // ⌘J opens (or focuses) a terminal owned by the selected agent's
            // workbench, below the primary pane.
            "j" => {
                if !self.open_auxiliary_terminal(_window, cx) {
                    return;
                }
            }
            digit @ ("1" | "2" | "3" | "4" | "5" | "6" | "7" | "8") => {
                // ⌘1–⌘8 select the nth session, matching the sidebar's row
                // hints; selection also focuses the terminal via
                // SessionActivated.
                let index = (digit.as_bytes()[0] - b'1') as usize;
                let selected = self
                    .sidebar
                    .update(cx, |sidebar, cx| sidebar.select_shortcut(index, cx));
                if !selected {
                    return;
                }
            }
            // ⌘9 jumps to the last session, the browser convention.
            "9" => {
                let selected = self
                    .sidebar
                    .update(cx, |sidebar, cx| sidebar.select_last(cx));
                if !selected {
                    return;
                }
            }
            // ⌃⌘↑/⌃⌘↓ move the selected row within its project group.
            "up" | "down" if event.keystroke.modifiers.control && !self.arrow_surface_visible() => {
                let delta = if event.keystroke.key == "up" { -1 } else { 1 };
                let moved = self
                    .sidebar
                    .update(cx, |sidebar, cx| sidebar.reorder_selected(delta, cx));
                if !moved {
                    return;
                }
            }
            // The explicit session-navigation shortcut steps through sidebar
            // order, wrapping. The switcher and overview own arrows while open.
            key if session_navigation_delta(
                key,
                event.keystroke.modifiers,
                self.arrow_surface_visible(),
            )
            .is_some() =>
            {
                let delta = session_navigation_delta(
                    key,
                    event.keystroke.modifiers,
                    self.arrow_surface_visible(),
                )
                .expect("guard checked navigation shortcut");
                let selected = self
                    .sidebar
                    .update(cx, |sidebar, cx| sidebar.select_relative(delta, cx));
                if !selected {
                    return;
                }
            }
            _ => return,
        }
        cx.stop_propagation();
    }

    pub(crate) fn on_key_up(
        &mut self,
        event: &KeyUpEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(surfaces) = &self.session_surfaces {
            surfaces.update(cx, |surfaces, cx| {
                surfaces.handle_key_up(event, window, cx);
            });
        }
    }

    pub(crate) fn on_modifiers_changed(
        &mut self,
        event: &ModifiersChangedEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(surfaces) = &self.session_surfaces {
            surfaces.update(cx, |surfaces, cx| {
                surfaces.handle_modifiers_changed(event, window, cx);
            });
        }
    }

    pub(crate) fn open_launcher(
        &mut self,
        _: &OpenLauncher,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.launcher
            .update(cx, |launcher, cx| launcher.open(window, cx));
        // Opening changes which main-pane branch RootView renders.
        cx.notify();
        // The launcher was not mounted while the terminal branch was active.
        // Focus it on the next frame, after GPUI has installed its focus node.
        let launcher = self.launcher.clone();
        cx.defer_in(window, move |_, window, cx| {
            launcher.update(cx, |launcher, cx| launcher.focus(window, cx));
        });
    }
}
