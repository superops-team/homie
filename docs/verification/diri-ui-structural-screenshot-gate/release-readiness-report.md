# Release Readiness Report: Diri UI Structural Screenshot Gate

```yaml
change_id: diri-ui-structural-screenshot-gate
beads: homie-nnp
status: pass
validated_at: 2026-08-07
```

## 1. Source

- PRD: `prd-spec/features/diri-ui-structural-screenshot-gate/2026-08-07-diri-ui-structural-screenshot-gate-design.md`
- OpenSpec: `openspec/changes/diri-ui-structural-screenshot-gate/`
- Functional cases: `docs/verification/diri-ui-structural-screenshot-gate/functional-cases.md`
- Beads: `homie-nnp`

## 2. Delivered

- Upgraded `make ui-screenshot-gate` to decode Homie and Diri PNG evidence and validate structural workbench metrics.
- Added left/center/right luma contrast checks for sidebar, terminal/workbench, and inspector zones.
- Added normalized vertical edge peak checks for workbench separators.
- Updated visual evidence report with structural metrics and limitations.

## 3. Gate Results

| Gate | Command | Result |
|------|---------|--------|
| Syntax | `python3 -m py_compile scripts/quality/check-ui-screenshot-evidence.py` | pass |
| UI screenshot | `make ui-screenshot-gate` | pass |
| Diff hygiene | scoped `git diff --check` | pass |
| Parity lock | `make parity-lock` | pass_with_remaining_gaps |

## 4. Parity Status

This slice strengthens evidence for `UI-001..UI-009`, but does not mark any row `implemented`.

Remaining blockers:

- GPUI interaction E2E for sidebar hover/drag/reorder.
- Terminal selection/find/scrollback live interaction E2E.
- Quick Open, Settings, Worktrees, Notifications, Inspector tab interaction E2E.
- Pixel/perceptual Diri side-by-side visual parity threshold.

## 5. Risk

Risk is low. The code is a local quality script with no runtime or product logic impact. The main residual risk is false confidence if structural screenshot metrics are treated as full UI parity; this report and the parity lock explicitly prevent that.
