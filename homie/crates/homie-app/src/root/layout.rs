use super::*;

impl RootView {
    pub(crate) fn window_bounds_changed(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let placement = crate::current_window_placement(window, cx);
        self.services
            .store
            .store
            .write()
            .expect("session store lock poisoned")
            .remember_window_placement(placement);

        if self.window_bounds_save.is_some() {
            return;
        }
        self.window_bounds_save = Some(cx.spawn_in(window, async move |this, cx| {
            cx.background_executor()
                .timer(WINDOW_BOUNDS_SAVE_DELAY)
                .await;
            let _ = this.update_in(cx, |this, _window, _cx| {
                this.window_bounds_save.take();
                if let Err(error) = this
                    .services
                    .store
                    .store
                    .write()
                    .expect("session store lock poisoned")
                    .persist_preferences()
                {
                    eprintln!("homie: could not remember window placement: {error}");
                }
            });
        }));
    }

    pub(crate) fn settled_sidebar_seam(&self, cx: &App) -> f32 {
        let sidebar = self.sidebar.read(cx);
        if sidebar.is_visible() {
            sidebar.width()
        } else {
            0.0
        }
    }

    pub(crate) fn begin_sidebar_slide(&mut self, cx: &mut Context<Self>) {
        let to = self.settled_sidebar_seam(cx);
        self.sidebar_slide = (!cx.reduce_motion())
            .then(|| SeamSlide::begin(self.sidebar_seam, to))
            .flatten();
        if self.sidebar_slide.is_none() {
            self.sidebar_seam = to;
        }
    }

    pub(crate) fn settled_inspector_seam(&self) -> f32 {
        if self.inspector_open {
            self.inspector_width.min(self.inspector_max_width)
        } else {
            0.0
        }
    }

    pub(crate) fn begin_inspector_slide(&mut self, cx: &mut Context<Self>) {
        let to = self.settled_inspector_seam();
        self.inspector_slide = (!cx.reduce_motion())
            .then(|| SeamSlide::begin(self.inspector_seam, to))
            .flatten();
        if self.inspector_slide.is_none() {
            self.inspector_seam = to;
        }
    }

    pub(crate) fn drag_resize(&mut self, pointer_x: f32, cx: &mut Context<Self>) {
        let Some((origin_x, base_width)) = self.resize_origin else {
            return;
        };
        let width = base_width + pointer_x - origin_x;
        self.sidebar
            .update(cx, |sidebar, cx| sidebar.set_width(width, cx));
    }

    pub(crate) fn drag_terminal_resize(&mut self, pointer_y: f32, cx: &mut Context<Self>) {
        let Some((origin_y, base_height)) = self.terminal_resize_origin else {
            return;
        };
        self.workbench_layout.resize_primary(
            base_height + pointer_y - origin_y,
            self.terminal_available_height,
        );
        cx.notify();
    }

    pub(crate) fn finish_terminal_resize(&mut self, cx: &mut Context<Self>) {
        if self.terminal_resize_origin.take().is_none() {
            return;
        }
        let fraction = self.workbench_layout.primary_fraction();
        if let Err(error) = self
            .services
            .store
            .store
            .write()
            .expect("session store lock poisoned")
            .update_preferences(|prefs| prefs.workbench_primary_fraction = fraction)
        {
            eprintln!("homie: could not remember workbench split: {error}");
        }
        cx.notify();
    }

    pub(crate) fn finish_resize(&mut self, cx: &mut Context<Self>) {
        if self.resize_origin.take().is_some() {
            self.sidebar
                .update(cx, |sidebar, cx| sidebar.commit_width(cx));
        }
    }

    pub(crate) fn drag_inspector_resize(&mut self, pointer_x: f32, cx: &mut Context<Self>) {
        let Some((origin_x, base_width)) = self.inspector_resize_origin else {
            return;
        };
        self.inspector_width = (base_width - pointer_x + origin_x).clamp(
            300.0_f32.min(self.inspector_max_width),
            self.inspector_max_width,
        );
        cx.notify();
    }

    pub(crate) fn finish_inspector_resize(&mut self, cx: &mut Context<Self>) {
        if self.inspector_resize_origin.take().is_none() {
            return;
        }
        let width = self.inspector_width;
        if let Err(error) = self
            .services
            .store
            .store
            .write()
            .expect("session store lock poisoned")
            .update_preferences(|prefs| prefs.inspector_width = width)
        {
            eprintln!("homie: could not remember inspector width: {error}");
        }
        cx.notify();
    }
}
