use std::sync::Arc;
use std::time::Duration;

use gpui::Context;

use super::Surface;

impl super::UtilitySurfaces {
    pub(crate) fn open_worktrees(&mut self, cx: &mut Context<Self>) {
        self.surface = Surface::Worktrees;
        self.refresh_worktrees(cx);
    }

    pub(super) fn refresh_worktrees(&mut self, cx: &mut Context<Self>) {
        self.worktrees.begin_refresh();
        let generation = self.next_worktrees_generation();
        cx.notify();
        let client = Arc::clone(self.store_runtime.client());
        let runtime = Arc::clone(&self.runtime);
        self.worktrees_task = Some(cx.spawn(async move |this, cx| {
            let task = runtime.spawn(async move {
                client.wait_until_connected(Duration::from_secs(5)).await?;
                client.worktree_overview().await
            });
            let result = match task.await {
                Ok(Ok(entries)) => Ok(entries),
                Ok(Err(error)) => Err(error.to_string()),
                Err(error) => Err(error.to_string()),
            };
            let _ = this.update(cx, |this, cx| {
                this.worktrees_task = None;
                if this.finish_worktrees_refresh(generation, result) {
                    cx.notify();
                }
            });
        }));
    }

    pub(super) fn finish_worktrees_refresh(
        &mut self,
        generation: u64,
        result: Result<Vec<homie_proto::WorktreeOverviewEntry>, String>,
    ) -> bool {
        if self.surface != Surface::Worktrees || self.worktrees_generation != generation {
            return false;
        }
        self.worktrees.finish_refresh(result);
        true
    }

    pub(super) fn confirm_cleanup(&mut self, cx: &mut Context<Self>) {
        let Some(params) = self.worktrees.confirm_cleanup() else {
            return;
        };
        self.worktrees.begin_refresh();
        let generation = self.next_worktrees_generation();
        let client = Arc::clone(self.store_runtime.client());
        let runtime = Arc::clone(&self.runtime);
        self.worktrees_task = Some(cx.spawn(async move |this, cx| {
            let task = runtime.spawn(async move {
                client.wait_until_connected(Duration::from_secs(5)).await?;
                client.worktree_remove(params).await?;
                client.worktree_overview().await
            });
            let result = match task.await {
                Ok(Ok(entries)) => Ok(entries),
                Ok(Err(error)) => Err(error.to_string()),
                Err(error) => Err(error.to_string()),
            };
            let _ = this.update(cx, |this, cx| {
                this.worktrees_task = None;
                if this.finish_worktrees_refresh(generation, result) {
                    cx.notify();
                }
            });
        }));
    }
}
