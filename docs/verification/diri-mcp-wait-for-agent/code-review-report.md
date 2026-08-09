# Code Review Report: Diri MCP wait_for_agent Runtime

```yaml
change_id: diri-mcp-wait-for-agent
beads: homie-trk
status: pass
reviewed_at: 2026-08-08
```

## Round 1: Explicit Defect Review

| Severity | Category | Location | Finding | Status |
|----------|----------|----------|---------|--------|
| high | Missing behavior | `crates/homie-cli/src/main.rs` | `wait_for_agent` was listed as a tool but runtime-backed calls returned unsupported. | fixed: dispatch now calls `wait_for_agent_payload`. |
| high | Real path | `mcp_wait_for_agent_cli.rs` | Tests must not mock status; they need real runtime state transitions. | fixed: tests use `session create`, `notify`, `session kill`, and `mcp-stdio --data-dir`. |
| medium | Parameter compatibility | `wait_for_agent_payload` | Diri uses `session_id`/`timeout_s`, while Homie has existing camelCase usage. | fixed: accepts both spellings. |

## Round 2: Hidden Risk Review

| Severity | Category | Location | Finding | Status |
|----------|----------|----------|---------|--------|
| medium | Timeout semantics | `wait_for_agent_payload` | `timeout_s:0` must still inspect current status once before returning timeout. | pass: implementation reads status before deadline check; test covers running timeout. |
| medium | Status encoding | `wait_for_agent_payload` | Runtime returns `SessionStatus` enum; MCP output must remain stable string JSON. | pass: `session_status_text` uses serde encoding and tests assert string values. |
| low | Scope | docs/specs | This is not the full Diri `events.wait` long-poll implementation. | accepted: scope notes and parity lock keep API rows partial. |

## Verification

| Command | Result |
|---------|--------|
| `cargo test -p homie-cli --test mcp_wait_for_agent_cli -- --nocapture` | pass |
| `cargo test -p homie-cli --test mcp_wait_children_cli -- --nocapture` | pass |
| `cargo check -p homie-client -p homie-cli` | pass |
| `cargo clippy -p homie-client -p homie-cli --all-targets -- -D warnings` | pass |
| `cargo fmt --all -- --check` | pass |
| scoped `git diff --check` | pass |
| `make parity-lock` | pass |
