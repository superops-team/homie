use super::*;

impl TerminalPane {
    pub(super) fn handle_key_down(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if event.keystroke.modifiers.platform {
            let handled = match event.keystroke.key.as_str() {
                "k" => self.navigation.as_ref().is_some_and(|navigation| {
                    navigation.update(cx, |navigation, cx| {
                        navigation.toggle_command_palette(&ToggleCommandPalette, window, cx);
                    });
                    true
                }),
                "p" => self.navigation.as_ref().is_some_and(|navigation| {
                    navigation.update(cx, |navigation, cx| {
                        navigation.toggle_quick_open(&ToggleQuickOpen, window, cx);
                    });
                    true
                }),
                "h" if event.keystroke.modifiers.shift => {
                    self.utility_surfaces.as_ref().is_some_and(|surfaces| {
                        surfaces.update(cx, |surfaces, cx| surfaces.toggle_history(cx));
                        true
                    })
                }
                "," => self.utility_surfaces.as_ref().is_some_and(|surfaces| {
                    surfaces.update(cx, |surfaces, cx| surfaces.open_settings(cx));
                    true
                }),
                "o" if event.keystroke.modifiers.shift => {
                    self.runtime
                        .store
                        .write()
                        .expect("session store lock poisoned")
                        .toggle_overview();
                    true
                }
                _ => false,
            };
            if handled {
                cx.stop_propagation();
                cx.notify();
                return;
            }
        }

        if let Some(navigation) = &self.navigation
            && navigation.read(cx).is_open()
        {
            navigation.update(cx, |navigation, cx| {
                navigation.on_key_down(event, window, cx);
            });
            cx.stop_propagation();
            return;
        }
        if let Some(surfaces) = &self.utility_surfaces
            && surfaces.read(cx).is_open()
        {
            surfaces.update(cx, |surfaces, cx| {
                surfaces.key_down(event, window, cx);
            });
            cx.stop_propagation();
            return;
        }

        let switcher_key = switcher_key(event);
        let switcher_handled = {
            let mut store = self
                .runtime
                .store
                .write()
                .expect("session store lock poisoned");
            let was_visible = store.switcher_state().is_visible();
            let handled = if was_visible
                || matches!(
                    switcher_key,
                    crate::switcher::SwitcherKey::Tab { control: true, .. }
                ) {
                store.handle_switcher_key(switcher_key)
            } else {
                false
            };
            if handled && !was_visible && store.switcher_state().is_visible() {
                store.dismiss_overview();
            }
            handled
        };
        if switcher_handled {
            cx.stop_propagation();
            cx.notify();
            return;
        }

        let Some(id) = self.selected_id() else {
            return;
        };
        let now = self.started_at.elapsed();
        let Some(resident) = self.residents.get_mut(&id) else {
            return;
        };

        if let Some(find) = resident.find.as_mut() {
            match event.keystroke.key.as_str() {
                "escape" => {
                    resident.find = None;
                    resident.element.set_find_highlights(Vec::new());
                    cx.notify();
                }
                "enter" => {
                    if event.keystroke.modifiers.shift {
                        resident.element.find_previous(find);
                    } else {
                        resident.element.find_next(find);
                    }
                    resident.element.sync_find_highlights(find);
                    cx.notify();
                }
                // Everything else is text editing, through the same key map the
                // command palette and Quick Open use.
                _ => {
                    let Some(edit) = query_editor::edit_for(&event.keystroke) else {
                        cx.propagate();
                        return;
                    };
                    let changed = match edit {
                        Edit::Local(local) => resident.find_query.apply(local),
                        Edit::Clipboard(ClipboardEdit::Copy) => {
                            query_editor::copy_selection(&resident.find_query, cx);
                            false
                        }
                        Edit::Clipboard(ClipboardEdit::Cut) => {
                            query_editor::cut_selection(&mut resident.find_query, cx)
                        }
                        // ⌘V is already an action (it also handles image
                        // pastes); claiming it here too would insert twice.
                        Edit::Clipboard(ClipboardEdit::Paste) => {
                            cx.propagate();
                            return;
                        }
                    };
                    if changed {
                        let query = resident.find_query.text().to_owned();
                        find.set_query(query, now);
                        self.schedule_find(id, Duration::from_millis(200), window, cx);
                    }
                }
            }
            cx.stop_propagation();
            cx.notify();
            return;
        }

        if event.keystroke.modifiers.platform && event.keystroke.key != "backspace" {
            cx.propagate();
            return;
        }
        let Some(term_event) = terminal_key_event(event) else {
            cx.propagate();
            return;
        };
        let modifiers = TermModifiers {
            shift: event.keystroke.modifiers.shift,
            ctrl: event.keystroke.modifiers.control,
            alt: event.keystroke.modifiers.alt,
            cmd: event.keystroke.modifiers.platform,
        };
        let bytes = encode_key(&term_event, modifiers, TermInputModes::default());
        if bytes.is_empty() {
            cx.propagate();
        } else {
            resident.attachment.input(bytes);
            cx.stop_propagation();
        }
    }

    pub(super) fn handle_key_up(
        &mut self,
        event: &KeyUpEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if matches!(event.keystroke.key.as_str(), "control" | "ctrl") {
            self.runtime
                .store
                .write()
                .expect("session store lock poisoned")
                .handle_switcher_modifiers_changed(false);
            cx.notify();
        }
    }

    pub(super) fn handle_modifiers_changed(
        &mut self,
        event: &ModifiersChangedEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let mut store = self
            .runtime
            .store
            .write()
            .expect("session store lock poisoned");
        let was_visible = store.switcher_state().is_visible();
        store.handle_switcher_modifiers_changed(event.modifiers.control);
        if was_visible != store.switcher_state().is_visible() {
            cx.notify();
        }
    }
}
