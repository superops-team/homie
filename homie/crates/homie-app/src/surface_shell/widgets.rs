use super::*;

pub(super) fn surface_button(
    label: impl Into<SharedString>,
    id: impl Into<SharedString>,
    colors: SemanticColors,
    cx: &mut Context<UtilitySurfaces>,
    handler: impl Fn(&mut UtilitySurfaces, &mut Context<UtilitySurfaces>) + 'static,
) -> impl IntoElement {
    div()
        .id(id.into())
        .h(px(26.0))
        .px(px(9.0))
        .rounded(px(Radius::BADGE))
        .border_1()
        .border_color(colors.primary.alpha(0.10))
        .bg(colors.primary.alpha(0.04))
        .flex()
        .items_center()
        .text_size(px(11.0))
        .cursor_pointer()
        .hover(move |style| style.bg(colors.primary.alpha(0.09)))
        .on_click(cx.listener(move |this, _, _, cx| handler(this, cx)))
        .child(label.into())
}

pub(super) fn settings_primary_button(
    label: &'static str,
    id: &'static str,
    icon: Option<&'static str>,
    cx: &mut Context<UtilitySurfaces>,
    handler: impl Fn(&mut UtilitySurfaces, &mut Window, &mut Context<UtilitySurfaces>) + 'static,
) -> impl IntoElement {
    div()
        .id(id)
        .h(px(28.0))
        .px(px(10.0))
        .rounded(px(Radius::BADGE))
        .bg(Palette::CLAY.alpha(0.90))
        .text_color(rgba(0x190f0bff))
        .flex()
        .items_center()
        .gap(px(6.0))
        .text_size(px(11.0))
        .font_weight(FontWeight::SEMIBOLD)
        .cursor_pointer()
        .hover(|style| style.bg(Palette::CLAY))
        .on_click(cx.listener(move |this, _, window, cx| handler(this, window, cx)))
        .when_some(icon, |button, icon| {
            button.child(sf_symbol(icon, 10.0, rgba(0x190f0bff)))
        })
        .child(label)
}

pub(super) fn settings_danger_button(
    label: &'static str,
    id: &'static str,
    cx: &mut Context<UtilitySurfaces>,
    handler: impl Fn(&mut UtilitySurfaces, &mut Context<UtilitySurfaces>) + 'static,
) -> impl IntoElement {
    div()
        .id(id)
        .h(px(26.0))
        .px(px(9.0))
        .rounded(px(Radius::BADGE))
        .bg(Ink::DANGER.alpha(0.11))
        .text_color(Ink::DANGER)
        .flex()
        .items_center()
        .text_size(px(11.0))
        .cursor_pointer()
        .hover(|style| style.bg(Ink::DANGER.alpha(0.18)))
        .on_click(cx.listener(move |this, _, _, cx| handler(this, cx)))
        .child(label)
}

pub(super) fn host_field_value(
    editor: &QueryEditor,
    placeholder: &'static str,
    active: bool,
    field: HostFormField,
    colors: SemanticColors,
) -> AnyElement {
    let debug_name = field.debug_name();
    let content = if active {
        div()
            .debug_selector(|| "host-field-caret".into())
            .child(query_label(editor))
            .into_any_element()
    } else if editor.is_empty() {
        div()
            .debug_selector(move || format!("HOST_FIELD_PLACEHOLDER_{debug_name}"))
            .text_color(colors.tertiary)
            .child(placeholder)
            .into_any_element()
    } else {
        div().child(editor.text().to_owned()).into_any_element()
    };
    div()
        .min_w(px(0.0))
        .w_full()
        .overflow_hidden()
        .whitespace_nowrap()
        .text_ellipsis()
        .child(content)
        .into_any_element()
}

pub(super) fn text_offset_for_x(
    text: &str,
    x: Pixels,
    window: &Window,
    colors: SemanticColors,
) -> usize {
    if text.is_empty() {
        return 0;
    }
    let run = TextRun {
        len: text.len(),
        font: font(crate::fonts::mono_family()),
        color: colors.primary.into(),
        ..TextRun::default()
    };
    window
        .text_system()
        .shape_line(text.to_owned().into(), px(11.0), &[run], None)
        .closest_index_for_x(x)
}

pub(super) fn danger_button(
    label: &'static str,
    cx: &mut Context<UtilitySurfaces>,
    handler: impl Fn(&mut UtilitySurfaces, &mut Context<UtilitySurfaces>) + 'static,
) -> impl IntoElement {
    div()
        .id("confirm-cleanup")
        .h(px(26.0))
        .px(px(9.0))
        .rounded(px(Radius::BADGE))
        .bg(Ink::DANGER.alpha(0.16))
        .text_color(Ink::DANGER)
        .flex()
        .items_center()
        .text_size(px(11.0))
        .cursor_pointer()
        .on_click(cx.listener(move |this, _, _, cx| handler(this, cx)))
        .child(label)
}

pub(super) fn toggle_row(
    label: &'static str,
    detail: &'static str,
    enabled: bool,
    id: &'static str,
    colors: SemanticColors,
    cx: &mut Context<UtilitySurfaces>,
    handler: impl Fn(&mut UtilitySurfaces, &mut Context<UtilitySurfaces>) + 'static,
) -> impl IntoElement {
    div()
        .id(id)
        .min_h(px(SETTINGS_ROW_HEIGHT))
        .px(px(12.0))
        .flex()
        .items_center()
        .justify_between()
        .gap(px(12.0))
        .cursor_pointer()
        .hover(move |style| style.bg(colors.primary.alpha(0.025)))
        .on_click(cx.listener(move |this, _, _, cx| handler(this, cx)))
        .child(setting_text_stack(label.into(), detail.into(), colors))
        .child(
            div()
                .flex_none()
                .w(px(30.0))
                .h(px(18.0))
                .p(px(2.0))
                .rounded(px(9.0))
                .bg(if enabled {
                    Ink::FRESH.alpha(0.72)
                } else {
                    colors.primary.alpha(0.14)
                })
                .flex()
                .justify_end()
                .when(!enabled, |toggle| toggle.justify_start())
                .child(div().size(px(14.0)).rounded(px(7.0)).bg(colors.primary)),
        )
}

pub(super) fn setting_section(
    title: &'static str,
    content: impl IntoElement,
    colors: SemanticColors,
) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap(px(6.0))
        .child(
            div()
                .px(px(1.0))
                .text_size(px(Typo::SECTION_HEADER.size))
                .font_weight(Typo::SECTION_HEADER.weight)
                .text_color(colors.tertiary)
                .child(title),
        )
        .child(
            div()
                .rounded(px(Radius::ROW))
                .border_1()
                .border_color(colors.primary.alpha(0.065))
                .bg(colors.primary.alpha(0.02))
                .overflow_hidden()
                .child(content),
        )
}

pub(super) fn setting_row(
    label: impl Into<SharedString>,
    detail: impl Into<SharedString>,
    control: impl IntoElement,
    colors: SemanticColors,
) -> impl IntoElement {
    let label = label.into();
    let detail = detail.into();
    div()
        .debug_selector(|| "settings-row".into())
        .min_h(px(SETTINGS_ROW_HEIGHT))
        .px(px(12.0))
        .flex()
        .items_center()
        .justify_between()
        .gap(px(12.0))
        .child(setting_text_stack(label, detail, colors))
        .child(control)
}

pub(super) fn setting_text_stack(
    label: SharedString,
    detail: SharedString,
    colors: SemanticColors,
) -> AnyElement {
    div()
        .flex_1()
        .min_w(px(0.0))
        .flex()
        .flex_col()
        .gap(px(2.0))
        .child(
            div()
                .min_w(px(0.0))
                .w_full()
                .whitespace_normal()
                .text_size(px(Typo::ROW_EMPHASIZED.size))
                .font_weight(Typo::ROW_EMPHASIZED.weight)
                .text_color(colors.primary)
                .child(wrappable_setting_copy(label)),
        )
        .child(
            div()
                .debug_selector(|| "settings-row-detail".into())
                .min_w(px(0.0))
                .w_full()
                .whitespace_normal()
                .text_size(px(Typo::META.size))
                .line_height(px(14.0))
                .text_color(colors.tertiary)
                .child(wrappable_setting_copy(detail)),
        )
        .into_any_element()
}

pub(super) fn wrappable_setting_copy(text: SharedString) -> SharedString {
    let mut output = String::with_capacity(text.len());
    let mut uninterrupted = 0usize;
    for character in text.chars() {
        output.push(character);
        if character.is_whitespace() {
            uninterrupted = 0;
            continue;
        }
        uninterrupted += 1;
        if matches!(
            character,
            '/' | '\\' | '.' | ':' | '-' | '_' | '@' | '?' | '&' | '='
        ) || uninterrupted >= 24
        {
            output.push('\u{200b}');
            uninterrupted = 0;
        }
    }
    output.into()
}

pub(super) fn settings_note(
    icon: &'static str,
    title: Option<&'static str>,
    body: &'static str,
    icon_color: Rgba,
    border_color: Rgba,
    background: Rgba,
    colors: SemanticColors,
) -> AnyElement {
    div()
        .p(px(12.0))
        .rounded(px(Radius::ROW))
        .border_1()
        .border_color(border_color)
        .bg(background)
        .flex()
        .items_start()
        .gap(px(10.0))
        .child(sf_symbol(icon, 15.0, icon_color))
        .child(
            div()
                .flex_1()
                .min_w(px(0.0))
                .flex()
                .flex_col()
                .gap(px(4.0))
                .when_some(title, |copy, title| {
                    copy.child(
                        div()
                            .min_w(px(0.0))
                            .w_full()
                            .whitespace_normal()
                            .text_size(px(Typo::TITLE.size))
                            .font_weight(Typo::TITLE.weight)
                            .child(wrappable_setting_copy(title.into())),
                    )
                })
                .child(
                    div()
                        .debug_selector(|| "SETTINGS_NOTE_COPY".into())
                        .min_w(px(0.0))
                        .w_full()
                        .whitespace_normal()
                        .text_size(px(11.0))
                        .line_height(px(17.0))
                        .text_color(colors.secondary)
                        .child(wrappable_setting_copy(body.into())),
                ),
        )
        .into_any_element()
}

pub(super) fn settings_page(
    title: &'static str,
    content: impl IntoElement,
    colors: SemanticColors,
) -> impl IntoElement {
    div()
        .w_full()
        .px(px(20.0))
        .pt(px(13.0))
        .pb(px(22.0))
        .flex()
        .flex_col()
        .gap(px(16.0))
        .child(
            div()
                .pr(px(34.0))
                .h(px(22.0))
                .flex()
                .items_center()
                .text_size(px(Typo::DISPLAY_TITLE.size))
                .font_weight(Typo::DISPLAY_TITLE.weight)
                .text_color(colors.primary)
                .child(title),
        )
        .child(content)
}

pub(super) fn setting_divider(colors: SemanticColors) -> impl IntoElement {
    div()
        .h(px(1.0))
        .mx(px(12.0))
        .bg(colors.primary.alpha(0.055))
}

pub(super) fn settings_select_button(
    label: impl Into<SharedString>,
    id: impl Into<SharedString>,
    open: bool,
    menu: SettingsMenu,
    colors: SemanticColors,
    cx: &mut Context<UtilitySurfaces>,
) -> impl IntoElement {
    div()
        .id(id.into())
        .h(px(28.0))
        .px(px(9.0))
        .rounded(px(Radius::BADGE))
        .border_1()
        .border_color(colors.primary.alpha(if open { 0.20 } else { 0.10 }))
        .bg(colors.primary.alpha(if open { 0.08 } else { 0.04 }))
        .flex()
        .items_center()
        .gap(px(8.0))
        .cursor_pointer()
        .hover(move |style| style.bg(colors.primary.alpha(0.075)))
        .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
        .on_click(cx.listener(move |this, _, _, cx| {
            this.settings_menu = if this.settings_menu == Some(menu) {
                None
            } else {
                Some(menu)
            };
            cx.notify();
        }))
        .child(
            div()
                .flex_1()
                .text_size(px(Typo::META.size))
                .font_weight(Typo::META.weight)
                .text_color(colors.primary)
                .child(label.into()),
        )
        .child(sf_symbol_weighted(
            if open { "chevron.up" } else { "chevron.down" },
            8.0,
            SymbolWeight::Semibold,
            colors.tertiary,
        ))
}

pub(super) fn settings_dropdown(
    content: impl IntoElement,
    width: f32,
    colors: SemanticColors,
) -> impl IntoElement {
    deferred(
        div()
            .absolute()
            .top(px(32.0))
            .right_0()
            .w(px(width))
            .occlude()
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .child(FloatingSurface::new(colors, content)),
    )
    .with_priority(5)
}

pub(super) fn settings_choice_row(
    id: impl Into<SharedString>,
    label: impl Into<SharedString>,
    selected: bool,
    colors: SemanticColors,
    cx: &mut Context<UtilitySurfaces>,
    handler: impl Fn(&mut UtilitySurfaces, &mut Context<UtilitySurfaces>) + 'static,
) -> impl IntoElement {
    div()
        .id(id.into())
        .h(px(Metrics::ROW_HEIGHT))
        .px(px(8.0))
        .rounded(px(Radius::ROW))
        .flex()
        .items_center()
        .gap(px(8.0))
        .bg(Fill::selected(colors, selected))
        .cursor_pointer()
        .hover(move |style| style.bg(colors.primary.alpha(0.08)))
        .on_click(cx.listener(move |this, _, _, cx| handler(this, cx)))
        .child(
            div()
                .flex_1()
                .text_size(px(Typo::ROW.size))
                .child(label.into()),
        )
        .when(selected, |row| {
            row.child(sf_symbol("checkmark", 10.0, colors.secondary))
        })
}

pub(super) fn theme_preview(theme: TermTheme, colors: SemanticColors) -> impl IntoElement {
    div()
        .p(px(10.0))
        .rounded(px(Radius::BADGE))
        .border_1()
        .border_color(colors.primary.alpha(0.10))
        .bg(theme.background)
        .flex()
        .flex_col()
        .gap(px(6.0))
        .font_family(crate::fonts::mono_family())
        .text_size(px(11.0))
        .child(
            div()
                .flex()
                .child(div().text_color(theme.ansi[2]).child("❯ "))
                .child(div().text_color(theme.foreground).child("cargo test"))
                .child(div().text_color(theme.cursor).child("█")),
        )
        .child(div().text_color(theme.ansi[8]).child("test result: ok"))
        .child(
            div().flex().gap(px(3.0)).children(
                theme
                    .ansi
                    .into_iter()
                    .map(|color| div().flex_1().h(px(10.0)).rounded(px(2.0)).bg(color)),
            ),
        )
}

pub(super) fn chip(label: String, colors: SemanticColors) -> impl IntoElement {
    div()
        .px(px(5.0))
        .py(px(2.0))
        .rounded(px(Radius::CHIP))
        .bg(colors.primary.alpha(0.09))
        .text_size(px(11.0))
        .text_color(colors.secondary)
        .child(label)
}

pub(super) fn colored_badge(label: &'static str, color: Rgba) -> impl IntoElement {
    div()
        .px(px(5.0))
        .py(px(2.0))
        .rounded(px(Radius::CHIP))
        .bg(color.alpha(0.12))
        .text_size(px(11.0))
        .text_color(color)
        .child(label)
}

pub(super) fn empty_label(label: &str, colors: SemanticColors) -> impl IntoElement {
    div()
        .h(px(42.0))
        .px(px(10.0))
        .flex()
        .items_center()
        .text_size(px(13.0))
        .text_color(colors.tertiary)
        .child(label.to_owned())
}
