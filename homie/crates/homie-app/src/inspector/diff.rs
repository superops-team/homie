use super::*;

struct DiffRowRenderContext {
    content_width: f32,
    colors: SemanticColors,
    inspector: Entity<WorkbenchInspector>,
    repo_root: PathBuf,
    layer: DiffLayer,
    armed_hunk: Option<u64>,
}

pub(super) fn render_rows(
    snapshot: &DiffSnapshot,
    range: Range<usize>,
    content_width: f32,
    colors: SemanticColors,
    inspector: Entity<WorkbenchInspector>,
    repo_root: &Path,
    armed_hunk: Option<u64>,
) -> Vec<AnyElement> {
    let context = DiffRowRenderContext {
        content_width,
        colors,
        inspector,
        repo_root: repo_root.to_path_buf(),
        layer: snapshot.layer,
        armed_hunk,
    };
    range
        .map(|index| {
            let owning_file = snapshot
                .file_diffs
                .iter()
                .find(|file| file.row_range.contains(&index));
            let file = (snapshot.rows[index].kind == DiffRowKind::File)
                .then(|| owning_file.cloned())
                .flatten();
            let hunk = (snapshot.rows[index].kind == DiffRowKind::Hunk)
                .then(|| {
                    owning_file.and_then(|file| {
                        file.hunks
                            .iter()
                            .find(|hunk| hunk.row_range.start == index)
                            .cloned()
                            .map(|hunk| (file.path.clone(), hunk))
                    })
                })
                .flatten();
            render_row(index, &snapshot.rows[index], &context, file, hunk)
        })
        .collect()
}

fn render_row(
    index: usize,
    row: &DiffRow,
    context: &DiffRowRenderContext,
    file: Option<DiffFile>,
    hunk: Option<(PathBuf, DiffHunk)>,
) -> AnyElement {
    let content_width = context.content_width;
    let colors = context.colors;
    let inspector = context.inspector.clone();
    let repo_root = &context.repo_root;
    let layer = context.layer;
    let armed_hunk = context.armed_hunk;
    let (background, foreground, marker) = match row.kind {
        DiffRowKind::Addition => (rgba(0x2f7d4a24), rgba(0xc7ebd2ff), "+"),
        DiffRowKind::Deletion => (rgba(0x9f3a4424), rgba(0xf0c4c8ff), "−"),
        DiffRowKind::Hunk => (rgba(0x4675a31c), rgba(0x9bbde0ff), ""),
        DiffRowKind::File => (rgba(0xffffff09), colors.primary, ""),
        DiffRowKind::Context => (rgba(0x00000000), rgba(0xffffffb8), ""),
        DiffRowKind::Meta => (rgba(0x00000000), rgba(0xffffff66), ""),
    };
    let line_number = |line: Option<u32>| line.map_or_else(String::new, |line| line.to_string());
    let text = if row.kind == DiffRowKind::File {
        SharedString::from(row.text.clone())
    } else {
        SharedString::from(format!("{marker}{}", row.text))
    };

    let reference = row.text.clone();
    let cwd = repo_root.to_path_buf();
    let open_inspector = inspector.clone();
    let mut actions = div()
        .absolute()
        .right(px(6.0))
        .top(px(2.0))
        .h(px(16.0))
        .flex()
        .items_center()
        .gap(px(2.0))
        .rounded(px(Radius::CHIP))
        .bg(colors.background.alpha(0.96))
        .border_1()
        .border_color(colors.primary.alpha(0.10));

    if let Some(file) = file.as_ref() {
        let ask_inspector = inspector.clone();
        let evidence = ReviewEvidence::File {
            path: file.path.clone(),
            layer: prompt_layer(layer),
            patch: file
                .hunks
                .iter()
                .map(|hunk| String::from_utf8_lossy(&hunk.patch))
                .collect::<Vec<_>>()
                .join("\n"),
        };
        actions = actions.child(
            div()
                .id(("ask-diff-file", index))
                .h_full()
                .px(px(5.0))
                .flex()
                .items_center()
                .gap(px(3.0))
                .rounded(px(Radius::CHIP))
                .cursor_pointer()
                .hover(move |button| button.bg(rgba(0xd9775722)))
                .text_size(px(8.5))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(rgba(0xe9a381ff))
                .child(sf_symbol("sparkles", 8.0, rgba(0xe9a381ff)))
                .child("Ask")
                .on_click(move |_, window, cx| {
                    ask_inspector.update(cx, |inspector, cx| {
                        inspector.open_ask(vec![evidence.clone()], window, cx);
                    });
                    cx.stop_propagation();
                }),
        );
        match layer {
            DiffLayer::Working => {
                let stage_inspector = inspector.clone();
                let path = file.path.clone();
                actions = actions.child(
                    div()
                        .id(("stage-diff-file", index))
                        .h_full()
                        .px(px(5.0))
                        .flex()
                        .items_center()
                        .rounded(px(Radius::CHIP))
                        .cursor_pointer()
                        .hover(move |button| button.bg(colors.primary.alpha(0.09)))
                        .text_size(px(8.5))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(colors.secondary)
                        .child("Stage")
                        .on_click(move |_, _, cx| {
                            stage_inspector.update(cx, |inspector, cx| {
                                inspector
                                    .run_review_action(ReviewAction::Stage(vec![path.clone()]), cx);
                            });
                            cx.stop_propagation();
                        }),
                );
            }
            DiffLayer::Staged => {
                let unstage_inspector = inspector.clone();
                let path = file.path.clone();
                actions = actions.child(
                    div()
                        .id(("unstage-diff-file", index))
                        .h_full()
                        .px(px(5.0))
                        .flex()
                        .items_center()
                        .rounded(px(Radius::CHIP))
                        .cursor_pointer()
                        .hover(move |button| button.bg(colors.primary.alpha(0.09)))
                        .text_size(px(8.5))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(colors.tertiary)
                        .child("Unstage")
                        .on_click(move |_, _, cx| {
                            unstage_inspector.update(cx, |inspector, cx| {
                                inspector.run_review_action(
                                    ReviewAction::Unstage(vec![path.clone()]),
                                    cx,
                                );
                            });
                            cx.stop_propagation();
                        }),
                );
            }
            DiffLayer::Branch => {}
        }
    }

    if let Some((path, hunk)) = hunk.as_ref() {
        let ask_inspector = inspector.clone();
        let evidence = ReviewEvidence::Hunk {
            path: path.clone(),
            layer: prompt_layer(layer),
            header: hunk.header.clone(),
            patch: String::from_utf8_lossy(&hunk.patch).into_owned(),
        };
        actions = actions.child(
            div()
                .id(("ask-diff-hunk", index))
                .h_full()
                .px(px(5.0))
                .flex()
                .items_center()
                .gap(px(3.0))
                .rounded(px(Radius::CHIP))
                .cursor_pointer()
                .hover(move |button| button.bg(rgba(0xd9775722)))
                .text_size(px(8.5))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(rgba(0xe9a381ff))
                .child(sf_symbol("sparkles", 8.0, rgba(0xe9a381ff)))
                .child("Ask")
                .on_click(move |_, window, cx| {
                    ask_inspector.update(cx, |inspector, cx| {
                        inspector.open_ask(vec![evidence.clone()], window, cx);
                    });
                    cx.stop_propagation();
                }),
        );
        let patch = hunk.patch.clone();
        match layer {
            DiffLayer::Working => {
                let stage_inspector = inspector.clone();
                let stage_patch = patch.clone();
                actions = actions.child(
                    div()
                        .id(("stage-diff-hunk", index))
                        .h_full()
                        .px(px(5.0))
                        .flex()
                        .items_center()
                        .rounded(px(Radius::CHIP))
                        .cursor_pointer()
                        .hover(move |button| button.bg(colors.primary.alpha(0.09)))
                        .text_size(px(8.5))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(colors.secondary)
                        .child("Stage")
                        .on_click(move |_, _, cx| {
                            stage_inspector.update(cx, |inspector, cx| {
                                inspector.run_review_action(
                                    ReviewAction::Patch {
                                        patch: stage_patch.clone(),
                                        mutation: PatchMutation::Stage,
                                    },
                                    cx,
                                );
                            });
                            cx.stop_propagation();
                        }),
                );
                if !patch_creates_file(&patch) {
                    let discard_inspector = inspector.clone();
                    let discard_patch = patch;
                    let fingerprint = hunk.fingerprint;
                    let armed = armed_hunk == Some(fingerprint);
                    actions = actions.child(
                        div()
                            .id(("discard-diff-hunk", index))
                            .h_full()
                            .px(px(5.0))
                            .flex()
                            .items_center()
                            .rounded(px(Radius::CHIP))
                            .cursor_pointer()
                            .bg(if armed {
                                Ink::DANGER.alpha(0.12)
                            } else {
                                colors.primary.alpha(0.0)
                            })
                            .hover(move |button| button.bg(Ink::DANGER.alpha(0.13)))
                            .text_size(px(8.5))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(Ink::DANGER)
                            .child(if armed { "Confirm" } else { "Discard" })
                            .on_click(move |_, _, cx| {
                                discard_inspector.update(cx, |inspector, cx| {
                                    if inspector.armed_hunk == Some(fingerprint) {
                                        inspector.run_review_action(
                                            ReviewAction::Patch {
                                                patch: discard_patch.clone(),
                                                mutation: PatchMutation::Discard,
                                            },
                                            cx,
                                        );
                                    } else {
                                        inspector.armed_hunk = Some(fingerprint);
                                        inspector.review_feedback = Some((
                                            false,
                                            "Click Confirm to discard this hunk".to_owned(),
                                        ));
                                        cx.notify();
                                    }
                                });
                                cx.stop_propagation();
                            }),
                    );
                }
            }
            DiffLayer::Staged => {
                let unstage_inspector = inspector.clone();
                actions = actions.child(
                    div()
                        .id(("unstage-diff-hunk", index))
                        .h_full()
                        .px(px(5.0))
                        .flex()
                        .items_center()
                        .rounded(px(Radius::CHIP))
                        .cursor_pointer()
                        .hover(move |button| button.bg(colors.primary.alpha(0.09)))
                        .text_size(px(8.5))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(colors.tertiary)
                        .child("Unstage")
                        .on_click(move |_, _, cx| {
                            unstage_inspector.update(cx, |inspector, cx| {
                                inspector.run_review_action(
                                    ReviewAction::Patch {
                                        patch: patch.clone(),
                                        mutation: PatchMutation::Unstage,
                                    },
                                    cx,
                                );
                            });
                            cx.stop_propagation();
                        }),
                );
            }
            DiffLayer::Branch => {}
        }
    }

    let has_actions = file.is_some() || hunk.is_some();
    div()
        .id(index)
        .relative()
        .h(px(DIFF_ROW_HEIGHT))
        .min_w(px(content_width))
        .w_full()
        .flex()
        .items_center()
        .bg(background)
        .when(row.kind == DiffRowKind::File, |line| {
            line.border_t_1()
                .border_color(colors.primary.alpha(0.08))
                .cursor_pointer()
                .hover(move |line| line.bg(colors.primary.alpha(0.07)))
                .on_click(move |_, _, cx| {
                    open_inspector.update(cx, |inspector, cx| {
                        inspector.open_file_reference(cwd.clone(), reference.clone(), cx);
                    });
                    cx.stop_propagation();
                })
        })
        .child(
            div()
                .w(px(GUTTER_WIDTH))
                .h_full()
                .flex_none()
                .pr(px(7.0))
                .flex()
                .items_center()
                .justify_end()
                .gap(px(7.0))
                .border_r_1()
                .border_color(colors.primary.alpha(0.055))
                .font_family(crate::fonts::mono_family())
                .text_size(px(10.5))
                .text_color(colors.primary.alpha(0.25))
                .child(line_number(row.old_line))
                .child(line_number(row.new_line)),
        )
        .child(
            div()
                .h_full()
                .flex()
                .items_center()
                .pl(px(if row.kind == DiffRowKind::File {
                    10.0
                } else {
                    8.0
                }))
                .gap(px(6.0))
                .font_family(crate::fonts::mono_family())
                .text_size(px(11.5))
                .font_weight(if row.kind == DiffRowKind::File {
                    FontWeight::MEDIUM
                } else {
                    FontWeight::NORMAL
                })
                .text_color(foreground)
                .when(row.kind == DiffRowKind::File, |content| {
                    content.child(sf_symbol(
                        "chevron.left.forwardslash.chevron.right",
                        13.0,
                        colors.secondary,
                    ))
                })
                .child(text),
        )
        .when(has_actions, |line| line.child(actions))
        .into_any_element()
}
