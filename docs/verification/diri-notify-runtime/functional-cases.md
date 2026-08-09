# Functional Cases: Diri Codex Notify Runtime

```yaml
change_id: diri-notify-runtime
beads: homie-qki
```

## FC-DNRT-001: Notify persists idle status

- Command: `cargo test -p homie-cli --test notify_runtime_cli -- --nocapture`
- Expected: real session receives Codex `agent-turn-complete`, then snapshot shows idle.

## FC-DNRT-002: Parse-only fallback

- Command: `cargo test -p homie-cli notify_command_outputs_codex_turn_complete -- --nocapture`
- Expected: no-data-dir notify output remains structured and parse-only.

## FC-DNRT-003: Quality gates

- Commands: check, clippy, diff, parity lock.

