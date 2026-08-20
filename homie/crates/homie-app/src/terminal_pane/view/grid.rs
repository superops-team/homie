use super::buttons::*;
use super::*;

impl TerminalPane {
    pub(crate) fn render_grid_and_overlays(
        &mut self,
        session: &SessionRecord,
        theme: TermTheme,
        font_size: f32,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        if session.is_archived() {
            return self.render_archived_overlay(session, cx);
        }
        let exited = matches!(session.status, SessionStatus::Exited(_));
        // An exited agent leaves its last screen behind in the daemon, and that
        // output is exactly what people want to read after closing an agent --
        // so only take the pane over when there is no terminal left to show.
        if exited && let Some(takeover) = self.render_exited_takeover(session, cx) {
            return takeover;
        }
        let Some(resident) = self.residents.get(&session.id) else {
            return centered_message("Preparing terminal…", "").into_any_element();
        };
        let element = resident
            .element
            .clone()
            .theme(theme)
            .font_size(px(font_size))
            .focus_handle(self.focus.clone());
        let view_offset = resident.element.view_offset();
        let attachment_state = resident.attachment_state;
        let overflow = self.grid_row_overflow(resident.element.grid_rows(), font_size, window);

        let id_for_focus = session.id.clone();
        let follows_selection = matches!(self.session_source, SessionSource::FollowSelection);
        let mut body = div()
            .relative()
            .flex_1()
            .overflow_hidden()
            .pt(px(2.0))
            .pb(px(10.0))
            .px(px(12.0))
            .bg(theme.background)
            .track_focus(&self.focus)
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, event: &gpui::MouseDownEvent, window, cx| {
                    window.focus(&this.focus, cx);
                    if follows_selection {
                        this.runtime
                            .store
                            .write()
                            .expect("session store lock poisoned")
                            .select(id_for_focus.clone());
                    }
                    let Some(id) = this.selected_id() else {
                        return;
                    };
                    let Some((col, row)) = this.grid_cell_at(event.position, window) else {
                        return;
                    };
                    let Some(resident) = this.residents.get(&id) else {
                        return;
                    };
                    if event.modifiers.platform {
                        match resident.element.reference_at(col, row) {
                            Some(TerminalReference::Url(url)) => cx.open_url(&url),
                            Some(TerminalReference::File(reference)) => {
                                let Some(session) = this.selected_session() else {
                                    return;
                                };
                                cx.emit(TerminalPaneEvent::OpenFileReference {
                                    reference,
                                    cwd: session.cwd.clone(),
                                    session_id: session.id.clone(),
                                });
                            }
                            None => {}
                        }
                        return;
                    }
                    // Mouse-reporting programs (Claude Code, vim) would
                    // normally own the pointer, but clicks are not forwarded
                    // to the PTY yet -- suppressing local selection here
                    // bought nothing and made text un-copyable. Revisit when
                    // click forwarding lands (then: plain drag to the app,
                    // option-drag for local selection, per terminal
                    // convention).
                    match event.click_count {
                        1 => resident.element.begin_selection(col, row),
                        _ => resident.element.select_word(col, row),
                    }
                    // notify, never window.refresh(): refresh() flags the
                    // whole frame as caching-disabled, repainting every cached
                    // view at pointer-event rate.
                    cx.notify();
                }),
            )
            .on_mouse_move(
                cx.listener(|this, event: &gpui::MouseMoveEvent, window, cx| {
                    if event.pressed_button != Some(MouseButton::Left) {
                        return;
                    }
                    let Some(id) = this.selected_id() else {
                        return;
                    };
                    let Some((col, row)) = this.grid_cell_at(event.position, window) else {
                        return;
                    };
                    let Some(resident) = this.residents.get(&id) else {
                        return;
                    };
                    resident.element.drag_selection(col, row);
                    cx.notify();
                }),
            )
            .on_scroll_wheel(cx.listener(Self::handle_scroll))
            .child(match overflow {
                // Settled: the mirrored screen fits, so the grid fills the pane
                // exactly as before.
                None => div().size_full().child(element),
                // The daemon's screen is still taller than the pane -- a shrink
                // that has not round-tripped yet. Give the grid its natural
                // height, bottom-anchored: the extra rows clip off the top, the
                // way a terminal drops scrollback, instead of the prompt and the
                // agent's input box vanishing off the bottom until the reflow
                // lands. Collapses back to the branch above on the next frame.
                Some(grid_height) => div().size_full().relative().overflow_hidden().child(
                    div()
                        .absolute()
                        .bottom(px(0.0))
                        .left(px(0.0))
                        .right(px(0.0))
                        .h(px(grid_height))
                        .child(element),
                ),
            });

        // The exit pill owns the bottom slot; the transient pills stack above it.
        let pill_bottom = if exited { 52.0 } else { 18.0 };
        if view_offset > 0 {
            let return_id = session.id.clone();
            body = body.child(
                div()
                    .id("scrolled-pill")
                    .absolute()
                    .bottom(px(pill_bottom))
                    .left_1_2()
                    .ml(px(-90.0))
                    .rounded(px(999.0))
                    .px(px(12.0))
                    .py(px(6.0))
                    .bg(rgba(0x303238e8))
                    .text_size(px(11.5))
                    .text_color(rgba(0xffffff99))
                    .cursor_pointer()
                    .flex()
                    .items_center()
                    .gap(px(5.0))
                    .child(sf_symbol("arrow.down", 11.5, rgba(0xffffff99)))
                    .child(format!("{view_offset} lines · Return to live"))
                    .on_click(cx.listener(move |this, _, _window, cx| {
                        if let Some(resident) = this.residents.get_mut(&return_id) {
                            resident
                                .element
                                .scroll_to_live(usize::from(resident.last_size.1));
                            cx.notify();
                        }
                    })),
            );
        }
        if attachment_state != AttachmentState::Live {
            let message = match attachment_state {
                AttachmentState::Attaching => "Attaching…",
                AttachmentState::Reconnecting => "Reconnecting terminal…",
                AttachmentState::Live => "",
            };
            body = body.child(
                div()
                    .absolute()
                    .bottom(px(pill_bottom))
                    .left_1_2()
                    .ml(px(-72.0))
                    .rounded(px(999.0))
                    .px(px(12.0))
                    .py(px(6.0))
                    .bg(rgba(0x303238e8))
                    .text_size(px(11.5))
                    .text_color(rgba(0xffffff99))
                    .child(message),
            );
        }
        if exited {
            body = body.child(self.render_exit_pill(session, cx));
        }
        body.into_any_element()
    }
}
