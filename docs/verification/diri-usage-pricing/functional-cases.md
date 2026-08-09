# Functional Cases: Diri Usage Pricing Estimate

```yaml
change_id: diri-usage-pricing
beads: homie-t3e
```

## FC-DUP-001: Claude pricing and cache write estimates

- Command: `cargo test -p homie-llm --test usage_pricing -- claude_estimate_matches_diri_cache_rates --nocapture`
- Expected:
  - `claude-sonnet` input 1M estimates 3.0.
  - cache read uses 0.1x input price.
  - cache write 5m uses 1.25x input price.
  - cache write 1h uses 2.0x input price.

## FC-DUP-002: OpenAI pricing and model matching

- Command: `cargo test -p homie-llm --test usage_pricing -- openai_estimate_matches_diri_model_rules --nocapture`
- Expected:
  - `codex` output/cache estimate matches Diri.
  - `gpt-5.4-mini` uses the specific mini rule before generic `gpt-5.4`.

## FC-DUP-003: Unknown and negative input safety

- Command: `cargo test -p homie-llm --test usage_pricing -- unknown_and_negative_inputs_are_safe --nocapture`
- Expected:
  - unknown model returns None.
  - negative token counts are clamped to zero.

## FC-DUP-004: Quality gates

- Commands:
  - `cargo test -p homie-llm`
  - `cargo check -p homie-llm`
  - `cargo clippy -p homie-llm --all-targets -- -D warnings`
  - `cargo fmt --all -- --check`
  - scoped `git diff --check`
  - `make parity-lock`
- Expected: all commands pass.
