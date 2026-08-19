//! Switcher filmstrip + preview panel rendering (Control-Tab surface).

use super::projection::{status_color, ui_agent_kind};
use super::*;

impl SessionSurfaces {
    pub(super) fn render_switcher(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let (state, sessions) = {
            let store = self.store.read().expect("session store lock poisoned");
            let state = store.switcher_state().clone();
            let sessions: Vec<_> = state
                .order()
                .iter()
                .filter_map(|id| store.sessions().get(id).cloned())
                .collect();
            (state, sessions)
        };
        let highlighted = sessions.get(state.index()).cloned();
        let colors = self.colors();

        let preview_content = highlighted.as_ref().map_or_else(
            || div().size_full().into_any_element(),
            |session| self.render_grid_or_logo(session, 56.0, 8.0, colors),
        );
        let footer = highlighted.as_ref().map(|session| {
            let kind = ui_agent_kind(session.effective_kind());
            let status = self.status_glyph(session, 22.0, colors, window, cx);
            let mut details = div().flex().flex_col().min_w_0().gap(px(2.0)).child(
                div()
                    .text_size(px(13.0))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(colors.primary)
                    .overflow_hidden()
                    .text_ellipsis()
                    .child(display_title(session)),
            );
            let folder = Path::new(&session.cwd)
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or(&session.cwd);
            let metadata = if let Some(branch) = session.git_branch.as_ref() {
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.0))
                    .child(sf_symbol("arrow.branch", 11.0, colors.secondary))
                    .child(branch.clone())
                    .child(folder.to_owned())
                    .into_any_element()
            } else {
                div().child(folder.to_owned()).into_any_element()
            };
            details = details.child(
                div()
                    .text_size(px(11.0))
                    .text_color(colors.secondary)
                    .overflow_hidden()
                    .text_ellipsis()
                    .child(metadata),
            );
            div()
                .flex()
                .items_center()
                .gap(px(10.0))
                .h(px(54.0))
                .px(px(14.0))
                .bg(colors.floating_surface())
                .child(AgentLogo::new(kind, 26.0, colors))
                .child(details)
                .child(div().flex_1())
                .child(status)
                .into_any_element()
        });

        let mut filmstrip = div()
            .id("switcher-filmstrip")
            .flex()
            .w(px(SWITCHER_PREVIEW_WIDTH))
            .gap(px(10.0))
            .p(px(6.0))
            .overflow_x_scroll();
        for (index, session) in sessions.iter().enumerate() {
            let active = index == state.index();
            filmstrip = filmstrip.child(
                div()
                    .id(("switcher-chip", index))
                    .flex()
                    .flex_none()
                    .items_center()
                    .gap(px(7.0))
                    .max_w(px(190.0))
                    .px(px(10.0))
                    .py(px(7.0))
                    .rounded(px(Radius::ROW))
                    .bg(if active {
                        Palette::CLAY.alpha(0.22)
                    } else {
                        colors.primary.alpha(0.06)
                    })
                    .border_1()
                    .border_color(if active {
                        Palette::CLAY
                    } else {
                        colors.primary.alpha(0.0)
                    })
                    .cursor_pointer()
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.store
                            .write()
                            .expect("session store lock poisoned")
                            .commit_switcher_index(index);
                        cx.notify();
                    }))
                    .child(AgentLogo::new(
                        ui_agent_kind(session.effective_kind()),
                        18.0,
                        colors,
                    ))
                    .child(
                        div()
                            .min_w_0()
                            .text_size(px(13.0))
                            .text_color(if active {
                                colors.primary
                            } else {
                                colors.secondary
                            })
                            .overflow_hidden()
                            .text_ellipsis()
                            .child(display_title(session)),
                    )
                    .child(
                        div()
                            .flex_none()
                            .size(px(5.0))
                            .rounded_full()
                            .bg(status_color(session, colors)),
                    )
                    .when(active, |chip| chip.shadow_sm())
                    .when(!active, |chip| chip.opacity(0.92)),
            );
        }

        let panel = div()
            .flex()
            .flex_col()
            .gap(px(14.0))
            .p(px(18.0))
            .rounded(px(22.0))
            .bg(colors.sidebar_surface())
            .border_1()
            .border_color(colors.primary.alpha(0.08))
            .shadow(vec![BoxShadow {
                color: rgba(0x00000080).into(),
                offset: point(px(0.0), px(18.0)),
                blur_radius: px(44.0),
                spread_radius: px(0.0),
                inset: false,
            }])
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .child(
                div()
                    .w(px(SWITCHER_PREVIEW_WIDTH))
                    .rounded(px(Radius::CARD))
                    .overflow_hidden()
                    .border_2()
                    .border_color(Palette::CLAY)
                    .child(
                        div()
                            .relative()
                            .w(px(SWITCHER_PREVIEW_WIDTH))
                            .h(px(SWITCHER_PREVIEW_HEIGHT))
                            .bg(colors.background)
                            .child(preview_content),
                    )
                    .children(footer),
            )
            .child(filmstrip);

        div()
            .id("switcher-scrim")
            .absolute()
            .inset_0()
            .flex()
            .items_center()
            .justify_center()
            .bg(rgba(0x0000001a))
            .occlude()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _, _, cx| {
                    this.store
                        .write()
                        .expect("session store lock poisoned")
                        .cancel_switcher();
                    cx.notify();
                    cx.stop_propagation();
                }),
            )
            .on_scroll_wheel(|_, _, cx| cx.stop_propagation())
            .child(panel)
            .into_any_element()
    }
}
