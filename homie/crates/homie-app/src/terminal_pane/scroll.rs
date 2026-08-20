use super::*;

impl TerminalPane {
    /// Starts the next queued scrollback fetch for `id`, if the viewport wants
    /// one and none is in flight. Called from wheel events AND from fetch
    /// completion: a fast wheel burst queues the next window while a fetch is
    /// in flight, and nothing else would ever start it — the stranded queue
    /// painted as a transient blank region in deep scrollback.
    pub(super) fn pump_scrollback_fetch(&mut self, id: &SessionId, visible_rows: usize) {
        let Some(resident) = self.residents.get_mut(id) else {
            return;
        };
        let Some(request) = resident.element.begin_scrollback_fetch(visible_rows) else {
            return;
        };
        let client = Arc::clone(self.runtime.client());
        let pane_tx = self.pane_tx.clone();
        let fetch_id = id.clone();
        self.tokio.spawn(async move {
            match client
                .read_scrollback_cells(&fetch_id, request.first_row, request.max_rows)
                .await
            {
                Ok(result) => {
                    let _ =
                        pane_tx.send(PaneEvent::ScrollbackCells(fetch_id, result, visible_rows));
                }
                Err(_) => {
                    let _ = pane_tx.send(PaneEvent::ScrollbackFailed(fetch_id));
                }
            }
        });
    }

    pub(super) fn handle_scroll(
        &mut self,
        event: &ScrollWheelEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(id) = self.selected_id() else {
            return;
        };
        let font_size = self
            .runtime
            .store
            .read()
            .expect("session store lock poisoned")
            .preferences()
            .terminal_font_size;
        let font = font(crate::fonts::mono_family());
        let metrics = CellMetrics::measure(window.text_system(), &font, px(font_size));
        let viewport = self.viewport.unwrap_or_default();
        let grid_x = viewport.x + GRID_HORIZONTAL_PADDING / 2.0;
        let grid_y = viewport.y + Metrics::TITLE_BAR + 2.0;
        let col = ((f32::from(event.position.x) - grid_x) / f32::from(metrics.cell_width))
            .floor()
            .max(0.0) as u16;
        let row = ((f32::from(event.position.y) - grid_y) / f32::from(metrics.line_height))
            .floor()
            .max(0.0) as u16;
        let delta = match event.delta {
            ScrollDelta::Pixels(point) => WheelDelta::PrecisePoints(f32::from(point.y)),
            ScrollDelta::Lines(point) => WheelDelta::Lines(point.y),
        };
        let Some(resident) = self.residents.get_mut(&id) else {
            return;
        };
        let visible_rows = resident.last_size.1.max(1);
        let route = resident.element.route_wheel(WheelEvent {
            delta,
            col,
            row,
            visible_rows,
            line_height: f32::from(metrics.line_height),
        });
        match route {
            Some(WheelRoute::Daemon {
                direction,
                lines,
                col,
                row,
            }) => resident.attachment.scroll(direction, lines, col, row),
            Some(WheelRoute::Local { .. }) => {
                self.pump_scrollback_fetch(&id, usize::from(visible_rows));
            }
            None => return,
        }
        cx.stop_propagation();
        cx.notify();
    }
}
