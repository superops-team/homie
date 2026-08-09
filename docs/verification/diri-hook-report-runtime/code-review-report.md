# Code Review Report: Diri Hook Report Runtime

```yaml
change_id: diri-hook-report-runtime
beads: homie-b6o
status: pass
reviewed_at: 2026-08-08
```

## Findings

| Severity | Category | Location | Evidence and impact | Status |
|----------|----------|----------|---------------------|--------|
| medium | Compatibility | `crates/homie-cli/src/main.rs` | Hook integrations must remain fail-open and not write local state unless explicitly configured. | fixed: only `--data-dir` writes runtime state; default stays parse-only. |
| medium | Security | `hook_report_runtime_cli` | Permission payload included a secret-like token; output and persisted summary must not leak it. | fixed: regression asserts output/snapshot summary exclude the secret value. |
| low | Scope | parity lock | Persisting PermissionRequest needs-input is not the complete Diri hook bus. | accepted: RT-005 remains partial. |

## Verification

| Command | Result |
|---------|--------|
| `cargo test -p homie-cli --test hook_report_runtime_cli -- --nocapture` | pass |
| `cargo test -p homie-cli hook_command_outputs_redacted_structured_event -- --nocapture` | pass |
| `cargo check -p homie-storage -p homie-runtime -p homie-client -p homie-cli` | pass |
| `cargo clippy -p homie-storage -p homie-runtime -p homie-client -p homie-cli --all-targets -- -D warnings` | pass |
| `cargo fmt --all -- --check` | pass |
| scoped `git diff --check` | pass |

