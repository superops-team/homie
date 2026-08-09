# Functional Verification Report: Diri Codex Notify Runtime

```yaml
change_id: diri-notify-runtime
beads: homie-qki
status: pass_with_scope_limit
verified_at: 2026-08-08
```

## RED Evidence

| Case | Command | RED result |
|------|---------|------------|
| FC-DNRT-001 | `cargo test -p homie-cli --test notify_runtime_cli -- --nocapture` | failed: `homie notify` did not accept `--data-dir` |

## GREEN Evidence

| Case | Command | Result |
|------|---------|--------|
| FC-DNRT-001 | `cargo test -p homie-cli --test notify_runtime_cli -- --nocapture` | pass |
| FC-DNRT-002 | `cargo test -p homie-cli notify_command_outputs_codex_turn_complete -- --nocapture` | pass |
| FC-DNRT-003 | `cargo check -p homie-runtime -p homie-client -p homie-cli` | pass |
| FC-DNRT-003 | `cargo clippy -p homie-runtime -p homie-client -p homie-cli --all-targets -- -D warnings` | pass |
| FC-DNRT-003 | `cargo fmt --all -- --check` | pass |
| FC-DNRT-003 | scoped `git diff --check` | pass |

## Scope Notes

- `notify --data-dir` persists Codex turn completion as session idle.
- Existing no-data-dir notify remains parse-only.
- Full notify bus/status matrix remains pending.
