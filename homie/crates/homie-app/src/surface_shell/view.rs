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

    fn render_settings(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = self.settings_colors();
        let tabs = SettingsTab::ALL
            .into_iter()
            .map(|tab| {
                let selected = tab == self.settings_tab;
                div()
                    .id(SharedString::from(format!("settings-{}", tab.label())))
                    .h(px(30.0))
                    .px(px(8.0))
                    .rounded(px(Radius::ROW))
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    .bg(Fill::selected(colors, selected))
                    .text_color(if selected {
                        colors.primary
                    } else {
                        colors.secondary
                    })
                    .cursor_pointer()
                    .hover(move |style| style.bg(Fill::hover(colors, true)))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.settings_tab = tab;
                        this.settings_menu = None;
                        this.host_editor = None;
                        if tab == SettingsTab::Remote {
                            this.reload_hosts();
                        }
                        cx.notify();
                    }))
                    .child(sf_symbol(
                        tab.icon(),
                        13.0,
                        if selected {
                            colors.primary
                        } else {
                            colors.tertiary
                        },
                    ))
                    .child(
                        div()
                            .text_size(px(Typo::ROW.size))
                            .font_weight(if selected {
                                FontWeight::MEDIUM
                            } else {
                                FontWeight::NORMAL
                            })
                            .child(tab.label()),
                    )
                    .into_any_element()
            })
            .collect::<Vec<_>>();
        let pane = match self.settings_tab {
            SettingsTab::General => self.general_settings(cx).into_any_element(),
            SettingsTab::Terminal => self.terminal_settings(cx).into_any_element(),
            SettingsTab::Resources => self.resource_settings(cx).into_any_element(),
            SettingsTab::Remote => self.remote_settings(cx).into_any_element(),
        };
        FloatingSurface::new(
            colors,
            div()
                .id("settings-dialog")
                .debug_selector(|| "settings-dialog".into())
                .w(px(SETTINGS_WIDTH))
                .h(px(SETTINGS_HEIGHT))
                .rounded(px(Radius::PANEL))
                .overflow_hidden()
                .flex()
                // The dialog owns clicks within its bounds. When a select is
                // open, any click elsewhere in Settings dismisses that select
                // before the clicked control runs its own action.
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(|this, _, _, cx| {
                        if this.settings_menu.take().is_some() {
                            cx.notify();
                        }
                        cx.stop_propagation();
                    }),
                )
                .child(
                    div()
                        .w(px(SETTINGS_NAV_WIDTH))
                        .h_full()
                        .bg(colors.primary.alpha(0.018))
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .h(px(Metrics::TITLE_BAR))
                                .px(px(Metrics::TOOLBAR_EDGE_INSET))
                                .flex()
                                .items_center()
                                .gap(px(Metrics::TOOLBAR_ITEM_GAP))
                                .child(sf_symbol("gearshape", 15.0, colors.secondary))
                                .child(
                                    div()
                                        .text_size(px(Typo::TITLE.size))
                                        .font_weight(Typo::TITLE.weight)
                                        .child("Settings"),
                                ),
                        )
                        .child(
                            div()
                                .px(px(Space::INSET))
                                .pt(px(2.0))
                                .pb(px(10.0))
                                .flex_1()
                                .flex()
                                .flex_col()
                                .gap(px(2.0))
                                .children(tabs),
                        )
                        .child(
                            div()
                                .px(px(Metrics::TOOLBAR_EDGE_INSET))
                                .pb(px(10.0))
                                .text_size(px(Typo::META.size))
                                .text_color(colors.tertiary)
                                .child(format!("homie {}", crate::updates::CURRENT_VERSION)),
                        ),
                )
                .child(HairlineDivider::vertical(colors))
                .child(
                    div()
                        .relative()
                        .flex_1()
                        .min_w(px(0.0))
                        .h_full()
                        .child(
                            div()
                                .id("settings-pane")
                                .debug_selector(|| "settings-pane".into())
                                .absolute()
                                .inset_0()
                                .overflow_y_scroll()
                                .child(pane),
                        )
                        .child(
                            div()
                                .absolute()
                                .debug_selector(|| "close-settings".into())
                                .top(px(8.0))
                                .right(px(Metrics::TOOLBAR_EDGE_INSET))
                                .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                                .child(
                                    Button::new(
                                        "close-settings",
                                        colors,
                                        sf_symbol_weighted(
                                            "xmark",
                                            13.5,
                                            SymbolWeight::Bold,
                                            colors.secondary,
                                        ),
                                    )
                                    .variant(ButtonVariant::Quiet)
                                    .size(ButtonSize::Toolbar)
                                    .on_click(cx.listener(
                                        |this, _, _, cx| {
                                            this.close_surface(cx);
                                        },
                                    )),
                                ),
                        ),
                ),
        )
    }

    fn general_settings(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = self.settings_colors();
        let quick_open_roots = if self.prefs.quick_open_roots.is_empty() {
            "~/fun".to_owned()
        } else {
            self.prefs.quick_open_roots.clone()
        };
        settings_page(
            "General",
            div()
                .flex()
                .flex_col()
                .gap(px(SETTINGS_SECTION_GAP))
                .child(setting_section(
                    "New sessions",
                    setting_row(
                        "Default agent",
                        "Used by ⌘T and Quick Open.",
                        self.default_agent_dropdown(cx),
                        colors,
                    ),
                    colors,
                ))
                .child(setting_section(
                    "Behavior",
                    div()
                        .flex()
                        .flex_col()
                        .child(toggle_row(
                            "Start homie at login",
                            "Open homie automatically after you sign in.",
                            self.prefs.start_at_login,
                            "toggle-login",
                            colors,
                            cx,
                            |this, cx| {
                                this.prefs.start_at_login = !this.prefs.start_at_login;
                                this.persist_prefs();
                                cx.notify();
                            },
                        ))
                        .child(setting_divider(colors))
                        .child(toggle_row(
                            "Confirm before closing a session",
                            "Ask before closing a session with a running process.",
                            self.prefs.confirm_before_closing_session,
                            "toggle-close-confirm",
                            colors,
                            cx,
                            |this, cx| {
                                this.prefs.confirm_before_closing_session =
                                    !this.prefs.confirm_before_closing_session;
                                this.persist_prefs();
                                cx.notify();
                            },
                        ))
                        .child(setting_divider(colors))
                        .child(toggle_row(
                            "Gentle status chimes",
                            "Quiet cues for input, completion, and memory pauses.",
                            self.prefs.status_sounds,
                            "toggle-status-sounds",
                            colors,
                            cx,
                            |this, cx| {
                                this.prefs.status_sounds = !this.prefs.status_sounds;
                                this.persist_prefs();
                                cx.notify();
                            },
                        )),
                    colors,
                ))
                .child(self.update_settings(cx))
                .child(setting_section(
                    "Quick Open",
                    div()
                        .p(px(12.0))
                        .flex()
                        .flex_col()
                        .gap(px(8.0))
                        .child(
                            div()
                                .text_size(px(13.0))
                                .font_weight(FontWeight::MEDIUM)
                                .child("Search roots"),
                        )
                        .child(
                            div()
                                .min_w(px(0.0))
                                .w_full()
                                .whitespace_normal()
                                .p(px(10.0))
                                .rounded(px(Radius::BADGE))
                                .bg(colors.primary.alpha(0.055))
                                .font_family(crate::fonts::mono_family())
                                .text_size(px(11.0))
                                .line_height(px(17.0))
                                .text_color(colors.secondary)
                                .child(wrappable_setting_copy(quick_open_roots.into())),
                        )
                        .child(
                            div()
                                .min_w(px(0.0))
                                .w_full()
                                .whitespace_normal()
                                .text_size(px(11.0))
                                .line_height(px(16.0))
                                .text_color(colors.tertiary)
                                .child(wrappable_setting_copy(
                                    if self.prefs.quick_open_roots.is_empty() {
                                        "Using the default folder plus project parent folders."
                                    } else {
                                        "One folder per line, scanned four levels deep."
                                    }
                                    .into(),
                                )),
                        ),
                    colors,
                )),
            colors,
        )
    }

    fn default_agent_dropdown(&self, cx: &mut Context<Self>) -> AnyElement {
        let colors = self.settings_colors();
        let selected = self.prefs.default_agent;
        let open = self.settings_menu == Some(SettingsMenu::DefaultAgent);
        let trigger = settings_select_button(
            default_agent_label(selected),
            "default-agent-dropdown",
            open,
            SettingsMenu::DefaultAgent,
            colors,
            cx,
        );

        let mut control = div().relative().min_w(px(154.0)).child(trigger);
        if open {
            let mut options = div().p(px(4.0)).flex().flex_col();
            for (index, agent) in DefaultAgent::ALL.into_iter().enumerate() {
                let is_selected = agent == selected;
                options = options.child(
                    div()
                        .id(SharedString::from(format!("default-agent-option-{index}")))
                        .h(px(Metrics::ROW_HEIGHT))
                        .px(px(8.0))
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .rounded(px(Radius::ROW))
                        .bg(Fill::selected(colors, is_selected))
                        .cursor_pointer()
                        .hover(move |style| style.bg(colors.primary.alpha(0.08)))
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.prefs.default_agent = agent;
                            this.settings_menu = None;
                            this.persist_prefs();
                            cx.notify();
                        }))
                        .child(AgentLogo::new(ui_default_agent(agent), 16.0, colors).badged(false))
                        .child(
                            div()
                                .flex_1()
                                .text_size(px(Typo::ROW.size))
                                .text_color(colors.primary)
                                .child(default_agent_label(agent)),
                        )
                        .when(is_selected, |row| {
                            row.child(sf_symbol_weighted(
                                "checkmark",
                                10.0,
                                SymbolWeight::Semibold,
                                colors.secondary,
                            ))
                        }),
                );
            }
            control = control.child(settings_dropdown(options, 204.0, colors));
        }
        control.into_any_element()
    }

    /// The UPDATES block in Settings → General.
    ///
    /// Only the *check* is automatic. Downloading and restarting stay behind
    /// the button here (and the sidebar pill), because homie holds live agent
    /// sessions and relaunching underneath them uninvited would be hostile.
    fn update_settings(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = self.settings_colors();
        let state = self.updates.state();
        let unsupported = matches!(state.phase, UpdatePhase::Unsupported(_));
        let action = match &state.phase {
            UpdatePhase::Available(_) => Some(("Download", UpdateCommand::Download)),
            UpdatePhase::Ready(_) => Some(("Restart", UpdateCommand::Install)),
            UpdatePhase::Checking | UpdatePhase::Downloading { .. } | UpdatePhase::Installing => {
                None
            }
            _ if unsupported => None,
            _ => Some((
                "Check Now",
                UpdateCommand::Check {
                    user_initiated: true,
                },
            )),
        };

        let summary = if unsupported {
            format!("homie {}", crate::updates::CURRENT_VERSION)
        } else {
            state.summary()
        };
        let action_control = div().when_some(action, |control, (label, command)| {
            control.child(surface_button(
                label,
                "update-action",
                colors,
                cx,
                move |this, _| {
                    this.updates.send(command.clone());
                },
            ))
        });
        let mut rows = div().flex().flex_col().child(setting_row(
            summary,
            update_detail(&state, unsupported),
            action_control,
            colors,
        ));

        // "Skip" only makes sense for a release that is offered but not yet
        // downloaded; once it is staged the bytes are already on disk.
        if let UpdatePhase::Available(release) = &state.phase {
            let version = release.version.clone();
            rows = rows.child(setting_divider(colors)).child(setting_row(
                "Skip this version",
                "Hide this release until a newer version is available.",
                surface_button("Skip", "skip-update", colors, cx, move |this, cx| {
                    this.prefs.skipped_update_version = version.clone();
                    this.persist_prefs();
                    this.updates.send(UpdateCommand::Skip);
                    cx.notify();
                }),
                colors,
            ));
        }
        if !unsupported {
            rows = rows.child(setting_divider(colors)).child(toggle_row(
                "Check automatically",
                "Look for new releases in the background.",
                self.prefs.automatic_updates,
                "toggle-automatic-updates",
                colors,
                cx,
                |this, cx| {
                    this.prefs.automatic_updates = !this.prefs.automatic_updates;
                    this.persist_prefs();
                    this.updates
                        .send(UpdateCommand::SetAutomatic(this.prefs.automatic_updates));
                    cx.notify();
                },
            ));
        }
        setting_section("Software updates", rows, colors)
    }

    fn terminal_settings(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = self.settings_colors();
        let selected = theme(&self.prefs.terminal_theme);
        let font_control = div()
            .h(px(28.0))
            .rounded(px(Radius::BADGE))
            .border_1()
            .border_color(colors.primary.alpha(0.11))
            .bg(colors.primary.alpha(0.045))
            .flex()
            .items_center()
            .child(
                div()
                    .id("font-smaller")
                    .w(px(30.0))
                    .h_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_size(px(15.0))
                    .cursor_pointer()
                    .hover(move |style| style.bg(colors.primary.alpha(0.08)))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.prefs.zoom_terminal(-1.0);
                        this.persist_prefs();
                        cx.notify();
                    }))
                    .child("−"),
            )
            .child(HairlineDivider::vertical(colors))
            .child(
                div()
                    .w(px(52.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .font_family(crate::fonts::mono_family())
                    .text_size(px(11.0))
                    .text_color(colors.secondary)
                    .child(format!("{:.0} pt", self.prefs.terminal_font_size)),
            )
            .child(HairlineDivider::vertical(colors))
            .child(
                div()
                    .id("font-larger")
                    .w(px(30.0))
                    .h_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_size(px(15.0))
                    .cursor_pointer()
                    .hover(move |style| style.bg(colors.primary.alpha(0.08)))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.prefs.zoom_terminal(1.0);
                        this.persist_prefs();
                        cx.notify();
                    }))
                    .child("+"),
            );

        settings_page(
            "Terminal",
            div()
                .flex()
                .flex_col()
                .gap(px(SETTINGS_SECTION_GAP))
                .child(setting_section(
                    "Appearance",
                    div()
                        .flex()
                        .flex_col()
                        .child(setting_row(
                            "Color theme",
                            "Applies immediately across the app and every open terminal.",
                            self.terminal_theme_dropdown(cx),
                            colors,
                        ))
                        .child(setting_divider(colors))
                        .child(div().p(px(12.0)).child(theme_preview(selected, colors))),
                    colors,
                ))
                .child(setting_section(
                    "Text",
                    setting_row(
                        "Font size",
                        "Adjust terminal text without changing the rest of the app.",
                        font_control,
                        colors,
                    ),
                    colors,
                )),
            colors,
        )
    }

    fn resource_settings(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = self.settings_colors();
        settings_page(
            "Resources",
            div()
                .flex()
                .flex_col()
                .gap(px(SETTINGS_SECTION_GAP))
                .child(setting_section(
                    "Session limits",
                    div()
                        .flex()
                        .flex_col()
                        .child(setting_row(
                            "Hibernate idle sessions",
                            "Freeze inactive sessions after this amount of time.",
                            self.hibernate_dropdown(cx),
                            colors,
                        ))
                        .child(setting_divider(colors))
                        .child(setting_row(
                            "Memory limit",
                            "Freeze an individual session when it reaches this size.",
                            self.memory_dropdown(cx),
                            colors,
                        )),
                    colors,
                ))
                .child(
                    settings_note(
                        "moon.fill",
                        None,
                        "Frozen sessions are never killed. Opening one wakes it immediately, exactly where you left it.",
                        colors.tertiary,
                        colors.primary.alpha(0.07),
                        colors.primary.alpha(0.035),
                        colors,
                    ),
                ),
            colors,
        )
    }

    fn remote_settings(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = self.settings_colors();
        settings_page(
            "Remote",
            div()
                .flex()
                .flex_col()
                .gap(px(SETTINGS_SECTION_GAP))
                .child(self.remote_hosts_section(cx))
                .child(
                    settings_note(
                        "lock.shield",
                        Some("OpenSSH transport"),
                        "Homie uses your SSH configuration without changing the host. Private-network and Tailscale names work transparently when OpenSSH can resolve them.",
                        colors.secondary,
                        colors.primary.alpha(0.08),
                        colors.primary.alpha(0.035),
                        colors,
                    ),
                ),
            colors,
        )
    }

    fn remote_hosts_section(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = self.settings_colors();
        let default_host = self
            .store
            .read()
            .expect("session store lock poisoned")
            .default_spawn_host();
        let mut catalog = div()
            .rounded(px(Radius::ROW))
            .border_1()
            .border_color(colors.primary.alpha(0.065))
            .bg(colors.primary.alpha(0.02))
            .overflow_hidden();

        if let Some(editor) = &self.host_editor {
            catalog = catalog.child(self.host_editor_panel(editor, cx));
        } else if self.hosts.is_empty() {
            catalog = catalog.child(
                div()
                    .px(px(14.0))
                    .py(px(16.0))
                    .flex()
                    .items_center()
                    .gap(px(12.0))
                    .child(
                        div()
                            .size(px(34.0))
                            .rounded(px(10.0))
                            .bg(colors.primary.alpha(0.065))
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(sf_symbol("network", 17.0, colors.tertiary)),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(3.0))
                            .child(
                                div()
                                    .text_size(px(13.0))
                                    .font_weight(FontWeight::MEDIUM)
                                    .child("No execution hosts yet"),
                            )
                            .child(
                                div()
                                    .text_size(px(11.0))
                                    .text_color(colors.tertiary)
                                    .child("Add any machine you can reach over SSH."),
                            ),
                    ),
            );
        } else {
            for (index, host) in self.hosts.iter().enumerate() {
                if index > 0 {
                    catalog = catalog.child(setting_divider(colors));
                }
                let id = host.id.clone();
                let name = host.display_name().to_owned();
                let destination = host.ssh.clone();
                let folder = host.default_cwd.clone();
                let first_party = host.node.is_some();
                let is_default = default_host.as_deref() == Some(host.id.as_str());
                catalog = catalog.child(
                    div()
                        .id(SharedString::from(format!("remote-host-{id}")))
                        .min_h(px(SETTINGS_ROW_HEIGHT))
                        .px(px(12.0))
                        .flex()
                        .items_center()
                        .gap(px(11.0))
                        .cursor_pointer()
                        .hover(move |style| style.bg(colors.primary.alpha(0.055)))
                        .on_click(cx.listener(move |this, _, window, cx| {
                            this.begin_editing_host(&id, window, cx);
                        }))
                        .child(
                            div()
                                .size(px(30.0))
                                .rounded(px(9.0))
                                .bg(colors.primary.alpha(0.06))
                                .flex()
                                .items_center()
                                .justify_center()
                                .child(sf_symbol("network", 15.0, colors.secondary)),
                        )
                        .child(
                            div()
                                .flex_1()
                                .min_w(px(0.0))
                                .flex()
                                .flex_col()
                                .gap(px(3.0))
                                .child(
                                    div()
                                        .flex()
                                        .min_w(px(0.0))
                                        .items_center()
                                        .gap(px(7.0))
                                        .text_size(px(13.0))
                                        .font_weight(FontWeight::MEDIUM)
                                        .child(
                                            div()
                                                .min_w(px(0.0))
                                                .flex_1()
                                                .overflow_hidden()
                                                .whitespace_nowrap()
                                                .text_ellipsis()
                                                .child(name),
                                        )
                                        .when(first_party, |row| {
                                            row.child(
                                                div()
                                                    .px(px(6.0))
                                                    .py(px(2.0))
                                                    .rounded(px(Radius::BADGE))
                                                    .bg(Ink::FRESH.alpha(0.10))
                                                    .text_size(px(9.0))
                                                    .font_weight(FontWeight::MEDIUM)
                                                    .text_color(Ink::FRESH)
                                                    .child("NODE"),
                                            )
                                        })
                                        .when(is_default, |row| {
                                            row.child(
                                                div()
                                                    .px(px(6.0))
                                                    .py(px(2.0))
                                                    .rounded(px(Radius::BADGE))
                                                    .bg(Palette::CLAY.alpha(0.12))
                                                    .text_size(px(9.0))
                                                    .font_weight(FontWeight::MEDIUM)
                                                    .text_color(Palette::CLAY)
                                                    .child("DEFAULT"),
                                            )
                                        }),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .min_w(px(0.0))
                                        .items_center()
                                        .gap(px(7.0))
                                        .overflow_hidden()
                                        .whitespace_nowrap()
                                        .text_ellipsis()
                                        .font_family(crate::fonts::mono_family())
                                        .text_size(px(10.5))
                                        .text_color(colors.tertiary)
                                        .child(destination)
                                        .when_some(folder, |line, folder| {
                                            line.child(
                                                div()
                                                    .text_color(colors.primary.alpha(0.22))
                                                    .child("·"),
                                            )
                                            .child(folder)
                                        }),
                                ),
                        )
                        .child(sf_symbol("chevron.right", 10.0, colors.tertiary)),
                );
            }
        }

        div()
            .flex()
            .flex_col()
            .gap(px(8.0))
            .child(
                div()
                    .h(px(28.0))
                    .px(px(2.0))
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .text_size(px(Typo::SECTION_HEADER.size))
                            .font_weight(Typo::SECTION_HEADER.weight)
                            .text_color(colors.tertiary)
                            .child("Execution hosts"),
                    )
                    .when(self.host_editor.is_none(), |header| {
                        header.child(settings_primary_button(
                            "Add Host",
                            "add-remote-host",
                            Some("plus"),
                            cx,
                            |this, window, cx| this.begin_adding_host(window, cx),
                        ))
                    }),
            )
            .when_some(self.host_initialization.clone(), |section, state| {
                section.child(self.host_initialization_card(state, cx))
            })
            .child(catalog)
    }

    fn host_initialization_card(
        &self,
        state: HostInitialization,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let colors = self.settings_colors();
        let HostInitializationCardModel {
            id,
            name,
            symbol,
            title,
            detail,
            tone,
            action,
            retry_kind,
        } = match state {
            HostInitialization::Running { id, name, kind, .. } => {
                let (title, detail) = match kind {
                    HostPreparationKind::Initialize => (
                        format!("Setting up {name}"),
                        "Connecting with SSH, verifying homie-remote, loading the login environment, and testing session persistence."
                            .to_owned(),
                    ),
                    HostPreparationKind::Reinstall => (
                        format!("Reinstalling {name}"),
                        "Uploading and verifying the packaged homie-remote build, then refreshing the remote environment checks. Running sessions are not interrupted."
                            .to_owned(),
                    ),
                };
                HostInitializationCardModel {
                    id,
                    name,
                    symbol: None,
                    title,
                    detail,
                    tone: Palette::CLAY,
                    action: None,
                    retry_kind: None,
                }
            }
            HostInitialization::Ready {
                id,
                name,
                kind,
                result,
                ..
            } => {
                let persistence = match result.persistence {
                    homie_proto::remote_pty::PersistenceCapability::NativeDetach => "native detach",
                    homie_proto::remote_pty::PersistenceCapability::UserSupervisor => {
                        "user supervisor"
                    }
                    homie_proto::remote_pty::PersistenceCapability::NonPersistent => {
                        "non-persistent"
                    }
                };
                let title = match kind {
                    HostPreparationKind::Initialize => format!("{name} is ready"),
                    HostPreparationKind::Reinstall => {
                        format!("Remote environment reinstalled on {name}")
                    }
                };
                let action = (kind == HostPreparationKind::Initialize).then_some("Use by default");
                HostInitializationCardModel {
                    id,
                    name: name.clone(),
                    symbol: Some("checkmark.circle.fill"),
                    title,
                    detail: format!(
                        "{} · {} · build {} · protocol {}.{} · {persistence}",
                        result.cwd,
                        result.shell,
                        result.helper_build_id,
                        result.protocol.major,
                        result.protocol.minor
                    ),
                    tone: Ink::FRESH,
                    action,
                    retry_kind: None,
                }
            }
            HostInitialization::Failed {
                id,
                name,
                kind,
                message,
                ..
            } => {
                let title = match kind {
                    HostPreparationKind::Initialize => {
                        format!("Could not initialize {name}")
                    }
                    HostPreparationKind::Reinstall => {
                        format!("Could not reinstall the remote environment on {name}")
                    }
                };
                HostInitializationCardModel {
                    id,
                    name,
                    symbol: Some("exclamationmark.triangle.fill"),
                    title,
                    detail: message,
                    tone: Ink::DANGER,
                    action: Some("Retry"),
                    retry_kind: Some(kind),
                }
            }
        };
        let action_id = id.clone();
        let status_mark = symbol.map_or_else(
            || LoadingIndicator::new("host-initialization-loading", 16.0, tone).into_any_element(),
            |symbol| sf_symbol(symbol, 13.0, tone),
        );
        div()
            .id("host-initialization")
            .debug_selector(|| "HOST_INITIALIZATION".into())
            .rounded(px(Radius::ROW))
            .border_1()
            .border_color(tone.alpha(0.22))
            .bg(tone.alpha(0.055))
            .px(px(12.0))
            .py(px(10.0))
            .flex()
            .items_start()
            .gap(px(9.0))
            .child(div().pt(px(1.0)).text_color(tone).child(status_mark))
            .child(
                div()
                    .min_w(px(0.0))
                    .flex_1()
                    .flex()
                    .flex_col()
                    .gap(px(3.0))
                    .child(
                        div()
                            .text_size(px(11.5))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(colors.primary)
                            .child(title),
                    )
                    .child(
                        div()
                            .min_w(px(0.0))
                            .w_full()
                            .whitespace_normal()
                            .text_size(px(10.5))
                            .line_height(px(15.0))
                            .text_color(colors.tertiary)
                            .child(wrappable_setting_copy(detail.into())),
                    ),
            )
            .when_some(action, |card, label| {
                card.child(
                    div()
                        .debug_selector(|| "HOST_INITIALIZATION_ACTION".into())
                        .child(surface_button(
                            label,
                            "host-initialization-action",
                            colors,
                            cx,
                            move |this, cx| {
                                if let Some(kind) = retry_kind {
                                    this.retry_host_initialization(&action_id, kind, cx);
                                } else {
                                    this.store
                                        .write()
                                        .expect("session store lock poisoned")
                                        .set_default_spawn_host(Some(action_id.clone()));
                                    this.activity =
                                        format!("{name} is now the default execution host");
                                    cx.notify();
                                }
                            },
                        )),
                )
            })
            .into_any_element()
    }

    fn host_editor_panel(&self, editor: &HostEditor, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = self.settings_colors();
        let editing = editor.original_id.is_some();
        let title = if editing { "Edit host" } else { "Add a host" };
        let mut form = div()
            .p(px(14.0))
            .flex()
            .flex_col()
            .gap(px(12.0))
            .child(
                div()
                    .flex()
                    .items_start()
                    .justify_between()
                    .gap(px(12.0))
                    .child(
                        div()
                            .flex_1()
                            .min_w(px(0.0))
                            .flex()
                            .flex_col()
                            .gap(px(3.0))
                            .child(
                                div()
                                    .text_size(px(Typo::TITLE.size))
                                    .font_weight(Typo::TITLE.weight)
                                    .child(title),
                            )
                            .child(
                                div()
                                    .text_size(px(11.0))
                                    .text_color(colors.tertiary)
                                    .child("SSH aliases from ~/.ssh/config work too."),
                            ),
                    )
                    .child(
                        div()
                            .id("cancel-host-editor")
                            .size(px(24.0))
                            .rounded(px(Radius::BADGE))
                            .flex()
                            .items_center()
                            .justify_center()
                            .cursor_pointer()
                            .hover(move |style| style.bg(colors.primary.alpha(0.08)))
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.host_editor = None;
                                cx.notify();
                            }))
                            .child(sf_symbol("xmark", 11.0, colors.secondary)),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(10.0))
                    .child(
                        div()
                            .flex()
                            .gap(px(10.0))
                            .child(
                                div().min_w(px(0.0)).flex_1().child(self.host_text_field(
                                    "Name",
                                    "Forge",
                                    &editor.name,
                                    HostFormField::Name,
                                    cx,
                                )),
                            )
                            .child(
                                div().min_w(px(0.0)).flex_1().child(self.host_text_field(
                                    "SSH destination",
                                    "you@forge",
                                    &editor.ssh,
                                    HostFormField::Ssh,
                                    cx,
                                )),
                            ),
                    )
                    .child(self.host_text_field(
                        "Default folder",
                        "~/code (optional)",
                        &editor.default_cwd,
                        HostFormField::DefaultCwd,
                        cx,
                    ))
                    .child(
                        div()
                            .pt(px(2.0))
                            .text_size(px(10.0))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(colors.secondary)
                            .child("First-party node (optional)"),
                    )
                    .child(
                        div()
                            .flex()
                            .gap(px(10.0))
                            .child(
                                div().min_w(px(0.0)).flex_1().child(self.host_text_field(
                                    "Node endpoint",
                                    "tcp://100.64.0.2:7337",
                                    &editor.node_endpoint,
                                    HostFormField::NodeEndpoint,
                                    cx,
                                )),
                            )
                            .child(
                                div().min_w(px(0.0)).flex_1().child(self.host_text_field(
                                    "Local token file",
                                    "~/.config/homie/forge.token",
                                    &editor.node_token_file,
                                    HostFormField::NodeTokenFile,
                                    cx,
                                )),
                            ),
                    )
                    .child(self.host_text_field(
                        "Pinned node ID",
                        "node-a1b2c3d4 (recommended after first hello)",
                        &editor.node_id,
                        HostFormField::NodeId,
                        cx,
                    ))
                    .child(
                        div()
                            .min_w(px(0.0))
                            .w_full()
                            .whitespace_normal()
                            .text_size(px(10.0))
                            .line_height(px(15.0))
                            .text_color(colors.tertiary)
                            .child(wrappable_setting_copy(
                                "The token stays in that owner-only file. SSH remains available for install and recovery."
                                    .into(),
                            )),
                    ),
            );

        if let Some(error) = &editor.error {
            form = form.child(
                div()
                    .px(px(10.0))
                    .py(px(8.0))
                    .rounded(px(Radius::BADGE))
                    .bg(Ink::DANGER.alpha(0.08))
                    .flex()
                    .items_center()
                    .gap(px(7.0))
                    .text_size(px(11.0))
                    .text_color(Ink::DANGER)
                    .child(sf_symbol("exclamationmark.triangle", 12.0, Ink::DANGER))
                    .child(
                        div()
                            .flex_1()
                            .min_w(px(0.0))
                            .whitespace_normal()
                            .child(wrappable_setting_copy(error.clone().into())),
                    ),
            );
        }

        if editor.confirm_remove {
            let name = if editor.name.is_empty() {
                "this host".to_owned()
            } else {
                editor.name.text().to_owned()
            };
            form = form.child(
                div()
                    .pt(px(2.0))
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .justify_between()
                    .gap(px(12.0))
                    .child(
                        div()
                            .flex_1()
                            .min_w(px(0.0))
                            .whitespace_normal()
                            .text_size(px(11.0))
                            .text_color(colors.secondary)
                            .child(wrappable_setting_copy(
                                format!("Remove {name}? Existing sessions must be moved first.")
                                    .into(),
                            )),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(7.0))
                            .child(surface_button(
                                "Keep Host",
                                "keep-host",
                                colors,
                                cx,
                                |this, cx| {
                                    if let Some(editor) = &mut this.host_editor {
                                        editor.confirm_remove = false;
                                    }
                                    cx.notify();
                                },
                            ))
                            .child(settings_danger_button(
                                "Remove Host",
                                "confirm-remove-host",
                                cx,
                                |this, cx| this.request_remove_host(cx),
                            )),
                    ),
            );
        } else {
            let reinstall_host_id = editor.original_id.clone();
            form = form.child(
                div()
                    .pt(px(2.0))
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap(px(8.0))
                    .child(div().when(editing, |left| {
                        let host_id = reinstall_host_id.expect("editing host id");
                        left.flex()
                            .flex_wrap()
                            .items_center()
                            .gap(px(7.0))
                            .child(settings_danger_button(
                                "Remove",
                                "remove-host",
                                cx,
                                |this, cx| this.request_remove_host(cx),
                            ))
                            .child(
                                div()
                                    .debug_selector(|| "REINSTALL_REMOTE_ENVIRONMENT".into())
                                    .child(surface_button(
                                        "Reinstall Environment",
                                        "reinstall-remote-environment",
                                        colors,
                                        cx,
                                        move |this, cx| this.reinstall_host(&host_id, cx),
                                    )),
                            )
                    }))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(7.0))
                            .child(surface_button(
                                "Cancel",
                                "cancel-host",
                                colors,
                                cx,
                                |this, cx| {
                                    this.host_editor = None;
                                    cx.notify();
                                },
                            ))
                            .child(settings_primary_button(
                                if editing { "Save Host" } else { "Add Host" },
                                "save-host",
                                None,
                                cx,
                                |this, _, cx| this.save_host(cx),
                            )),
                    ),
            );
        }
        form
    }

    fn host_text_field(
        &self,
        label: &'static str,
        placeholder: &'static str,
        editor: &QueryEditor,
        field: HostFormField,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let colors = self.settings_colors();
        let active = self
            .host_editor
            .as_ref()
            .is_some_and(|host_editor| host_editor.active_field == field);
        let value = host_field_value(editor, placeholder, active, field, colors);
        let bounds_slot = Rc::clone(&self.host_field_bounds[field.index()]);
        div()
            .min_w(px(0.0))
            .flex()
            .flex_col()
            .gap(px(5.0))
            .child(
                div()
                    .text_size(px(10.0))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(colors.secondary)
                    .child(label),
            )
            .child({
                let debug_name = field.debug_name();
                div()
                    .id(SharedString::from(format!("host-field-{field:?}")))
                    .debug_selector(move || format!("HOST_FIELD_{debug_name}"))
                    .relative()
                    .min_w(px(0.0))
                    .h(px(34.0))
                    .px(px(10.0))
                    .rounded(px(Radius::BADGE))
                    .border_1()
                    .border_color(colors.primary.alpha(if active { 0.26 } else { 0.11 }))
                    .bg(colors.primary.alpha(if active { 0.075 } else { 0.04 }))
                    .flex()
                    .items_center()
                    .overflow_hidden()
                    .font_family(crate::fonts::mono_family())
                    .text_size(px(11.0))
                    .text_color(colors.primary)
                    .cursor(CursorStyle::IBeam)
                    .on_click(cx.listener(move |this, event: &ClickEvent, window, cx| {
                        this.select_host_field(field, window, cx);
                        let Some(bounds) = this.host_field_bounds[field.index()].get() else {
                            return;
                        };
                        let x = (event.position().x - bounds.left() - px(10.0)).max(px(0.0));
                        let offset = this.host_editor.as_ref().map_or(0, |editor| {
                            text_offset_for_x(editor.field(field).text(), x, window, colors)
                        });
                        if let Some(editor) = &mut this.host_editor {
                            editor
                                .field_mut()
                                .set_cursor(offset, event.modifiers().shift);
                        }
                        cx.notify();
                    }))
                    .child(value)
                    .child(
                        canvas(
                            move |bounds, _, _| bounds_slot.set(Some(bounds)),
                            |_, _, _, _| {},
                        )
                        .absolute()
                        .inset_0(),
                    )
            })
    }

    fn terminal_theme_dropdown(&self, cx: &mut Context<Self>) -> AnyElement {
        let selected = theme(&self.prefs.terminal_theme);
        let colors = self.settings_colors();
        let open = self.settings_menu == Some(SettingsMenu::TerminalTheme);
        let mut control = div()
            .relative()
            .min_w(px(158.0))
            .child(settings_select_button(
                selected.name,
                "terminal-theme-dropdown",
                open,
                SettingsMenu::TerminalTheme,
                colors,
                cx,
            ));
        if open {
            let mut options = div()
                .id("terminal-theme-options")
                .max_h(px(300.0))
                .overflow_y_scroll()
                .p(px(4.0))
                .flex()
                .flex_col();
            for appearance in ThemeAppearance::ALL {
                options = options.child(
                    div()
                        .h(px(24.0))
                        .px(px(8.0))
                        .flex()
                        .items_center()
                        .text_size(px(Typo::META.size - 1.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(colors.tertiary)
                        .child(appearance.label()),
                );
                for (index, candidate) in TermTheme::CATALOG
                    .into_iter()
                    .enumerate()
                    .filter(|(_, theme)| theme.appearance == appearance)
                {
                    let is_selected = candidate.id == selected.id;
                    options =
                        options.child(
                            div()
                                .id(SharedString::from(format!("terminal-theme-option-{index}")))
                                .h(px(Metrics::ROW_HEIGHT))
                                .px(px(8.0))
                                .rounded(px(Radius::ROW))
                                .flex()
                                .items_center()
                                .gap(px(8.0))
                                .bg(Fill::selected(colors, is_selected))
                                .cursor_pointer()
                                .hover(move |style| style.bg(colors.primary.alpha(0.08)))
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.prefs.terminal_theme = candidate.id.to_owned();
                                    this.settings_menu = None;
                                    this.persist_prefs();
                                    cx.notify();
                                }))
                                .child(
                                    div()
                                        .size(px(16.0))
                                        .rounded(px(5.0))
                                        .bg(candidate.background)
                                        .border_1()
                                        .border_color(colors.primary.alpha(0.16))
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .child(
                                            div()
                                                .size(px(5.0))
                                                .rounded(px(2.5))
                                                .bg(candidate.foreground),
                                        ),
                                )
                                .child(div().flex().gap(px(2.0)).children(
                                    [candidate.ansi[2], candidate.ansi[4], candidate.ansi[5]].map(
                                        |color| div().size(px(5.0)).rounded(px(2.5)).bg(color),
                                    ),
                                ))
                                .child(
                                    div()
                                        .flex_1()
                                        .text_size(px(Typo::ROW.size))
                                        .child(candidate.name),
                                )
                                .when(is_selected, |row| {
                                    row.child(sf_symbol("checkmark", 10.0, colors.secondary))
                                }),
                        );
                }
            }
            control = control.child(settings_dropdown(options, 252.0, colors));
        }
        control.into_any_element()
    }

    fn hibernate_dropdown(&self, cx: &mut Context<Self>) -> AnyElement {
        const OPTIONS: [(u32, &str); 5] = [
            (0, "Off"),
            (5, "5 minutes"),
            (15, "15 minutes"),
            (30, "30 minutes"),
            (60, "1 hour"),
        ];
        let colors = self.settings_colors();
        let selected_label = OPTIONS
            .into_iter()
            .find_map(|(value, label)| {
                (value == self.prefs.hibernate_after_minutes).then_some(label)
            })
            .unwrap_or("Off");
        let open = self.settings_menu == Some(SettingsMenu::HibernateAfter);
        let mut control = div()
            .relative()
            .min_w(px(132.0))
            .child(settings_select_button(
                selected_label,
                "hibernate-dropdown",
                open,
                SettingsMenu::HibernateAfter,
                colors,
                cx,
            ));
        if open {
            let mut options = div().p(px(4.0)).flex().flex_col();
            for (index, (value, label)) in OPTIONS.into_iter().enumerate() {
                let is_selected = value == self.prefs.hibernate_after_minutes;
                options = options.child(settings_choice_row(
                    format!("hibernate-option-{index}"),
                    label,
                    is_selected,
                    colors,
                    cx,
                    move |this, cx| {
                        this.prefs.hibernate_after_minutes = value;
                        this.settings_menu = None;
                        this.persist_prefs();
                        cx.notify();
                    },
                ));
            }
            control = control.child(settings_dropdown(options, 172.0, colors));
        }
        control.into_any_element()
    }

    fn memory_dropdown(&self, cx: &mut Context<Self>) -> AnyElement {
        const OPTIONS: [u64; 4] = [2, 4, 6, 8];
        let colors = self.settings_colors();
        let open = self.settings_menu == Some(SettingsMenu::MemoryLimit);
        let mut control = div()
            .relative()
            .min_w(px(104.0))
            .child(settings_select_button(
                format!("{} GB", self.prefs.memory_hard_limit_gb),
                "memory-dropdown",
                open,
                SettingsMenu::MemoryLimit,
                colors,
                cx,
            ));
        if open {
            let mut options = div().p(px(4.0)).flex().flex_col();
            for (index, value) in OPTIONS.into_iter().enumerate() {
                let is_selected = value == self.prefs.memory_hard_limit_gb;
                options = options.child(settings_choice_row(
                    format!("memory-option-{index}"),
                    format!("{value} GB"),
                    is_selected,
                    colors,
                    cx,
                    move |this, cx| {
                        this.prefs.memory_hard_limit_gb = value;
                        this.settings_menu = None;
                        this.persist_prefs();
                        cx.notify();
                    },
                ));
            }
            control = control.child(settings_dropdown(options, 132.0, colors));
        }
        control.into_any_element()
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

fn surface_button(
    label: impl Into<SharedString>,
    id: impl Into<SharedString>,
    colors: SemanticColors,
    cx: &mut Context<UtilitySurfaces>,
    handler: impl Fn(&mut UtilitySurfaces, &mut Context<UtilitySurfaces>) + 'static,
) -> impl IntoElement {
    div()
        .id(id.into())
        .h(px(26.0))
        .px(px(9.0))
        .rounded(px(Radius::BADGE))
        .border_1()
        .border_color(colors.primary.alpha(0.10))
        .bg(colors.primary.alpha(0.04))
        .flex()
        .items_center()
        .text_size(px(11.0))
        .cursor_pointer()
        .hover(move |style| style.bg(colors.primary.alpha(0.09)))
        .on_click(cx.listener(move |this, _, _, cx| handler(this, cx)))
        .child(label.into())
}

fn settings_primary_button(
    label: &'static str,
    id: &'static str,
    icon: Option<&'static str>,
    cx: &mut Context<UtilitySurfaces>,
    handler: impl Fn(&mut UtilitySurfaces, &mut Window, &mut Context<UtilitySurfaces>) + 'static,
) -> impl IntoElement {
    div()
        .id(id)
        .h(px(28.0))
        .px(px(10.0))
        .rounded(px(Radius::BADGE))
        .bg(Palette::CLAY.alpha(0.90))
        .text_color(rgba(0x190f0bff))
        .flex()
        .items_center()
        .gap(px(6.0))
        .text_size(px(11.0))
        .font_weight(FontWeight::SEMIBOLD)
        .cursor_pointer()
        .hover(|style| style.bg(Palette::CLAY))
        .on_click(cx.listener(move |this, _, window, cx| handler(this, window, cx)))
        .when_some(icon, |button, icon| {
            button.child(sf_symbol(icon, 10.0, rgba(0x190f0bff)))
        })
        .child(label)
}

fn settings_danger_button(
    label: &'static str,
    id: &'static str,
    cx: &mut Context<UtilitySurfaces>,
    handler: impl Fn(&mut UtilitySurfaces, &mut Context<UtilitySurfaces>) + 'static,
) -> impl IntoElement {
    div()
        .id(id)
        .h(px(26.0))
        .px(px(9.0))
        .rounded(px(Radius::BADGE))
        .bg(Ink::DANGER.alpha(0.11))
        .text_color(Ink::DANGER)
        .flex()
        .items_center()
        .text_size(px(11.0))
        .cursor_pointer()
        .hover(|style| style.bg(Ink::DANGER.alpha(0.18)))
        .on_click(cx.listener(move |this, _, _, cx| handler(this, cx)))
        .child(label)
}

fn host_field_value(
    editor: &QueryEditor,
    placeholder: &'static str,
    active: bool,
    field: HostFormField,
    colors: SemanticColors,
) -> AnyElement {
    let debug_name = field.debug_name();
    let content = if active {
        div()
            .debug_selector(|| "host-field-caret".into())
            .child(query_label(editor))
            .into_any_element()
    } else if editor.is_empty() {
        div()
            .debug_selector(move || format!("HOST_FIELD_PLACEHOLDER_{debug_name}"))
            .text_color(colors.tertiary)
            .child(placeholder)
            .into_any_element()
    } else {
        div().child(editor.text().to_owned()).into_any_element()
    };
    div()
        .min_w(px(0.0))
        .w_full()
        .overflow_hidden()
        .whitespace_nowrap()
        .text_ellipsis()
        .child(content)
        .into_any_element()
}

fn text_offset_for_x(text: &str, x: Pixels, window: &Window, colors: SemanticColors) -> usize {
    if text.is_empty() {
        return 0;
    }
    let run = TextRun {
        len: text.len(),
        font: font(crate::fonts::mono_family()),
        color: colors.primary.into(),
        ..TextRun::default()
    };
    window
        .text_system()
        .shape_line(text.to_owned().into(), px(11.0), &[run], None)
        .closest_index_for_x(x)
}

fn danger_button(
    label: &'static str,
    cx: &mut Context<UtilitySurfaces>,
    handler: impl Fn(&mut UtilitySurfaces, &mut Context<UtilitySurfaces>) + 'static,
) -> impl IntoElement {
    div()
        .id("confirm-cleanup")
        .h(px(26.0))
        .px(px(9.0))
        .rounded(px(Radius::BADGE))
        .bg(Ink::DANGER.alpha(0.16))
        .text_color(Ink::DANGER)
        .flex()
        .items_center()
        .text_size(px(11.0))
        .cursor_pointer()
        .on_click(cx.listener(move |this, _, _, cx| handler(this, cx)))
        .child(label)
}

fn toggle_row(
    label: &'static str,
    detail: &'static str,
    enabled: bool,
    id: &'static str,
    colors: SemanticColors,
    cx: &mut Context<UtilitySurfaces>,
    handler: impl Fn(&mut UtilitySurfaces, &mut Context<UtilitySurfaces>) + 'static,
) -> impl IntoElement {
    div()
        .id(id)
        .min_h(px(SETTINGS_ROW_HEIGHT))
        .px(px(12.0))
        .flex()
        .items_center()
        .justify_between()
        .gap(px(12.0))
        .cursor_pointer()
        .hover(move |style| style.bg(colors.primary.alpha(0.025)))
        .on_click(cx.listener(move |this, _, _, cx| handler(this, cx)))
        .child(setting_text_stack(label.into(), detail.into(), colors))
        .child(
            div()
                .flex_none()
                .w(px(30.0))
                .h(px(18.0))
                .p(px(2.0))
                .rounded(px(9.0))
                .bg(if enabled {
                    Ink::FRESH.alpha(0.72)
                } else {
                    colors.primary.alpha(0.14)
                })
                .flex()
                .justify_end()
                .when(!enabled, |toggle| toggle.justify_start())
                .child(div().size(px(14.0)).rounded(px(7.0)).bg(colors.primary)),
        )
}

fn setting_section(
    title: &'static str,
    content: impl IntoElement,
    colors: SemanticColors,
) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap(px(6.0))
        .child(
            div()
                .px(px(1.0))
                .text_size(px(Typo::SECTION_HEADER.size))
                .font_weight(Typo::SECTION_HEADER.weight)
                .text_color(colors.tertiary)
                .child(title),
        )
        .child(
            div()
                .rounded(px(Radius::ROW))
                .border_1()
                .border_color(colors.primary.alpha(0.065))
                .bg(colors.primary.alpha(0.02))
                .overflow_hidden()
                .child(content),
        )
}

pub(crate) fn setting_row(
    label: impl Into<SharedString>,
    detail: impl Into<SharedString>,
    control: impl IntoElement,
    colors: SemanticColors,
) -> impl IntoElement {
    let label = label.into();
    let detail = detail.into();
    div()
        .debug_selector(|| "settings-row".into())
        .min_h(px(SETTINGS_ROW_HEIGHT))
        .px(px(12.0))
        .flex()
        .items_center()
        .justify_between()
        .gap(px(12.0))
        .child(setting_text_stack(label, detail, colors))
        .child(control)
}

fn setting_text_stack(
    label: SharedString,
    detail: SharedString,
    colors: SemanticColors,
) -> AnyElement {
    div()
        .flex_1()
        .min_w(px(0.0))
        .flex()
        .flex_col()
        .gap(px(2.0))
        .child(
            div()
                .min_w(px(0.0))
                .w_full()
                .whitespace_normal()
                .text_size(px(Typo::ROW_EMPHASIZED.size))
                .font_weight(Typo::ROW_EMPHASIZED.weight)
                .text_color(colors.primary)
                .child(wrappable_setting_copy(label)),
        )
        .child(
            div()
                .debug_selector(|| "settings-row-detail".into())
                .min_w(px(0.0))
                .w_full()
                .whitespace_normal()
                .text_size(px(Typo::META.size))
                .line_height(px(14.0))
                .text_color(colors.tertiary)
                .child(wrappable_setting_copy(detail)),
        )
        .into_any_element()
}

fn wrappable_setting_copy(text: SharedString) -> SharedString {
    let mut output = String::with_capacity(text.len());
    let mut uninterrupted = 0usize;
    for character in text.chars() {
        output.push(character);
        if character.is_whitespace() {
            uninterrupted = 0;
            continue;
        }
        uninterrupted += 1;
        if matches!(
            character,
            '/' | '\\' | '.' | ':' | '-' | '_' | '@' | '?' | '&' | '='
        ) || uninterrupted >= 24
        {
            output.push('\u{200b}');
            uninterrupted = 0;
        }
    }
    output.into()
}

fn settings_note(
    icon: &'static str,
    title: Option<&'static str>,
    body: &'static str,
    icon_color: Rgba,
    border_color: Rgba,
    background: Rgba,
    colors: SemanticColors,
) -> AnyElement {
    div()
        .p(px(12.0))
        .rounded(px(Radius::ROW))
        .border_1()
        .border_color(border_color)
        .bg(background)
        .flex()
        .items_start()
        .gap(px(10.0))
        .child(sf_symbol(icon, 15.0, icon_color))
        .child(
            div()
                .flex_1()
                .min_w(px(0.0))
                .flex()
                .flex_col()
                .gap(px(4.0))
                .when_some(title, |copy, title| {
                    copy.child(
                        div()
                            .min_w(px(0.0))
                            .w_full()
                            .whitespace_normal()
                            .text_size(px(Typo::TITLE.size))
                            .font_weight(Typo::TITLE.weight)
                            .child(wrappable_setting_copy(title.into())),
                    )
                })
                .child(
                    div()
                        .debug_selector(|| "SETTINGS_NOTE_COPY".into())
                        .min_w(px(0.0))
                        .w_full()
                        .whitespace_normal()
                        .text_size(px(11.0))
                        .line_height(px(17.0))
                        .text_color(colors.secondary)
                        .child(wrappable_setting_copy(body.into())),
                ),
        )
        .into_any_element()
}

fn settings_page(
    title: &'static str,
    content: impl IntoElement,
    colors: SemanticColors,
) -> impl IntoElement {
    div()
        .w_full()
        .px(px(20.0))
        .pt(px(13.0))
        .pb(px(22.0))
        .flex()
        .flex_col()
        .gap(px(16.0))
        .child(
            div()
                .pr(px(34.0))
                .h(px(22.0))
                .flex()
                .items_center()
                .text_size(px(Typo::DISPLAY_TITLE.size))
                .font_weight(Typo::DISPLAY_TITLE.weight)
                .text_color(colors.primary)
                .child(title),
        )
        .child(content)
}

fn setting_divider(colors: SemanticColors) -> impl IntoElement {
    div()
        .h(px(1.0))
        .mx(px(12.0))
        .bg(colors.primary.alpha(0.055))
}

fn settings_select_button(
    label: impl Into<SharedString>,
    id: impl Into<SharedString>,
    open: bool,
    menu: SettingsMenu,
    colors: SemanticColors,
    cx: &mut Context<UtilitySurfaces>,
) -> impl IntoElement {
    div()
        .id(id.into())
        .h(px(28.0))
        .px(px(9.0))
        .rounded(px(Radius::BADGE))
        .border_1()
        .border_color(colors.primary.alpha(if open { 0.20 } else { 0.10 }))
        .bg(colors.primary.alpha(if open { 0.08 } else { 0.04 }))
        .flex()
        .items_center()
        .gap(px(8.0))
        .cursor_pointer()
        .hover(move |style| style.bg(colors.primary.alpha(0.075)))
        .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
        .on_click(cx.listener(move |this, _, _, cx| {
            this.settings_menu = if this.settings_menu == Some(menu) {
                None
            } else {
                Some(menu)
            };
            cx.notify();
        }))
        .child(
            div()
                .flex_1()
                .text_size(px(Typo::META.size))
                .font_weight(Typo::META.weight)
                .text_color(colors.primary)
                .child(label.into()),
        )
        .child(sf_symbol_weighted(
            if open { "chevron.up" } else { "chevron.down" },
            8.0,
            SymbolWeight::Semibold,
            colors.tertiary,
        ))
}

fn settings_dropdown(
    content: impl IntoElement,
    width: f32,
    colors: SemanticColors,
) -> impl IntoElement {
    deferred(
        div()
            .absolute()
            .top(px(32.0))
            .right_0()
            .w(px(width))
            .occlude()
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .child(FloatingSurface::new(colors, content)),
    )
    .with_priority(5)
}

fn settings_choice_row(
    id: impl Into<SharedString>,
    label: impl Into<SharedString>,
    selected: bool,
    colors: SemanticColors,
    cx: &mut Context<UtilitySurfaces>,
    handler: impl Fn(&mut UtilitySurfaces, &mut Context<UtilitySurfaces>) + 'static,
) -> impl IntoElement {
    div()
        .id(id.into())
        .h(px(Metrics::ROW_HEIGHT))
        .px(px(8.0))
        .rounded(px(Radius::ROW))
        .flex()
        .items_center()
        .gap(px(8.0))
        .bg(Fill::selected(colors, selected))
        .cursor_pointer()
        .hover(move |style| style.bg(colors.primary.alpha(0.08)))
        .on_click(cx.listener(move |this, _, _, cx| handler(this, cx)))
        .child(
            div()
                .flex_1()
                .text_size(px(Typo::ROW.size))
                .child(label.into()),
        )
        .when(selected, |row| {
            row.child(sf_symbol("checkmark", 10.0, colors.secondary))
        })
}

fn theme_preview(theme: TermTheme, colors: SemanticColors) -> impl IntoElement {
    div()
        .p(px(10.0))
        .rounded(px(Radius::BADGE))
        .border_1()
        .border_color(colors.primary.alpha(0.10))
        .bg(theme.background)
        .flex()
        .flex_col()
        .gap(px(6.0))
        .font_family(crate::fonts::mono_family())
        .text_size(px(11.0))
        .child(
            div()
                .flex()
                .child(div().text_color(theme.ansi[2]).child("❯ "))
                .child(div().text_color(theme.foreground).child("cargo test"))
                .child(div().text_color(theme.cursor).child("█")),
        )
        .child(div().text_color(theme.ansi[8]).child("test result: ok"))
        .child(
            div().flex().gap(px(3.0)).children(
                theme
                    .ansi
                    .into_iter()
                    .map(|color| div().flex_1().h(px(10.0)).rounded(px(2.0)).bg(color)),
            ),
        )
}

fn chip(label: String, colors: SemanticColors) -> impl IntoElement {
    div()
        .px(px(5.0))
        .py(px(2.0))
        .rounded(px(Radius::CHIP))
        .bg(colors.primary.alpha(0.09))
        .text_size(px(11.0))
        .text_color(colors.secondary)
        .child(label)
}

fn colored_badge(label: &'static str, color: Rgba) -> impl IntoElement {
    div()
        .px(px(5.0))
        .py(px(2.0))
        .rounded(px(Radius::CHIP))
        .bg(color.alpha(0.12))
        .text_size(px(11.0))
        .text_color(color)
        .child(label)
}

fn empty_label(label: &str, colors: SemanticColors) -> impl IntoElement {
    div()
        .h(px(42.0))
        .px(px(10.0))
        .flex()
        .items_center()
        .text_size(px(13.0))
        .text_color(colors.tertiary)
        .child(label.to_owned())
}
