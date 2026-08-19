use super::widgets::*;
use super::*;

impl UtilitySurfaces {
    fn render_history(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = self.colors();
        let entries = self.visible_history();
        let empty = entries.is_empty() && !self.history_loading;
        let rows = entries.into_iter().enumerate().map(|(index, entry)| {
            let selected = index == self.history_highlight;
            let resumable = entry.cwd_exists;
            let folder = folder_name(&entry.cwd).to_owned();
            let parent = relative_parent(&entry.cwd);
            let title = entry
                .title
                .clone()
                .unwrap_or_else(|| "Untitled conversation".to_owned());
            let age = relative_time(entry.last_active_at.0);
            let agent = ui_agent(&entry.kind);
            div()
                .id(("history-row", index))
                .h(px(46.0))
                .px(px(10.0))
                .rounded(px(Radius::ROW))
                .flex()
                .items_center()
                .gap(px(9.0))
                .opacity(if resumable { 1.0 } else { 0.45 })
                .bg(Fill::selected(colors, selected))
                .cursor_pointer()
                .hover(move |style| style.bg(Fill::hover(colors, true)))
                .on_click(cx.listener(move |this, _, _, cx| {
                    if resumable {
                        this.resume_history(entry.clone(), cx);
                    }
                }))
                .child(AgentLogo::new(agent, 18.0, colors))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .flex_1()
                        .min_w(px(0.0))
                        .gap(px(2.0))
                        .child(
                            div()
                                .text_size(px(13.0))
                                .text_color(colors.primary)
                                .overflow_hidden()
                                .child(title),
                        )
                        .child(
                            div()
                                .flex()
                                .gap(px(5.0))
                                .text_size(px(11.0))
                                .child(div().text_color(colors.secondary).child(folder))
                                .child(div().text_color(colors.tertiary).child(parent)),
                        ),
                )
                .child(chip(
                    if !resumable {
                        "folder gone".to_owned()
                    } else if selected {
                        "↵ resume".to_owned()
                    } else {
                        age
                    },
                    colors,
                ))
        });

        FloatingSurface::new(
            self.colors(),
            div()
                .w(px(560.0))
                .max_h(px(440.0))
                .flex()
                .flex_col()
                .child(
                    div()
                        .h(px(48.0))
                        .px(px(16.0))
                        .flex()
                        .items_center()
                        .gap(px(10.0))
                        .text_size(px(15.0))
                        .child(sf_symbol("magnifyingglass", 13.0, colors.tertiary))
                        .child(
                            div()
                                .flex_1()
                                .text_color(if self.history_query.is_empty() {
                                    colors.tertiary
                                } else {
                                    colors.primary
                                })
                                .child(if self.history_query.is_empty() {
                                    div().child("Search past conversations…").into_any_element()
                                } else {
                                    query_label(&self.history_query)
                                }),
                        )
                        .child(chip("esc".to_owned(), colors)),
                )
                .child(HairlineDivider::horizontal(colors))
                .child(
                    div()
                        .id("history-results")
                        .max_h(px(380.0))
                        .overflow_y_scroll()
                        .p(px(6.0))
                        .flex()
                        .flex_col()
                        .gap(px(1.0))
                        .when(self.history_loading, |view| {
                            view.child(empty_label(
                                "Scanning Claude and Codex transcripts…",
                                colors,
                            ))
                        })
                        .when_some(self.history_error.clone(), |view, error| {
                            view.child(empty_label(&error, colors))
                        })
                        .when(empty, |view| {
                            view.child(empty_label(
                                if self.history_query.is_empty() {
                                    "No past conversations"
                                } else {
                                    "No matches"
                                },
                                colors,
                            ))
                        })
                        .children(rows),
                ),
        )
    }

    fn render_worktrees(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = self.colors();
        let cards = self
            .worktrees
            .entries
            .iter()
            .map(|entry| {
                let path = entry.path.clone();
                let branch = entry
                    .branch
                    .clone()
                    .unwrap_or_else(|| "detached".to_owned());
                let project = folder_name(&entry.project_root).to_owned();
                let stale = entry.stale_suggestion;
                div()
                    .w(px(278.0))
                    .min_h(px(132.0))
                    .p(px(12.0))
                    .rounded(px(Radius::CARD))
                    .border_1()
                    .border_color(if stale {
                        Ink::ATTENTION.alpha(0.6)
                    } else {
                        colors.primary.alpha(0.06)
                    })
                    .bg(Fill::subtle(colors))
                    .flex()
                    .flex_col()
                    .gap(px(8.0))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(6.0))
                            .text_size(px(13.0))
                            .font_weight(FontWeight::MEDIUM)
                            .child(sf_symbol("arrow.branch", 13.0, colors.secondary))
                            .child(branch),
                    )
                    .child(
                        div()
                            .text_size(px(11.0))
                            .text_color(colors.tertiary)
                            .child(project),
                    )
                    .child(
                        div()
                            .flex()
                            .gap(px(5.0))
                            .when(entry.dirty, |row| {
                                row.child(colored_badge("dirty", Ink::ATTENTION))
                            })
                            .when(entry.merged, |row| {
                                row.child(colored_badge("merged", Ink::FRESH))
                            })
                            .child(chip(format!("{}d", entry.age_days), colors)),
                    )
                    .when(stale, |card| {
                        card.child(
                            div()
                                .id(SharedString::from(format!("cleanup-{path}")))
                                .text_size(px(11.0))
                                .text_color(Ink::DANGER)
                                .cursor_pointer()
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.worktrees.request_cleanup(&path);
                                    cx.notify();
                                }))
                                .flex()
                                .items_center()
                                .gap(px(6.0))
                                .child(sf_symbol_weighted(
                                    "trash",
                                    11.0,
                                    SymbolWeight::Regular,
                                    Ink::DANGER,
                                ))
                                .child("Clean Up"),
                        )
                    })
                    .into_any_element()
            })
            .collect::<Vec<_>>();
        let pending = self.worktrees.pending_cleanup.clone();
        FloatingSurface::new(
            self.colors(),
            div()
                .w(px(620.0))
                .h(px(460.0))
                .flex()
                .flex_col()
                .child(
                    div()
                        .h(px(52.0))
                        .px(px(16.0))
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(
                            div()
                                .text_size(px(15.0))
                                .font_weight(FontWeight::SEMIBOLD)
                                .child("Worktrees"),
                        )
                        .child(
                            div()
                                .flex()
                                .gap(px(8.0))
                                .child(surface_button(
                                    "Refresh",
                                    "refresh-worktrees",
                                    colors,
                                    cx,
                                    |this, cx| {
                                        this.refresh_worktrees(cx);
                                    },
                                ))
                                .child(surface_button(
                                    "Done",
                                    "done-worktrees",
                                    colors,
                                    cx,
                                    |this, cx| {
                                        this.close_surface(cx);
                                    },
                                )),
                        ),
                )
                .child(HairlineDivider::horizontal(colors))
                .child(
                    div()
                        .id("worktree-cards")
                        .flex_1()
                        .overflow_y_scroll()
                        .p(px(16.0))
                        .flex()
                        .flex_wrap()
                        .content_start()
                        .gap(px(12.0))
                        .when(self.worktrees.loading, |view| {
                            view.child(empty_label("Refreshing worktrees…", colors))
                        })
                        .when_some(self.worktrees.error.clone(), |view, error| {
                            view.child(empty_label(&error, colors))
                        })
                        .when(
                            self.worktrees.entries.is_empty() && !self.worktrees.loading,
                            |view| view.child(empty_label("No worktrees", colors)),
                        )
                        .children(cards),
                )
                .when_some(pending, |sheet, entry| {
                    sheet.child(
                        div()
                            .absolute()
                            .inset_0()
                            .occlude()
                            .bg(rgba(0x00000088))
                            .flex()
                            .items_end()
                            .justify_center()
                            .pb(px(18.0))
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|this, _, _, cx| {
                                    this.worktrees.cancel_cleanup();
                                    cx.notify();
                                    cx.stop_propagation();
                                }),
                            )
                            .child(FloatingSurface::new(
                                self.colors(),
                                div()
                                    .w(px(560.0))
                                    .p(px(16.0))
                                    .flex()
                                    .items_center()
                                    .gap(px(12.0))
                                    .on_mouse_down(MouseButton::Left, |_, _, cx| {
                                        cx.stop_propagation()
                                    })
                                    .child(
                                        div()
                                            .flex_1()
                                            .flex()
                                            .flex_col()
                                            .gap(px(4.0))
                                            .child(
                                                div()
                                                    .font_weight(FontWeight::SEMIBOLD)
                                                    .child("Remove worktree?"),
                                            )
                                            .child(
                                                div()
                                                    .text_size(px(11.0))
                                                    .text_color(colors.secondary)
                                                    .child(format!(
                                                        "{} is merged and clean.",
                                                        entry.path
                                                    )),
                                            ),
                                    )
                                    .child(surface_button(
                                        "Cancel",
                                        "cancel-cleanup",
                                        colors,
                                        cx,
                                        |this, cx| {
                                            this.worktrees.cancel_cleanup();
                                            cx.notify();
                                        },
                                    ))
                                    .child(danger_button("Remove Worktree", cx, |this, cx| {
                                        this.confirm_cleanup(cx);
                                    })),
                            )),
                    )
                }),
        )
    }
}

impl Render for UtilitySurfaces {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let overlay = match self.surface {
            Surface::None => None,
            Surface::History => Some(self.render_history(cx).into_any_element()),
            Surface::Worktrees => Some(self.render_worktrees(cx).into_any_element()),
            Surface::Settings => Some(self.render_settings(cx).into_any_element()),
        };
        let root = div()
            .id("utility-surfaces")
            .track_focus(&self.focus)
            .key_context("Homie")
            .on_key_down(cx.listener(Self::key_down))
            .on_action(cx.listener(|this, _: &ToggleHistory, _, cx| {
                if this.surface == Surface::History {
                    this.close_surface(cx);
                } else {
                    this.open_history(cx);
                }
            }))
            .on_action(cx.listener(|this, _: &OpenWorktrees, _, cx| {
                this.open_worktrees(cx);
            }))
            .on_action(cx.listener(|this, _: &OpenSettings, _, cx| {
                this.open_settings(cx);
            }))
            .on_action(cx.listener(|this, _: &CloseSurface, _, cx| this.close_surface(cx)))
            .on_action(cx.listener(|this, _: &MoveUp, _, cx| this.move_history(-1, cx)))
            .on_action(cx.listener(|this, _: &MoveDown, _, cx| this.move_history(1, cx)))
            .on_action(cx.listener(|this, _: &Activate, _, cx| this.activate_history(cx)))
            .absolute()
            // Cached entity roots are laid out independently, so insets alone
            // leave this absolute root without a definite size: its height
            // collapses to its in-flow content, which is nothing. The backdrop
            // then paints no dim at all and the dialog centers on the window's
            // top edge, putting its tab list and close control off-window.
            .size_full()
            .text_color(self.colors().primary);
        if let Some(overlay) = overlay {
            root.inset_0().child(
                div()
                    .absolute()
                    .inset_0()
                    .debug_selector(|| "surface-backdrop".into())
                    // The modal backdrop dismisses the topmost surface while
                    // still protecting terminal selection and scrollback.
                    .occlude()
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _, _, cx| {
                            this.close_surface(cx);
                            cx.stop_propagation();
                        }),
                    )
                    .on_scroll_wheel(|_, _, cx| cx.stop_propagation())
                    .bg(rgba(0x00000040))
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        div()
                            .occlude()
                            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                            .child(overlay),
                    ),
            )
        } else {
            root.size(px(0.0))
        }
    }
}
