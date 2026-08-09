# Release Readiness Report: Diri Usage Summary CLI

```yaml
change_id: diri-usage-summary-cli
beads: homie-163
status: pass_with_scope_limit
validated_at: 2026-08-08
```

## Delivered

- `homie usage summary`.
- JSON output for token/cache/cost totals and authoritative billing flag.
- Filters for session/provider/model/time window.
- CLI E2E over seeded storage usage records.

## Gate Results

| Gate | Command | Result |
|------|---------|--------|
| CLI usage summary | `cargo test -p homie-cli --test usage_summary_cli -- --nocapture` | pass |
| Build | `cargo check -p homie-storage -p homie-cli` | pass |
| Lint | `cargo clippy -p homie-storage -p homie-cli --all-targets -- -D warnings` | pass |
| Format | `cargo fmt --all -- --check` | pass |

## Remaining Work

- Transcript watcher/parser.
- Pricing table parity.
- Fleet usage merge.
- Usage UI E2E.
