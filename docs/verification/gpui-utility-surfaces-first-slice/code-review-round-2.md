# GPUI UtilitySurfaces First Slice Code Review Round 2

## 1. Scope

Second-pass review focused on hidden lifecycle, scope and evidence risks.

## 2. Checks

| Check | Result | Notes |
|-------|--------|-------|
| History/Worktrees lifecycle tasks are held | pass | `history_task` and `worktrees_task` fields are assigned returned `Task<()>` values |
| Stale results are guarded | pass | finish helpers check both surface and generation |
| Completed stale task handles are cleared | pass | completion callbacks clear task fields before applying result |
| Closing surface cancels UI lifecycle task | pass | `close_surface` clears relevant task and increments generation |
| Out-of-scope crates unchanged | pass | no diff in `homie-ui/src`, `homie-engine`, `homie-client`, or `root.rs` |
| Remaining `.detach()` | accepted | one remaining `.detach()` belongs to remote host initialization, explicitly outside this slice |
| Icon asset change | accepted | required baseline remediation; no source code change under `homie-ui/src` |

## 3. Residual Risk

Dropping a held GPUI task can cancel UI-side orchestration, but cannot guarantee
daemon-side cancellation after a request has already been sent. This is recorded
in the PRD and mitigated by generation guards that prevent late results from
rewriting visible UI state.

## 4. Conclusion

Round 2 passed. No P0/P1 issues remain.
