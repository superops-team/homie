use super::*;

impl TerminalPane {
    pub(crate) fn render_sidebar_reveal_control(
        &self,
        colors: SemanticColors,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        div()
            .flex_none()
            .flex()
            .items_center()
            .gap(px(Metrics::TOOLBAR_ITEM_GAP))
            // The visible lights need more breathing room than their native
            // frames imply, so this is an intentional optical safe area.
            .child(div().w(px(Metrics::TOOLBAR_TRAFFIC_LIGHT_LANE)).flex_none())
            .child(
                div()
                    .id("show-sidebar")
                    .debug_selector(|| "show-sidebar".into())
                    .size(px(Metrics::TOOLBAR_CONTROL_SIZE))
                    .flex_none()
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(px(Radius::BADGE))
                    .cursor_pointer()
                    .hover(move |button| button.bg(Fill::subtle(colors)))
                    .child(sf_symbol("sidebar.left", 15.0, colors.secondary))
                    .on_click(cx.listener(|_, _, _, cx| {
                        cx.emit(TerminalPaneEvent::ToggleSidebar);
                        cx.stop_propagation();
                    })),
            )
            .into_any_element()
    }

    pub(crate) fn render_header(
        &self,
        session: &SessionRecord,
        colors: SemanticColors,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let glyph = self.glyphs.get(&session.id).cloned();
        let branch = session.git_branch.clone();
        let host = session.host.as_ref().map(|host| {
            self.runtime
                .store
                .read()
                .expect("session store lock poisoned")
                .host_display_name(host)
        });
        let kind = ui_agent_kind(session.effective_kind());
        let shell_controls = matches!(self.session_source, SessionSource::FollowSelection);
        let show_sidebar = shell_controls && !self.sidebar_visible;
        let sidebar_reveal = show_sidebar.then(|| self.render_sidebar_reveal_control(colors, cx));
        let inspector_open = self.inspector_open;
        div()
            .h(px(Metrics::TITLE_BAR))
            .flex_none()
            .px(px(Metrics::TOOLBAR_EDGE_INSET))
            .flex()
            .items_center()
            .justify_between()
            .bg(colors.sidebar_surface())
            .child(
                div()
                    .min_w(px(0.0))
                    .flex_1()
                    .flex()
                    .items_center()
                    .gap(px(Metrics::TOOLBAR_ITEM_GAP))
                    .overflow_hidden()
                    .when_some(sidebar_reveal, |title, control| title.child(control))
                    .child(sf_symbol("terminal", 15.0, colors.secondary))
                    .child(
                        div()
                            .min_w(px(0.0))
                            .flex_1()
                            .overflow_hidden()
                            .whitespace_nowrap()
                            .text_size(px(Typo::TITLE.size))
                            .font_weight(Typo::TITLE.weight)
                            .text_color(colors.primary)
                            .child(session.title.clone()),
                    )
                    .when_some(branch, |title, branch| {
                        title.child(
                            div()
                                .flex_none()
                                .flex()
                                .items_center()
                                .gap(px(Metrics::TOOLBAR_COMPACT_GAP))
                                .px(px(5.0))
                                .py(px(2.0))
                                .rounded(px(Radius::CHIP))
                                .bg(Fill::subtle(colors))
                                .overflow_hidden()
                                .whitespace_nowrap()
                                .text_size(px(Typo::META.size))
                                .text_color(colors.tertiary)
                                .child(sf_symbol("arrow.branch", 10.5, colors.tertiary))
                                .child(branch),
                        )
                    })
                    .when_some(host, |title, host| {
                        // Remote-host chip: the agent runs on that configured machine.
                        title.child(
                            div()
                                .flex_none()
                                .flex()
                                .items_center()
                                .gap(px(Metrics::TOOLBAR_COMPACT_GAP))
                                .rounded(px(Radius::CHIP))
                                .px(px(5.0))
                                .py(px(2.0))
                                .bg(Fill::subtle(colors))
                                .text_size(px(Typo::META.size))
                                .text_color(colors.secondary)
                                .child(sf_symbol("network", 9.0, colors.secondary))
                                .child(host),
                        )
                    }),
            )
            .child(
                div()
                    .flex_none()
                    .pl(px(Metrics::TOOLBAR_EDGE_INSET))
                    .flex()
                    .items_center()
                    .gap(px(Metrics::TOOLBAR_ITEM_GAP))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(Metrics::TOOLBAR_COMPACT_GAP))
                            .when_some(glyph, |identity, glyph| identity.child(glyph))
                            .child(
                                div()
                                    .text_size(px(Typo::META.size))
                                    .text_color(colors.tertiary)
                                    .child(kind.label()),
                            ),
                    )
                    .when(shell_controls, |trailing| {
                        trailing.child(
                            div()
                                .id("toggle-inspector")
                                .size(px(Metrics::TOOLBAR_CONTROL_SIZE))
                                .flex_none()
                                .flex()
                                .items_center()
                                .justify_center()
                                .rounded(px(Radius::BADGE))
                                .cursor_pointer()
                                .when(inspector_open, |button| button.bg(Fill::subtle(colors)))
                                .hover(move |button| button.bg(Fill::subtle(colors)))
                                .child(sf_symbol(
                                    "sidebar.right",
                                    15.0,
                                    if inspector_open {
                                        colors.primary
                                    } else {
                                        colors.secondary
                                    },
                                ))
                                .on_click(cx.listener(|_, _, _, cx| {
                                    cx.emit(TerminalPaneEvent::ToggleInspector);
                                    cx.stop_propagation();
                                })),
                        )
                    }),
            )
            .into_any_element()
    }
}
