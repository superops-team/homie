use super::*;

impl Sidebar {
    pub(crate) fn popover(
        &self,
        colors: SemanticColors,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> Option<AnyElement> {
        match self.ui.popover.clone()? {
            Popover::NewAgent { directory, host } => {
                Some(self.new_agent_popover(directory, host, colors, cx))
            }
            Popover::Account => Some(self.account_popover(colors, window, cx)),
            Popover::ProjectActions { id, position } => {
                Some(self.project_actions_popover(id, position, colors, cx))
            }
            Popover::SessionActions { id, position } => {
                Some(self.session_actions_popover(id, position, colors, cx))
            }
        }
    }

    pub(crate) fn popover_shell(
        &self,
        top: f32,
        child: impl IntoElement,
        colors: SemanticColors,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        self.popover_shell_at(
            point(px(12.0), px(top)),
            Anchor::TopLeft,
            244.0,
            child,
            colors,
            cx,
        )
    }

    pub(crate) fn popover_shell_above_footer(
        &self,
        child: impl IntoElement,
        colors: SemanticColors,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let footer_top = f32::from(window.viewport_size().height) - 44.0;
        self.popover_shell_at(
            point(px(12.0), px(footer_top)),
            Anchor::BottomLeft,
            244.0,
            child,
            colors,
            cx,
        )
    }

    pub(crate) fn popover_shell_at(
        &self,
        position: Point<Pixels>,
        anchor: Anchor,
        width: f32,
        child: impl IntoElement,
        colors: SemanticColors,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        div()
            .absolute()
            .inset_0()
            .occlude()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _, _, cx| {
                    this.ui.popover = None;
                    cx.notify();
                }),
            )
            .child(
                deferred(
                    anchored()
                        .position(position)
                        .anchor(anchor)
                        .snap_to_window_with_margin(px(8.0))
                        .child(
                            div()
                                .w(px(width))
                                .occlude()
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(|_, _, _, cx| cx.stop_propagation()),
                                )
                                .on_mouse_down_out(cx.listener(|this, _, _, cx| {
                                    this.ui.popover = None;
                                    cx.notify();
                                }))
                                .child(FloatingSurface::new(
                                    colors,
                                    div()
                                        .rounded(px(Radius::PANEL))
                                        .overflow_hidden()
                                        .py(px(4.0))
                                        .child(child),
                                )),
                        ),
                )
                .with_priority(1),
            )
            .into_any_element()
    }
}
