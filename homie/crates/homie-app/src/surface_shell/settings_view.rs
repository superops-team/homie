use super::widgets::*;
use super::*;

impl UtilitySurfaces {
    pub(super) fn render_settings(&self, cx: &mut Context<Self>) -> impl IntoElement {
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
