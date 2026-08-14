# GPUI UtilitySurfaces First Slice Plan

## 1. Scope

This change implements the first code-bearing GPUI architecture hardening slice:
History and Worktrees lifecycle task ownership in `UtilitySurfaces`.

## 2. In Scope

- Add generation counters and held `Task<()>` fields for History and Worktrees.
- Convert History load/resume operations from detached lifecycle tasks to held tasks.
- Convert Worktrees refresh/cleanup operations from detached lifecycle tasks to held tasks.
- Reject stale async results using generation and current surface checks.
- Clear held tasks when closing the corresponding utility surface.
- Add focused tests for stale and closed-surface result guards.

## 3. Out Of Scope

- RootView restructuring.
- Splitting `surface_shell.rs` into multiple files.
- New `homie-ui` primitives.
- Daemon/client API changes.
- Settings/Remote Host initialization task lifecycle.

## 4. Implementation Strategy

1. Add fields to `UtilitySurfaces`.
2. Add small private helper methods for generation and result application.
3. Rewrite the four lifecycle operations:
   - `open_history`
   - `resume_history`
   - `refresh_worktrees`
   - `confirm_cleanup`
4. Add tests using private helper methods to avoid nondeterministic daemon timing.
5. Execute functional cases.

## 5. Risk Controls

- Keep UI render output unchanged.
- Do not move code into new modules in this slice.
- Keep side-effect semantics explicit: dropping a task protects UI lifecycle but
  does not promise daemon-side cancellation after a request is already sent.
