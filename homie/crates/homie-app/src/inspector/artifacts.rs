use super::*;

pub(super) fn render_artifact_row(
    artifact: &SessionArtifact,
    colors: SemanticColors,
) -> AnyElement {
    let (symbol, kind_label) = match artifact.kind {
        ArtifactKind::PullRequest => ("arrow.triangle.pull", "Pull request"),
        ArtifactKind::LinearIssue => ("checklist", "Linear issue"),
        ArtifactKind::Preview => ("network", "Preview"),
        ArtifactKind::Link | ArtifactKind::Unknown => ("link", "Link"),
    };
    let title = artifact_title(artifact);
    let url = artifact.url.clone();
    div()
        .id(SharedString::from(format!(
            "inspector-artifact-{}",
            artifact.url
        )))
        .min_h(px(54.0))
        .px(px(11.0))
        .py(px(9.0))
        .flex()
        .items_center()
        .gap(px(10.0))
        .rounded(px(Radius::ROW))
        .bg(colors.primary.alpha(0.035))
        .border_1()
        .border_color(colors.primary.alpha(0.06))
        .cursor_pointer()
        .hover(move |row| row.bg(colors.primary.alpha(0.065)))
        .child(artifact_icon(symbol, colors))
        .child(
            div()
                .min_w(px(0.0))
                .flex_1()
                .flex()
                .flex_col()
                .gap(px(2.0))
                .child(
                    div()
                        .truncate()
                        .text_size(px(Typo::ROW_EMPHASIZED.size))
                        .font_weight(Typo::ROW_EMPHASIZED.weight)
                        .text_color(colors.primary)
                        .child(title),
                )
                .child(
                    div()
                        .truncate()
                        .text_size(px(Typo::META.size))
                        .text_color(colors.tertiary)
                        .child(kind_label),
                ),
        )
        .child(sf_symbol("arrow.up.right", 11.0, colors.tertiary))
        .on_click(move |_, _, cx| cx.open_url(&url))
        .into_any_element()
}

pub(super) fn artifact_icon(symbol: &'static str, colors: SemanticColors) -> AnyElement {
    div()
        .size(px(30.0))
        .flex_none()
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(Radius::BADGE))
        .bg(Fill::subtle(colors))
        .child(sf_symbol(symbol, 13.0, colors.secondary))
        .into_any_element()
}
