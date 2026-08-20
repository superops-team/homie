//! GPUI render facade for the terminal pane.
//!
//! Owns the render tree (the `impl Render` block, the `render_*` methods, and
//! the free render helpers). Effect dispatch stays in `mod.rs`.

use super::*;

mod buttons;
mod chrome;
mod find_bar;
mod grid;
mod status;

impl Render for TerminalPane {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.reconcile_residency();
        let (theme, colors, sidebar_colors, font_size) = {
            let store = self
                .runtime
                .store
                .read()
                .expect("session store lock poisoned");
            let theme_id = &store.preferences().terminal_theme;
            (
                crate::app_theme::terminal_theme(theme_id),
                crate::app_theme::colors(theme_id),
                crate::app_theme::sidebar_colors(theme_id),
                store.preferences().terminal_font_size,
            )
        };
        self.sync_status_glyphs(colors, window, cx);
        self.update_selected_geometry(window, cx);

        let selected = self.selected_session();

        let content = if let Some(session) = selected {
            let mut pane = div()
                .relative()
                .flex()
                .flex_col()
                .flex_1()
                .h_full()
                .overflow_hidden()
                .border_l_1()
                .border_color(sidebar_colors.primary.alpha(0.08))
                .bg(sidebar_colors.sidebar_surface())
                .child(self.render_header(&session, sidebar_colors, cx));
            let terminal_surface = div()
                .relative()
                .min_h(px(0.0))
                .flex_1()
                .flex()
                .flex_col()
                .rounded_tl(px(Radius::CARD))
                .rounded_tr(px(Radius::CARD))
                .overflow_hidden()
                .bg(theme.background)
                .child(self.render_grid_and_overlays(&session, theme, font_size, window, cx));
            pane = pane.child(terminal_surface);
            if let Some(find) = self.render_find_bar(&session, colors, cx) {
                pane = pane.child(find);
            }
            pane.into_any_element()
        } else {
            let show_sidebar = matches!(self.session_source, SessionSource::FollowSelection)
                && !self.sidebar_visible;
            let sidebar_reveal =
                show_sidebar.then(|| self.render_sidebar_reveal_control(sidebar_colors, cx));
            div()
                .flex_1()
                .h_full()
                .flex()
                .flex_col()
                .bg(theme.background)
                .when_some(sidebar_reveal, |pane, control| {
                    pane.child(
                        div()
                            .h(px(Metrics::TITLE_BAR))
                            .flex_none()
                            .px(px(Metrics::TOOLBAR_EDGE_INSET))
                            .flex()
                            .items_center()
                            .bg(sidebar_colors.sidebar_surface())
                            .child(control),
                    )
                })
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_size(px(13.0))
                        .text_color(sidebar_colors.tertiary)
                        .child("Start a terminal from the sidebar"),
                )
                .into_any_element()
        };

        let root_id = match &self.session_source {
            SessionSource::FollowSelection => SharedString::from("homie-terminal-root"),
            SessionSource::Fixed(id) => SharedString::from(format!("homie-terminal-root-{}", id.0)),
        };
        div()
            .id(root_id)
            .track_focus(&self.focus)
            .flex()
            .size_full()
            .text_color(colors.primary)
            .on_action(cx.listener(Self::open_find))
            .on_action(cx.listener(Self::find_next))
            .on_action(cx.listener(Self::find_previous))
            .on_action(cx.listener(Self::close_find))
            .on_action(cx.listener(Self::zoom_in))
            .on_action(cx.listener(Self::zoom_out))
            .on_action(cx.listener(Self::reset_zoom))
            .on_action(cx.listener(Self::paste))
            .on_action(cx.listener(Self::copy_selection))
            .on_key_down(cx.listener(Self::handle_key_down))
            .on_key_up(cx.listener(Self::handle_key_up))
            .on_modifiers_changed(cx.listener(Self::handle_modifiers_changed))
            .child(content)
    }
}
