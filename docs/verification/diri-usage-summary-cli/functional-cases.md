# Functional Cases: Diri Usage Summary CLI

```yaml
change_id: diri-usage-summary-cli
beads: homie-163
```

## FC-DUSC-001: Usage totals JSON

- Command: `cargo test -p homie-cli --test usage_summary_cli -- summarizes_usage_records_from_storage --nocapture`
- Expected: CLI reports token/cache/cost totals from seeded storage records.

## FC-DUSC-002: Empty summary

- Command: `cargo test -p homie-cli --test usage_summary_cli -- reports_empty_usage_summary --nocapture`
- Expected: empty DB reports zero totals.

## FC-DUSC-003: Quality gates

- Commands: check, clippy, diff, parity lock.

