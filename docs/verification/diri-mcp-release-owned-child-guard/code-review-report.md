# Code Review Report: Diri MCP release_agent Owned-child Guard

```yaml
change_id: diri-mcp-release-owned-child-guard
beads: homie-al5
status: pass
reviewed_at: 2026-08-08
```

## Round 1: Explicit Defect Review

| Severity | Category | Location | Finding | Status |
|----------|----------|----------|---------|--------|
| high | Permission | `crates/homie-cli/src/main.rs` `release_agent` | Before this slice, sibling and unrelated targets could reach `terminate_session`. | fixed: only `relation == "child"` reaches `terminate_session`. |
| high | Side effect | `mcp_release_owned_child_guard_cli.rs` | Deny tests must prove the target survives, otherwise a terminate-before-error bug could pass. | fixed: tests call real `session snapshot` after denial and assert status remains `running`. |
| medium | Regression | `release_agent` branch | Adding allow-list could accidentally block direct child release. | verified: `mcp_release_agent_cli` still passes `releases_direct_child`. |

## Round 2: Hidden Risk Review

| Severity | Category | Location | Finding | Status |
|----------|----------|----------|---------|--------|
| medium | Relation matrix | `lineage_relation` and `release_agent` | The relation matrix must preserve dedicated self and upstream messages before the generic deny branch. | pass: `self` and `parent`/`ancestor` checks remain before generic non-child denial. |
| medium | Test realism | CLI integration tests | Tests should use the same MCP stdio and runtime-backed storage path a managed agent would use. | pass: tests drive `homie mcp-stdio --data-dir --session-id` and `homie session snapshot`. |
| low | Scope | parity lock | Owned-child guard is still not full recursive lineage permission. | accepted: API-005 remains `partial`. |

## Verification

| Command | Result |
|---------|--------|
| `cargo test -p homie-cli --test mcp_release_owned_child_guard_cli -- --nocapture` | pass |
| `cargo test -p homie-cli --test mcp_release_agent_cli -- --nocapture` | pass |
| `cargo test -p homie-cli --test mcp_release_ancestor_guard_cli -- --nocapture` | pass |
| `cargo check -p homie-client -p homie-cli` | pass |
| `cargo clippy -p homie-client -p homie-cli --all-targets -- -D warnings` | pass |
| `cargo fmt --all -- --check` | pass |
| scoped `git diff --check` | pass |
| `make parity-lock` | pass |
