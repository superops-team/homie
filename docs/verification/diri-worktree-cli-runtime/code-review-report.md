# Code Review Report: Diri Worktree CLI Runtime

```yaml
change_id: diri-worktree-cli-runtime
beads: homie-ye8
status: pass
reviewed_at: 2026-08-08
```

## Findings

| Severity | Category | Location | Evidence and impact | Status |
|----------|----------|----------|---------------------|--------|
| medium | Correctness | `crates/homie-runtime/src/lib.rs` | First GREEN attempt compared non-canonical worktree paths; macOS temp paths can differ between `/var` and `/private/var`, causing false misses after `git worktree add`. | fixed: compare canonical paths and return git-listed entry. |
| low | Scope | parity lock | Worktree CLI/runtime E2E does not complete app Worktrees sheet interaction. | accepted: `GIT-002` / `UI-007` remain partial. |

## Verification

| Command | Result |
|---------|--------|
| `cargo test -p homie-runtime --test worktree_git -- --nocapture` | pass |
| `cargo test -p homie-cli --test worktree_cli -- --nocapture` | pass |
| `cargo check -p homie-proto -p homie-runtime -p homie-client -p homie-cli` | pass |
| `cargo clippy -p homie-runtime -p homie-client -p homie-cli --all-targets -- -D warnings` | pass |
| `cargo fmt --all -- --check` | pass |
| scoped `git diff --check` | pass |

## Remaining Risk

- App sheet action flow and screenshot/interaction evidence still need a UI lane.
