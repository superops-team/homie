use super::*;

impl RootView {
    pub(crate) fn set_inspector_open(&mut self, open: bool, cx: &mut Context<Self>) {
        if self.preview || self.inspector_open == open {
            return;
        }
        let now = Instant::now();
        if !toggle_has_settled(self.inspector_toggled_at.map(|at| now.duration_since(at))) {
            return;
        }
        self.inspector_toggled_at = Some(now);
        self.inspector_open = open;
        if let Some(inspector) = &self.inspector {
            inspector.update(cx, |inspector, cx| inspector.set_visible(open, cx));
        }
        if let Err(error) = self
            .services
            .store
            .store
            .write()
            .expect("session store lock poisoned")
            .update_preferences(|prefs| prefs.inspector_open = open)
        {
            eprintln!("homie: could not remember inspector visibility: {error}");
        }
        self.begin_inspector_slide(cx);
        cx.notify();
    }

    pub(crate) fn toggle_inspector(&mut self, cx: &mut Context<Self>) {
        self.set_inspector_open(!self.inspector_open, cx);
    }

    pub(crate) fn reveal_inspector(&mut self, cx: &mut Context<Self>) {
        self.inspector_toggled_at = None;
        self.set_inspector_open(true, cx);
    }
}
