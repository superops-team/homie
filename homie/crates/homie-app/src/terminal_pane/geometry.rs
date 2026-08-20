use super::*;

impl TerminalPane {
    pub(super) fn zoom_in(&mut self, _: &ZoomIn, window: &mut Window, cx: &mut Context<Self>) {
        self.change_zoom(1.0, false, window, cx);
    }

    pub(super) fn zoom_out(&mut self, _: &ZoomOut, window: &mut Window, cx: &mut Context<Self>) {
        self.change_zoom(-1.0, false, window, cx);
    }

    pub(super) fn reset_zoom(
        &mut self,
        _: &ResetZoom,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.change_zoom(0.0, true, window, cx);
    }

    fn change_zoom(
        &mut self,
        delta: f32,
        reset: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let result = {
            let mut store = self
                .runtime
                .store
                .write()
                .expect("session store lock poisoned");
            if reset {
                store.reset_terminal_zoom()
            } else {
                store.zoom_terminal(delta)
            }
        };
        if result.is_ok() {
            self.update_selected_geometry(window, cx);
            cx.stop_propagation();
            cx.notify();
        }
    }
    /// Grid cell under a window-space pointer position, using the same
    /// geometry as `handle_scroll`.
    pub(super) fn grid_cell_at(
        &self,
        position: gpui::Point<gpui::Pixels>,
        window: &mut Window,
    ) -> Option<(usize, usize)> {
        self.selected_session()?;
        let store = self
            .runtime
            .store
            .read()
            .expect("session store lock poisoned");
        let font_size = store.preferences().terminal_font_size;
        drop(store);
        let metrics = CellMetrics::measure(
            window.text_system(),
            &font(crate::fonts::mono_family()),
            px(font_size),
        );
        let viewport = self.viewport.unwrap_or_default();
        let grid_x = viewport.x + GRID_HORIZONTAL_PADDING / 2.0;
        // An overflowing grid is bottom-anchored (see render_grid_and_overlays),
        // so its first row sits above the surface -- selection has to follow it
        // or clicks land on the wrong line while a resize is in flight.
        let grid_rows = self
            .selected_id()
            .and_then(|id| self.residents.get(&id))
            .map_or(0, |resident| resident.element.grid_rows());
        let anchor = self
            .grid_row_overflow(grid_rows, font_size, window)
            .map_or(0.0, |grid_height| self.grid_inner_height() - grid_height);
        let grid_y = viewport.y + Metrics::TITLE_BAR + 2.0 + anchor;
        let col = ((f32::from(position.x) - grid_x) / f32::from(metrics.cell_width))
            .floor()
            .max(0.0) as usize;
        let row = ((f32::from(position.y) - grid_y) / f32::from(metrics.line_height))
            .floor()
            .max(0.0) as usize;
        Some((col, row))
    }

    /// The height the mirrored grid needs when the daemon's screen is taller
    /// than the pane can show, or `None` when it fits. Only a resize still in
    /// flight puts the two out of step, so this is `None` on settled frames.
    pub(super) fn grid_row_overflow(
        &self,
        grid_rows: u16,
        font_size: f32,
        window: &mut Window,
    ) -> Option<f32> {
        if grid_rows == 0 || self.viewport.is_none() {
            return None;
        }
        let metrics = CellMetrics::measure(
            window.text_system(),
            &font(crate::fonts::mono_family()),
            px(font_size),
        );
        // A pixel of slack on top of the exact row height: the element derives
        // its row count back out with `floor(height / line_height)`, and an
        // exactly-sized box loses its last row to float error or to layout
        // rounding -- which is the row this anchoring exists to keep on screen.
        (grid_rows > metrics.rows_for_height(px(self.grid_inner_height())))
            .then(|| f32::from(metrics.line_height).mul_add(f32::from(grid_rows), ANCHOR_SLACK))
    }

    /// Height available to `TerminalElement` inside the terminal surface -- the
    /// same figure [`estimated_grid_size`] turns into a row count.
    fn grid_inner_height(&self) -> f32 {
        let height = self.viewport.map_or(0.0, |viewport| viewport.height);
        (height - Metrics::TITLE_BAR - GRID_VERTICAL_PADDING - GRID_LAYOUT_VERTICAL_CHROME).max(1.0)
    }
    pub(super) fn update_selected_geometry(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(session) = self.selected_session() else {
            return;
        };
        let font_size = self
            .runtime
            .store
            .read()
            .expect("session store lock poisoned")
            .preferences()
            .terminal_font_size;
        let viewport = self.viewport.unwrap_or_else(|| {
            let bounds = window.inner_window_bounds().get_bounds();
            TerminalViewport {
                x: 0.0,
                y: 0.0,
                width: f32::from(bounds.size.width),
                height: f32::from(bounds.size.height),
            }
        });
        let metrics = CellMetrics::measure(
            window.text_system(),
            &font(crate::fonts::mono_family()),
            px(font_size),
        );
        let size = estimated_grid_size(viewport.width, viewport.height, 0.0, metrics);
        if let Some(resident) = self.residents.get_mut(&session.id)
            && resident.last_size != size
        {
            // Leading edge: an isolated change (first measure after attach, a
            // session switch, a window snap, the first frame of a drag) reaches
            // the daemon immediately so the pane feels instant.
            let previous = resident.last_size;
            let first_measure = previous == (0, 0);
            resident.last_size = size;
            let now = Instant::now();
            let since_sent = self.last_resize_sent.map(|at| now.duration_since(at));
            let delay = match plan_resize(first_measure, since_sent, self.resize_flush_armed) {
                ResizePlan::SendNow => {
                    self.last_resize_sent = Some(now);
                    self.pending_resizes.remove(&session.id);
                    resident.attachment.resize(size.0, size.1);
                    if should_hold_reflow(previous, size, since_sent) {
                        self.hold_reflow(session.id.clone(), window, cx);
                    }
                    return;
                }
                // Mid-drag: fold into the tick already armed. It is never
                // rescheduled by a later frame -- it fires on the cadence
                // carrying whatever the newest size is by then -- so a
                // continuous drag keeps the PTY reflowing at ~20Hz instead of
                // waiting for the mouse to stop.
                ResizePlan::Fold => {
                    self.pending_resizes.insert(session.id.clone(), size);
                    return;
                }
                ResizePlan::Arm(delay) => delay,
            };
            self.pending_resizes.insert(session.id.clone(), size);
            self.resize_flush_armed = true;
            let timer = cx.background_executor().timer(delay);
            self.resize_flush = Some(cx.spawn(async move |this, cx| {
                timer.await;
                let _ = this.update(cx, |this, _cx| {
                    this.resize_flush_armed = false;
                    this.last_resize_sent = Some(Instant::now());
                    let pending = std::mem::take(&mut this.pending_resizes);
                    for (id, size) in pending {
                        if let Some(resident) = this.residents.get(&id) {
                            resident.attachment.resize(size.0, size.1);
                        }
                    }
                });
            }));
        }
    }
}
