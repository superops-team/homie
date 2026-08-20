use super::*;

pub(crate) fn find_icon_button(
    id: &'static str,
    system_image: &'static str,
    cx: &mut Context<TerminalPane>,
    handler: impl Fn(&mut TerminalPane, &mut Window, &mut Context<TerminalPane>) + 'static,
) -> AnyElement {
    div()
        .id(id)
        .size(px(20.0))
        .rounded(px(4.0))
        .flex()
        .items_center()
        .justify_center()
        .text_size(px(11.0))
        .text_color(rgba(0xffffff99))
        .hover(|style| style.bg(rgba(0xffffff0f)))
        .cursor_pointer()
        .child(sf_symbol_weighted(
            system_image,
            11.0,
            SymbolWeight::Semibold,
            rgba(0xffffff99),
        ))
        .on_click(cx.listener(move |this, _, window, cx| handler(this, window, cx)))
        .into_any_element()
}

pub(crate) fn primary_button(
    id: &'static str,
    label: &'static str,
    cx: &mut Context<TerminalPane>,
    handler: impl Fn(&mut TerminalPane, &mut Context<TerminalPane>) + 'static,
) -> AnyElement {
    div()
        .id(id)
        .mt(px(2.0))
        .rounded(px(7.0))
        .px(px(14.0))
        .py(px(7.0))
        .bg(rgba(0xffffffeb))
        .text_size(px(13.0))
        .font_weight(Typo::ROW_EMPHASIZED.weight)
        .text_color(rgba(0x121318ff))
        .hover(|style| style.bg(rgba(0xffffffff)))
        .cursor_pointer()
        .child(label)
        .on_click(cx.listener(move |this, _, _, cx| handler(this, cx)))
        .into_any_element()
}

pub(crate) fn centered_message(icon: &str, message: &str) -> gpui::Div {
    div()
        .flex_1()
        .size_full()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap(px(12.0))
        .when(!icon.is_empty(), |content| {
            content.child(
                div()
                    .text_size(px(30.0))
                    .text_color(rgba(0xffffff4d))
                    .child(icon.to_owned()),
            )
        })
        .when(!message.is_empty(), |content| {
            content.child(
                div()
                    .text_size(px(13.0))
                    .text_color(rgba(0xffffff99))
                    .child(message.to_owned()),
            )
        })
}

pub(crate) fn centered_symbol_message(system_image: &str, size: f32, message: &str) -> gpui::Div {
    div()
        .flex_1()
        .size_full()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap(px(12.0))
        .child(sf_symbol_weighted(
            system_image,
            size,
            SymbolWeight::Regular,
            rgba(0xffffff4d),
        ))
        .when(!message.is_empty(), |content| {
            content.child(
                div()
                    .text_size(px(13.0))
                    .text_color(rgba(0xffffff99))
                    .child(message.to_owned()),
            )
        })
}
