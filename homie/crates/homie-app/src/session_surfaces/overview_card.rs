//! Overview card / list-row / bulk-close-bar rendering.

use super::projection::{state_badge, state_badge_color, status_color};
use super::*;

impl SessionSurfaces {
    pub(super) fn overview_card(
        &mut self,
        session: &SessionRecord,
        state: &crate::switcher::SessionOverviewState,
        colors: SemanticColors,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let selected = state.selection().contains(&session.id);
        let focused = state.focused() == Some(&session.id);
        let has_selection = !state.selection().is_empty();
        let id = session.id.clone();
        let close_id = id.clone();
        let status = self.status_glyph(session, 14.0, colors, window, cx);
        let preview = self.render_grid_or_logo(session, 34.0, 6.5, colors);

        let mut thumbnail = div()
            .relative()
            .w_full()
            .h(px(112.0))
            .rounded(px(Radius::ROW))
            .overflow_hidden()
            .bg(colors.background)
            .border_1()
            .border_color(colors.primary.alpha(0.075))
            .when(session.hibernation.is_some(), |thumbnail| {
                thumbnail.opacity(0.68)
            })
            .child(preview);
        if selected {
            thumbnail = thumbnail.child(
                div()
                    .absolute()
                    .top(px(6.0))
                    .right(px(6.0))
                    .size(px(18.0))
                    .rounded_full()
                    .bg(Palette::CLAY)
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(sf_symbol_weighted(
                        "checkmark",
                        9.0,
                        SymbolWeight::Bold,
                        colors.primary,
                    )),
            );
        } else if !has_selection {
            thumbnail = thumbnail.child(
                div()
                    .id(SharedString::from(format!("close-card-{}", close_id.0)))
                    .absolute()
                    .top(px(6.0))
                    .left(px(6.0))
                    .size(px(20.0))
                    .rounded_full()
                    .bg(rgba(0x20222bd9))
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_size(px(10.0))
                    .font_weight(FontWeight::BOLD)
                    .text_color(colors.primary)
                    .cursor_pointer()
                    .invisible()
                    .group_hover("overview-card", |style| style.visible())
                    .on_click(cx.listener(move |this, _, _, cx| {
                        cx.stop_propagation();
                        this.store
                            .write()
                            .expect("session store lock poisoned")
                            .close_overview_session(close_id.clone());
                        cx.notify();
                    }))
                    .child(sf_symbol_weighted(
                        "xmark",
                        9.0,
                        SymbolWeight::Bold,
                        colors.primary,
                    )),
            );
        }

        div()
            .id(SharedString::from(format!("overview-card-{}", id.0)))
            .debug_selector(|| format!("OVERVIEW_CARD_{}", id.0))
            .group("overview-card")
            .flex()
            .flex_none()
            .flex_col()
            .gap(px(8.0))
            .p(px(8.0))
            .rounded(px(Radius::CARD))
            .bg(if selected {
                Palette::CLAY.alpha(0.085)
            } else if focused {
                colors.primary.alpha(0.055)
            } else {
                colors.primary.alpha(0.032)
            })
            .border_1()
            .border_color(if selected {
                Palette::CLAY
            } else if focused {
                colors.primary.alpha(0.24)
            } else {
                colors.primary.alpha(0.06)
            })
            .cursor_pointer()
            .hover(|style| {
                if selected {
                    style
                        .bg(Palette::CLAY.alpha(0.105))
                        .border_color(Palette::CLAY)
                } else {
                    style
                        .bg(colors.primary.alpha(0.06))
                        .border_color(colors.primary.alpha(0.12))
                }
            })
            .active(|style| style.bg(colors.primary.alpha(0.075)))
            .on_click(cx.listener(move |this, event: &ClickEvent, _, cx| {
                if event.modifiers().platform {
                    this.store
                        .write()
                        .expect("session store lock poisoned")
                        .toggle_overview_selection(id.clone());
                } else {
                    this.store
                        .write()
                        .expect("session store lock poisoned")
                        .activate_overview_session(id.clone());
                }
                cx.notify();
            }))
            .child(thumbnail)
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(7.0))
                    .px(px(2.0))
                    .pb(px(1.0))
                    .child(status)
                    .child(
                        div()
                            .min_w_0()
                            .flex_1()
                            .text_size(px(13.0))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(colors.primary)
                            .overflow_hidden()
                            .text_ellipsis()
                            .child(display_title(session)),
                    )
                    .child(
                        div()
                            .flex_none()
                            .text_size(px(11.0))
                            .text_color(state_badge_color(session, colors))
                            .child(state_badge(session)),
                    ),
            )
            .into_any_element()
    }

    pub(super) fn overview_list_row(
        &mut self,
        session: &SessionRecord,
        state: &crate::switcher::SessionOverviewState,
        colors: SemanticColors,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let selected = state.selection().contains(&session.id);
        let focused = state.focused() == Some(&session.id);
        let id = session.id.clone();
        let close_id = id.clone();
        let status = self.status_glyph(session, 16.0, colors, window, cx);
        div()
            .id(SharedString::from(format!("overview-row-{}", id.0)))
            .flex()
            .flex_none()
            .items_center()
            .gap(px(10.0))
            .h(px(68.0))
            .px(px(10.0))
            .rounded(px(Radius::ROW))
            .bg(colors.primary.alpha(if selected {
                0.085
            } else if focused {
                0.055
            } else {
                0.028
            }))
            .border_1()
            .border_color(if selected {
                Palette::CLAY
            } else {
                colors.primary.alpha(if focused { 0.18 } else { 0.06 })
            })
            .cursor_pointer()
            .hover(|style| {
                if selected {
                    style
                        .bg(colors.primary.alpha(0.10))
                        .border_color(Palette::CLAY)
                } else {
                    style.bg(colors.primary.alpha(0.06))
                }
            })
            .active(|style| style.bg(colors.primary.alpha(0.075)))
            .on_click(cx.listener(move |this, event: &ClickEvent, _, cx| {
                if event.modifiers().platform {
                    this.store
                        .write()
                        .expect("session store lock poisoned")
                        .toggle_overview_selection(id.clone());
                } else {
                    this.store
                        .write()
                        .expect("session store lock poisoned")
                        .activate_overview_session(id.clone());
                }
                cx.notify();
            }))
            .child(
                div()
                    .flex_none()
                    .w(px(88.0))
                    .h(px(50.0))
                    .rounded(px(Radius::BADGE))
                    .overflow_hidden()
                    .bg(colors.background)
                    .child(self.render_grid_or_logo(session, 24.0, 4.5, colors)),
            )
            .child(status)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .min_w_0()
                    .flex_1()
                    .gap(px(3.0))
                    .child(
                        div()
                            .text_size(px(13.0))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(colors.primary)
                            .overflow_hidden()
                            .text_ellipsis()
                            .child(display_title(session)),
                    )
                    .child(
                        div()
                            .min_w_0()
                            .text_size(px(11.0))
                            .text_color(colors.tertiary)
                            .overflow_hidden()
                            .text_ellipsis()
                            .child(session.cwd.clone()),
                    ),
            )
            .child(
                div()
                    .flex_none()
                    .px(px(8.0))
                    .h(px(22.0))
                    .flex()
                    .items_center()
                    .rounded_full()
                    .bg(status_color(session, colors).alpha(0.12))
                    .text_size(px(11.0))
                    .text_color(status_color(session, colors))
                    .child(OverviewLane::for_session(session).label()),
            )
            .child(
                div()
                    .id(SharedString::from(format!("close-row-{}", close_id.0)))
                    .size(px(24.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded_full()
                    .text_color(colors.secondary)
                    .hover(|style| style.bg(colors.primary.alpha(0.08)))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        cx.stop_propagation();
                        this.store
                            .write()
                            .expect("session store lock poisoned")
                            .close_overview_session(close_id.clone());
                        cx.notify();
                    }))
                    .child(sf_symbol_weighted(
                        "xmark",
                        10.0,
                        SymbolWeight::Bold,
                        colors.secondary,
                    )),
            )
            .into_any_element()
    }

    pub(super) fn bulk_close_bar(
        &self,
        count: usize,
        visible_count: usize,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let colors = self.colors();
        div()
            .absolute()
            .bottom(px(20.0))
            .left_0()
            .right_0()
            .flex()
            .justify_center()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(14.0))
                    .px(px(16.0))
                    .py(px(10.0))
                    .rounded_full()
                    .bg(rgba(0x2a2c35f5))
                    .border_1()
                    .border_color(colors.primary.alpha(0.10))
                    .shadow_lg()
                    .child(
                        div()
                            .text_size(px(13.0))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(colors.primary)
                            .child(format!("{count} selected")),
                    )
                    .when(count < visible_count, |bar| {
                        bar.child(
                            div()
                                .id("select-all-overview")
                                .text_size(px(13.0))
                                .text_color(colors.secondary)
                                .cursor_pointer()
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.store
                                        .write()
                                        .expect("session store lock poisoned")
                                        .select_all_overview_sessions();
                                    cx.notify();
                                }))
                                .child("Select All"),
                        )
                    })
                    .child(div().w(px(1.0)).h(px(16.0)).bg(colors.primary.alpha(0.10)))
                    .child(
                        div()
                            .id("cancel-overview-selection")
                            .text_size(px(13.0))
                            .text_color(colors.secondary)
                            .cursor_pointer()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.store
                                    .write()
                                    .expect("session store lock poisoned")
                                    .clear_overview_selection();
                                cx.notify();
                            }))
                            .child("Cancel"),
                    )
                    .child(
                        div()
                            .id("close-overview-selection")
                            .px(px(12.0))
                            .h(px(28.0))
                            .flex()
                            .items_center()
                            .rounded_full()
                            .bg(Ink::DANGER)
                            .text_size(px(13.0))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(colors.primary)
                            .cursor_pointer()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.store
                                    .write()
                                    .expect("session store lock poisoned")
                                    .close_overview_selection();
                                cx.notify();
                            }))
                            .child(if count == 1 {
                                "Close 1 Session".to_owned()
                            } else {
                                format!("Close {count} Sessions")
                            }),
                    ),
            )
            .into_any_element()
    }
}
