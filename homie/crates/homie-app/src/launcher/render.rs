use super::*;

impl super::LauncherOverlay {
    fn render_harness_picker(&self, colors: SemanticColors, cx: &mut Context<Self>) -> AnyElement {
        let mut list = div()
            .id("launcher-harness-list")
            .py(px(6.0))
            .w(px(260.0))
            .max_h(px(PICKER_HEIGHT))
            .overflow_y_scroll();
        for (index, choice) in self.harness_choices().into_iter().enumerate() {
            let selected = choice.kind == self.selected_harness;
            let enabled = choice.available;
            let highlighted = self.highlight == index;
            let kind = choice.kind.clone();
            let logo = ui_agent_kind(&choice.kind);
            list = list.child(
                div()
                    .id(format!("launcher-harness-{index}"))
                    .mx(px(6.0))
                    .h(px(38.0))
                    .px(px(9.0))
                    .flex()
                    .items_center()
                    .gap(px(9.0))
                    .rounded(px(8.0))
                    .text_size(px(12.0))
                    .text_color(if enabled {
                        colors.primary
                    } else {
                        colors.tertiary
                    })
                    .when(highlighted && enabled, |row| {
                        row.bg(colors.primary.alpha(0.08))
                    })
                    .when(enabled, |row| {
                        row.cursor_pointer()
                            .hover(move |row| row.bg(colors.primary.alpha(0.06)))
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.selected_harness = kind.clone();
                                this.picker = None;
                                cx.notify();
                            }))
                    })
                    .child(AgentLogo::new(logo, 21.0, colors))
                    .child(div().flex_1().child(choice.label))
                    .when(!enabled, |row| {
                        row.child(
                            div()
                                .text_size(px(9.0))
                                .text_color(colors.tertiary)
                                .child("Unavailable"),
                        )
                    })
                    .when(selected, |row| {
                        row.child(sf_symbol_weighted(
                            "checkmark",
                            9.0,
                            SymbolWeight::Semibold,
                            colors.secondary,
                        ))
                    }),
            );
        }
        FloatingSurface::new(colors, list).into_any_element()
    }

    fn render_project_picker(&self, colors: SemanticColors, cx: &mut Context<Self>) -> AnyElement {
        let projects = self.projects();
        let mut list = div()
            .id("launcher-project-list")
            .py(px(6.0))
            .w(px(310.0))
            .max_h(px(PICKER_HEIGHT))
            .overflow_y_scroll();
        if projects.is_empty() {
            list = list.child(
                div()
                    .h(px(38.0))
                    .px(px(11.0))
                    .flex()
                    .items_center()
                    .text_size(px(11.0))
                    .text_color(colors.tertiary)
                    .child("No recent projects"),
            );
        }
        for (index, project) in projects.into_iter().enumerate() {
            let selected = project.root == self.selected_root;
            let highlighted = self.highlight == index;
            let root = project.root.clone();
            list = list.child(
                div()
                    .id(format!("launcher-project-{index}"))
                    .mx(px(6.0))
                    .min_h(px(44.0))
                    .px(px(9.0))
                    .py(px(6.0))
                    .flex()
                    .items_center()
                    .gap(px(9.0))
                    .rounded(px(8.0))
                    .cursor_pointer()
                    .when(highlighted, |row| row.bg(colors.primary.alpha(0.08)))
                    .hover(move |row| row.bg(colors.primary.alpha(0.06)))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.selected_root.clone_from(&root);
                        this.picker = None;
                        cx.notify();
                    }))
                    .child(sf_symbol("folder", 12.0, colors.secondary))
                    .child(
                        div()
                            .flex_1()
                            .min_w(px(0.0))
                            .flex()
                            .flex_col()
                            .gap(px(1.0))
                            .child(
                                div()
                                    .text_size(px(12.0))
                                    .text_color(colors.primary)
                                    .child(project.name),
                            )
                            .child(
                                div()
                                    .text_size(px(9.0))
                                    .text_color(colors.tertiary)
                                    .whitespace_nowrap()
                                    .overflow_hidden()
                                    .child(project.root),
                            ),
                    )
                    .when(selected, |row| {
                        row.child(sf_symbol_weighted(
                            "checkmark",
                            9.0,
                            SymbolWeight::Semibold,
                            colors.secondary,
                        ))
                    }),
            );
        }
        FloatingSurface::new(colors, list).into_any_element()
    }

    fn render_panel(
        &self,
        colors: SemanticColors,
        focused: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let can_submit = self.can_submit();
        let harness_open = self.picker == Some(Picker::Harness);
        let project_open = self.picker == Some(Picker::Project);
        let text_height = composer_text_height(self.prompt.line_count());
        let composer_height = text_height + COMPOSER_CONTROLS_HEIGHT;
        // The pickers hang off the bottom of the panel, which now moves with
        // the composer.
        let picker_top = TITLE_HEIGHT + TITLE_GAP + composer_height + SHELF_HEIGHT + 8.0;
        let blocker = self.blocker();
        let harness_label = self.selected_harness_label();
        let project_label = self.selected_project_label();
        let logo = ui_agent_kind(&self.selected_harness);
        let composer_fill = if colors.appearance == homie_ui::Appearance::Dark {
            rgba(0x26282dff)
        } else {
            rgba(0xf2f1efff)
        };
        let shelf_fill = if colors.appearance == homie_ui::Appearance::Dark {
            rgba(0x1d1f23ff)
        } else {
            rgba(0xe8e7e4ff)
        };

        // Wrapped lines are children of a scroll container so the composer's
        // handle can scroll BY LINE to keep the caret on screen — the whole
        // point of the rewrite. An empty prompt shows the placeholder in the
        // same row the caret is on, so the two do not fight over the baseline.
        let prompt = if self.prompt.is_empty() {
            div()
                .h(px(COMPOSER_LINE_HEIGHT))
                .flex()
                .items_center()
                .when(focused, |line| {
                    line.child(div().text_color(colors.primary.alpha(0.92)).child(CARET))
                })
                .child(
                    div()
                        .text_color(colors.tertiary)
                        .child("Describe the task…"),
                )
                .into_any_element()
        } else {
            div()
                .id("launcher-prompt-lines")
                .size_full()
                .flex()
                .flex_col()
                .overflow_y_scroll()
                .track_scroll(self.prompt.scroll_handle())
                .children(self.prompt.render_lines(
                    px(COMPOSER_LINE_HEIGHT),
                    focused.then_some(CARET),
                    HighlightStyle {
                        background_color: Some(Palette::CLAY.alpha(0.35).into()),
                        ..HighlightStyle::default()
                    },
                ))
                .into_any_element()
        };

        let panel = div()
            .relative()
            .w(px(PANEL_WIDTH))
            .child(
                div()
                    .h(px(TITLE_HEIGHT))
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        div()
                            .text_size(px(22.0))
                            .font_weight(FontWeight::NORMAL)
                            .text_color(colors.primary.alpha(0.94))
                            .child("What should we work on?"),
                    ),
            )
            .child(
                div()
                    .relative()
                    .mt(px(TITLE_GAP))
                    .mx(px(COMPOSER_INSET))
                    .h(px(composer_height))
                    .rounded(px(Radius::PANEL))
                    .bg(composer_fill)
                    .border_1()
                    .border_color(if focused {
                        Palette::CLAY.alpha(0.42)
                    } else {
                        colors.primary.alpha(0.09)
                    })
                    .cursor_text()
                    .on_mouse_down(MouseButton::Left, {
                        let focus = self.focus.clone();
                        move |_, window, cx| window.focus(&focus, cx)
                    })
                    .child(
                        div()
                            .h(px(text_height))
                            .px(px(COMPOSER_PADDING))
                            .pt(px(COMPOSER_PAD_TOP))
                            .pb(px(COMPOSER_PAD_BOTTOM))
                            .text_size(px(COMPOSER_FONT_SIZE))
                            .line_height(px(COMPOSER_LINE_HEIGHT))
                            .text_color(colors.primary)
                            .child(prompt),
                    )
                    .child(
                        div()
                            .h(px(COMPOSER_CONTROLS_HEIGHT))
                            .px(px(10.0))
                            .pb(px(8.0))
                            .flex()
                            .items_end()
                            .justify_between()
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(8.0))
                                    .child(
                                        div()
                                            .id("launcher-add-project")
                                            .size(px(CONTROL_SIZE))
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .rounded(px(CONTROL_RADIUS))
                                            .cursor_pointer()
                                            .hover(move |button| button.bg(Fill::subtle(colors)))
                                            .active(move |button| {
                                                button.bg(colors.primary.alpha(0.10))
                                            })
                                            .child(sf_symbol("plus", 11.0, colors.secondary))
                                            .on_click(cx.listener(|this, _, window, cx| {
                                                this.choose_folder(window, cx);
                                            })),
                                    )
                                    .child(
                                        div()
                                            .text_size(px(10.0))
                                            .text_color(colors.tertiary)
                                            .child(blocker.clone().unwrap_or_else(|| {
                                                "⇧↵  New line   ⇥  Agent".to_owned()
                                            })),
                                    ),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(7.0))
                                    .child(
                                        div()
                                            .id("launcher-harness-button")
                                            .h(px(CONTROL_SIZE))
                                            .px(px(10.0))
                                            .flex()
                                            .items_center()
                                            .gap(px(7.0))
                                            .rounded(px(CONTROL_RADIUS))
                                            .cursor_pointer()
                                            .text_size(px(12.0))
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(colors.secondary)
                                            .bg(if harness_open {
                                                colors.primary.alpha(0.10)
                                            } else {
                                                Fill::subtle(colors)
                                            })
                                            .hover(move |button| {
                                                button.bg(colors.primary.alpha(0.09))
                                            })
                                            .active(move |button| {
                                                button.bg(colors.primary.alpha(0.12))
                                            })
                                            .child(AgentLogo::new(logo, 16.0, colors).badged(false))
                                            .child(harness_label)
                                            .child(sf_symbol("chevron.down", 7.5, colors.tertiary))
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.toggle_picker(Picker::Harness);
                                                cx.notify();
                                            })),
                                    )
                                    .child(
                                        div()
                                            .id("launcher-submit")
                                            .size(px(CONTROL_SIZE))
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .rounded(px(CONTROL_RADIUS))
                                            .bg(if can_submit {
                                                colors.primary
                                            } else {
                                                Fill::subtle(colors)
                                            })
                                            .when(can_submit, |button| {
                                                button
                                                    .cursor_pointer()
                                                    .hover(move |button| button.opacity(0.86))
                                                    .active(move |button| button.opacity(0.72))
                                                    .on_click(cx.listener(|this, _, _, cx| {
                                                        this.submit(cx);
                                                    }))
                                            })
                                            .child(sf_symbol_weighted(
                                                "chevron.up",
                                                10.0,
                                                SymbolWeight::Bold,
                                                if can_submit {
                                                    colors.background
                                                } else {
                                                    colors.tertiary
                                                },
                                            )),
                                    ),
                            ),
                    ),
            )
            .child(
                div()
                    .relative()
                    .mx(px(16.0))
                    .h(px(SHELF_HEIGHT))
                    .px(px(12.0))
                    .flex()
                    .items_center()
                    .justify_between()
                    .rounded_bl(px(Radius::PANEL))
                    .rounded_br(px(Radius::PANEL))
                    .bg(shelf_fill)
                    .border_1()
                    .border_color(colors.primary.alpha(0.055))
                    .child(
                        div()
                            .id("launcher-project-button")
                            .h(px(CONTROL_SIZE - 2.0))
                            .px(px(8.0))
                            .flex()
                            .items_center()
                            .gap(px(7.0))
                            .rounded(px(CONTROL_RADIUS - 1.0))
                            .cursor_pointer()
                            .bg(if project_open {
                                colors.primary.alpha(0.08)
                            } else {
                                colors.primary.alpha(0.0)
                            })
                            .hover(move |button| button.bg(colors.primary.alpha(0.08)))
                            .active(move |button| button.bg(colors.primary.alpha(0.11)))
                            .child(sf_symbol("folder", 11.0, colors.secondary))
                            .child(
                                div()
                                    .text_size(px(12.0))
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(colors.primary.alpha(0.86))
                                    .child(project_label),
                            )
                            .child(sf_symbol("chevron.down", 8.0, colors.tertiary))
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.toggle_picker(Picker::Project);
                                cx.notify();
                            })),
                    )
                    .child(
                        div()
                            .id("launcher-new-project")
                            .h(px(CONTROL_SIZE - 2.0))
                            .px(px(8.0))
                            .flex()
                            .items_center()
                            .gap(px(6.0))
                            .rounded(px(CONTROL_RADIUS - 1.0))
                            .cursor_pointer()
                            .text_size(px(11.0))
                            .text_color(colors.secondary)
                            .hover(move |button| button.bg(colors.primary.alpha(0.08)))
                            .active(move |button| button.bg(colors.primary.alpha(0.11)))
                            .child(sf_symbol("plus", 9.0, colors.tertiary))
                            .child("New project")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.choose_folder(window, cx);
                            })),
                    ),
            )
            .when(harness_open, |panel| {
                panel.child(
                    self.floating(picker_top, cx)
                        .right(px(COMPOSER_INSET))
                        .child(self.render_harness_picker(colors, cx)),
                )
            })
            .when(project_open, |panel| {
                panel.child(
                    self.floating(picker_top, cx)
                        .left(px(COMPOSER_INSET))
                        .child(self.render_project_picker(colors, cx)),
                )
            });

        panel.into_any_element()
    }

    /// Wrapper for a picker popover. It swallows its own mouse-down so the
    /// canvas behind it — which closes any open picker — does not tear the
    /// list away between press and release, which would eat the click.
    fn floating(&self, top: f32, cx: &mut Context<Self>) -> gpui::Div {
        div().absolute().top(px(top)).on_mouse_down(
            MouseButton::Left,
            cx.listener(|_, _, _, cx| {
                cx.stop_propagation();
            }),
        )
    }
}

impl Render for super::LauncherOverlay {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let root = div()
            .id("new-session-launcher")
            .key_context("HomieLauncher")
            .track_focus(&self.focus)
            .on_key_down(cx.listener(|this, event, window, cx| {
                this.handle_key_down(event, window, cx);
            }));
        if !self.open {
            return root.size(px(0.0));
        }

        // Soft-wrapping needs the text system, which only exists here. Doing
        // it before the panel is built is what lets the composer size itself
        // to the prompt and scroll the caret into view.
        self.prompt.layout(
            px(COMPOSER_TEXT_WIDTH),
            gpui::font(crate::fonts::ui_family()),
            px(COMPOSER_FONT_SIZE),
            window,
        );

        // The session workbench is intentionally always dark, independent of
        // macOS appearance. This is a destination in that workbench—not a
        // translucent window overlay—so paint the same fully opaque surface.
        let colors = SemanticColors::dark();
        let focused = self.focus.is_focused(window);
        root.size_full()
            .relative()
            .flex()
            .items_center()
            .justify_center()
            .bg(colors.background)
            // The entire empty workbench behaves like the editor's canvas: a
            // click anywhere returns to the prompt and dismisses whichever
            // picker was open, which previously stayed up until you found the
            // button again or pressed Escape.
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _, window, cx| {
                    this.picker = None;
                    window.focus(&this.focus, cx);
                    cx.notify();
                }),
            )
            // Command-N is a high-frequency keyboard action; the destination
            // appears immediately rather than making the user wait on motion.
            .child(self.render_panel(colors, focused, cx))
    }
}
