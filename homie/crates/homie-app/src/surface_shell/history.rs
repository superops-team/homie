use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use gpui::Context;

use homie_proto::HistoryEntry;

use super::{RESULT_LIMIT, Surface};

impl super::UtilitySurfaces {
    pub(crate) fn open_history(&mut self, cx: &mut Context<Self>) {
        self.surface = Surface::History;
        self.history_query.clear();
        self.history_highlight = 0;
        self.history_loading = true;
        self.history_error = None;
        let generation = self.next_history_generation();
        cx.notify();

        let roots = crate::history::HistoryRoots::current_user();
        let client = Arc::clone(self.store_runtime.client());
        let runtime = Arc::clone(&self.runtime);
        self.history_task = Some(cx.spawn(async move |this, cx| {
            let task = runtime.spawn(async move {
                let tracked = if client
                    .wait_until_connected(Duration::from_secs(5))
                    .await
                    .is_ok()
                {
                    client
                        .sessions()
                        .await
                        .map(|result| {
                            result
                                .sessions
                                .into_iter()
                                .filter_map(|session| session.agent_session_id)
                                .collect()
                        })
                        .unwrap_or_default()
                } else {
                    HashSet::new()
                };
                tokio::task::spawn_blocking(move || crate::history::scan(&roots, &tracked))
                    .await
                    .map_err(|error| error.to_string())
            });
            let result = task
                .await
                .map_err(|error| error.to_string())
                .and_then(|r| r);
            let _ = this.update(cx, |this, cx| {
                this.history_task = None;
                if this.finish_history_load(generation, result) {
                    cx.notify();
                }
            });
        }));
    }

    pub(super) fn finish_history_load(
        &mut self,
        generation: u64,
        result: Result<Vec<HistoryEntry>, String>,
    ) -> bool {
        if self.surface != Surface::History || self.history_generation != generation {
            return false;
        }
        self.history_loading = false;
        match result {
            Ok(entries) => {
                self.activity = format!("{} past conversations found", entries.len());
                self.history = entries;
                self.history_error = None;
            }
            Err(error) => self.history_error = Some(error),
        }
        true
    }

    fn finish_history_resume(
        &mut self,
        generation: u64,
        result: Result<homie_proto::SessionId, String>,
    ) -> bool {
        if self.surface != Surface::History || self.history_generation != generation {
            return false;
        }
        self.history_loading = false;
        match result {
            Ok(id) => {
                self.surface = Surface::None;
                self.activity = format!("Resumed conversation in session {}", id.0);
                self.history_error = None;
            }
            Err(error) => self.history_error = Some(error),
        }
        true
    }

    pub(super) fn resume_history(&mut self, entry: HistoryEntry, cx: &mut Context<Self>) {
        let Some(params) = crate::history::resume_spawn(&entry) else {
            self.history_error = Some("The conversation folder is no longer available".to_owned());
            cx.notify();
            return;
        };
        self.history_loading = true;
        self.history_error = None;
        let generation = self.next_history_generation();
        let client = Arc::clone(self.store_runtime.client());
        let runtime = Arc::clone(&self.runtime);
        self.history_task = Some(cx.spawn(async move |this, cx| {
            let task = runtime.spawn(async move {
                client.wait_until_connected(Duration::from_secs(5)).await?;
                client.spawn(params).await
            });
            let result = match task.await {
                Ok(result) => result.map_err(|error| error.to_string()),
                Err(error) => Err(error.to_string()),
            };
            let _ = this.update(cx, |this, cx| {
                this.history_task = None;
                if this.finish_history_resume(generation, result) {
                    cx.notify();
                }
            });
        }));
    }

    pub(super) fn visible_history(&self) -> Vec<HistoryEntry> {
        self.history
            .iter()
            .filter(|entry| crate::history::matches_query(entry, self.history_query.text()))
            .take(RESULT_LIMIT)
            .cloned()
            .collect()
    }

    pub(super) fn move_history(&mut self, delta: isize, cx: &mut Context<Self>) {
        if self.surface != Surface::History {
            return;
        }
        let count = self.visible_history().len();
        if count == 0 {
            return;
        }
        self.history_highlight =
            (self.history_highlight as isize + delta).rem_euclid(count as isize) as usize;
        cx.notify();
    }

    pub(super) fn activate_history(&mut self, cx: &mut Context<Self>) {
        if self.surface != Surface::History {
            return;
        }
        if let Some(entry) = self.visible_history().get(self.history_highlight).cloned() {
            self.resume_history(entry, cx);
        }
    }
}
