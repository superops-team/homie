# Functional Verification Report: Diri PR Monitor Parser

```yaml
change_id: diri-pr-monitor-parser
beads: homie-jkj
status: pass_with_scope_limit
validated_at: 2026-08-08
```

## Summary

This slice advances `ART-003` from missing to parser/model foundation:

- `homie-runtime::pr_monitor` parses `gh pr view --json` fixture payloads.
- It projects PR state, title, review decision, mergeability, line stats, comment/review counts, checks and fetched timestamp.
- It parses GraphQL review thread resolved/total counts.
- It parses GitHub PR URL coordinates.
- It implements Diri's `overall` rollup ladder.

`ART-003` remains `partial`; background polling, session artifact status wiring, and app chips/popover E2E are still pending.

## Results

| Case | Command | Result |
|------|---------|--------|
| FC-DPRM-001 | `cargo test -p homie-runtime --test pr_monitor -- --nocapture` | pass |
| FC-DPRM-002 | `cargo check -p homie-runtime` | pass |
| FC-DPRM-003 | `cargo clippy -p homie-runtime --all-targets -- -D warnings` | pass |
| FC-DPRM-004 | scoped `git diff --check` | pass |
| FC-DPRM-004 | `make parity-lock` | pass_with_remaining_gaps |

