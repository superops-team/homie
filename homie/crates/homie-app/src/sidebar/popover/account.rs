use super::*;

impl Sidebar {
    pub(crate) fn account_popover(
        &self,
        colors: SemanticColors,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let mut usage = div()
            .flex()
            .flex_col()
            .child(section_label("Usage", colors));
        if self.preview {
            usage = usage
                .child(usage_row("Session", "resets in 2h 14m", "$2.31", colors))
                .child(usage_row("Today", "1.8M tokens", "$4.82", colors))
                .child(usage_row("This month", "", "$86.40", colors));
        } else if let Some(snapshot) = self.usage {
            usage = usage
                .child(usage_row(
                    "Session",
                    snapshot
                        .session_remaining_seconds
                        .map(|seconds| format!("resets in {}", compact_duration(seconds)))
                        .as_deref()
                        .unwrap_or("idle"),
                    &snapshot
                        .session_cost
                        .map(UsageFormat::money)
                        .unwrap_or_else(|| "—".into()),
                    colors,
                ))
                .child(usage_row(
                    "Today",
                    &format!(
                        "{} tokens",
                        UsageFormat::tokens(snapshot.today().total_tokens())
                    ),
                    &UsageFormat::money(snapshot.today().cost),
                    colors,
                ))
                .child(usage_row(
                    "This month",
                    "",
                    &UsageFormat::money(snapshot.month().cost),
                    colors,
                ));
        } else {
            usage = usage.child(
                div()
                    .px(px(14.0))
                    .py(px(6.0))
                    .text_size(px(Typo::ROW.size))
                    .text_color(colors.tertiary)
                    .child("Measuring…"),
            );
        }
        let content = div()
            .flex()
            .flex_col()
            .child(usage)
            .child(div().mt(px(8.0)).h(px(1.0)).bg(colors.primary.alpha(0.06)))
            .child(section_label("Version", colors))
            .child(self.update_menu_row(colors, cx))
            .child(div().mt(px(8.0)).h(px(1.0)).bg(colors.primary.alpha(0.06)))
            .child(section_label("Remote", colors))
            .child(
                div()
                    .id("quick-add-remote-host")
                    .debug_selector(|| "quick-add-remote-host".into())
                    .mx(px(6.0))
                    .px(px(8.0))
                    .h(px(30.0))
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    .rounded(px(Radius::ROW))
                    .cursor_pointer()
                    .hover(move |element| element.bg(colors.primary.alpha(0.06)))
                    .text_size(px(Typo::ROW.size))
                    .text_color(colors.primary)
                    .child(sf_symbol("plus", 11.0, colors.secondary))
                    .child(div().flex_1().child("Add remote host"))
                    .child(
                        div()
                            .text_size(px(Typo::META.size))
                            .text_color(colors.tertiary)
                            .child("SSH"),
                    )
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.ui.popover = None;
                        cx.emit(SidebarEvent::AddRemoteHost);
                        cx.notify();
                    })),
            )
            .child(div().mt(px(8.0)).h(px(1.0)).bg(colors.primary.alpha(0.06)))
            .child(section_label("Account", colors))
            .child(
                div()
                    .id("account-active")
                    .mx(px(6.0))
                    .px(px(8.0))
                    .h(px(28.0))
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    .rounded(px(Radius::ROW))
                    .text_size(px(Typo::ROW.size))
                    .text_color(colors.primary)
                    .child(sf_symbol_weighted(
                        "checkmark",
                        10.0,
                        SymbolWeight::Semibold,
                        colors.secondary,
                    ))
                    .child(if self.preview {
                        "preview@homie.local"
                    } else {
                        "Local agents"
                    }),
            )
            .child(
                div()
                    .id("dismiss-account")
                    .mx(px(6.0))
                    .my(px(6.0))
                    .px(px(8.0))
                    .h(px(28.0))
                    .flex()
                    .items_center()
                    .rounded(px(Radius::ROW))
                    .cursor_pointer()
                    .hover(move |element| element.bg(colors.primary.alpha(0.06)))
                    .text_size(px(Typo::ROW.size))
                    .text_color(colors.secondary)
                    .child("Done")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.ui.popover = None;
                        cx.notify();
                    })),
            );
        self.popover_shell_above_footer(content, colors, window, cx)
    }
}
