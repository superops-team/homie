# Functional Cases: Diri MCP release_agent Owned-child Guard

```yaml
change_id: diri-mcp-release-owned-child-guard
beads: homie-al5
```

## FC-DMRO-001: Sibling release is refused without side effects

- Command: `cargo test -p homie-cli --test mcp_release_owned_child_guard_cli -- sibling_release_is_refused_and_target_survives --nocapture`
- Setup: create root session, spawn sibling A and sibling B through real `homie mcp-stdio --data-dir`.
- Expected:
  - sibling A calling `release_agent(sessionId=siblingB)` returns JSON-RPC error code `-32000`.
  - error message contains `only release a direct child`.
  - sibling B remains visible through `homie session snapshot`.

## FC-DMRO-002: Unrelated release is refused without side effects

- Command: `cargo test -p homie-cli --test mcp_release_owned_child_guard_cli -- unrelated_release_is_refused_and_target_survives --nocapture`
- Setup: create two independent root sessions.
- Expected:
  - root A calling `release_agent(sessionId=rootB)` returns JSON-RPC error code `-32000`.
  - error message contains `only release a direct child`.
  - root B remains visible through `homie session snapshot`.

## FC-DMRO-003: Existing release guards do not regress

- Commands:
  - `cargo test -p homie-cli --test mcp_release_agent_cli -- --nocapture`
  - `cargo test -p homie-cli --test mcp_release_ancestor_guard_cli -- --nocapture`
- Expected:
  - direct child release still succeeds.
  - self release still fails with the calling-session guard.
  - parent/ancestor release still fails with the `spawned you` guard.

## FC-DMRO-004: Quality gates

- Commands:
  - `cargo check -p homie-client -p homie-cli`
  - `cargo clippy -p homie-client -p homie-cli --all-targets -- -D warnings`
  - `cargo fmt --all -- --check`
  - scoped `git diff --check`
  - `make parity-lock`
- Expected: all commands pass; parity lock remains honest about remaining `partial` rows.
