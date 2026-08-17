use super::*;

pub(super) fn render_pull_request(
    pull_request: &PullRequestStatus,
    colors: SemanticColors,
    inspector: Entity<WorkbenchInspector>,
    body: Option<Arc<MarkdownDocument>>,
) -> AnyElement {
    let number = if pull_request.number > 0 {
        format!("PR #{}", pull_request.number)
    } else {
        "Pull request".to_owned()
    };
    let title = pull_request.title.clone().unwrap_or_else(|| number.clone());
    let author = pull_request.author.as_deref().unwrap_or("contributor");
    let (state_label, state_color) = pull_request_state(pull_request, colors);
    let checks_total =
        pull_request.checks_passed + pull_request.checks_failed + pull_request.checks_pending;
    let discussion_total = pull_request.comment_count + pull_request.review_count;
    let can_merge = pull_request_can_merge(pull_request);
    let view_url = pull_request.url.clone();
    let merge_url = pull_request.url.clone();
    let checks = sorted_pr_checks(pull_request);
    let discussion = pull_request.discussion.as_deref().unwrap_or_default();
    let ask_evidence = ReviewEvidence::PullRequest {
        url: pull_request.url.clone(),
        title: title.clone(),
        body: body.as_ref().map(|document| document.plain_text()),
        base: pull_request.base_ref_name.clone(),
        head: pull_request.head_ref_name.clone(),
    };
    let ask_inspector = inspector.clone();

    let mut surface = div()
        .id(SharedString::from(format!(
            "inspector-pr-{}",
            pull_request.url
        )))
        .flex()
        .flex_col()
        .gap(px(14.0))
        .rounded(px(Radius::CARD))
        .bg(colors.primary.alpha(0.022))
        .border_1()
        .border_color(colors.primary.alpha(0.075))
        .overflow_hidden()
        .child(
            div()
                .p(px(13.0))
                .pb(px(12.0))
                .flex()
                .flex_col()
                .gap(px(10.0))
                .child(
                    div()
                        .flex()
                        .items_start()
                        .gap(px(9.0))
                        .child(
                            div()
                                .size(px(30.0))
                                .flex_none()
                                .flex()
                                .items_center()
                                .justify_center()
                                .rounded_full()
                                .bg(state_color.alpha(0.12))
                                .child(sf_symbol_weighted(
                                    "arrow.triangle.pull",
                                    13.0,
                                    SymbolWeight::Semibold,
                                    state_color,
                                )),
                        )
                        .child(
                            div()
                                .min_w(px(0.0))
                                .flex_1()
                                .flex()
                                .flex_col()
                                .gap(px(3.0))
                                .child(
                                    div()
                                        .line_height(px(17.0))
                                        .text_size(px(13.0))
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(colors.primary)
                                        .child(title),
                                )
                                .child(
                                    div()
                                        .text_size(px(Typo::META.size))
                                        .text_color(colors.tertiary)
                                        .child(format!("{author} opened {number}")),
                                ),
                        )
                        .child(
                            div()
                                .id(SharedString::from(format!(
                                    "inspector-pr-ask-{}",
                                    pull_request.number
                                )))
                                .debug_selector(|| "INSPECTOR_PR_ASK".to_owned())
                                .h(px(24.0))
                                .px(px(8.0))
                                .flex_none()
                                .flex()
                                .items_center()
                                .gap(px(5.0))
                                .rounded(px(Radius::CHIP))
                                .bg(rgba(0xd9775717))
                                .cursor_pointer()
                                .hover(|button| button.bg(rgba(0xd9775728)))
                                .text_size(px(9.5))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(rgba(0xe9a381ff))
                                .child(sf_symbol("sparkles", 9.5, rgba(0xe9a381ff)))
                                .child("Ask")
                                .on_click(move |_, window, cx| {
                                    ask_inspector.update(cx, |inspector, cx| {
                                        inspector.open_ask(vec![ask_evidence.clone()], window, cx);
                                    });
                                    cx.stop_propagation();
                                }),
                        )
                        .child(
                            div()
                                .flex_none()
                                .px(px(7.0))
                                .h(px(21.0))
                                .flex()
                                .items_center()
                                .rounded_full()
                                .bg(state_color.alpha(0.12))
                                .text_size(px(10.0))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(state_color)
                                .child(state_label),
                        )
                        .child(
                            div()
                                .id(SharedString::from(format!(
                                    "inspector-pr-open-{}",
                                    pull_request.number
                                )))
                                .debug_selector(|| "INSPECTOR_PR_OPEN".to_owned())
                                .size(px(24.0))
                                .flex_none()
                                .flex()
                                .items_center()
                                .justify_center()
                                .rounded(px(Radius::CHIP))
                                .cursor_pointer()
                                .hover(move |button| button.bg(colors.primary.alpha(0.06)))
                                .child(sf_symbol("arrow.up.right", 10.5, colors.tertiary))
                                .on_click(move |_, _, cx| cx.open_url(&view_url)),
                        ),
                )
                .when(
                    pull_request.head_ref_name.is_some() || pull_request.base_ref_name.is_some(),
                    |header| {
                        let head = pull_request
                            .head_ref_name
                            .clone()
                            .unwrap_or_else(|| "head".to_owned());
                        let base = pull_request
                            .base_ref_name
                            .clone()
                            .unwrap_or_else(|| "base".to_owned());
                        header.child(
                            div()
                                .h(px(24.0))
                                .flex()
                                .items_center()
                                .gap(px(6.0))
                                .child(branch_badge(base, colors))
                                .child(sf_symbol("arrow.left", 9.5, colors.tertiary))
                                .child(branch_badge(head, colors)),
                        )
                    },
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(7.0))
                        .child(diff_stat(
                            format!("+{}", pull_request.additions),
                            Ink::FRESH,
                        ))
                        .child(diff_stat(
                            format!("−{}", pull_request.deletions),
                            Ink::DANGER,
                        ))
                        .child(
                            div()
                                .text_size(px(Typo::META.size))
                                .text_color(colors.tertiary)
                                .child(format!(
                                    "{} changed {}",
                                    pull_request.changed_files,
                                    if pull_request.changed_files == 1 {
                                        "file"
                                    } else {
                                        "files"
                                    }
                                )),
                        )
                        .when_some(pull_request.total_threads, |stats, total| {
                            stats.child(
                                div()
                                    .ml_auto()
                                    .text_size(px(10.5))
                                    .text_color(colors.tertiary)
                                    .child(format!(
                                        "{}/{} resolved",
                                        pull_request.resolved_threads.unwrap_or(0),
                                        total
                                    )),
                            )
                        }),
                )
                .when_some(body, |header, body| {
                    header.child(
                        div()
                            .mt(px(1.0))
                            .p(px(11.0))
                            .rounded(px(Radius::BADGE))
                            .bg(colors.primary.alpha(0.035))
                            .border_1()
                            .border_color(colors.primary.alpha(0.055))
                            .child(render_markdown(&body, colors)),
                    )
                }),
        );

    if checks_total > 0 {
        let (checks_label, checks_color) = checks_rollup(pull_request);
        let mut check_rows = div()
            .rounded(px(Radius::BADGE))
            .border_1()
            .border_color(colors.primary.alpha(0.07))
            .overflow_hidden();
        for (index, check) in checks.iter().enumerate() {
            check_rows = check_rows.child(render_pr_check(
                check,
                index,
                checks.len(),
                pull_request.number,
                colors,
                inspector.clone(),
            ));
        }
        surface = surface.child(
            div()
                .px(px(13.0))
                .flex()
                .flex_col()
                .gap(px(7.0))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .child(section_label("Checks", colors))
                        .child(
                            div()
                                .ml_auto()
                                .text_size(px(10.5))
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(checks_color)
                                .child(checks_label),
                        ),
                )
                .child(check_rows),
        );
    }

    if discussion_total > 0 {
        let mut conversation = div().px(px(13.0)).flex().flex_col().gap(px(8.0)).child(
            div()
                .flex()
                .items_center()
                .child(section_label("Conversation", colors))
                .child(
                    div()
                        .ml_auto()
                        .text_size(px(10.5))
                        .text_color(colors.tertiary)
                        .child(format!("{discussion_total} items")),
                ),
        );
        if discussion.is_empty() {
            conversation = conversation.child(render_discussion_fallback(pull_request, colors));
        } else {
            for (index, item) in discussion.iter().enumerate() {
                conversation = conversation.child(render_discussion_item(
                    item,
                    index,
                    discussion.len(),
                    pull_request.number,
                    colors,
                ));
            }
        }
        surface = surface.child(conversation);
    }

    if pull_request.state == "OPEN" {
        let (merge_detail, merge_color) = if can_merge {
            ("Ready to merge", Ink::FRESH)
        } else {
            (merge_blocker_label(pull_request), Ink::ATTENTION)
        };
        surface = surface.child(
            div()
                .mt(px(1.0))
                .p(px(13.0))
                .flex()
                .items_center()
                .gap(px(10.0))
                .border_t_1()
                .border_color(colors.primary.alpha(0.07))
                .bg(merge_color.alpha(0.045))
                .child(
                    div()
                        .min_w(px(0.0))
                        .flex_1()
                        .flex()
                        .flex_col()
                        .gap(px(2.0))
                        .child(
                            div()
                                .text_size(px(Typo::META.size))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(colors.primary)
                                .child(merge_detail),
                        )
                        .child(
                            div()
                                .text_size(px(10.0))
                                .text_color(colors.tertiary)
                                .child("Review and confirm on GitHub"),
                        ),
                )
                .child(
                    div()
                        .id(SharedString::from(format!(
                            "inspector-pr-merge-{}",
                            pull_request.number
                        )))
                        .debug_selector(|| "INSPECTOR_PR_MERGE".to_owned())
                        .h(px(30.0))
                        .px(px(10.0))
                        .flex_none()
                        .flex()
                        .items_center()
                        .gap(px(6.0))
                        .rounded(px(Radius::BADGE))
                        .cursor_pointer()
                        .bg(merge_color.alpha(if can_merge { 0.86 } else { 0.13 }))
                        .text_size(px(11.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(if can_merge {
                            rgba(0xffffffff)
                        } else {
                            merge_color
                        })
                        .hover(move |button| {
                            button.bg(merge_color.alpha(if can_merge { 1.0 } else { 0.19 }))
                        })
                        .child("Merge pull request")
                        .child(sf_symbol(
                            "arrow.up.right",
                            9.0,
                            if can_merge {
                                rgba(0xffffffff)
                            } else {
                                merge_color
                            },
                        ))
                        .on_click(move |_, _, cx| cx.open_url(&merge_url)),
                ),
        );
    }

    surface.pb(px(13.0)).into_any_element()
}

fn branch_badge(branch: String, colors: SemanticColors) -> AnyElement {
    div()
        .min_w(px(0.0))
        .max_w(px(158.0))
        .h(px(22.0))
        .px(px(7.0))
        .flex()
        .items_center()
        .rounded(px(Radius::CHIP))
        .bg(colors.primary.alpha(0.045))
        .font_family(crate::fonts::mono_family())
        .text_size(px(10.0))
        .text_color(colors.secondary)
        .truncate()
        .child(branch)
        .into_any_element()
}

fn diff_stat(label: String, color: gpui::Rgba) -> AnyElement {
    div()
        .px(px(7.0))
        .h(px(22.0))
        .flex()
        .items_center()
        .rounded(px(Radius::CHIP))
        .bg(color.alpha(0.09))
        .text_size(px(10.5))
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(color)
        .child(label)
        .into_any_element()
}

fn render_pr_check(
    check: &PrCheck,
    index: usize,
    total: usize,
    pr_number: i64,
    colors: SemanticColors,
    inspector: Entity<WorkbenchInspector>,
) -> AnyElement {
    let (symbol, color, status) = match check.result.as_str() {
        "pass" => ("checkmark.circle.fill", Ink::FRESH, "Passed"),
        "fail" => ("xmark.circle.fill", Ink::DANGER, "Failed"),
        "pending" => ("clock.fill", Ink::ATTENTION, "Running"),
        _ => ("circle", colors.tertiary, "Unknown"),
    };
    let detail = check
        .detail
        .as_deref()
        .map(humanize_github_state)
        .filter(|detail| detail != status)
        .unwrap_or_else(|| status.to_owned());
    let url = check.url.clone();
    let ask_evidence = ReviewEvidence::Check {
        name: check.name.clone(),
        result: check.result.clone(),
        detail: check.detail.clone(),
    };
    div()
        .id(SharedString::from(format!(
            "inspector-pr-{pr_number}-check-{index}"
        )))
        .debug_selector(move || format!("INSPECTOR_PR_CHECK_{index}"))
        .min_h(px(34.0))
        .px(px(9.0))
        .flex()
        .items_center()
        .gap(px(8.0))
        .bg(colors.primary.alpha(if check.result == "pending" {
            0.025
        } else {
            0.0
        }))
        .when(index + 1 < total, |row| {
            row.border_b_1().border_color(colors.primary.alpha(0.055))
        })
        .when(url.is_some(), |row| {
            row.cursor_pointer()
                .hover(move |row| row.bg(colors.primary.alpha(0.045)))
        })
        .child(sf_symbol(symbol, 12.0, color))
        .child(
            div()
                .min_w(px(0.0))
                .flex_1()
                .truncate()
                .text_size(px(Typo::META.size))
                .font_weight(FontWeight::MEDIUM)
                .text_color(colors.secondary)
                .child(check.name.clone()),
        )
        .child(
            div()
                .flex_none()
                .text_size(px(10.0))
                .font_weight(FontWeight::MEDIUM)
                .text_color(color)
                .child(detail),
        )
        .child(
            div()
                .id(("ask-pr-check", index))
                .size(px(20.0))
                .flex_none()
                .flex()
                .items_center()
                .justify_center()
                .rounded(px(Radius::CHIP))
                .cursor_pointer()
                .hover(move |button| button.bg(rgba(0xd9775722)))
                .child(sf_symbol("sparkles", 8.5, rgba(0xe9a381ff)))
                .on_click(move |_, window, cx| {
                    inspector.update(cx, |inspector, cx| {
                        inspector.open_ask(vec![ask_evidence.clone()], window, cx);
                    });
                    cx.stop_propagation();
                }),
        )
        .when_some(url, |row, url| {
            row.child(sf_symbol("arrow.up.right", 9.0, colors.tertiary))
                .on_click(move |_, _, cx| cx.open_url(&url))
        })
        .into_any_element()
}

fn render_discussion_item(
    item: &PrDiscussionItem,
    index: usize,
    total: usize,
    pr_number: i64,
    colors: SemanticColors,
) -> AnyElement {
    let author = item.author.clone();
    let initial = author
        .chars()
        .next()
        .map(|character| character.to_uppercase().collect::<String>())
        .unwrap_or_else(|| "?".to_owned());
    let is_review = item.kind == "review";
    let (review_label, review_color) = discussion_state(item, colors);
    let body = MarkdownDocument::parse(&item.body);
    let body_fallback = if item.body.trim().is_empty() {
        review_label
            .clone()
            .unwrap_or_else(|| "Commented".to_owned())
    } else {
        String::new()
    };
    let time = item.created_at.as_ref().map(|date| relative_time(date.0));
    let url = item.url.clone();

    div()
        .id(SharedString::from(format!(
            "inspector-pr-{pr_number}-comment-{index}"
        )))
        .debug_selector(move || format!("INSPECTOR_PR_COMMENT_{index}"))
        .flex()
        .items_stretch()
        .gap(px(8.0))
        .child(
            div()
                .w(px(26.0))
                .flex_none()
                .flex()
                .flex_col()
                .items_center()
                .child(
                    div()
                        .size(px(24.0))
                        .flex_none()
                        .flex()
                        .items_center()
                        .justify_center()
                        .rounded_full()
                        .bg(if is_review {
                            review_color.alpha(0.13)
                        } else {
                            colors.primary.alpha(0.075)
                        })
                        .text_size(px(9.5))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(if is_review {
                            review_color
                        } else {
                            colors.secondary
                        })
                        .child(initial),
                )
                .when(index + 1 < total, |rail| {
                    rail.child(
                        div()
                            .mt(px(4.0))
                            .w(px(1.0))
                            .flex_1()
                            .min_h(px(10.0))
                            .bg(colors.primary.alpha(0.08)),
                    )
                }),
        )
        .child(
            div()
                .id(SharedString::from(format!(
                    "inspector-pr-{pr_number}-comment-card-{index}"
                )))
                .min_w(px(0.0))
                .flex_1()
                .mb(px(if index + 1 < total { 2.0 } else { 0.0 }))
                .rounded(px(Radius::BADGE))
                .border_1()
                .border_color(colors.primary.alpha(0.07))
                .bg(colors.primary.alpha(0.025))
                .when(url.is_some(), |card| {
                    card.cursor_pointer()
                        .hover(move |card| card.bg(colors.primary.alpha(0.05)))
                })
                .child(
                    div()
                        .min_h(px(29.0))
                        .px(px(9.0))
                        .flex()
                        .items_center()
                        .gap(px(5.0))
                        .border_b_1()
                        .border_color(colors.primary.alpha(0.055))
                        .child(
                            div()
                                .text_size(px(10.5))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(colors.primary)
                                .child(author),
                        )
                        .when_some(review_label, |header, label| {
                            header.child(
                                div()
                                    .px(px(5.0))
                                    .h(px(17.0))
                                    .flex()
                                    .items_center()
                                    .rounded_full()
                                    .bg(review_color.alpha(0.11))
                                    .text_size(px(9.0))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(review_color)
                                    .child(label),
                            )
                        })
                        .when_some(time, |header, time| {
                            header.child(
                                div()
                                    .ml_auto()
                                    .text_size(px(9.5))
                                    .text_color(colors.tertiary)
                                    .child(time),
                            )
                        }),
                )
                .child(
                    div()
                        .px(px(9.0))
                        .py(px(8.0))
                        .child(if body_fallback.is_empty() {
                            render_markdown(&body, colors)
                        } else {
                            div()
                                .text_size(px(Typo::META.size))
                                .text_color(colors.secondary)
                                .child(body_fallback)
                                .into_any_element()
                        }),
                )
                .when_some(url, |card, url| {
                    card.on_click(move |_, _, cx| cx.open_url(&url))
                }),
        )
        .into_any_element()
}

fn render_discussion_fallback(
    pull_request: &PullRequestStatus,
    colors: SemanticColors,
) -> AnyElement {
    let discussion = pull_request_discussion(pull_request)
        .unwrap_or_else(|| "Open the conversation on GitHub".to_owned());
    let url = pull_request.url.clone();
    div()
        .id(SharedString::from(format!(
            "inspector-pr-discussion-{}",
            pull_request.number
        )))
        .min_h(px(38.0))
        .px(px(9.0))
        .flex()
        .items_center()
        .gap(px(8.0))
        .rounded(px(Radius::BADGE))
        .border_1()
        .border_color(colors.primary.alpha(0.07))
        .bg(colors.primary.alpha(0.025))
        .cursor_pointer()
        .hover(move |row| row.bg(colors.primary.alpha(0.05)))
        .child(sf_symbol(
            "bubble.left.and.bubble.right",
            12.0,
            colors.secondary,
        ))
        .child(
            div()
                .flex_1()
                .text_size(px(Typo::META.size))
                .text_color(colors.secondary)
                .child(discussion),
        )
        .child(sf_symbol("arrow.up.right", 9.0, colors.tertiary))
        .on_click(move |_, _, cx| cx.open_url(&url))
        .into_any_element()
}
