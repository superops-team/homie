use super::*;

impl WorkbenchInspector {
    fn render_diff(
        &mut self,
        snapshot: Arc<DiffSnapshot>,
        colors: SemanticColors,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let row_count = snapshot.rows.len();
        let content_width =
            (GUTTER_WIDTH + 28.0 + snapshot.max_text_columns as f32 * 7.1).clamp(320.0, 3700.0);
        let inspector = cx.entity();
        let repo_root = snapshot.repo_root.clone();
        let armed_hunk = self.armed_hunk;
        let list = uniform_list("inspector-diff", row_count, move |range, _, _| {
            render_rows(
                &snapshot,
                range,
                content_width,
                colors,
                inspector.clone(),
                &repo_root,
                armed_hunk,
            )
        })
        .with_horizontal_sizing_behavior(ListHorizontalSizingBehavior::Unconstrained)
        .track_scroll(&self.scroll)
        .size_full();
        let scrollbar = self.render_scrollbar(colors, cx);

        // The list's scroll bounds are available after its first layout pass.
        // Re-render once on the next frame so the fixed overlay can size itself.
        if !self.scrollbar_layout_primed {
            self.scrollbar_layout_primed = true;
            cx.on_next_frame(window, |_, _, cx| cx.notify());
        }

        div()
            .relative()
            .size_full()
            .overflow_hidden()
            .on_drag_move(cx.listener(
                |this, event: &DragMoveEvent<DraggedDiffScrollbar>, _, cx| {
                    this.set_scrollbar_offset(f32::from(event.event.position.y), cx);
                },
            ))
            .on_mouse_up_out(
                MouseButton::Left,
                cx.listener(|this, _, _, cx| this.finish_scrollbar_drag(cx)),
            )
            .child(list)
            .when_some(scrollbar, |body, scrollbar| body.child(scrollbar))
    }

    fn comparison_label(&self) -> String {
        if let LoadState::Ready(snapshot) = &self.state
            && let Some(base_ref) = snapshot.base_ref.as_deref()
        {
            return base_ref.to_owned();
        }
        match self.comparison {
            SessionDiffBase::DefaultBranch => "default branch".to_owned(),
            SessionDiffBase::Head => "HEAD".to_owned(),
        }
    }

    fn render_comparison_option(
        &self,
        comparison: SessionDiffBase,
        colors: SemanticColors,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let (title, detail, selector) = match comparison {
            SessionDiffBase::DefaultBranch => (
                "Default branch",
                "Committed and working changes",
                "INSPECTOR_COMPARE_DEFAULT",
            ),
            SessionDiffBase::Head => ("HEAD", "Uncommitted changes only", "INSPECTOR_COMPARE_HEAD"),
        };
        let selected = self.comparison == comparison;
        div()
            .id(SharedString::from(format!("compare-option-{title}")))
            .debug_selector(move || selector.to_owned())
            .min_h(px(48.0))
            .px(px(10.0))
            .py(px(7.0))
            .flex()
            .items_center()
            .gap(px(9.0))
            .cursor_pointer()
            .bg(if selected {
                colors.primary.alpha(0.075)
            } else {
                colors.primary.alpha(0.0)
            })
            .hover(move |row| row.bg(colors.primary.alpha(0.09)))
            .child(
                div()
                    .w(px(14.0))
                    .flex_none()
                    .flex()
                    .justify_center()
                    .when(selected, |slot| {
                        slot.child(sf_symbol("checkmark", 10.5, colors.primary))
                    }),
            )
            .child(
                div()
                    .min_w(px(0.0))
                    .flex_1()
                    .flex()
                    .flex_col()
                    .gap(px(1.0))
                    .child(
                        div()
                            .text_size(px(Typo::ROW.size))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(colors.primary)
                            .child(title),
                    )
                    .child(
                        div()
                            .text_size(px(Typo::META.size))
                            .text_color(colors.tertiary)
                            .child(detail),
                    ),
            )
            .on_click(cx.listener(move |this, _, _, cx| {
                this.select_comparison(comparison, cx);
                cx.stop_propagation();
            }))
            .into_any_element()
    }

    fn render_layer_option(
        &self,
        layer: DiffLayer,
        colors: SemanticColors,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let (label, selector) = match layer {
            DiffLayer::Branch => ("Branch", "INSPECTOR_LAYER_BRANCH"),
            DiffLayer::Working => ("Working", "INSPECTOR_LAYER_WORKING"),
            DiffLayer::Staged => ("Staged", "INSPECTOR_LAYER_STAGED"),
        };
        let selected = self.diff_layer == layer;
        div()
            .id(SharedString::from(format!("review-layer-{label}")))
            .debug_selector(move || selector.to_owned())
            .h(px(25.0))
            .px(px(8.0))
            .flex()
            .items_center()
            .rounded(px(Radius::CHIP))
            .bg(if selected {
                colors.primary.alpha(0.11)
            } else {
                colors.primary.alpha(0.0)
            })
            .cursor_pointer()
            .hover(move |button| button.bg(colors.primary.alpha(0.085)))
            .text_size(px(9.5))
            .font_weight(if selected {
                FontWeight::SEMIBOLD
            } else {
                FontWeight::MEDIUM
            })
            .text_color(if selected {
                colors.primary
            } else {
                colors.tertiary
            })
            .child(label)
            .on_click(cx.listener(move |this, _, _, cx| {
                this.select_diff_layer(layer, cx);
                cx.stop_propagation();
            }))
            .into_any_element()
    }

    fn render_file_navigator(
        &self,
        snapshot: Arc<DiffSnapshot>,
        colors: SemanticColors,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let mut files = div()
            .id("review-file-navigator-list")
            .max_h(px(390.0))
            .py(px(4.0))
            .overflow_y_scroll();
        for (index, file) in snapshot.file_diffs.iter().enumerate() {
            let row = file.row_range.start;
            files = files.child(
                div()
                    .id(("review-file-navigator-row", index))
                    .debug_selector(move || format!("INSPECTOR_REVIEW_FILE_{index}"))
                    .min_h(px(38.0))
                    .px(px(10.0))
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    .cursor_pointer()
                    .hover(move |item| item.bg(colors.primary.alpha(0.065)))
                    .child(sf_symbol(
                        "chevron.left.forwardslash.chevron.right",
                        11.5,
                        colors.tertiary,
                    ))
                    .child(
                        div()
                            .min_w(px(0.0))
                            .flex_1()
                            .truncate()
                            .font_family(crate::fonts::mono_family())
                            .text_size(px(10.0))
                            .text_color(colors.secondary)
                            .child(file.path.to_string_lossy().into_owned()),
                    )
                    .child(
                        div()
                            .flex_none()
                            .text_size(px(9.0))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(Ink::FRESH)
                            .child(format!("+{}", file.additions)),
                    )
                    .child(
                        div()
                            .flex_none()
                            .text_size(px(9.0))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(Ink::DANGER)
                            .child(format!("−{}", file.deletions)),
                    )
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.jump_to_diff_row(row, cx);
                        cx.stop_propagation();
                    })),
            );
        }

        div()
            .absolute()
            .inset_0()
            .child(div().absolute().inset_0().occlude().on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _, _, cx| {
                    this.files_open = false;
                    cx.notify();
                    cx.stop_propagation();
                }),
            ))
            .child(
                div()
                    .id("review-file-navigator")
                    .debug_selector(|| "INSPECTOR_FILE_NAVIGATOR".to_owned())
                    .absolute()
                    .top(px(40.0))
                    .right(px(9.0))
                    .w(px(330.0))
                    .occlude()
                    .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                    .on_mouse_down_out(cx.listener(|this, _, _, cx| {
                        this.files_open = false;
                        cx.notify();
                    }))
                    .child(FloatingSurface::new(colors, files)),
            )
            .into_any_element()
    }

    pub(super) fn render_changes(
        &mut self,
        colors: SemanticColors,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let label = self.comparison_label();
        let remote = self.context.as_ref().is_some_and(|context| context.remote);
        let empty_detail = match self.diff_layer {
            DiffLayer::Branch => format!("This branch matches {label}."),
            DiffLayer::Working => "The working tree matches the index.".to_owned(),
            DiffLayer::Staged => "The index matches HEAD.".to_owned(),
        };
        let body = match self.state.clone() {
            LoadState::Ready(snapshot) if snapshot.rows.is_empty() => self
                .render_message(colors, "checkmark.circle", "No changes", empty_detail)
                .into_any_element(),
            LoadState::Ready(snapshot) => self
                .render_diff(snapshot, colors, window, cx)
                .into_any_element(),
            LoadState::Loading => self
                .render_message(
                    colors,
                    "ellipsis",
                    "Loading changes",
                    "Reading the working tree…",
                )
                .into_any_element(),
            LoadState::NoSession => self
                .render_message(
                    colors,
                    "sidebar.left",
                    "Select a session",
                    "Changes follow the active agent.",
                )
                .into_any_element(),
            LoadState::Error(error) => self
                .render_message(
                    colors,
                    "exclamationmark.triangle",
                    "Couldn't load changes",
                    error,
                )
                .into_any_element(),
        };
        let comparison_open = remote && self.comparison_menu_open;
        let snapshot = match &self.state {
            LoadState::Ready(snapshot) => Some(Arc::clone(snapshot)),
            _ => None,
        };
        let file_count = snapshot
            .as_ref()
            .map_or(0, |snapshot| snapshot.file_diffs.len());
        let menu = if !remote && self.files_open {
            snapshot.map(|snapshot| self.render_file_navigator(snapshot, colors, cx))
        } else if comparison_open {
            Some(
                div()
                    .absolute()
                    .inset_0()
                    .child(div().absolute().inset_0().occlude().on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _, _, cx| {
                            this.comparison_menu_open = false;
                            cx.notify();
                            cx.stop_propagation();
                        }),
                    ))
                    .child(
                        div()
                            .id("inspector-comparison-menu")
                            .absolute()
                            .top(px(40.0))
                            .right(px(10.0))
                            .w(px(230.0))
                            .occlude()
                            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                            .on_mouse_down_out(cx.listener(|this, _, _, cx| {
                                this.comparison_menu_open = false;
                                cx.notify();
                            }))
                            .child(FloatingSurface::new(
                                colors,
                                div()
                                    .py(px(4.0))
                                    .rounded(px(Radius::PANEL))
                                    .overflow_hidden()
                                    .child(self.render_comparison_option(
                                        SessionDiffBase::DefaultBranch,
                                        colors,
                                        cx,
                                    ))
                                    .child(self.render_comparison_option(
                                        SessionDiffBase::Head,
                                        colors,
                                        cx,
                                    )),
                            )),
                    )
                    .into_any_element(),
            )
        } else {
            None
        };

        let toolbar = if remote {
            div()
                .h(px(38.0))
                .flex_none()
                .px(px(10.0))
                .flex()
                .items_center()
                .justify_between()
                .border_b_1()
                .border_color(colors.primary.alpha(0.06))
                .child(
                    div()
                        .text_size(px(Typo::META.size))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(colors.tertiary)
                        .child("Remote branch"),
                )
                .child(
                    div()
                        .id("inspector-comparison-button")
                        .debug_selector(|| "INSPECTOR_COMPARE_BUTTON".to_owned())
                        .max_w(px(184.0))
                        .h(px(26.0))
                        .px(px(8.0))
                        .flex()
                        .items_center()
                        .gap(px(5.0))
                        .rounded(px(Radius::BADGE))
                        .bg(colors
                            .primary
                            .alpha(if comparison_open { 0.10 } else { 0.055 }))
                        .cursor_pointer()
                        .hover(move |button| button.bg(colors.primary.alpha(0.10)))
                        .child(sf_symbol("arrow.branch", 11.0, colors.secondary))
                        .child(
                            div()
                                .min_w(px(0.0))
                                .truncate()
                                .text_size(px(Typo::META.size))
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(colors.secondary)
                                .child(format!("vs {label}")),
                        )
                        .child(sf_symbol("chevron.down", 9.0, colors.tertiary))
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.comparison_menu_open = !this.comparison_menu_open;
                            cx.notify();
                            cx.stop_propagation();
                        })),
                )
                .into_any_element()
        } else {
            div()
                .h(px(38.0))
                .flex_none()
                .px(px(9.0))
                .flex()
                .items_center()
                .gap(px(7.0))
                .border_b_1()
                .border_color(colors.primary.alpha(0.06))
                .child(
                    div()
                        .h(px(29.0))
                        .p(px(2.0))
                        .flex()
                        .items_center()
                        .gap(px(1.0))
                        .rounded(px(Radius::BADGE))
                        .bg(colors.primary.alpha(0.035))
                        .border_1()
                        .border_color(colors.primary.alpha(0.055))
                        .child(self.render_layer_option(DiffLayer::Branch, colors, cx))
                        .child(self.render_layer_option(DiffLayer::Working, colors, cx))
                        .child(self.render_layer_option(DiffLayer::Staged, colors, cx)),
                )
                .child(div().flex_1())
                .child(
                    div()
                        .id("review-files-button")
                        .debug_selector(|| "INSPECTOR_REVIEW_FILES".to_owned())
                        .h(px(26.0))
                        .px(px(8.0))
                        .flex()
                        .items_center()
                        .gap(px(5.0))
                        .rounded(px(Radius::BADGE))
                        .bg(colors
                            .primary
                            .alpha(if self.files_open { 0.10 } else { 0.045 }))
                        .text_size(px(9.5))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(if file_count > 0 {
                            colors.secondary
                        } else {
                            colors.primary.alpha(0.28)
                        })
                        .when(file_count > 0, |button| {
                            button
                                .cursor_pointer()
                                .hover(move |button| button.bg(colors.primary.alpha(0.09)))
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.files_open = !this.files_open;
                                    cx.notify();
                                    cx.stop_propagation();
                                }))
                        })
                        .child(sf_symbol("list.bullet", 10.5, colors.tertiary))
                        .child(format!("{file_count} files"))
                        .child(sf_symbol("chevron.down", 8.5, colors.tertiary)),
                )
                .into_any_element()
        };

        div()
            .relative()
            .size_full()
            .flex()
            .flex_col()
            .child(toolbar)
            .child(self.render_review_controls(colors, window, cx))
            .child(div().min_h(px(0.0)).flex_1().overflow_hidden().child(body))
            .when_some(menu, |panel, menu| panel.child(menu))
            .into_any_element()
    }
}
