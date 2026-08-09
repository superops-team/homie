# OpenSpec Plan: Diri Usage Pricing Estimate

```yaml
change_id: diri-usage-pricing
beads: homie-t3e
source_prd: prd-spec/features/diri-usage-pricing/2026-08-08-diri-usage-pricing-design.md
component_specs:
  - specs/llm-proxy/README.md
```

## 1. Scope

Add Diri-compatible API-equivalent pricing estimate helpers to `homie-llm`. This is a pure local pricing slice for USAGE-001.

## 2. Current State

- Homie storage can record/query usage totals.
- Homie CLI can summarize seeded usage records.
- Homie lacks Diri model pricing helpers in `homie-llm`.

## 3. Target State

```text
match_claude/model -> ModelPricing
match_openai/model -> ModelPricing
estimate(tokens, cache tokens) -> Option<f64>
```

## 4. Module Changes

| Module | Change |
|--------|--------|
| `homie-llm` | Add Diri-compatible pricing helper functions. |
| `homie-llm tests` | Add usage pricing tests for model matching, cache rates, unknown models, negative tokens. |
| `specs/llm-proxy` | Document pricing helper as the local API-equivalent estimate source. |
| parity docs | Add USAGE-001 evidence while keeping parser/watcher/UI/fleet gaps partial. |

## 5. Verification

- FC-DUP-001 Claude pricing/cache write.
- FC-DUP-002 OpenAI pricing/model matching.
- FC-DUP-003 unknown/negative safety.
- FC-DUP-004 quality gates.
