# Code Review Report: Diri Usage Summary CLI

```yaml
change_id: diri-usage-summary-cli
beads: homie-163
status: pass
reviewed_at: 2026-08-08
```

## Findings

| Severity | Category | Location | Evidence and impact | Status |
|----------|----------|----------|---------------------|--------|
| low | Scope | parity lock | Usage summary is not transcript parsing/watching or UI/fleet accounting. | accepted: USAGE-001 remains partial. |

## Verification

| Command | Result |
|---------|--------|
| `cargo test -p homie-cli --test usage_summary_cli -- --nocapture` | pass |
| `cargo check -p homie-storage -p homie-cli` | pass |
| `cargo clippy -p homie-storage -p homie-cli --all-targets -- -D warnings` | pass |
| `cargo fmt --all -- --check` | pass |

