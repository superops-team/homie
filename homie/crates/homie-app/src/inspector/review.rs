use super::*;

impl WorkbenchInspector {
    pub(super) fn render_review_controls(
        &mut self,
        colors: SemanticColors,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let ReviewLoadState::Ready(status) = &self.review_state else {
            let (symbol, label) = match &self.review_state {
                ReviewLoadState::NoSession => ("minus.circle", "Select an agent to review"),
                ReviewLoadState::Remote => (
                    "network",
                    "Remote changes are view-only until Git actions move into the daemon",
                ),
                ReviewLoadState::Loading => ("ellipsis", "Reading index and working tree…"),
                ReviewLoadState::Error(error) => {
                    return div()
                        .px(px(10.0))
                        .py(px(7.0))
                        .flex()
                        .items_center()
                        .gap(px(7.0))
                        .border_b_1()
                        .border_color(colors.primary.alpha(0.06))
                        .text_size(px(Typo::META.size))
                        .text_color(Ink::ATTENTION)
                        .child(sf_symbol("exclamationmark.triangle", 11.0, Ink::ATTENTION))
                        .child(error.clone())
                        .into_any_element();
                }
                ReviewLoadState::Ready(_) => unreachable!(),
            };
            return div()
                .h(px(36.0))
                .flex_none()
                .px(px(10.0))
                .flex()
                .items_center()
                .gap(px(7.0))
                .border_b_1()
                .border_color(colors.primary.alpha(0.06))
                .text_size(px(Typo::META.size))
                .text_color(colors.tertiary)
                .child(sf_symbol(symbol, 11.0, colors.tertiary))
                .child(label)
                .into_any_element();
        };
        let status = Arc::clone(status);
        let staged_paths: Vec<_> = status
            .staged
            .iter()
            .map(|change| change.path.clone())
            .collect();
        // Conflicted paths are deliberately excluded. `git add` on a file that
        // still carries conflict markers both stages the markers and collapses
        // index stages 1/2/3, after which `git checkout --merge` can no longer
        // reconstruct the conflict. Resolving stays an explicit, per-file act.
        let mut stage_paths: Vec<_> = status
            .unstaged
            .iter()
            .chain(status.untracked.iter())
            .map(|change| change.path.clone())
            .collect();
        stage_paths.sort();
        stage_paths.dedup();
        let discard_paths: Vec<_> = status
            .unstaged
            .iter()
            .map(|change| change.path.clone())
            .collect();
        let staged_count = status.staged.len();
        let working_count = status.unstaged.len() + status.untracked.len();
        let conflicted_count = status.conflicted.len();
        let branch = status
            .branch
            .name
            .clone()
            .unwrap_or_else(|| "Detached HEAD".to_owned());
        let busy = self.review_action_busy;
        let commit_open = self.commit_open;
        let discard_armed = self.discard_armed;

        let mut actions = div().flex().items_center().gap(px(5.0));
        if self.diff_layer == DiffLayer::Working && !stage_paths.is_empty() {
            let paths = stage_paths;
            actions = actions.child(
                div()
                    .id("review-stage-all")
                    .h(px(25.0))
                    .px(px(8.0))
                    .flex()
                    .items_center()
                    .gap(px(5.0))
                    .rounded(px(Radius::BADGE))
                    .bg(if staged_count == 0 {
                        rgba(0xd9775724)
                    } else {
                        colors.primary.alpha(0.055)
                    })
                    .text_size(px(10.5))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(if staged_count == 0 {
                        rgba(0xe89a7cff)
                    } else {
                        colors.secondary
                    })
                    .when(!busy, |button| {
                        button
                            .cursor_pointer()
                            .hover(move |button| button.bg(colors.primary.alpha(0.10)))
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.run_review_action(ReviewAction::Stage(paths.clone()), cx);
                                cx.stop_propagation();
                            }))
                    })
                    .child(sf_symbol("plus", 9.0, colors.secondary))
                    .child("Stage all"),
            );
        }
        if self.diff_layer == DiffLayer::Staged && !staged_paths.is_empty() {
            let paths = staged_paths;
            actions = actions.child(
                div()
                    .id("review-unstage-all")
                    .h(px(25.0))
                    .px(px(8.0))
                    .flex()
                    .items_center()
                    .rounded(px(Radius::BADGE))
                    .text_size(px(10.5))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(colors.tertiary)
                    .when(!busy, |button| {
                        button
                            .cursor_pointer()
                            .hover(move |button| button.bg(colors.primary.alpha(0.07)))
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.run_review_action(ReviewAction::Unstage(paths.clone()), cx);
                                cx.stop_propagation();
                            }))
                    })
                    .child("Unstage"),
            );
            actions = actions.child(
                div()
                    .id("review-open-commit")
                    .h(px(25.0))
                    .px(px(9.0))
                    .flex()
                    .items_center()
                    .gap(px(5.0))
                    .rounded(px(Radius::BADGE))
                    .bg(rgba(0xd9775730))
                    .text_size(px(10.5))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(rgba(0xf0aa8fff))
                    .when(!busy, |button| {
                        button
                            .cursor_pointer()
                            .hover(|button| button.bg(rgba(0xd9775744)))
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.commit_open = !this.commit_open;
                                this.discard_armed = false;
                                this.ask_draft = None;
                                this.ask_feedback = None;
                                this.ask_query.clear();
                                if this.commit_open {
                                    window.focus(&this.focus, cx);
                                }
                                cx.notify();
                                cx.stop_propagation();
                            }))
                    })
                    .child(sf_symbol("checkmark", 9.5, rgba(0xf0aa8fff)))
                    .child(if commit_open { "Cancel" } else { "Commit" }),
            );
        }
        if self.diff_layer == DiffLayer::Working && !discard_paths.is_empty() {
            let paths = discard_paths;
            actions = actions.child(
                div()
                    .id("review-discard-all")
                    .h(px(25.0))
                    .px(px(7.0))
                    .flex()
                    .items_center()
                    .gap(px(4.0))
                    .rounded(px(Radius::BADGE))
                    .text_size(px(10.5))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(if discard_armed {
                        Ink::DANGER
                    } else {
                        colors.tertiary
                    })
                    .when(!busy, |button| {
                        button
                            .cursor_pointer()
                            .hover(move |button| button.bg(Ink::DANGER.alpha(0.09)))
                            .on_click(cx.listener(move |this, _, _, cx| {
                                if this.discard_armed {
                                    this.run_review_action(
                                        ReviewAction::Discard(paths.clone()),
                                        cx,
                                    );
                                } else {
                                    this.discard_armed = true;
                                    this.commit_open = false;
                                    cx.notify();
                                }
                                cx.stop_propagation();
                            }))
                    })
                    .child(sf_symbol(
                        "trash",
                        9.5,
                        if discard_armed {
                            Ink::DANGER
                        } else {
                            colors.tertiary
                        },
                    ))
                    .child(if discard_armed { "Discard?" } else { "Discard" }),
            );
        }

        let branch_detail = match (status.branch.ahead, status.branch.behind) {
            (0, 0) => None,
            (ahead, 0) => Some(format!("↑{ahead}")),
            (0, behind) => Some(format!("↓{behind}")),
            (ahead, behind) => Some(format!("↑{ahead} ↓{behind}")),
        };
        let counts = format!(
            "{staged_count} staged · {working_count} working{}",
            if conflicted_count > 0 {
                format!(" · {conflicted_count} conflicted")
            } else {
                String::new()
            }
        );
        let mut panel = div()
            .flex_none()
            .px(px(10.0))
            .py(px(7.0))
            .flex()
            .flex_col()
            .gap(px(7.0))
            .border_b_1()
            .border_color(colors.primary.alpha(0.06))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(7.0))
                    .child(sf_symbol("arrow.branch", 11.0, colors.secondary))
                    .child(
                        div()
                            .min_w(px(0.0))
                            .flex_1()
                            .truncate()
                            .font_family(crate::fonts::mono_family())
                            .text_size(px(10.5))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(colors.secondary)
                            .child(branch),
                    )
                    .when_some(branch_detail, |row, detail| {
                        row.child(
                            div()
                                .font_family(crate::fonts::mono_family())
                                .text_size(px(9.5))
                                .text_color(colors.tertiary)
                                .child(detail),
                        )
                    })
                    .child(
                        div()
                            .text_size(px(9.5))
                            .text_color(if conflicted_count > 0 {
                                Ink::DANGER
                            } else {
                                colors.tertiary
                            })
                            .child(counts),
                    ),
            )
            .when(self.diff_layer != DiffLayer::Branch, |panel| {
                panel.child(actions)
            })
            .when(self.diff_layer == DiffLayer::Branch, |panel| {
                panel.child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.0))
                        .text_size(px(9.5))
                        .text_color(colors.tertiary)
                        .child(sf_symbol("scope", 9.5, colors.tertiary))
                        .child("Overview only · choose Working or Staged to mutate hunks"),
                )
            });

        if self.commit_open {
            let empty = self.commit_query.is_empty();
            panel = panel.child(
                div()
                    .p(px(8.0))
                    .flex()
                    .items_center()
                    .gap(px(7.0))
                    .rounded(px(Radius::ROW))
                    .bg(colors.primary.alpha(0.035))
                    .border_1()
                    .border_color(colors.primary.alpha(0.08))
                    .child(
                        div()
                            .id("review-commit-message")
                            .min_w(px(0.0))
                            .h(px(27.0))
                            .flex_1()
                            .px(px(8.0))
                            .flex()
                            .items_center()
                            .rounded(px(Radius::CHIP))
                            .bg(colors.background)
                            .border_1()
                            .border_color(colors.primary.alpha(0.10))
                            .cursor_text()
                            .font_family(crate::fonts::mono_family())
                            .text_size(px(10.5))
                            .text_color(colors.primary)
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|this, _, window, cx| {
                                    window.focus(&this.focus, cx);
                                    cx.stop_propagation();
                                }),
                            )
                            .child(if empty {
                                div()
                                    .text_color(colors.tertiary)
                                    .child("Commit message…")
                                    .into_any_element()
                            } else {
                                crate::navigation::query_label(&self.commit_query)
                            }),
                    )
                    .child(
                        div()
                            .id("review-submit-commit")
                            .h(px(27.0))
                            .px(px(9.0))
                            .flex()
                            .items_center()
                            .rounded(px(Radius::CHIP))
                            .bg(if empty {
                                colors.primary.alpha(0.035)
                            } else {
                                rgba(0xd9775730)
                            })
                            .text_size(px(10.5))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(if empty {
                                colors.primary.alpha(0.25)
                            } else {
                                rgba(0xf0aa8fff)
                            })
                            .when(!empty && !busy, |button| {
                                button
                                    .cursor_pointer()
                                    .hover(|button| button.bg(rgba(0xd9775744)))
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.submit_commit(cx);
                                        cx.stop_propagation();
                                    }))
                            })
                            .child("Commit"),
                    ),
            );
        }
        if let Some((success, message)) = &self.review_feedback {
            let accent = if *success { Ink::FRESH } else { Ink::DANGER };
            panel = panel.child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.0))
                    .text_size(px(10.0))
                    .text_color(accent)
                    .child(sf_symbol(
                        if *success {
                            "checkmark.circle.fill"
                        } else {
                            "exclamationmark.circle.fill"
                        },
                        10.5,
                        accent,
                    ))
                    .child(message.clone()),
            );
        }
        let _ = window;
        panel.into_any_element()
    }
}
