use super::*;

impl Sidebar {
    pub(crate) fn new_agent_row(
        &self,
        colors: SemanticColors,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let hovering = self.ui.hovered_control == Some("new-agent");
        let (agent, location) = {
            let store = self.store.read().expect("session store lock poisoned");
            let agent = store.preferences().default_agent.display_name().to_owned();
            let location = store
                .default_spawn_host()
                .map_or_else(|| "This Mac".to_owned(), |id| store.host_display_name(&id));
            (agent, location)
        };
        div()
            .id("new-agent")
            .mx(px(Space::INSET))
            .mb(px(4.0))
            .px(px(Space::ROW_H))
            .h(px(44.0))
            .flex()
            .items_center()
            .gap(px(8.0))
            .rounded(px(Radius::ROW))
            .bg(Fill::hover(colors, hovering))
            .cursor_pointer()
            .text_size(px(Typo::ROW.size))
            .text_color(colors.text(homie_ui::TextTone::Label))
            .on_hover(cx.listener(|this, hovered: &bool, _, cx| {
                this.ui.hovered_control = hovered.then_some("new-agent");
                cx.notify();
            }))
            .on_click(cx.listener(|this, _, _, cx| {
                this.commit_rename();
                this.open_new_agent_popover(None, cx);
            }))
            .child(
                div()
                    .w(px(16.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(sf_symbol("square.and.pencil", 13.0, colors.secondary)),
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
                            .whitespace_nowrap()
                            .overflow_hidden()
                            .text_ellipsis()
                            .child("New Agent"),
                    )
                    .child(
                        div()
                            .whitespace_nowrap()
                            .overflow_hidden()
                            .text_ellipsis()
                            .text_size(px(Typo::META.size))
                            .text_color(colors.tertiary)
                            .child(format!("{agent} · {location}")),
                    ),
            )
            .child(
                div()
                    .text_size(px(Typo::META.size))
                    .text_color(colors.tertiary)
                    .child("⌘T"),
            )
            .into_any_element()
    }

    pub(crate) fn top_bar(&self, colors: SemanticColors, cx: &mut Context<Self>) -> AnyElement {
        let settings_hover = self.ui.hovered_control == Some("settings");
        let toggle_hover = self.ui.hovered_control == Some("sidebar-toggle");
        div()
            .h(px(Metrics::TITLE_BAR))
            .flex_none()
            .flex()
            .items_center()
            .justify_end()
            .pr(px(Metrics::TOOLBAR_EDGE_INSET))
            .gap(px(Metrics::TOOLBAR_COMPACT_GAP))
            .child(icon_button(
                "settings",
                "gearshape",
                settings_hover,
                colors,
                cx.listener(|this, _, _, cx| {
                    this.ui.popover = None;
                    cx.emit(SidebarEvent::OpenSettings);
                }),
                cx.listener(|this, hovered: &bool, _, cx| {
                    this.ui.hovered_control = hovered.then_some("settings");
                    cx.notify();
                }),
            ))
            .child(icon_button(
                "sidebar-toggle",
                "sidebar.left",
                toggle_hover,
                colors,
                cx.listener(|this, _, _, cx| this.toggle(cx)),
                cx.listener(|this, hovered: &bool, _, cx| {
                    this.ui.hovered_control = hovered.then_some("sidebar-toggle");
                    cx.notify();
                }),
            ))
            .into_any_element()
    }

    pub(crate) fn empty_state(&self, colors: SemanticColors, cx: &mut Context<Self>) -> AnyElement {
        div()
            .flex_1()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap(px(12.0))
            .child(AgentLogo::new(AgentKind::ClaudeCode, 44.0, colors).badged(false))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(px(3.0))
                    .child(
                        div()
                            .text_size(px(Typo::ROW_EMPHASIZED.size))
                            .font_weight(Typo::ROW_EMPHASIZED.weight)
                            .text_color(colors.secondary)
                            .child("Bring up your first agent"),
                    )
                    .child(
                        div()
                            .text_size(px(Typo::META.size))
                            .text_color(colors.tertiary)
                            .child("⌘T"),
                    ),
            )
            .child(
                div()
                    .id("empty-new-agent")
                    .px(px(10.0))
                    .h(px(28.0))
                    .flex()
                    .items_center()
                    .rounded(px(Radius::ROW))
                    .text_size(px(Typo::ROW.size))
                    .text_color(colors.secondary)
                    .cursor_pointer()
                    .hover(move |element| element.bg(colors.primary.alpha(0.06)))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.open_new_agent_popover(None, cx);
                    }))
                    .gap(px(7.0))
                    .child(sf_symbol("square.and.pencil", 13.0, colors.secondary))
                    .child("New Agent"),
            )
            .into_any_element()
    }
}
