# Functional Verification Report: Diri Hook Report Runtime

```yaml
change_id: diri-hook-report-runtime
beads: homie-b6o
status: pass_with_scope_limit
verified_at: 2026-08-08
```

## RED Evidence

| Case | Command | RED result |
|------|---------|------------|
| FC-DHRR-001 | `cargo test -p homie-cli --test hook_report_runtime_cli -- --nocapture` | failed: `homie hook` did not accept `--data-dir` |

## GREEN Evidence

| Case | Command | Result |
|------|---------|--------|
| FC-DHRR-001 | `cargo test -p homie-cli --test hook_report_runtime_cli -- --nocapture` | pass |
| FC-DHRR-002 | `cargo test -p homie-cli hook_command_outputs_redacted_structured_event -- --nocapture` | pass |
| FC-DHRR-003 | `cargo check -p homie-storage -p homie-runtime -p homie-client -p homie-cli` | pass |
| FC-DHRR-003 | `cargo clippy -p homie-storage -p homie-runtime -p homie-client -p homie-cli --all-targets -- -D warnings` | pass |
| FC-DHRR-003 | `cargo fmt --all -- --check` | pass |
| FC-DHRR-003 | scoped `git diff --check` | pass |

## Scope Notes

- `hook --data-dir` persists non-subagent needs-input into session metadata.
- No `--data-dir` remains parse-only and redacted.
- Full hook bus/status matrix remains pending, so RT-005 remains partial.
