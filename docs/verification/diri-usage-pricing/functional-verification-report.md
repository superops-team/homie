# Functional Verification Report: Diri Usage Pricing Estimate

```yaml
change_id: diri-usage-pricing
beads: homie-t3e
status: pass_with_scope_limit
verified_at: 2026-08-08
```

## RED Evidence

| Case | Command | RED result |
|------|---------|------------|
| FC-DUP-001..003 | `cargo test -p homie-llm --test usage_pricing -- --nocapture` | failed: `claude_estimate`, `openai_estimate`, `match_claude`, and `match_openai` were not exported by `homie-llm`. |

## GREEN Evidence

| Case | Command | Result |
|------|---------|--------|
| FC-DUP-001 | `cargo test -p homie-llm --test usage_pricing -- claude_estimate_matches_diri_cache_rates --nocapture` | pass |
| FC-DUP-002 | `cargo test -p homie-llm --test usage_pricing -- openai_estimate_matches_diri_model_rules --nocapture` | pass |
| FC-DUP-003 | `cargo test -p homie-llm --test usage_pricing -- unknown_and_negative_inputs_are_safe --nocapture` | pass |
| FC-DUP-001..003 | `cargo test -p homie-llm --test usage_pricing -- --nocapture` | pass: 3 passed |
| FC-DUP-004 | `cargo test -p homie-llm` | pass: usage pricing and virtual key tests passed |
| FC-DUP-004 | `cargo check -p homie-llm` | pass |
| FC-DUP-004 | `cargo clippy -p homie-llm --all-targets -- -D warnings` | pass |
| FC-DUP-004 | `cargo fmt --all -- --check` | pass after running `cargo fmt --all` for the new test file |
| FC-DUP-004 | scoped `git diff --check` | pass |
| FC-DUP-004 | `make parity-lock` | pass; remaining unrelated partial rows listed honestly |

## Scope Notes

- Implements Diri-compatible API-equivalent pricing estimate helpers only.
- Does not implement transcript watcher, storage usage write path, usage UI, fleet merge, or billed spend.
