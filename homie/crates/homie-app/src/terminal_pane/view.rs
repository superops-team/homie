//! GPUI render facade for the terminal pane.
//!
//! Owns the render tree (the `impl Render` block, the `render_*` methods, and
//! the free render helpers). Effect dispatch stays in `mod.rs`.

use super::*;

impl TerminalPane {
    pub(super) fn render_sidebar_reveal_control(
        &self,
        colors: SemanticColors,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        div()
            .flex_none()
            .flex()
            .items_center()
            .gap(px(Metrics::TOOLBAR_ITEM_GAP))
            // The visible lights need more breathing room than their native
            // frames imply, so this is an intentional optical safe area.
            .child(div().w(px(Metrics::TOOLBAR_TRAFFIC_LIGHT_LANE)).flex_none())
            .child(
                div()
                    .id("show-sidebar")
                    .debug_selector(|| "show-sidebar".into())
                    .size(px(Metrics::TOOLBAR_CONTROL_SIZE))
                    .flex_none()
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(px(Radius::BADGE))
                    .cursor_pointer()
                    .hover(move |button| button.bg(Fill::subtle(colors)))
                    .child(sf_symbol("sidebar.left", 15.0, colors.secondary))
                    .on_click(cx.listener(|_, _, _, cx| {
                        cx.emit(TerminalPaneEvent::ToggleSidebar);
                        cx.stop_propagation();
                    })),
            )
            .into_any_element()
    }

    pub(super) fn render_header(
        &self,
        session: &SessionRecord,
        colors: SemanticColors,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let glyph = self.glyphs.get(&session.id).cloned();
        let branch = session.git_branch.clone();
        let host = session.host.as_ref().map(|host| {
            self.runtime
                .store
                .read()
                .expect("session store lock poisoned")
                .host_display_name(host)
        });
        let kind = ui_agent_kind(session.effective_kind());
        let shell_controls = matches!(self.session_source, SessionSource::FollowSelection);
        let show_sidebar = shell_controls && !self.sidebar_visible;
        let sidebar_reveal = show_sidebar.then(|| self.render_sidebar_reveal_control(colors, cx));
        let inspector_open = self.inspector_open;
        div()
            .h(px(Metrics::TITLE_BAR))
            .flex_none()
            .px(px(Metrics::TOOLBAR_EDGE_INSET))
            .flex()
            .items_center()
            .justify_between()
            .bg(colors.sidebar_surface())
            .child(
                div()
                    .min_w(px(0.0))
                    .flex_1()
                    .flex()
                    .items_center()
                    .gap(px(Metrics::TOOLBAR_ITEM_GAP))
                    .overflow_hidden()
                    .when_some(sidebar_reveal, |title, control| title.child(control))
                    .child(sf_symbol("terminal", 15.0, colors.secondary))
                    .child(
                        div()
                            .min_w(px(0.0))
                            .flex_1()
                            .overflow_hidden()
                            .whitespace_nowrap()
                            .text_size(px(Typo::TITLE.size))
                            .font_weight(Typo::TITLE.weight)
                            .text_color(colors.primary)
                            .child(session.title.clone()),
                    )
                    .when_some(branch, |title, branch| {
                        title.child(
                            div()
                                .flex_none()
                                .flex()
                                .items_center()
                                .gap(px(Metrics::TOOLBAR_COMPACT_GAP))
                                .px(px(5.0))
                                .py(px(2.0))
                                .rounded(px(Radius::CHIP))
                                .bg(Fill::subtle(colors))
                                .overflow_hidden()
                                .whitespace_nowrap()
                                .text_size(px(Typo::META.size))
                                .text_color(colors.tertiary)
                                .child(sf_symbol("arrow.branch", 10.5, colors.tertiary))
                                .child(branch),
                        )
                    })
                    .when_some(host, |title, host| {
                        // Remote-host chip: the agent runs on that configured machine.
                        title.child(
                            div()
                                .flex_none()
                                .flex()
                                .items_center()
                                .gap(px(Metrics::TOOLBAR_COMPACT_GAP))
                                .rounded(px(Radius::CHIP))
                                .px(px(5.0))
                                .py(px(2.0))
                                .bg(Fill::subtle(colors))
                                .text_size(px(Typo::META.size))
                                .text_color(colors.secondary)
                                .child(sf_symbol("network", 9.0, colors.secondary))
                                .child(host),
                        )
                    }),
            )
            .child(
                div()
                    .flex_none()
                    .pl(px(Metrics::TOOLBAR_EDGE_INSET))
                    .flex()
                    .items_center()
                    .gap(px(Metrics::TOOLBAR_ITEM_GAP))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(Metrics::TOOLBAR_COMPACT_GAP))
                            .when_some(glyph, |identity, glyph| identity.child(glyph))
                            .child(
                                div()
                                    .text_size(px(Typo::META.size))
                                    .text_color(colors.tertiary)
                                    .child(kind.label()),
                            ),
                    )
                    .when(shell_controls, |trailing| {
                        trailing.child(
                            div()
                                .id("toggle-inspector")
                                .size(px(Metrics::TOOLBAR_CONTROL_SIZE))
                                .flex_none()
                                .flex()
                                .items_center()
                                .justify_center()
                                .rounded(px(Radius::BADGE))
                                .cursor_pointer()
                                .when(inspector_open, |button| button.bg(Fill::subtle(colors)))
                                .hover(move |button| button.bg(Fill::subtle(colors)))
                                .child(sf_symbol(
                                    "sidebar.right",
                                    15.0,
                                    if inspector_open {
                                        colors.primary
                                    } else {
                                        colors.secondary
                                    },
                                ))
                                .on_click(cx.listener(|_, _, _, cx| {
                                    cx.emit(TerminalPaneEvent::ToggleInspector);
                                    cx.stop_propagation();
                                })),
                        )
                    }),
            )
            .into_any_element()
    }

    pub(super) fn render_grid_and_overlays(
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

    /// Slim status pill over an exited session's last screen: says what happened
    /// and offers the resume that the pane-filling card used to.
    pub(super) fn render_exit_pill(
        &self,
        session: &SessionRecord,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let id = session.id.clone();
        let resumable = session.resumability == Resumability::Resumable;
        let mut pill = div()
            .id("exit-pill")
            .rounded(px(999.0))
            .pl(px(12.0))
            .pr(if resumable { px(4.0) } else { px(12.0) })
            .py(px(4.0))
            .bg(rgba(0x303238e8))
            .flex()
            .items_center()
            .gap(px(8.0))
            .text_size(px(11.5))
            .text_color(rgba(0xffffff99))
            .child(sf_symbol("power", 11.0, rgba(0xffffff66)))
            .child(exit_description(session));
        if resumable {
            pill = pill.child(
                div()
                    .id("exit-pill-resume")
                    .rounded(px(999.0))
                    .px(px(9.0))
                    .py(px(3.0))
                    .bg(rgba(0xffffff1a))
                    .hover(|style| style.bg(rgba(0xffffff2e)))
                    .cursor_pointer()
                    .text_color(rgba(0xffffffe6))
                    .child("Resume")
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.runtime
                            .store
                            .read()
                            .expect("session store lock poisoned")
                            .resume(id.clone());
                        cx.notify();
                    })),
            );
        } else if session.resumability == Resumability::TranscriptMissing {
            pill = pill.child(
                div()
                    .text_color(rgba(0xffffff4d))
                    .child("· transcript gone"),
            );
        }
        // Centered by a full-width row rather than a guessed half-width offset,
        // since the description's length varies with the exit reason.
        div()
            .absolute()
            .bottom(px(18.0))
            .left_0()
            .right_0()
            .flex()
            .justify_center()
            .child(pill)
            .into_any_element()
    }

    pub(super) fn render_find_bar(
        &self,
        session: &SessionRecord,
        colors: SemanticColors,
        cx: &mut Context<Self>,
    ) -> Option<AnyElement> {
        let resident = self.residents.get(&session.id)?;
        let find = resident.find.as_ref()?;
        let count = if find.matches().is_empty() {
            if find.query().is_empty() {
                String::new()
            } else {
                "No matches".to_owned()
            }
        } else {
            format!("{}/{}", find.current_index() + 1, find.matches().len())
        };
        let query = if resident.find_query.is_empty() {
            div().child("Find").into_any_element()
        } else {
            query_label(&resident.find_query)
        };
        let alt_screen = find.is_alt_screen();
        Some(
            div()
                .id("find-bar")
                .absolute()
                .top(px(Metrics::TITLE_BAR + 6.0))
                .right(px(16.0))
                .w(px(360.0))
                .child(FloatingSurface::new(
                    colors,
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(3.0))
                        .px(px(10.0))
                        .py(px(7.0))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(8.0))
                                .text_size(px(Typo::ROW.size))
                                .text_color(rgba(0xffffffd9))
                                .child(sf_symbol("magnifyingglass", 12.0, rgba(0xffffff66)))
                                .child(div().flex_1().child(query))
                                .child(
                                    div()
                                        .text_size(px(Typo::META.size))
                                        .text_color(rgba(0xffffff4d))
                                        .child(count),
                                )
                                .child(div().w(px(1.0)).h(px(16.0)).bg(rgba(0xffffff1a)))
                                .child(find_icon_button(
                                    "find-previous",
                                    "chevron.up",
                                    cx,
                                    |this, _w, cx| {
                                        this.navigate_find(true, cx);
                                    },
                                ))
                                .child(find_icon_button(
                                    "find-next",
                                    "chevron.down",
                                    cx,
                                    |this, _w, cx| {
                                        this.navigate_find(false, cx);
                                    },
                                ))
                                .child(find_icon_button(
                                    "find-close",
                                    "xmark",
                                    cx,
                                    |this, _w, cx| {
                                        this.close_find_for_selected();
                                        cx.notify();
                                    },
                                )),
                        )
                        .when(alt_screen, |bar| {
                            bar.child(
                                div()
                                    .pl(px(20.0))
                                    .text_size(px(Typo::META.size))
                                    .text_color(rgba(0xffffff4d))
                                    .child("full-screen app — screen only"),
                            )
                        }),
                ))
                .into_any_element(),
        )
    }

    /// The pane-filling card for an exited session, or `None` when the terminal
    /// itself should stay on screen (with [`Self::render_exit_pill`] over it).
    pub(super) fn render_exited_takeover(
        &self,
        session: &SessionRecord,
        cx: &mut Context<Self>,
    ) -> Option<AnyElement> {
        let (auto_resuming, migrating) = {
            let store = self
                .runtime
                .store
                .read()
                .expect("session store lock poisoned");
            (
                store.auto_resuming().contains(&session.id),
                store.migrating().contains(&session.id),
            )
        };
        // Mid-migration the source agent is briefly down; show the busy state
        // instead of an exit card with a doomed Resume button.
        if migrating {
            return Some(centered_message("◌", "Moving session…").into_any_element());
        }
        if auto_resuming {
            return Some(centered_message("◌", "Resuming conversation…").into_any_element());
        }
        if self
            .residents
            .get(&session.id)
            .is_some_and(|resident| resident.element.has_content())
        {
            return None;
        }
        Some(self.render_exited_card(session, cx))
    }

    pub(super) fn render_exited_card(
        &self,
        session: &SessionRecord,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let id = session.id.clone();
        let content = centered_message("", &exit_description(session));
        if session.resumability == Resumability::Resumable {
            content
                .child(primary_button(
                    "resume-conversation",
                    "Resume Conversation",
                    cx,
                    move |this, cx| {
                        this.runtime
                            .store
                            .read()
                            .expect("session store lock poisoned")
                            .resume(id.clone());
                        cx.notify();
                    },
                ))
                .into_any_element()
        } else if session.resumability == Resumability::TranscriptMissing {
            content
                .child(
                    div()
                        .text_size(px(11.5))
                        .text_color(rgba(0xffffff4d))
                        .child("Transcript is gone — start a fresh session in the same folder."),
                )
                .into_any_element()
        } else {
            content.into_any_element()
        }
    }

    pub(super) fn render_archived_overlay(
        &self,
        session: &SessionRecord,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let id = session.id.clone();
        let mut content = centered_symbol_message("archivebox", 30.0, &session.title).child(
            div()
                .text_size(px(13.0))
                .text_color(rgba(0xffffff99))
                .child("Archived"),
        );
        if session.resumability == Resumability::NotResumable {
            content = content.child(
                div()
                    .max_w(px(320.0))
                    .text_size(px(11.5))
                    .text_color(rgba(0xffffff4d))
                    .child(
                        "This session can't resume its conversation; revive restores it as ended.",
                    ),
            );
        }
        content
            .child(primary_button(
                "revive-session",
                "Revive Session",
                cx,
                move |this, cx| {
                    this.runtime
                        .store
                        .write()
                        .expect("session store lock poisoned")
                        .revive_sessions(vec![id.clone()]);
                    this.reconcile_residency();
                    cx.notify();
                },
            ))
            .into_any_element()
    }
}

impl Render for TerminalPane {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.reconcile_residency();
        let (theme, colors, sidebar_colors, font_size) = {
            let store = self
                .runtime
                .store
                .read()
                .expect("session store lock poisoned");
            let theme_id = &store.preferences().terminal_theme;
            (
                crate::app_theme::terminal_theme(theme_id),
                crate::app_theme::colors(theme_id),
                crate::app_theme::sidebar_colors(theme_id),
                store.preferences().terminal_font_size,
            )
        };
        self.sync_status_glyphs(colors, window, cx);
        self.update_selected_geometry(window, cx);

        let selected = self.selected_session();

        let content = if let Some(session) = selected {
            let mut pane = div()
                .relative()
                .flex()
                .flex_col()
                .flex_1()
                .h_full()
                .overflow_hidden()
                .border_l_1()
                .border_color(sidebar_colors.primary.alpha(0.08))
                .bg(sidebar_colors.sidebar_surface())
                .child(self.render_header(&session, sidebar_colors, cx));
            let terminal_surface = div()
                .relative()
                .min_h(px(0.0))
                .flex_1()
                .flex()
                .flex_col()
                .rounded_tl(px(Radius::CARD))
                .rounded_tr(px(Radius::CARD))
                .overflow_hidden()
                .bg(theme.background)
                .child(self.render_grid_and_overlays(&session, theme, font_size, window, cx));
            pane = pane.child(terminal_surface);
            if let Some(find) = self.render_find_bar(&session, colors, cx) {
                pane = pane.child(find);
            }
            pane.into_any_element()
        } else {
            let show_sidebar = matches!(self.session_source, SessionSource::FollowSelection)
                && !self.sidebar_visible;
            let sidebar_reveal =
                show_sidebar.then(|| self.render_sidebar_reveal_control(sidebar_colors, cx));
            div()
                .flex_1()
                .h_full()
                .flex()
                .flex_col()
                .bg(theme.background)
                .when_some(sidebar_reveal, |pane, control| {
                    pane.child(
                        div()
                            .h(px(Metrics::TITLE_BAR))
                            .flex_none()
                            .px(px(Metrics::TOOLBAR_EDGE_INSET))
                            .flex()
                            .items_center()
                            .bg(sidebar_colors.sidebar_surface())
                            .child(control),
                    )
                })
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_size(px(13.0))
                        .text_color(sidebar_colors.tertiary)
                        .child("Start a terminal from the sidebar"),
                )
                .into_any_element()
        };

        let root_id = match &self.session_source {
            SessionSource::FollowSelection => SharedString::from("homie-terminal-root"),
            SessionSource::Fixed(id) => SharedString::from(format!("homie-terminal-root-{}", id.0)),
        };
        div()
            .id(root_id)
            .track_focus(&self.focus)
            .flex()
            .size_full()
            .text_color(colors.primary)
            .on_action(cx.listener(Self::open_find))
            .on_action(cx.listener(Self::find_next))
            .on_action(cx.listener(Self::find_previous))
            .on_action(cx.listener(Self::close_find))
            .on_action(cx.listener(Self::zoom_in))
            .on_action(cx.listener(Self::zoom_out))
            .on_action(cx.listener(Self::reset_zoom))
            .on_action(cx.listener(Self::paste))
            .on_action(cx.listener(Self::copy_selection))
            .on_key_down(cx.listener(Self::handle_key_down))
            .on_key_up(cx.listener(Self::handle_key_up))
            .on_modifiers_changed(cx.listener(Self::handle_modifiers_changed))
            .child(content)
    }
}

fn find_icon_button(
    id: &'static str,
    system_image: &'static str,
    cx: &mut Context<TerminalPane>,
    handler: impl Fn(&mut TerminalPane, &mut Window, &mut Context<TerminalPane>) + 'static,
) -> AnyElement {
    div()
        .id(id)
        .size(px(20.0))
        .rounded(px(4.0))
        .flex()
        .items_center()
        .justify_center()
        .text_size(px(11.0))
        .text_color(rgba(0xffffff99))
        .hover(|style| style.bg(rgba(0xffffff0f)))
        .cursor_pointer()
        .child(sf_symbol_weighted(
            system_image,
            11.0,
            SymbolWeight::Semibold,
            rgba(0xffffff99),
        ))
        .on_click(cx.listener(move |this, _, window, cx| handler(this, window, cx)))
        .into_any_element()
}

fn primary_button(
    id: &'static str,
    label: &'static str,
    cx: &mut Context<TerminalPane>,
    handler: impl Fn(&mut TerminalPane, &mut Context<TerminalPane>) + 'static,
) -> AnyElement {
    div()
        .id(id)
        .mt(px(2.0))
        .rounded(px(7.0))
        .px(px(14.0))
        .py(px(7.0))
        .bg(rgba(0xffffffeb))
        .text_size(px(13.0))
        .font_weight(Typo::ROW_EMPHASIZED.weight)
        .text_color(rgba(0x121318ff))
        .hover(|style| style.bg(rgba(0xffffffff)))
        .cursor_pointer()
        .child(label)
        .on_click(cx.listener(move |this, _, _, cx| handler(this, cx)))
        .into_any_element()
}

fn centered_message(icon: &str, message: &str) -> gpui::Div {
    div()
        .flex_1()
        .size_full()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap(px(12.0))
        .when(!icon.is_empty(), |content| {
            content.child(
                div()
                    .text_size(px(30.0))
                    .text_color(rgba(0xffffff4d))
                    .child(icon.to_owned()),
            )
        })
        .when(!message.is_empty(), |content| {
            content.child(
                div()
                    .text_size(px(13.0))
                    .text_color(rgba(0xffffff99))
                    .child(message.to_owned()),
            )
        })
}

fn centered_symbol_message(system_image: &str, size: f32, message: &str) -> gpui::Div {
    div()
        .flex_1()
        .size_full()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap(px(12.0))
        .child(sf_symbol_weighted(
            system_image,
            size,
            SymbolWeight::Regular,
            rgba(0xffffff4d),
        ))
        .when(!message.is_empty(), |content| {
            content.child(
                div()
                    .text_size(px(13.0))
                    .text_color(rgba(0xffffff99))
                    .child(message.to_owned()),
            )
        })
}
