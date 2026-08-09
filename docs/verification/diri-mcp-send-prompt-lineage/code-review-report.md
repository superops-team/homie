# Code Review Report: Diri MCP send_prompt Lineage

```yaml
change_id: diri-mcp-send-prompt-lineage
beads: homie-ggi
status: pass
reviewed_at: 2026-08-08
```

## Findings

| Severity | Category | Location | Evidence and impact | Status |
|----------|----------|----------|---------------------|--------|
| high | Safety | `crates/homie-cli/src/main.rs` | `send_prompt` to self would feed tool output back into the caller. | fixed: self target returns safe runtime error. |
| medium | Correctness | `crates/homie-cli/src/main.rs` | Sibling/unrelated writes lacked provenance, so target could confuse them with user input. | fixed: non-verbatim relations get provenance header. |
| low | Scope | parity lock | This is not full recursive lineage permission. | accepted: API-005 remains partial. |

## Verification

| Command | Result |
|---------|--------|
| `cargo test -p homie-cli --test mcp_send_prompt_lineage_cli -- --nocapture` | pass |
| `cargo check -p homie-client -p homie-cli` | pass |
| `cargo clippy -p homie-client -p homie-cli --all-targets -- -D warnings` | pass |
| `cargo fmt --all -- --check` | pass |
| scoped `git diff --check` | pass |

