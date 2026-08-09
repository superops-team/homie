# Functional Verification Report: Diri MCP release_agent Ancestor Guard

```yaml
change_id: diri-mcp-release-ancestor-guard
beads: homie-4na
status: pass_with_scope_limit
verified_at: 2026-08-08
```

## Case Results

| Case | Command | Result |
|------|---------|--------|
| FC-DMRG-001 | `cargo test -p homie-cli --test mcp_release_ancestor_guard_cli -- --nocapture` | pass: child-to-parent and grandchild-to-root `release_agent` calls return JSON-RPC `-32000` and include `spawned you` in the error message. |
| FC-DMRG-002 | `cargo test -p homie-cli --test mcp_release_agent_cli -- --nocapture` | pass: direct child release still succeeds and self-release still returns the existing safe error. |
| FC-DMRG-003 | `cargo clippy -p homie-client -p homie-cli --all-targets -- -D warnings` | pass. |

## Evidence

```text
$ cargo test -p homie-cli --test mcp_release_ancestor_guard_cli -- --nocapture
running 1 test
test release_agent_refuses_parent_and_ancestor ... ok
test result: ok. 1 passed; 0 failed

$ cargo test -p homie-cli --test mcp_release_agent_cli -- --nocapture
running 2 tests
test rejects_releasing_calling_session ... ok
test releases_direct_child ... ok
test result: ok. 2 passed; 0 failed

$ cargo clippy -p homie-client -p homie-cli --all-targets -- -D warnings
Finished `dev` profile [unoptimized + debuginfo] target(s)
```

## Scope Notes

- This slice only closes the parent/ancestor release safety gap.
- API-005 remains partial because recursive permission enforcement and full lineage E2E are still separate parity gaps.
