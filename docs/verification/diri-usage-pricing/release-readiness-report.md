# Release Readiness Report: Diri Usage Pricing Estimate

```yaml
change_id: diri-usage-pricing
beads: homie-t3e
status: pass_with_scope_limit
validated_at: 2026-08-08
```

## Delivered

- Diri-compatible `ModelPricing`.
- Claude model matching and cost estimate helper.
- OpenAI/Codex model matching and cost estimate helper.
- Cache read/write 5m/write 1h estimate semantics.
- Unknown model and negative-token safety behavior.

## Gate Results

| Gate | Command | Result |
|------|---------|--------|
| Pricing tests | `cargo test -p homie-llm --test usage_pricing -- --nocapture` | pass |
| homie-llm tests | `cargo test -p homie-llm` | pass |
| Build | `cargo check -p homie-llm` | pass |
| Lint | `cargo clippy -p homie-llm --all-targets -- -D warnings` | pass |
| Format | `cargo fmt --all -- --check` | pass |
| Diff hygiene | scoped `git diff --check` | pass |
| Parity lock | `make parity-lock` | pass |

## Remaining Work

- Transcript watcher/parser that feeds usage records.
- Pricing snapshots persisted into storage.
- Usage UI/fleet merge.
- Provider billed spend reconciliation.
