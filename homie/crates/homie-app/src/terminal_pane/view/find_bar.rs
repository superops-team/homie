use super::buttons::*;
use super::*;

impl TerminalPane {
    pub(crate) fn render_find_bar(
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
}
