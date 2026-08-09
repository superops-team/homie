# Code Review Report: Diri Codex Notify Runtime

```yaml
change_id: diri-notify-runtime
beads: homie-qki
status: pass
reviewed_at: 2026-08-08
```

## Findings

| Severity | Category | Location | Evidence and impact | Status |
|----------|----------|----------|---------------------|--------|
| medium | Correctness | `crates/homie-runtime/src/lib.rs` | Opening a new runtime adopted live holders and overwrote persisted idle with running. | fixed: adoption preserves persisted idle/needs-input; send_text moves session back to running. |
| low | Scope | parity lock | Codex turn-complete persistence does not complete every notify event type. | accepted: RT-005 remains partial. |

## Verification

| Command | Result |
|---------|--------|
| `cargo test -p homie-cli --test notify_runtime_cli -- --nocapture` | pass |
| `cargo test -p homie-cli notify_command_outputs_codex_turn_complete -- --nocapture` | pass |
| `cargo check -p homie-runtime -p homie-client -p homie-cli` | pass |
| `cargo clippy -p homie-runtime -p homie-client -p homie-cli --all-targets -- -D warnings` | pass |
| `cargo fmt --all -- --check` | pass |
| scoped `git diff --check` | pass |

