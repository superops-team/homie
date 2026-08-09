# T-010 Worktree Project Management Report

```yaml
change_id: reference-parity-v1
openspec_task: T-010
beads: homie-wd7
status: pass
functional_cases:
  - FC-010
```

## 1. Summary

T-010 implemented the first non-destructive worktree safety slice in `homie-runtime`.

Implemented:

- `WorktreeOverviewEntry`.
- `WorktreeSheet`.
- `WorktreeCleanupRequest`.
- Cleanup eligibility logic that rejects main/master, dirty, unmerged, and non-stale worktrees.

This slice does not execute `git worktree remove`; it only defines the safe cleanup decision contract.

## 2. RED

Added failing safety test:

- `crates/homie-runtime/tests/worktree_safety.rs`

The test requires cleanup to be available only for stale, clean, merged, non-main worktrees.

## 3. GREEN

Implemented:

- Worktree overview and cleanup request types in `crates/homie-runtime/src/lib.rs`.
- Non-destructive cleanup eligibility logic.

## 4. Verification

Focused command:

```bash
cargo test -p homie-runtime
```

Result:

- Exit code: 0
- Runtime lifecycle tests: 1 passed
- Worktree safety tests: 1 passed

Workspace regression command:

```bash
cargo test --workspace
```

Result:

- Exit code: 0
- Homie agents/app/CLI/LLM/proto/runtime/storage tests passed.

Safety checks:

```bash
rg -n -i "<old-reference-name-pattern>" .
git diff --check
```

Result:

- Old reference name scan: no matches.
- Markdown/patch whitespace check: pass.

## 5. Remaining Scope

Still deferred:

- Real git worktree create/list/remove implementation.
- Storage repository integration for worktrees/projects.
- CLI and UI worktree surfaces.

