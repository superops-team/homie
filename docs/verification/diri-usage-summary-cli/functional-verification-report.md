# Functional Verification Report: Diri Usage Summary CLI

```yaml
change_id: diri-usage-summary-cli
beads: homie-163
status: pass_with_scope_limit
verified_at: 2026-08-08
```

## RED Evidence

| Case | Command | RED result |
|------|---------|------------|
| FC-DUSC-001..002 | `cargo test -p homie-cli --test usage_summary_cli -- --nocapture` | failed: unrecognized subcommand `usage` |

## GREEN Evidence

| Case | Command | Result |
|------|---------|--------|
| FC-DUSC-001 | `cargo test -p homie-cli --test usage_summary_cli -- summarizes_usage_records_from_storage --nocapture` | pass |
| FC-DUSC-002 | `cargo test -p homie-cli --test usage_summary_cli -- reports_empty_usage_summary --nocapture` | pass |
| FC-DUSC-003 | `cargo check -p homie-storage -p homie-cli` | pass |
| FC-DUSC-003 | `cargo clippy -p homie-storage -p homie-cli --all-targets -- -D warnings` | pass |
| FC-DUSC-003 | `cargo fmt --all -- --check` | pass |

## Scope Notes

- CLI summary uses existing storage `query_usage_totals`.
- Transcript watcher/parser, pricing table parity, fleet merge, and usage UI remain pending.
