//! Directory indexing and Quick Open snapshot building for the navigation
//! overlay.
//!
//! These methods own the disk cache load and the background filesystem scan,
//! producing the `DirectoryIndex` and `QuickOpenSnapshot` that ranking and
//! rendering read. State mutation (scheduling ranks, notifying) stays in
//! `super`.

use super::*;

impl NavigationOverlay {
    /// The roots to index, and where their cached index lives.
    fn index_roots(&mut self) -> (Vec<PathBuf>, Vec<PathBuf>, PathBuf) {
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/nonexistent"));
        let projects = self.project_roots();
        let mut fallback = vec![PathBuf::from("~/fun")];
        fallback.extend(
            projects
                .iter()
                .filter_map(|(root, _)| root.parent().map(Path::to_path_buf)),
        );
        let quick_open_roots = self
            .store
            .read()
            .expect("session store lock poisoned")
            .preferences()
            .quick_open_roots
            .clone();
        let roots = quick_open::resolve_roots(&quick_open_roots, &fallback, &home);
        let cache = quick_open::cache_file(&home);
        (roots, vec![home], cache)
    }

    /// Populate the index from the previous run's scan. Costs one file read, so
    /// the first ⌘P of a launch has results to show instead of "Scanning…".
    pub(super) fn load_cached_index(&mut self, cx: &mut Context<Self>) {
        let (roots, _, cache) = self.index_roots();
        let (projects, cwds) = self.snapshot_inputs();
        self.cache_task = Some(cx.spawn(async move |this, cx| {
            let built = cx
                .background_spawn(async move {
                    let entries = quick_open::load_cache(&cache, &roots)?;
                    let snapshot = quick_open::build_snapshot(&entries, &projects, &cwds);
                    Some((entries, snapshot))
                })
                .await;
            let Some((entries, snapshot)) = built else {
                return;
            };
            this.update(cx, |this, cx| {
                this.directory_index.adopt_cached(entries);
                this.quick_snapshot = snapshot;
                cx.notify();
            })
            .ok();
        }));
    }

    pub(super) fn refresh_directory_index(&mut self, cx: &mut Context<Self>) {
        if !self.directory_index.needs_scan(Instant::now()) || !self.directory_index.begin_scan() {
            return;
        }
        let (roots, standalone, cache) = self.index_roots();
        let (projects, cwds) = self.snapshot_inputs();

        self.scan_task = Some(cx.spawn(async move |this, cx| {
            // Scan, persist, and prepare 20 000 ranking candidates all on the
            // background executor: preparing them on the main thread cost ~13 ms,
            // which is a dropped frame on any display and most of two at 120 Hz.
            let (entries, snapshot) = cx
                .background_spawn(async move {
                    let entries = quick_open::scan(&roots, &standalone);
                    quick_open::store_cache(&cache, &roots, &entries);
                    let snapshot = quick_open::build_snapshot(&entries, &projects, &cwds);
                    (entries, snapshot)
                })
                .await;
            this.update(cx, |this, cx| {
                this.directory_index.finish_scan(entries, Instant::now());
                this.quick_snapshot = snapshot;
                if !this.query.text().trim().is_empty() {
                    this.schedule_rank(cx);
                }
                cx.notify();
            })
            .ok();
        }));
    }

    /// The Recent section's contents: configured projects first, then session
    /// working directories in most-recently-updated order.
    fn snapshot_inputs(&mut self) -> (Vec<(PathBuf, String)>, Vec<PathBuf>) {
        let projects = self.project_roots();
        let store = self.store.read().expect("session store lock poisoned");
        let mut sessions: Vec<_> = store.sessions().values().collect();
        sessions.sort_by(|left, right| {
            right
                .updated_at
                .partial_cmp(&left.updated_at)
                .unwrap_or(Ordering::Equal)
        });
        let cwds = sessions
            .into_iter()
            .map(|session| PathBuf::from(&session.cwd))
            .collect();
        (projects, cwds)
    }

    fn project_roots(&mut self) -> Vec<(PathBuf, String)> {
        self.store
            .write()
            .expect("session store lock poisoned")
            .sidebar_projection()
            .projects
            .iter()
            .map(|entry| {
                (
                    PathBuf::from(&entry.project.root),
                    entry.project.name.clone(),
                )
            })
            .collect()
    }
}
