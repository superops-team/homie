# Code Review Report: Diri MCP release_agent Ancestor Guard

```yaml
change_id: diri-mcp-release-ancestor-guard
beads: homie-4na
status: pass
reviewed_at: 2026-08-08
```

## Round 1: Correctness Review

| Severity | Category | Location | Finding | Status |
|----------|----------|----------|---------|--------|
| high | Safety | `crates/homie-cli/src/main.rs` `release_agent` branch | A child session must not be able to terminate the parent or any ancestor that is waiting for its result. | fixed: `lineage_relation` returns `parent`/`ancestor`, and `release_agent` rejects both with JSON-RPC runtime error `-32000`. |
| medium | Regression | `crates/homie-cli/src/main.rs` `release_agent` branch | The guard must not block the intended direct-child release path. | verified: `mcp_release_agent_cli` still passes `releases_direct_child`. |
| medium | Side effect | `crates/homie-cli/src/main.rs` `release_agent` branch | Earlier release code risked duplicate termination calls while adding guard logic. | fixed: current branch calls `terminate_session` exactly once after all guard checks. |

## Round 2: Hidden Risk Review

| Severity | Category | Location | Finding | Status |
|----------|----------|----------|---------|--------|
| medium | Lineage semantics | `lineage_relation` | Parent/ancestor detection must walk the caller parent chain rather than only comparing one hop. | pass: implementation walks `parent_session_id` until root and returns `ancestor` for higher generations. |
| low | Error contract | `mcp_release_ancestor_guard_cli.rs` | Test should assert the user-facing safety reason, not only the error code. | pass: test asserts code `-32000` and message substring `spawned you`. |
| low | Scope | parity lock | This slice is not full MCP permission enforcement. | accepted: `API-005` remains `partial`. |

## Verification

| Command | Result |
|---------|--------|
| `cargo test -p homie-cli --test mcp_release_ancestor_guard_cli -- --nocapture` | pass |
| `cargo test -p homie-cli --test mcp_release_agent_cli -- --nocapture` | pass |
| `cargo clippy -p homie-client -p homie-cli --all-targets -- -D warnings` | pass |

