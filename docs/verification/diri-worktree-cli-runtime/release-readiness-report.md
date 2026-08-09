# Release Readiness Report: Diri Worktree CLI Runtime

```yaml
change_id: diri-worktree-cli-runtime
beads: homie-ye8
status: pass_with_scope_limit
validated_at: 2026-08-08
```

## Delivered

- `WorktreeInfo`, `WorktreeCreateRequest`, `WorktreeListRequest`, `WorktreeRemoveRequest`.
- Runtime real git worktree list/create/remove helpers.
- HomieClient worktree methods and protocol dispatch.
- `homie worktree list/create/remove`.
- Runtime and CLI real git fixture E2E.

## Gate Results

| Gate | Command | Result |
|------|---------|--------|
| Runtime worktree | `cargo test -p homie-runtime --test worktree_git -- --nocapture` | pass |
| CLI worktree | `cargo test -p homie-cli --test worktree_cli -- --nocapture` | pass |
| Build | `cargo check -p homie-proto -p homie-runtime -p homie-client -p homie-cli` | pass |
| Lint | `cargo clippy -p homie-runtime -p homie-client -p homie-cli --all-targets -- -D warnings` | pass |
| Format | `cargo fmt --all -- --check` | pass |
| Diff hygiene | scoped `git diff --check` | pass |

## Beads

- `homie-ye8` is complete for this bounded slice.
- Parent worktree/UI/API parity groups remain open.

## Remaining Work

- App worktree sheet create/remove interaction E2E.
- Worktree cleanup suggestion real UI path.
- Ports/forwarding adjacent CLI parity.
