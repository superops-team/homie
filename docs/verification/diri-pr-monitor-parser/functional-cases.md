# Functional Cases: Diri PR Monitor Parser

```yaml
change_id: diri-pr-monitor-parser
beads: homie-jkj
```

## FC-DPRM-001: PR monitor parser fixtures

- Command: `cargo test -p homie-runtime --test pr_monitor -- --nocapture`
- Expected:
  - Full gh payload fixture parses state/title/review/merge/check/comment stats.
  - Minimal payload accepts missing optional fields.
  - Garbage payload is rejected.
  - Overall rollup ladder matches Diri.
  - GraphQL review thread fixture parses resolved/total counts.
  - PR URL coordinates parse owner/repo/number.

## FC-DPRM-002: Runtime check

- Command: `cargo check -p homie-runtime`
- Expected: exit code 0.

## FC-DPRM-003: Runtime lint

- Command: `cargo clippy -p homie-runtime --all-targets -- -D warnings`
- Expected: exit code 0.

## FC-DPRM-004: Hygiene and parity lock

- Commands:
  - `git diff --check -- crates/homie-runtime prd-spec/features/diri-pr-monitor-parser openspec/changes/diri-pr-monitor-parser docs/verification/diri-pr-monitor-parser`
  - `make parity-lock`
- Expected:
  - diff check passes.
  - parity lock moves `ART-003` to partial after parser evidence is recorded, not implemented.
