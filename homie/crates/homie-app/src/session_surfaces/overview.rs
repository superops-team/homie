//! Overview chrome rendering (header, filters, mode selector, empty state, board, list).

use super::*;

impl SessionSurfaces {
    pub(super) fn render_overview(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let (sessions, state) = {
            let mut store = self.store.write().expect("session store lock poisoned");
            (store.ordered_sessions(), store.overview_state().clone())
        };
        let colors = self.colors();
        let project_count = sessions
            .iter()
            .map(|session| &session.project_id)
            .collect::<HashSet<_>>()
            .len();
        let summary = format!(
            "{} session{} · {} project{}",
            sessions.len(),
            if sessions.len() == 1 { "" } else { "s" },
            project_count,
            if project_count == 1 { "" } else { "s" }
        );

        let mode_selector = div()
            .flex()
            .items_center()
            .gap(px(2.0))
            .p(px(2.0))
            .rounded(px(Radius::ROW))
            .bg(colors.primary.alpha(0.045))
            .border_1()
            .border_color(colors.primary.alpha(0.07))
            .child(self.mode_button(OverviewMode::Board, state.mode(), "Board", colors, cx))
            .child(self.mode_button(OverviewMode::List, state.mode(), "List", colors, cx));

        let header = div()
            .flex()
            .flex_none()
            .items_center()
            .gap(px(10.0))
            .h(px(50.0))
            .px(px(20.0))
            .child(
                div()
                    .text_size(px(16.0))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(colors.primary)
                    .child("All Sessions"),
            )
            .child(
                div()
                    .text_size(px(12.0))
                    .text_color(colors.tertiary)
                    .child(summary),
            )
            .child(div().flex_1())
            .child(
                div()
                    .text_size(px(11.0))
                    .text_color(colors.tertiary)
                    .child("⌘ click to select"),
            )
            .child(mode_selector)
            .child(
                div()
                    .id("close-overview")
                    .flex()
                    .items_center()
                    .justify_center()
                    .size(px(26.0))
                    .rounded_full()
                    .bg(colors.primary.alpha(0.045))
                    .border_1()
                    .border_color(colors.primary.alpha(0.07))
                    .text_size(px(11.0))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(colors.secondary)
                    .cursor_pointer()
                    .hover(|style| style.bg(colors.primary.alpha(0.10)))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.store
                            .write()
                            .expect("session store lock poisoned")
                            .dismiss_overview();
                        cx.notify();
                    }))
                    .child(sf_symbol_weighted(
                        "xmark",
                        11.0,
                        SymbolWeight::Semibold,
                        colors.secondary,
                    )),
            );

        let mut filters = div()
            .id("overview-filters")
            .flex()
            .flex_none()
            .items_center()
            .gap(px(6.0))
            .h(px(38.0))
            .px(px(20.0))
            .overflow_x_scroll();
        filters = filters.child(self.filter_chip(
            OverviewFilter::All,
            "All",
            sessions.len(),
            state.filter(),
            colors,
            cx,
        ));
        for lane in OverviewLane::ALL {
            let count = sessions
                .iter()
                .filter(|session| OverviewLane::for_session(session) == lane)
                .count();
            if count > 0 {
                filters = filters.child(self.filter_chip(
                    OverviewFilter::Lane(lane),
                    lane.label(),
                    count,
                    state.filter(),
                    colors,
                    cx,
                ));
            }
        }
        if !state.query().is_empty() {
            filters = filters.child(
                div()
                    .flex()
                    .flex_none()
                    .items_center()
                    .gap(px(5.0))
                    .h(px(22.0))
                    .px(px(9.0))
                    .rounded_full()
                    .bg(colors.primary.alpha(0.10))
                    .border_1()
                    .border_color(colors.primary.alpha(0.16))
                    .text_size(px(11.0))
                    .text_color(colors.primary)
                    .child(sf_symbol("magnifyingglass", 11.0, colors.secondary))
                    .child(state.query().to_owned())
                    .child(div().text_color(colors.tertiary).child("⌫")),
            );
        }

        let visible_count = state.visible_sessions(&sessions).count();
        let body = if visible_count == 0 {
            self.overview_empty_state(&state, colors)
        } else if state.mode() == OverviewMode::Board {
            self.overview_board(&sessions, &state, colors, window, cx)
        } else {
            self.overview_list(&sessions, &state, colors, window, cx)
        };

        let chrome = div()
            .flex()
            .flex_none()
            .flex_col()
            .bg(rgba(0x171921ff))
            .child(header)
            .child(filters)
            .child(HairlineDivider::horizontal(colors));

        let content = div()
            .debug_selector(|| "OVERVIEW_CONTENT".into())
            .absolute()
            .inset_0()
            .size_full()
            .flex()
            .flex_col()
            .pt(px(42.0))
            .bg(colors.background)
            .overflow_hidden()
            .occlude()
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .child(chrome)
            .child(body)
            .when(!state.selection().is_empty(), |content| {
                content.child(self.bulk_close_bar(state.selection().len(), visible_count, cx))
            });

        div()
            .id("overview-scrim")
            .absolute()
            .inset_0()
            .size_full()
            .bg(rgba(0x000000ff))
            .occlude()
            .on_scroll_wheel(|_, _, cx| cx.stop_propagation())
            .on_click(cx.listener(|this, _, _, cx| {
                this.store
                    .write()
                    .expect("session store lock poisoned")
                    .overview_escape();
                cx.notify();
            }))
            .child(content)
            .into_any_element()
    }

    fn mode_button(
        &self,
        mode: OverviewMode,
        current: OverviewMode,
        label: &'static str,
        colors: SemanticColors,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let active = mode == current;
        div()
            .id(SharedString::from(format!("overview-mode-{label}")))
            .flex_none()
            .h(px(24.0))
            .px(px(10.0))
            .flex()
            .items_center()
            .rounded(px(Radius::BADGE))
            .bg(colors.primary.alpha(if active { 0.10 } else { 0.0 }))
            .text_size(px(11.0))
            .text_color(if active {
                colors.primary
            } else {
                colors.secondary
            })
            .cursor_pointer()
            .hover(|style| style.bg(colors.primary.alpha(if active { 0.12 } else { 0.055 })))
            .on_click(cx.listener(move |this, _, _, cx| {
                this.store
                    .write()
                    .expect("session store lock poisoned")
                    .set_overview_mode(mode);
                cx.notify();
            }))
            .child(label)
            .into_any_element()
    }

    fn filter_chip(
        &self,
        filter: OverviewFilter,
        label: &'static str,
        count: usize,
        current: OverviewFilter,
        colors: SemanticColors,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let active = filter == current;
        div()
            .id(SharedString::from(format!("overview-filter-{label}")))
            .flex()
            .flex_none()
            .items_center()
            .gap(px(5.0))
            .h(px(22.0))
            .px(px(9.0))
            .rounded_full()
            .bg(colors.primary.alpha(if active { 0.10 } else { 0.0 }))
            .border_1()
            .border_color(colors.primary.alpha(if active { 0.16 } else { 0.08 }))
            .text_size(px(11.0))
            .text_color(if active {
                colors.primary
            } else {
                colors.secondary
            })
            .cursor_pointer()
            .hover(|style| style.bg(colors.primary.alpha(if active { 0.12 } else { 0.05 })))
            .on_click(cx.listener(move |this, _, _, cx| {
                this.store
                    .write()
                    .expect("session store lock poisoned")
                    .set_overview_filter(filter);
                cx.notify();
            }))
            .child(label)
            .child(div().text_color(colors.tertiary).child(count.to_string()))
            .into_any_element()
    }

    fn overview_empty_state(
        &self,
        state: &crate::switcher::SessionOverviewState,
        colors: SemanticColors,
    ) -> AnyElement {
        let title = if !state.query().is_empty() {
            format!("No matches for “{}”", state.query())
        } else {
            match state.filter() {
                OverviewFilter::All => "No sessions".to_owned(),
                OverviewFilter::Lane(lane) => {
                    format!("No {} sessions", lane.label().to_lowercase())
                }
            }
        };
        div()
            .flex()
            .flex_1()
            .flex_col()
            .items_center()
            .justify_center()
            .gap(px(10.0))
            .text_color(colors.tertiary)
            .child(sf_symbol_weighted(
                "square.grid.2x2",
                26.0,
                SymbolWeight::Regular,
                colors.tertiary,
            ))
            .child(
                div()
                    .text_size(px(15.0))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(colors.secondary)
                    .child(title),
            )
            .into_any_element()
    }

    fn overview_board(
        &mut self,
        sessions: &[SessionRecord],
        state: &crate::switcher::SessionOverviewState,
        colors: SemanticColors,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let lanes: Vec<_> = match state.filter() {
            OverviewFilter::All => OverviewLane::ALL.to_vec(),
            OverviewFilter::Lane(lane) => vec![lane],
        };
        let bottom_padding = if state.selection().is_empty() {
            18.0
        } else {
            82.0
        };
        let mut board = div()
            .id("overview-board")
            .debug_selector(|| "OVERVIEW_BOARD".into())
            .flex()
            .flex_1()
            .min_w_0()
            .min_h_0()
            .items_stretch()
            .gap(px(12.0))
            .px(px(20.0))
            .pt(px(14.0))
            .pb(px(bottom_padding))
            .track_scroll(&self.overview_board_scroll)
            .overflow_x_scroll();
        // Do not reinterpret a vertical wheel as horizontal board movement;
        // the lane under the pointer owns that axis.
        board.style().restrict_scroll_to_axis = Some(true);
        for lane in lanes {
            let lane_sessions = state.lane_sessions(lane, sessions);
            let count = lane_sessions.len();
            let lane_scroll = self
                .overview_lane_scrolls
                .get(&lane)
                .expect("every overview lane has a scroll handle")
                .clone();
            let mut cards = div()
                .id(SharedString::from(format!(
                    "overview-lane-{}",
                    lane.label()
                )))
                .debug_selector(|| format!("OVERVIEW_LANE_{}", lane.label().to_uppercase()))
                .flex()
                .flex_1()
                .flex_col()
                .min_h_0()
                .gap(px(10.0))
                .p(px(10.0))
                .pt(px(8.0))
                .track_scroll(&lane_scroll)
                .overflow_y_scroll();
            cards.style().restrict_scroll_to_axis = Some(true);
            for session in lane_sessions {
                cards = cards.child(self.overview_card(session, state, colors, window, cx));
            }
            board = board.child(
                div()
                    .flex()
                    .flex_col()
                    .flex_none()
                    .w(px(OVERVIEW_LANE_WIDTH))
                    .min_h_0()
                    .overflow_hidden()
                    .rounded(px(Radius::PANEL))
                    .bg(colors.primary.alpha(0.026))
                    .border_1()
                    .border_color(colors.primary.alpha(0.065))
                    .child(
                        div()
                            .flex()
                            .flex_none()
                            .items_center()
                            .gap(px(6.0))
                            .h(px(36.0))
                            .px(px(11.0))
                            .text_size(px(11.0))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(colors.secondary)
                            .child(lane.label())
                            .child(div().flex_1())
                            .child(
                                div()
                                    .min_w(px(20.0))
                                    .h(px(20.0))
                                    .px(px(6.0))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .rounded_full()
                                    .bg(colors.primary.alpha(0.055))
                                    .text_color(colors.tertiary)
                                    .child(count.to_string()),
                            ),
                    )
                    .child(HairlineDivider::horizontal(colors))
                    .child(cards),
            );
        }
        board.into_any_element()
    }

    fn overview_list(
        &mut self,
        sessions: &[SessionRecord],
        state: &crate::switcher::SessionOverviewState,
        colors: SemanticColors,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let visible: Vec<_> = state.visible_sessions(sessions).cloned().collect();
        let bottom_padding = if state.selection().is_empty() {
            18.0
        } else {
            82.0
        };
        let mut list = div()
            .id("overview-list")
            .debug_selector(|| "OVERVIEW_LIST".into())
            .flex()
            .flex_1()
            .min_h_0()
            .flex_col()
            .gap(px(8.0))
            .px(px(20.0))
            .pt(px(14.0))
            .pb(px(bottom_padding))
            .track_scroll(&self.overview_list_scroll)
            .overflow_y_scroll();
        list.style().restrict_scroll_to_axis = Some(true);
        for session in &visible {
            list = list.child(self.overview_list_row(session, state, colors, window, cx));
        }
        list.into_any_element()
    }
}
