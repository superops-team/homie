use super::*;

impl WorkbenchInspector {
    fn scrollbar_metrics(&self) -> Option<ScrollbarMetrics> {
        let base = self.scroll.0.borrow().base_handle.clone();
        let bounds = base.bounds();
        let viewport_height = f32::from(bounds.size.height);
        let max_offset = f32::from(base.max_offset().y).max(0.0);
        if max_offset <= 0.0 || viewport_height <= SCROLLBAR_MIN_THUMB {
            return None;
        }

        let track_height = (viewport_height - SCROLLBAR_INSET * 2.0).max(0.0);
        let content_height = viewport_height + max_offset;
        let thumb_height = (track_height * viewport_height / content_height)
            .max(SCROLLBAR_MIN_THUMB)
            .min(track_height);
        let thumb_travel = (track_height - thumb_height).max(0.0);
        let progress = (-f32::from(base.offset().y) / max_offset).clamp(0.0, 1.0);

        Some(ScrollbarMetrics {
            track_top: f32::from(bounds.origin.y) + SCROLLBAR_INSET,
            track_height,
            thumb_height,
            thumb_top: thumb_travel * progress,
        })
    }

    pub(super) fn set_scrollbar_offset(&mut self, pointer_y: f32, cx: &mut Context<Self>) {
        let Some(metrics) = self.scrollbar_metrics() else {
            return;
        };
        let thumb_travel = (metrics.track_height - metrics.thumb_height).max(0.0);
        if thumb_travel <= 0.0 {
            return;
        }

        let thumb_top = (pointer_y - metrics.track_top - self.scrollbar_interaction.grab_offset)
            .clamp(0.0, thumb_travel);
        let base = self.scroll.0.borrow().base_handle.clone();
        let max_offset = f32::from(base.max_offset().y).max(0.0);
        let current = base.offset();
        base.set_offset(point(
            current.x,
            px(-(max_offset * thumb_top / thumb_travel)),
        ));
        cx.notify();
    }

    pub(super) fn finish_scrollbar_drag(&mut self, cx: &mut Context<Self>) {
        if self.scrollbar_interaction.dragging {
            self.scrollbar_interaction.dragging = false;
            cx.notify();
        }
    }

    pub(super) fn render_scrollbar(
        &self,
        colors: SemanticColors,
        cx: &mut Context<Self>,
    ) -> Option<AnyElement> {
        let metrics = self.scrollbar_metrics()?;
        let dragging = self.scrollbar_interaction.dragging;

        let thumb = div()
            .id("diff-scrollbar-thumb")
            .absolute()
            .top(px(metrics.thumb_top))
            .left(px(3.0))
            .right(px(3.0))
            .h(px(metrics.thumb_height))
            .rounded(px(3.0))
            .bg(colors.primary.alpha(if dragging { 0.46 } else { 0.24 }))
            .group_hover("diff-scrollbar", move |style| {
                style.bg(colors.primary.alpha(0.40))
            })
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, event: &gpui::MouseDownEvent, _, cx| {
                    this.scrollbar_interaction.dragging = true;
                    this.scrollbar_interaction.grab_offset =
                        (f32::from(event.position.y) - metrics.track_top - metrics.thumb_top)
                            .clamp(0.0, metrics.thumb_height);
                    cx.notify();
                    cx.stop_propagation();
                }),
            )
            .on_drag(DraggedDiffScrollbar, |value, _, _, cx| {
                cx.stop_propagation();
                cx.new(|_| *value)
            });

        Some(
            div()
                .id("diff-scrollbar-track")
                .group("diff-scrollbar")
                .absolute()
                .top(px(SCROLLBAR_INSET))
                .bottom(px(SCROLLBAR_INSET))
                .right(px(2.0))
                .w(px(12.0))
                .rounded(px(6.0))
                .occlude()
                .hover(move |style| style.bg(colors.primary.alpha(0.055)))
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, event: &gpui::MouseDownEvent, _, cx| {
                        this.scrollbar_interaction.dragging = true;
                        this.scrollbar_interaction.grab_offset = metrics.thumb_height / 2.0;
                        this.set_scrollbar_offset(f32::from(event.position.y), cx);
                        cx.stop_propagation();
                    }),
                )
                .on_drag(DraggedDiffScrollbar, |value, _, _, cx| {
                    cx.stop_propagation();
                    cx.new(|_| *value)
                })
                .on_mouse_up(
                    MouseButton::Left,
                    cx.listener(|this, _, _, cx| {
                        this.finish_scrollbar_drag(cx);
                        cx.stop_propagation();
                    }),
                )
                .on_mouse_up_out(
                    MouseButton::Left,
                    cx.listener(|this, _, _, cx| this.finish_scrollbar_drag(cx)),
                )
                .child(thumb)
                .into_any_element(),
        )
    }
}
