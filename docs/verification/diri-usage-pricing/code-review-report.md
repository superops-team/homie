# Code Review Report: Diri Usage Pricing Estimate

```yaml
change_id: diri-usage-pricing
beads: homie-t3e
status: pass
reviewed_at: 2026-08-08
```

## Round 1: Explicit Defect Review

| Severity | Category | Location | Finding | Status |
|----------|----------|----------|---------|--------|
| high | Missing behavior | `crates/homie-llm/src/lib.rs` | Homie had usage storage but no Diri-compatible pricing estimate helper. | fixed: added `ModelPricing`, Claude/OpenAI matchers, and estimate functions. |
| high | Pricing correctness | `match_claude`, `match_openai` | Specific model rules must precede generic rules. | fixed and tested for `opus-4-1` and `gpt-5.4-mini`. |
| medium | Cache accounting | estimate functions | Cache read/write rates differ by provider and duration. | fixed and tested for OpenAI cache read and Claude cache write 5m/1h. |

## Round 2: Hidden Risk Review

| Severity | Category | Location | Finding | Status |
|----------|----------|----------|---------|--------|
| medium | Billing semantics | docs/spec | Estimated API-equivalent cost must not be mislabeled as billed spend. | pass: PRD/spec/readiness use estimate language. |
| medium | Input safety | estimate functions | Negative token counts should not reduce cost. | pass: functions clamp token counts with `max(0)`. |
| low | Scope | parity lock | Pricing helper is not transcript watcher/UI/fleet parity. | accepted: USAGE-001 remains `partial`. |

## Verification

| Command | Result |
|---------|--------|
| `cargo test -p homie-llm --test usage_pricing -- --nocapture` | pass |
| `cargo test -p homie-llm` | pass |
| `cargo check -p homie-llm` | pass |
| `cargo clippy -p homie-llm --all-targets -- -D warnings` | pass |
| `cargo fmt --all -- --check` | pass |
| scoped `git diff --check` | pass |
| `make parity-lock` | pass |
