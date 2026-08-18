use super::*;

pub(crate) fn icon_button(
    id: &'static str,
    system_image: &'static str,
    hovering: bool,
    colors: SemanticColors,
    on_click: impl Fn(&gpui::ClickEvent, &mut Window, &mut App) + 'static,
    on_hover: impl Fn(&bool, &mut Window, &mut App) + 'static,
) -> AnyElement {
    div()
        .id(id)
        .size(px(Metrics::TOOLBAR_CONTROL_SIZE))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(Radius::BADGE))
        .bg(Fill::hover(colors, hovering))
        .cursor_pointer()
        .text_size(px(15.0))
        .text_color(colors.secondary)
        .on_click(on_click)
        .on_hover(on_hover)
        .child(sf_symbol(system_image, 15.0, colors.secondary))
        .into_any_element()
}

/// Title `display_title` gives a placeholder-named session that has exited.
/// The "Ended" chip stands down when the title already says it.
pub(crate) const ENDED_TITLE: &str = "Ended";

/// One leading column per ancestor level. A column is drawn full height while
/// that ancestor still has siblings below, and stops halfway on the last child
/// so a subtree visibly closes instead of trailing a rail into the next row.
pub(crate) fn indent_rails(
    row: &crate::store::SidebarRow,
    colors: SemanticColors,
) -> Vec<AnyElement> {
    (0..row.depth)
        .map(|column| {
            let continues = row.rails & (1u32 << column.min(31)) != 0;
            let last_column = column + 1 == row.depth;
            div()
                .w(px(Space::INDENT))
                .h(px(Metrics::ROW_HEIGHT))
                .flex_none()
                .flex()
                .justify_center()
                .child(
                    div()
                        .w(px(1.0))
                        // A rail that neither continues nor elbows into this
                        // row has no business being drawn at all.
                        .h(px(if continues {
                            Metrics::ROW_HEIGHT
                        } else if last_column {
                            Metrics::ROW_HEIGHT / 2.0
                        } else {
                            0.0
                        }))
                        .bg(colors.primary.alpha(0.10)),
                )
                .into_any_element()
        })
        .collect()
}

pub(crate) fn pin_mark(colors: SemanticColors) -> AnyElement {
    div()
        .flex_none()
        .flex()
        .items_center()
        .child(sf_symbol("pin.fill", 9.0, colors.tertiary))
        .into_any_element()
}

/// The row's shared chip: one state, stated in the smallest space that still
/// reads. Every chip on a row is the same shape so they scan as one lane.
pub(crate) fn state_chip(
    label: impl Into<SharedString>,
    tint: Rgba,
    colors: SemanticColors,
) -> AnyElement {
    div()
        .flex_none()
        .px(px(5.0))
        .py(px(1.0))
        .rounded(px(Radius::CHIP))
        .bg(Fill::subtle(colors))
        .text_size(px(Typo::META.size))
        .font_weight(Typo::META.weight)
        .text_color(tint)
        .whitespace_nowrap()
        .child(label.into())
        .into_any_element()
}

/// A chip that has to outrank the rest of the lane. Same geometry as
/// [`state_chip`] so the row still scans as one lane, tinted so it does not.
pub(crate) fn alert_chip(label: impl Into<SharedString>) -> AnyElement {
    div()
        .flex_none()
        .px(px(5.0))
        .py(px(1.0))
        .rounded(px(Radius::CHIP))
        .bg(Ink::DANGER.alpha(0.12))
        .text_size(px(Typo::META.size))
        .font_weight(Typo::META.weight)
        .text_color(Ink::DANGER)
        .whitespace_nowrap()
        .child(label.into())
        .into_any_element()
}

pub(crate) fn project_badge(colors: SemanticColors) -> AnyElement {
    div()
        .flex_none()
        .size(px(18.0))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(Radius::CHIP))
        .bg(colors.primary.alpha(0.08))
        .text_size(px(9.0))
        .text_color(colors.secondary)
        .child(sf_symbol("folder.fill", 9.0, colors.secondary))
        .into_any_element()
}

pub(crate) fn menu_row(
    label: impl Into<SharedString>,
    colors: SemanticColors,
    on_click: impl Fn(&gpui::ClickEvent, &mut Window, &mut App) + 'static,
) -> AnyElement {
    let label = label.into();
    div()
        .id(label.clone())
        .px(px(8.0))
        .h(px(28.0))
        .flex()
        .items_center()
        .rounded(px(Radius::ROW))
        .cursor_pointer()
        .hover(move |element| element.bg(colors.primary.alpha(0.06)))
        .text_size(px(Typo::ROW.size))
        .text_color(colors.primary)
        .child(label)
        .on_click(on_click)
        .into_any_element()
}

pub(crate) fn directory_row(
    symbol: &'static str,
    label: String,
    colors: SemanticColors,
    on_click: impl Fn(&gpui::ClickEvent, &mut Window, &mut App) + 'static,
) -> AnyElement {
    let row_id = format!("directory-row-{label}");
    div()
        .id(row_id)
        .px(px(10.0))
        .h(px(30.0))
        .flex()
        .items_center()
        .gap(px(8.0))
        .rounded(px(Radius::ROW))
        .cursor_pointer()
        .hover(move |row| row.bg(colors.primary.alpha(0.06)))
        .child(sf_symbol(symbol, 11.0, colors.secondary))
        .child(
            div()
                .min_w(px(0.0))
                .flex_1()
                .whitespace_nowrap()
                .overflow_hidden()
                .text_ellipsis()
                .text_size(px(Typo::ROW.size))
                .text_color(colors.primary)
                .child(label),
        )
        .on_click(on_click)
        .into_any_element()
}

pub(crate) fn menu_divider(colors: SemanticColors) -> AnyElement {
    div()
        .my(px(3.0))
        .child(HairlineDivider::horizontal(colors))
        .into_any_element()
}

pub(crate) fn copy_session_id_row(
    id: SessionId,
    colors: SemanticColors,
    cx: &mut Context<Sidebar>,
) -> AnyElement {
    menu_row(
        "Copy Session ID",
        colors,
        cx.listener(move |this, _, _, cx| {
            cx.write_to_clipboard(gpui::ClipboardItem::new_string(id.0.clone()));
            this.ui.popover = None;
            cx.notify();
        }),
    )
}

pub(crate) fn section_label(label: &'static str, colors: SemanticColors) -> AnyElement {
    div()
        .px(px(14.0))
        .pt(px(10.0))
        .pb(px(3.0))
        .text_size(px(Typo::SECTION_HEADER.size))
        .font_weight(Typo::SECTION_HEADER.weight)
        .text_color(colors.tertiary)
        .child(label)
        .into_any_element()
}

pub(crate) fn usage_row(
    label: &str,
    detail: &str,
    value: &str,
    colors: SemanticColors,
) -> AnyElement {
    div()
        .px(px(14.0))
        .h(px(24.0))
        .flex()
        .items_center()
        .gap(px(8.0))
        .text_size(px(Typo::ROW.size))
        .text_color(colors.text(homie_ui::TextTone::Label))
        .child(label.to_owned())
        .child(div().flex_1())
        .when(!detail.is_empty(), |row| {
            row.child(
                div()
                    .text_size(px(Typo::META.size))
                    .text_color(colors.tertiary)
                    .child(detail.to_owned()),
            )
        })
        .child(
            div()
                .font_family(crate::fonts::mono_family())
                .text_size(px(Typo::META_MONO.size))
                .text_color(colors.secondary)
                .child(value.to_owned()),
        )
        .into_any_element()
}

pub(crate) fn hover_detail(
    icon: &str,
    text: &str,
    mono: bool,
    colors: SemanticColors,
) -> AnyElement {
    div()
        .flex()
        .items_center()
        .gap(px(7.0))
        .child(
            div()
                .w(px(13.0))
                .flex()
                .items_center()
                .justify_center()
                .child(sf_symbol(icon, 10.0, colors.secondary)),
        )
        .child(
            div()
                .min_w(px(0.0))
                .flex_1()
                .whitespace_nowrap()
                .overflow_hidden()
                .text_ellipsis()
                .when(mono, |text| text.font_family(crate::fonts::mono_family()))
                .text_size(px(Typo::META.size))
                .text_color(colors.primary.alpha(0.82))
                .child(text.to_owned()),
        )
        .into_any_element()
}
