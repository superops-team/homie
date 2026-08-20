use super::*;

impl TerminalPane {
    pub(super) fn schedule_find(
        &self,
        id: SessionId,
        delay: Duration,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        cx.spawn_in(window, async move |this, cx| {
            cx.background_executor().timer(delay).await;
            let _ = this.update_in(cx, |this, _window, _cx| this.start_due_find(&id));
        })
        .detach();
    }

    fn start_due_find(&mut self, id: &SessionId) {
        let now = self.started_at.elapsed();
        let Some(request) = self
            .residents
            .get_mut(id)
            .and_then(|resident| resident.find.as_mut())
            .and_then(|find| find.take_due_search(now))
        else {
            return;
        };
        let client = Arc::clone(self.runtime.client());
        let pane_tx = self.pane_tx.clone();
        let id = id.clone();
        self.tokio.spawn(async move {
            if let Ok(snapshot) = client.read_scrollback(&id).await {
                let _ = pane_tx.send(PaneEvent::FindSnapshot(id, request, snapshot.into()));
            }
        });
    }

    pub(super) fn open_find(&mut self, _: &OpenFind, window: &mut Window, cx: &mut Context<Self>) {
        let Some(id) = self.selected_id() else {
            return;
        };
        let Some(resident) = self.residents.get_mut(&id) else {
            return;
        };
        if resident.find.is_none() {
            resident.find = Some(TerminalFindModel::default());
            // Reopening keeps the last query but selects it, so ⌘F then typing
            // starts a new search while ⌘F then ⏎ repeats the old one.
            resident.find_query.select_all();
        }
        window.focus(&self.focus, cx);
        cx.stop_propagation();
        cx.notify();
    }

    pub(super) fn close_find(
        &mut self,
        _: &CloseFind,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.close_find_for_selected() {
            cx.stop_propagation();
            cx.notify();
        } else {
            cx.propagate();
        }
    }

    pub(super) fn close_find_for_selected(&mut self) -> bool {
        let Some(id) = self.selected_id() else {
            return false;
        };
        let Some(resident) = self.residents.get_mut(&id) else {
            return false;
        };
        if resident.find.take().is_none() {
            return false;
        }
        resident.element.set_find_highlights(Vec::new());
        true
    }

    pub(super) fn find_next(&mut self, _: &FindNext, _window: &mut Window, cx: &mut Context<Self>) {
        self.navigate_find(false, cx);
    }

    pub(super) fn find_previous(
        &mut self,
        _: &FindPrevious,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.navigate_find(true, cx);
    }

    pub(super) fn navigate_find(&mut self, backwards: bool, cx: &mut Context<Self>) {
        let Some(id) = self.selected_id() else {
            return;
        };
        let Some(resident) = self.residents.get_mut(&id) else {
            return;
        };
        let Some(find) = resident.find.as_mut() else {
            return;
        };
        if backwards {
            resident.element.find_previous(find);
        } else {
            resident.element.find_next(find);
        }
        resident.element.sync_find_highlights(find);
        cx.stop_propagation();
        cx.notify();
    }
}
