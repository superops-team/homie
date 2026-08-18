use super::*;

impl RootView {
    /// The grab strip that straddles the sidebar/terminal seam.
    ///
    /// Two things make this reliable, and both are easy to lose:
    ///  - `deferred` + `occlude` put the strip above the terminal card, which
    ///    is a later sibling and would otherwise win the hit test on the half
    ///    of the strip that overhangs it.
    ///  - the drag is tracked with `on_drag`/`on_drag_move` (see `RootView::
    ///    render`) rather than `on_mouse_move`, because plain move listeners
    ///    only fire while the hitbox is hovered -- so any pointer motion that
    ///    outran the 9px strip silently dropped the resize.
    pub(crate) fn resize_handle(&self, cx: &mut Context<Self>) -> AnyElement {
        div()
            .relative()
            .flex_none()
            .w(px(0.0))
            .h_full()
            .child(deferred(
                div()
                    .id("sidebar-resize-handle")
                    .absolute()
                    .left(px(-4.5))
                    .top(px(0.0))
                    .w(px(9.0))
                    .h_full()
                    .cursor(CursorStyle::ResizeLeftRight)
                    .occlude()
                    .on_drag(DraggedSidebarEdge, |edge, _, _, cx| {
                        cx.stop_propagation();
                        cx.new(|_| *edge)
                    })
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, event: &gpui::MouseDownEvent, _, cx| {
                            let width = this.sidebar.read(cx).width();
                            this.resize_origin = Some((f32::from(event.position.x), width));
                            cx.stop_propagation();
                        }),
                    )
                    .on_mouse_up(
                        MouseButton::Left,
                        cx.listener(|this, event: &gpui::MouseUpEvent, _, cx| {
                            if event.click_count == 2 {
                                this.sidebar
                                    .update(cx, |sidebar, cx| sidebar.reset_width(cx));
                                cx.stop_propagation();
                            }
                            this.finish_resize(cx);
                        }),
                    )
                    .on_mouse_up_out(
                        MouseButton::Left,
                        cx.listener(|this, _, _, cx| this.finish_resize(cx)),
                    ),
            ))
            .into_any_element()
    }
    pub(crate) fn terminal_resize_handle(&self, cx: &mut Context<Self>) -> AnyElement {
        let line = rgba(0xffffff18);
        div()
            .relative()
            .flex_none()
            .h(px(1.0))
            .w_full()
            .bg(line)
            .child(deferred(
                div()
                    .id("terminal-resize-handle")
                    .absolute()
                    .top(px(-4.0))
                    .left(px(0.0))
                    .h(px(9.0))
                    .w_full()
                    .cursor(CursorStyle::ResizeUpDown)
                    .occlude()
                    .on_drag(DraggedTerminalEdge, |edge, _, _, cx| {
                        cx.stop_propagation();
                        cx.new(|_| *edge)
                    })
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, event: &gpui::MouseDownEvent, _, cx| {
                            let primary = this
                                .workbench_layout
                                .pane_heights(this.terminal_available_height)
                                .primary;
                            this.terminal_resize_origin =
                                Some((f32::from(event.position.y), primary));
                            cx.stop_propagation();
                        }),
                    )
                    .on_mouse_up(
                        MouseButton::Left,
                        cx.listener(|this, event: &gpui::MouseUpEvent, _, cx| {
                            if event.click_count == 2 {
                                this.workbench_layout.reset();
                            }
                            this.finish_terminal_resize(cx);
                        }),
                    )
                    .on_mouse_up_out(
                        MouseButton::Left,
                        cx.listener(|this, _, _, cx| this.finish_terminal_resize(cx)),
                    ),
            ))
            .into_any_element()
    }
    pub(crate) fn inspector_resize_handle(&self, cx: &mut Context<Self>) -> AnyElement {
        div()
            .relative()
            .flex_none()
            .w(px(0.0))
            .h_full()
            .child(deferred(
                div()
                    .id("inspector-resize-handle")
                    .absolute()
                    .left(px(-4.5))
                    .top(px(0.0))
                    .w(px(9.0))
                    .h_full()
                    .cursor(CursorStyle::ResizeLeftRight)
                    .occlude()
                    .on_drag(DraggedInspectorEdge, |edge, _, _, cx| {
                        cx.stop_propagation();
                        cx.new(|_| *edge)
                    })
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, event: &gpui::MouseDownEvent, _, cx| {
                            this.inspector_resize_origin =
                                Some((f32::from(event.position.x), this.inspector_width));
                            cx.stop_propagation();
                        }),
                    )
                    .on_mouse_up(
                        MouseButton::Left,
                        cx.listener(|this, event: &gpui::MouseUpEvent, _, cx| {
                            if event.click_count == 2 {
                                this.inspector_width = 440.0_f32.min(this.inspector_max_width);
                            }
                            this.finish_inspector_resize(cx);
                        }),
                    )
                    .on_mouse_up_out(
                        MouseButton::Left,
                        cx.listener(|this, _, _, cx| this.finish_inspector_resize(cx)),
                    ),
            ))
            .into_any_element()
    }
    /// While a resize drag is active, keep pointer motion from reaching the
    /// terminal's selection layer. The drag payload still routes to RootView,
    /// while this transparent hitbox owns everything underneath it.
    pub(crate) fn resize_shield(&self, cx: &mut Context<Self>) -> AnyElement {
        let vertical = self.terminal_resize_origin.is_some();
        deferred(
            div()
                .id("active-resize-shield")
                .absolute()
                .inset_0()
                .cursor(if vertical {
                    CursorStyle::ResizeUpDown
                } else {
                    CursorStyle::ResizeLeftRight
                })
                .occlude()
                .on_mouse_up(
                    MouseButton::Left,
                    cx.listener(|this, _, _, cx| {
                        this.finish_resize(cx);
                        this.finish_terminal_resize(cx);
                        this.finish_inspector_resize(cx);
                        cx.stop_propagation();
                    }),
                )
                .on_mouse_up_out(
                    MouseButton::Left,
                    cx.listener(|this, _, _, cx| {
                        this.finish_resize(cx);
                        this.finish_terminal_resize(cx);
                        this.finish_inspector_resize(cx);
                    }),
                ),
        )
        .into_any_element()
    }
    /// `visible_sidebar` and `inspector_width` are the settled layout and drive
    /// everything the terminal is *told* -- viewport geometry, and whether its
    /// chrome offers a "show sidebar" button. The two `*_seam` widths are what
    /// is being painted this frame and drive only the card's own top corners,
    /// so each radius appears the moment its panel finishes clearing rather
    /// than at the start of the slide. Keeping the two apart is what stops a
    /// 260ms slide from firing a PTY resize on every frame of it.
    pub(crate) fn terminal_card(
        &mut self,
        visible_sidebar: bool,
        seam: f32,
        inspector_width: f32,
        inspector_seam: f32,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let terminal = self.colors();
        let bounds = window.inner_window_bounds().get_bounds();
        let sidebar_width = if visible_sidebar {
            self.sidebar.read(cx).width()
        } else {
            0.0
        };
        let card_width = (f32::from(bounds.size.width) - sidebar_width - inspector_width).max(0.0);
        let card_height = f32::from(bounds.size.height).max(0.0);
        let selected = self
            .services
            .store
            .store
            .read()
            .expect("session store lock poisoned")
            .selected_session_id()
            .cloned();
        let split_open = self.auxiliary_terminal.is_some()
            || selected
                .as_ref()
                .is_some_and(|id| self.auxiliary_spawn_parent.as_ref() == Some(id));
        let mut card = div()
            .relative()
            .flex_1()
            .flex()
            .flex_col()
            .h_full()
            .min_w(px(0.0))
            .when(seam <= 0.0, |card| card.rounded_tl(px(Radius::CARD)))
            .when(inspector_seam <= 0.0, |card| {
                card.rounded_tr(px(Radius::CARD))
            })
            .rounded_bl(px(Radius::CARD))
            .bg(terminal.background)
            .overflow_hidden()
            .text_color(terminal.primary);

        // Paint the frame independently from layout. A normal border shrinks
        // the content box, putting this title bar one pixel below the
        // borderless sidebar title bar even though both are 42 points tall.
        let card_outline = div()
            .absolute()
            .inset_0()
            .when(seam <= 0.0, |outline| outline.rounded_tl(px(Radius::CARD)))
            .when(inspector_seam <= 0.0, |outline| {
                outline.rounded_tr(px(Radius::CARD))
            })
            .rounded_bl(px(Radius::CARD))
            .border_1()
            .border_color(terminal.primary.alpha(0.10));

        if self.preview {
            card = card.child(self.preview_workbench(terminal));
        } else if split_open {
            let available_height = (card_height - 1.0).max(0.0);
            self.terminal_available_height = available_height;
            let heights = self.workbench_layout.pane_heights(available_height);
            if let Some(primary) = &self.terminal {
                primary.update(cx, |terminal, cx| {
                    terminal.set_shell_chrome(visible_sidebar, self.inspector_open, cx);
                    terminal.set_viewport(
                        TerminalViewport {
                            x: sidebar_width,
                            y: 0.0,
                            width: card_width,
                            height: heights.primary,
                        },
                        cx,
                    );
                });
                card = card.child(
                    div()
                        .flex_none()
                        .w_full()
                        .h(px(heights.primary))
                        .min_h(px(0.0))
                        .overflow_hidden()
                        .child(primary.clone()),
                );
            }
            card = card.child(self.terminal_resize_handle(cx));

            let mut auxiliary = div()
                .relative()
                .flex_none()
                .w_full()
                .h(px(heights.auxiliary))
                .min_h(px(0.0))
                .overflow_hidden();
            if let Some(terminal) = &self.auxiliary_terminal {
                terminal.update(cx, |terminal, cx| {
                    terminal.set_viewport(
                        TerminalViewport {
                            x: sidebar_width,
                            y: heights.primary + 1.0,
                            width: card_width,
                            height: heights.auxiliary,
                        },
                        cx,
                    );
                });
                auxiliary = auxiliary.child(terminal.clone());
            } else {
                auxiliary = auxiliary.child(
                    div()
                        .size_full()
                        .flex()
                        .items_center()
                        .justify_center()
                        .bg(terminal.background)
                        .text_size(px(12.0))
                        .text_color(terminal.secondary)
                        .child("Opening terminal…"),
                );
            }
            if let Some(id) = self.auxiliary_id.clone() {
                let store = Arc::clone(&self.services.store);
                let primary = self.terminal.clone();
                auxiliary = auxiliary.child(
                    div()
                        .id("close-auxiliary-terminal")
                        .absolute()
                        .top(px(12.0))
                        .right(px(12.0))
                        .size(px(24.0))
                        .flex()
                        .items_center()
                        .justify_center()
                        .rounded(px(Radius::BADGE))
                        .cursor_pointer()
                        .text_color(terminal.secondary)
                        .hover(move |button| button.bg(terminal.primary.alpha(0.08)))
                        .child(sf_symbol("xmark", 10.5, terminal.secondary))
                        .on_click(move |_, window, cx| {
                            store
                                .store
                                .write()
                                .expect("session store lock poisoned")
                                .remove_sessions(vec![id.clone()]);
                            if let Some(primary) = &primary {
                                primary.update(cx, |terminal, cx| terminal.focus(window, cx));
                            }
                            cx.stop_propagation();
                        }),
                );
            }
            card = card.child(auxiliary);
        } else if let Some(primary) = &self.terminal {
            self.terminal_available_height = card_height;
            primary.update(cx, |terminal, cx| {
                terminal.set_shell_chrome(visible_sidebar, self.inspector_open, cx);
                terminal.set_viewport(
                    TerminalViewport {
                        x: sidebar_width,
                        y: 0.0,
                        width: card_width,
                        height: card_height,
                    },
                    cx,
                );
            });
            card = card.child(primary.clone());
        }

        card.child(card_outline).into_any_element()
    }
    pub(crate) fn preview_workbench(&self, colors: SemanticColors) -> AnyElement {
        let scenario = match self.preview_scenario {
            PreviewScenario::Typical => "Typical",
            PreviewScenario::Stress => "Stress",
            PreviewScenario::Empty => "Empty",
            PreviewScenario::Artifacts => "Artifacts",
        };
        div()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .child(
                div()
                    .w(px(360.0))
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(px(22.0))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .items_center()
                            .gap(px(7.0))
                            .child(
                                div()
                                    .text_size(px(25.0))
                                    .font_weight(FontWeight::THIN)
                                    .text_color(colors.secondary)
                                    .child(sf_symbol_weighted(
                                        "sidebar.left",
                                        25.0,
                                        SymbolWeight::Regular,
                                        colors.secondary,
                                    )),
                            )
                            .child(
                                div()
                                    .text_size(px(17.0))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child("Sidebar design preview"),
                            )
                            .child(
                                div()
                                    .text_size(px(Typo::META.size))
                                    .text_color(colors.secondary)
                                    .child("Mock data only · no daemon connection"),
                            ),
                    )
                    .child(preview_control("Content", scenario, colors))
                    .child(preview_control("Appearance", "Dark", colors))
                    .child(
                        div()
                            .w_full()
                            .p(px(14.0))
                            .flex()
                            .flex_col()
                            .gap(px(9.0))
                            .rounded(px(Radius::PANEL))
                            .bg(colors.primary.alpha(0.045))
                            .border_1()
                            .border_color(colors.primary.alpha(0.07))
                            .child(preview_hint(
                                "cursorarrow.rays",
                                "Hover rows and project headers",
                                colors,
                            ))
                            .child(preview_hint(
                                "cursorarrow.click.2",
                                "Select, collapse, rename, and drag mock sessions",
                                colors,
                            ))
                            .child(preview_hint(
                                "arrow.left.and.right",
                                "Resize the sidebar from its trailing edge",
                                colors,
                            )),
                    ),
            )
            .into_any_element()
    }
    pub(crate) fn close_confirmation(
        &self,
        colors: SemanticColors,
        cx: &mut Context<Self>,
    ) -> Option<AnyElement> {
        let (title, message) = self.sidebar.read(cx).pending_close_copy()?;
        Some(
            div()
                .absolute()
                .inset_0()
                .occlude()
                .flex()
                .items_center()
                .justify_center()
                .bg(gpui::rgba(0x00000055))
                .on_mouse_down(MouseButton::Left, {
                    let sidebar = self.sidebar.clone();
                    move |_, _, cx| {
                        sidebar.update(cx, |sidebar, cx| sidebar.cancel_close(cx));
                        cx.stop_propagation();
                    }
                })
                .on_scroll_wheel(|_, _, cx| cx.stop_propagation())
                .child(FloatingSurface::new(
                    colors,
                    div()
                        .w(px(320.0))
                        .p(px(18.0))
                        .flex()
                        .flex_col()
                        .gap(px(10.0))
                        .occlude()
                        .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                        .child(
                            div()
                                .text_size(px(Typo::DISPLAY_TITLE.size))
                                .font_weight(Typo::DISPLAY_TITLE.weight)
                                .text_color(colors.primary)
                                .child(title),
                        )
                        .child(
                            div()
                                .text_size(px(Typo::ROW.size))
                                .text_color(colors.secondary)
                                .child(message),
                        )
                        .child(
                            div()
                                .mt(px(6.0))
                                .flex()
                                .justify_end()
                                .gap(px(8.0))
                                .child(
                                    div()
                                        .id("cancel-close")
                                        .px(px(12.0))
                                        .h(px(30.0))
                                        .flex()
                                        .items_center()
                                        .rounded(px(Radius::ROW))
                                        .cursor_pointer()
                                        .text_size(px(Typo::ROW.size))
                                        .text_color(colors.secondary)
                                        .hover(move |button| button.bg(colors.primary.alpha(0.06)))
                                        .child("Cancel")
                                        .on_click({
                                            let sidebar = self.sidebar.clone();
                                            move |_, _, cx| {
                                                sidebar.update(cx, |sidebar, cx| {
                                                    sidebar.cancel_close(cx)
                                                });
                                            }
                                        }),
                                )
                                .child(
                                    div()
                                        .id("confirm-close")
                                        .px(px(12.0))
                                        .h(px(30.0))
                                        .flex()
                                        .items_center()
                                        .rounded(px(Radius::ROW))
                                        .cursor_pointer()
                                        .bg(homie_ui::Ink::DANGER.alpha(0.16))
                                        .text_size(px(Typo::ROW.size))
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(homie_ui::Ink::DANGER)
                                        .child("Close")
                                        .on_click({
                                            let sidebar = self.sidebar.clone();
                                            move |_, _, cx| {
                                                sidebar.update(cx, |sidebar, cx| {
                                                    sidebar.confirm_close(cx)
                                                });
                                            }
                                        }),
                                ),
                        ),
                ))
                .into_any_element(),
        )
    }
    pub(crate) fn status_banner(
        &self,
        colors: SemanticColors,
        cx: &mut Context<Self>,
    ) -> Option<AnyElement> {
        let banner = self.status_banner.as_ref()?;
        Some(
            deferred(
                div()
                    .absolute()
                    .right(px(16.0))
                    .bottom(px(16.0))
                    .w(px(360.0))
                    .p(px(13.0))
                    .flex()
                    .items_start()
                    .gap(px(10.0))
                    .rounded(px(Radius::PANEL))
                    .bg(colors.background)
                    .border_1()
                    .border_color(colors.floating_stroke())
                    .shadow_lg()
                    .occlude()
                    .child(
                        div()
                            .min_w(px(0.0))
                            .flex_1()
                            .flex()
                            .flex_col()
                            .gap(px(3.0))
                            .child(
                                div()
                                    .text_size(px(Typo::ROW_EMPHASIZED.size))
                                    .font_weight(Typo::ROW_EMPHASIZED.weight)
                                    .text_color(colors.primary)
                                    .child(banner.title.clone()),
                            )
                            .child(
                                div()
                                    .text_size(px(Typo::META.size))
                                    .text_color(colors.secondary)
                                    .child(banner.body.clone()),
                            ),
                    )
                    .child(
                        div()
                            .id("dismiss-status-banner")
                            .size(px(22.0))
                            .flex_none()
                            .flex()
                            .items_center()
                            .justify_center()
                            .rounded(px(Radius::CHIP))
                            .cursor_pointer()
                            .text_color(colors.tertiary)
                            .hover(move |button| button.bg(colors.primary.alpha(0.06)))
                            .child(sf_symbol_weighted(
                                "xmark",
                                8.5,
                                SymbolWeight::Bold,
                                colors.tertiary,
                            ))
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.status_banner_generation =
                                    this.status_banner_generation.wrapping_add(1);
                                this.status_banner = None;
                                cx.notify();
                            })),
                    ),
            )
            .into_any_element(),
        )
    }
}

pub(crate) fn preview_control(label: &str, value: &str, colors: SemanticColors) -> AnyElement {
    div()
        .w(px(330.0))
        .flex()
        .items_center()
        .child(
            div()
                .w(px(82.0))
                .text_size(px(Typo::ROW_EMPHASIZED.size))
                .font_weight(Typo::ROW_EMPHASIZED.weight)
                .text_color(colors.secondary)
                .child(label.to_owned()),
        )
        .child(
            div()
                .flex_1()
                .h(px(26.0))
                .flex()
                .items_center()
                .justify_center()
                .rounded(px(Radius::BADGE))
                .bg(colors.primary.alpha(0.08))
                .text_size(px(Typo::META.size))
                .text_color(colors.primary)
                .child(value.to_owned()),
        )
        .into_any_element()
}
pub(crate) fn preview_hint(system_image: &str, label: &str, colors: SemanticColors) -> AnyElement {
    div()
        .flex()
        .items_center()
        .gap(px(9.0))
        .child(
            div()
                .w(px(15.0))
                .flex()
                .items_center()
                .justify_center()
                .child(sf_symbol(system_image, 11.0, colors.secondary)),
        )
        .child(
            div()
                .text_size(px(Typo::ROW.size))
                .text_color(colors.primary.alpha(0.82))
                .child(label.to_owned()),
        )
        .into_any_element()
}
