use super::*;

impl ControlServer {
    /// The aggregated staleness view: every worktree of every project,
    /// joined with the session (live wins) occupying it, its dirtiness,
    /// merged-ness into the default branch, and age — plus the "safe to
    /// clean up" suggestion. The staleness join itself lives in `crate::git`.
    pub(crate) fn worktree_overview(&self) -> Result<JsonValue, ControlError> {
        let (records, roots) = {
            let registry = self.registry.lock().map_err(poisoned)?;
            let roots: Vec<String> = registry
                .projects_raw()
                .iter()
                .filter_map(|project| project.get("root").and_then(|value| value.as_str()))
                .map(str::to_string)
                .collect();
            (registry.records(), roots)
        };
        encode(&crate::git::worktree_overview(&records, roots))
    }

    pub(crate) fn worktree_create(
        &self,
        params: Option<JsonValue>,
    ) -> Result<JsonValue, ControlError> {
        let p: homie_proto::WorktreeCreateParams = decode(params)?;
        let info = crate::git::create_worktree(
            Path::new(&p.repo_path),
            p.branch.as_deref(),
            p.base.as_deref(),
        )
        .map_err(io_control_error)?;
        self.events.publish(
            "worktree.created",
            json!({ "repoPath": p.repo_path, "path": info.path, "branch": info.branch }),
            None,
        );
        encode(&worktree_to_wire(info))
    }

    pub(crate) fn worktree_list(
        &self,
        params: Option<JsonValue>,
    ) -> Result<JsonValue, ControlError> {
        let p: homie_proto::WorktreeListParams = decode(params)?;
        let list = crate::git::list_worktrees(Path::new(&p.repo_path)).map_err(io_control_error)?;
        encode(&list.into_iter().map(worktree_to_wire).collect::<Vec<_>>())
    }

    pub(crate) fn worktree_remove(
        &self,
        params: Option<JsonValue>,
    ) -> Result<JsonValue, ControlError> {
        let p: homie_proto::WorktreeRemoveParams = decode(params)?;
        crate::git::remove_worktree(Path::new(&p.repo_path), &p.worktree_path, p.force)
            .map_err(io_control_error)?;
        self.events.publish(
            "worktree.removed",
            json!({ "repoPath": p.repo_path, "path": p.worktree_path }),
            None,
        );
        Ok(json!({}))
    }
}
