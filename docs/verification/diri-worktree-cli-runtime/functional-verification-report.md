# Functional Verification Report: Diri Worktree CLI Runtime

```yaml
change_id: diri-worktree-cli-runtime
beads: homie-ye8
status: pass_with_scope_limit
verified_at: 2026-08-08
```

## RED Evidence

| Case | Command | RED result |
|------|---------|------------|
| FC-DWCR-001..002 | `cargo test -p homie-runtime --test worktree_git -- --nocapture` | failed: unresolved runtime worktree types/functions |
| FC-DWCR-003 | `cargo test -p homie-cli --test worktree_cli -- --nocapture` | failed: unrecognized subcommand `worktree` |

## GREEN Evidence

| Case | Command | Result |
|------|---------|--------|
| FC-DWCR-001 | `cargo test -p homie-runtime --test worktree_git -- parses_worktree_porcelain_like_diri --nocapture` | pass |
| FC-DWCR-002 | `cargo test -p homie-runtime --test worktree_git -- creates_lists_and_removes_real_git_worktree --nocapture` | pass |
| FC-DWCR-003 | `cargo test -p homie-cli --test worktree_cli -- --nocapture` | pass |
| FC-DWCR-004 | `cargo check -p homie-proto -p homie-runtime -p homie-client -p homie-cli` | pass |
| FC-DWCR-004 | `cargo clippy -p homie-runtime -p homie-client -p homie-cli --all-targets -- -D warnings` | pass |
| FC-DWCR-004 | `cargo fmt --all -- --check` | pass |
| FC-DWCR-004 | scoped `git diff --check` | pass |

## Scope Notes

- Runtime calls real `/usr/bin/git worktree list/add/remove`.
- CLI E2E initializes a real temp repository, creates a worktree, lists it, removes it, and verifies the path is gone.
- UI worktree sheet full interaction remains pending, so `GIT-002` and `UI-007` remain partial.
