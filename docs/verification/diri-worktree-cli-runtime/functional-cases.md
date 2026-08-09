# Functional Cases: Diri Worktree CLI Runtime

```yaml
change_id: diri-worktree-cli-runtime
beads: homie-ye8
```

## FC-DWCR-001: Runtime porcelain model

- Command: `cargo test -p homie-runtime --test worktree_git -- parses_worktree_porcelain_like_diri --nocapture`
- Expected: parses path, branch, bare, detached, prunable with final block flush.

## FC-DWCR-002: Runtime real git operations

- Command: `cargo test -p homie-runtime --test worktree_git -- creates_lists_and_removes_real_git_worktree --nocapture`
- Expected: initializes real repo, creates branch worktree, lists it, removes it.

## FC-DWCR-003: CLI real git E2E

- Command: `cargo test -p homie-cli --test worktree_cli -- --nocapture`
- Expected: `homie worktree create/list/remove --json` works against real temp repo.

## FC-DWCR-004: Quality gates

- Commands:
  - `cargo check -p homie-proto -p homie-runtime -p homie-client -p homie-cli`
  - `cargo clippy -p homie-runtime -p homie-client -p homie-cli --all-targets -- -D warnings`
  - scoped `git diff --check`
  - `make parity-lock`
- Expected: all pass; GIT-002 remains partial because app UI E2E is still pending.
