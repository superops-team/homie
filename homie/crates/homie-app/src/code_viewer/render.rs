use super::highlight::source_row;
use super::*;

impl super::CodeViewer {
    fn render_toolbar(
        &self,
        snapshot: Option<&SourceSnapshot>,
        colors: SemanticColors,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let can_back = !self.history.is_empty() && self.history_index > 0;
        let can_forward = self.history_index + 1 < self.history.len();
        let picker_open = self.picker_open;
        let (path, location) = snapshot.map_or_else(
            || ("No file open".to_owned(), None),
            |snapshot| {
                (
                    snapshot.relative_path.to_string_lossy().into_owned(),
                    snapshot.target.map(|target| {
                        if target.column > 1 {
                            format!("{}:{}", target.line, target.column)
                        } else {
                            target.line.to_string()
                        }
                    }),
                )
            },
        );
        let nav_button = |id: &'static str,
                          symbol: &'static str,
                          enabled: bool,
                          delta: isize,
                          cx: &mut Context<Self>| {
            div()
                .id(id)
                .size(px(24.0))
                .flex_none()
                .flex()
                .items_center()
                .justify_center()
                .rounded(px(Radius::BADGE))
                .text_color(if enabled {
                    colors.secondary
                } else {
                    colors.primary.alpha(0.20)
                })
                .when(enabled, |button| {
                    button
                        .cursor_pointer()
                        .hover(move |button| button.bg(colors.primary.alpha(0.07)))
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.navigate(delta, cx);
                            cx.stop_propagation();
                        }))
                })
                .child(sf_symbol_weighted(
                    symbol,
                    10.0,
                    SymbolWeight::Semibold,
                    if enabled {
                        colors.secondary
                    } else {
                        colors.primary.alpha(0.20)
                    },
                ))
        };

        div()
            .h(px(38.0))
            .flex_none()
            .px(px(9.0))
            .flex()
            .items_center()
            .gap(px(3.0))
            .border_b_1()
            .border_color(colors.primary.alpha(0.06))
            .child(nav_button(
                "code-history-back",
                "chevron.left",
                can_back,
                -1,
                cx,
            ))
            .child(nav_button(
                "code-history-forward",
                "chevron.right",
                can_forward,
                1,
                cx,
            ))
            .child(
                div()
                    .id("code-open-file-picker")
                    .size(px(24.0))
                    .ml(px(2.0))
                    .flex_none()
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(px(Radius::BADGE))
                    .bg(if picker_open {
                        colors.primary.alpha(0.09)
                    } else {
                        colors.primary.alpha(0.0)
                    })
                    .cursor_pointer()
                    .hover(move |button| button.bg(colors.primary.alpha(0.07)))
                    .child(sf_symbol("magnifyingglass", 10.5, colors.secondary))
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.toggle_picker(window, cx);
                        cx.stop_propagation();
                    })),
            )
            .child(
                div()
                    .ml(px(2.0))
                    .min_w(px(0.0))
                    .flex_1()
                    .flex()
                    .items_center()
                    .gap(px(6.0))
                    .child(sf_symbol(
                        "doc.text",
                        11.5,
                        if snapshot.is_some() {
                            colors.secondary
                        } else {
                            colors.tertiary
                        },
                    ))
                    .child(
                        div()
                            .min_w(px(0.0))
                            .truncate()
                            .font_family(crate::fonts::mono_family())
                            .text_size(px(Typo::META.size))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(colors.secondary)
                            .child(path),
                    ),
            )
            .when_some(location, |bar, location| {
                bar.child(
                    div()
                        .px(px(6.0))
                        .h(px(20.0))
                        .flex()
                        .items_center()
                        .rounded(px(Radius::CHIP))
                        .bg(colors.primary.alpha(0.055))
                        .font_family(crate::fonts::mono_family())
                        .text_size(px(9.5))
                        .text_color(colors.tertiary)
                        .child(location),
                )
            })
            .into_any_element()
    }

    fn render_source(&self, snapshot: Arc<SourceSnapshot>, colors: SemanticColors) -> AnyElement {
        let rows = snapshot.lines.len();
        let target = snapshot.target.map(|target| target.line);
        let extension = snapshot
            .relative_path
            .extension()
            .and_then(|extension| extension.to_str())
            .unwrap_or("")
            .to_owned();
        let content_width = snapshot
            .lines
            .iter()
            .map(|line| snapshot.text[line.range.clone()].trim_end().chars().count())
            .max()
            .unwrap_or(0) as f32
            * 7.1
            + SOURCE_GUTTER_WIDTH
            + 24.0;
        uniform_list("code-viewer-source", rows, move |range, _, _| {
            range
                .map(|index| {
                    source_row(
                        &snapshot,
                        index,
                        target == Some(index + 1),
                        &extension,
                        content_width.max(320.0),
                        colors,
                    )
                })
                .collect()
        })
        .with_horizontal_sizing_behavior(ListHorizontalSizingBehavior::Unconstrained)
        .track_scroll(&self.scroll)
        .size_full()
        .into_any_element()
    }

    fn render_picker(&self, colors: SemanticColors, cx: &mut Context<Self>) -> Option<AnyElement> {
        if !self.picker_open {
            return None;
        }
        let query_empty = self.query.is_empty();
        let mut results = div()
            .id("code-search-results")
            .max_h(px(330.0))
            .overflow_y_scroll()
            .py(px(4.0));
        if self.results.is_empty() {
            results = results.child(
                div()
                    .h(px(72.0))
                    .px(px(18.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_size(px(Typo::META.size))
                    .text_color(colors.tertiary)
                    .child(if self.intelligence.is_some() {
                        if query_empty {
                            "Indexing workspace…"
                        } else {
                            "No matching files or symbols"
                        }
                    } else {
                        "Open one file to establish the workspace"
                    }),
            );
        } else {
            for (index, hit) in self.results.iter().take(40).enumerate() {
                let selected = index == self.highlighted_result;
                let path = hit.relative_path.to_string_lossy().into_owned();
                let preview = hit.preview.clone();
                let symbol = hit.kind == SearchHitKind::Symbol;
                results = results.child(
                    div()
                        .id(("code-search-result", index))
                        .min_h(px(39.0))
                        .px(px(9.0))
                        .py(px(5.0))
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .rounded(px(Radius::ROW))
                        .bg(if selected {
                            colors.primary.alpha(0.085)
                        } else {
                            colors.primary.alpha(0.0)
                        })
                        .cursor_pointer()
                        .hover(move |row| row.bg(colors.primary.alpha(0.07)))
                        .child(sf_symbol(
                            if symbol { "curlybraces" } else { "doc.text" },
                            11.5,
                            if symbol {
                                rgba(0xc792eaff)
                            } else {
                                colors.secondary
                            },
                        ))
                        .child(
                            div()
                                .min_w(px(0.0))
                                .flex_1()
                                .flex()
                                .flex_col()
                                .gap(px(1.0))
                                .child(
                                    div()
                                        .truncate()
                                        .font_family(crate::fonts::mono_family())
                                        .text_size(px(10.5))
                                        .font_weight(if symbol {
                                            FontWeight::MEDIUM
                                        } else {
                                            FontWeight::NORMAL
                                        })
                                        .text_color(colors.primary)
                                        .child(preview),
                                )
                                .child(
                                    div()
                                        .truncate()
                                        .font_family(crate::fonts::mono_family())
                                        .text_size(px(9.0))
                                        .text_color(colors.tertiary)
                                        .child(path),
                                ),
                        )
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.highlighted_result = index;
                            this.open_highlighted(cx);
                            cx.stop_propagation();
                        })),
                );
            }
        }

        let query = if query_empty {
            div()
                .text_color(colors.tertiary)
                .child("Search files and symbols…")
                .into_any_element()
        } else {
            crate::navigation::query_label(&self.query)
        };
        Some(
            div()
                .absolute()
                .top(px(40.0))
                .left(px(8.0))
                .right(px(8.0))
                .occlude()
                .child(FloatingSurface::new(
                    colors,
                    div()
                        .rounded(px(Radius::PANEL))
                        .overflow_hidden()
                        .child(
                            div()
                                .id("code-search-input")
                                .h(px(38.0))
                                .px(px(10.0))
                                .flex()
                                .items_center()
                                .gap(px(7.0))
                                .border_b_1()
                                .border_color(colors.primary.alpha(0.08))
                                .cursor_text()
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(|this, _, window, cx| {
                                        window.focus(&this.focus, cx);
                                        cx.stop_propagation();
                                    }),
                                )
                                .child(sf_symbol("magnifyingglass", 11.0, colors.tertiary))
                                .child(
                                    div()
                                        .min_w(px(0.0))
                                        .flex_1()
                                        .font_family(crate::fonts::mono_family())
                                        .text_size(px(11.0))
                                        .text_color(colors.primary)
                                        .child(query),
                                ),
                        )
                        .child(results),
                ))
                .into_any_element(),
        )
    }

    fn render_message(
        &self,
        colors: SemanticColors,
        symbol: &'static str,
        title: impl Into<SharedString>,
        body: impl Into<SharedString>,
    ) -> AnyElement {
        div()
            .size_full()
            .px(px(28.0))
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap(px(8.0))
            .text_center()
            .child(sf_symbol(symbol, 28.0, colors.tertiary))
            .child(
                div()
                    .text_size(px(Typo::ROW_EMPHASIZED.size))
                    .font_weight(Typo::ROW_EMPHASIZED.weight)
                    .text_color(colors.primary.alpha(0.88))
                    .child(title.into()),
            )
            .child(
                div()
                    .max_w(px(300.0))
                    .text_size(px(Typo::META.size))
                    .text_color(colors.tertiary)
                    .child(body.into()),
            )
            .into_any_element()
    }
}
impl Render for super::CodeViewer {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = SemanticColors::dark();
        let snapshot = match &self.state {
            ViewerState::Ready(snapshot) => Some(Arc::clone(snapshot)),
            _ => None,
        };
        let body = match &self.state {
            ViewerState::Empty => self.render_message(
                colors,
                "cursorarrow.click.2",
                "Open code from the terminal",
                "⌘-click a file path, stack frame, or compiler location to inspect it here.",
            ),
            ViewerState::Loading { reference } => self.render_message(
                colors,
                "ellipsis",
                "Opening file",
                format!("Resolving {reference}…"),
            ),
            ViewerState::Ready(snapshot) => self.render_source(Arc::clone(snapshot), colors),
            ViewerState::Error { reference, message } => self.render_message(
                colors,
                "exclamationmark.triangle",
                format!("Couldn’t open {reference}"),
                message.clone(),
            ),
        };
        let picker = self.render_picker(colors, cx);
        div()
            .relative()
            .size_full()
            .flex()
            .flex_col()
            .overflow_hidden()
            .bg(colors.background)
            .track_focus(&self.focus)
            .on_key_down(cx.listener(Self::handle_key_down))
            .child(self.render_toolbar(snapshot.as_deref(), colors, cx))
            .child(div().min_h(px(0.0)).flex_1().overflow_hidden().child(body))
            .when_some(picker, |viewer, picker| viewer.child(picker))
    }
}
