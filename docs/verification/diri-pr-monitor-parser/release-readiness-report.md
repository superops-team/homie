# Release Readiness Report: Diri PR Monitor Parser

```yaml
change_id: diri-pr-monitor-parser
beads: homie-jkj
status: pass_with_scope_limit
validated_at: 2026-08-08
```

## 1. Source

- PRD: `prd-spec/features/diri-pr-monitor-parser/2026-08-08-diri-pr-monitor-parser-design.md`
- OpenSpec: `openspec/changes/diri-pr-monitor-parser/`
- Functional cases: `docs/verification/diri-pr-monitor-parser/functional-cases.md`
- Beads: `homie-jkj`

## 2. Delivered

- New `homie-runtime::pr_monitor` parser/model module.
- Diri-equivalent `gh pr view --json` parser fixture coverage.
- Review thread GraphQL parser fixture coverage.
- PR coordinate parser and overall rollup ladder.
- Parity lock updated from `missing` to `partial` for `ART-003`.

## 3. Gate Results

| Gate | Command | Result |
|------|---------|--------|
| Parser tests | `cargo test -p homie-runtime --test pr_monitor -- --nocapture` | pass |
| Build | `cargo check -p homie-runtime` | pass |
| Lint | `cargo clippy -p homie-runtime --all-targets -- -D warnings` | pass |
| Diff hygiene | scoped `git diff --check` | pass |
| Parity lock | `make parity-lock` | pass_with_remaining_gaps |

## 4. Parity Status

`ART-003` is now `partial`, not implemented. Remaining work:

- background polling with `gh` budget/TTL;
- applying PR status to session artifacts;
- app terminal/inspector PR chips and checks popover;
- live or fixture E2E covering full session artifact flow.

