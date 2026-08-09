# Release Readiness Report: Diri MCP release_agent Owned-child Guard

```yaml
change_id: diri-mcp-release-owned-child-guard
beads: homie-al5
status: pass_with_scope_limit
validated_at: 2026-08-08
```

## Delivered

- `release_agent` now allows only direct child targets to reach `terminate_session`.
- Sibling and unrelated targets return JSON-RPC runtime error `-32000` before termination.
- Deny tests verify target sessions remain `running` after refusal.
- Existing self, parent, ancestor and direct-child behavior remains covered by regression tests.

## Gate Results

| Gate | Command | Result |
|------|---------|--------|
| Owned-child permission | `cargo test -p homie-cli --test mcp_release_owned_child_guard_cli -- --nocapture` | pass |
| Direct child/self regression | `cargo test -p homie-cli --test mcp_release_agent_cli -- --nocapture` | pass |
| Parent/ancestor regression | `cargo test -p homie-cli --test mcp_release_ancestor_guard_cli -- --nocapture` | pass |
| Build | `cargo check -p homie-client -p homie-cli` | pass |
| Lint | `cargo clippy -p homie-client -p homie-cli --all-targets -- -D warnings` | pass |
| Format | `cargo fmt --all -- --check` | pass |
| Diff hygiene | scoped `git diff --check` | pass |
| Parity lock | `make parity-lock` | pass |

## Remaining Work

- Recursive descendant release semantics.
- Full permission profile storage and enforcement.
- UI-visible lineage controls and audit trail.
