# Functional Verification Report: Diri MCP release_agent Owned-child Guard

```yaml
change_id: diri-mcp-release-owned-child-guard
beads: homie-al5
status: pass_with_scope_limit
verified_at: 2026-08-08
```

## RED Evidence

| Case | Command | RED result |
|------|---------|------------|
| FC-DMRO-001 | `cargo test -p homie-cli --test mcp_release_owned_child_guard_cli -- --nocapture` | failed: sibling release returned success instead of JSON-RPC error `-32000`. |
| FC-DMRO-002 | `cargo test -p homie-cli --test mcp_release_owned_child_guard_cli -- --nocapture` | failed: unrelated release returned success instead of JSON-RPC error `-32000`. |

During test hardening, the snapshot helper was corrected to match the existing CLI contract: `homie session snapshot --id <ID> --data-dir <DATA_DIR>` returns JSON by default.

## GREEN Evidence

| Case | Command | Result |
|------|---------|--------|
| FC-DMRO-001 | `cargo test -p homie-cli --test mcp_release_owned_child_guard_cli -- sibling_release_is_refused_and_target_survives --nocapture` | pass |
| FC-DMRO-002 | `cargo test -p homie-cli --test mcp_release_owned_child_guard_cli -- unrelated_release_is_refused_and_target_survives --nocapture` | pass |
| FC-DMRO-001..002 | `cargo test -p homie-cli --test mcp_release_owned_child_guard_cli -- --nocapture` | pass: 2 passed |
| FC-DMRO-003 | `cargo test -p homie-cli --test mcp_release_agent_cli -- --nocapture` | pass: 2 passed |
| FC-DMRO-003 | `cargo test -p homie-cli --test mcp_release_ancestor_guard_cli -- --nocapture` | pass: 1 passed |
| FC-DMRO-004 | `cargo check -p homie-client -p homie-cli` | pass |
| FC-DMRO-004 | `cargo clippy -p homie-client -p homie-cli --all-targets -- -D warnings` | pass |
| FC-DMRO-004 | `cargo fmt --all -- --check` | pass |
| FC-DMRO-004 | scoped `git diff --check` | pass |
| FC-DMRO-004 | `make parity-lock` | pass; remaining unrelated partial rows listed honestly |

## Scope Notes

- Implements direct-child-only `release_agent` permission.
- Does not implement recursive descendant release, full permission profiles, or UI controls.
