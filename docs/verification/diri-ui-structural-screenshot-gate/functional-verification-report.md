# Functional Verification Report: Diri UI Structural Screenshot Gate

```yaml
change_id: diri-ui-structural-screenshot-gate
beads: homie-nnp
status: pass
validated_at: 2026-08-07
```

## Summary

This slice upgrades `make ui-screenshot-gate` from a nonblank PNG check to a structural Homie/Diri screenshot comparison. It validates that both screenshots contain a Diri-style workbench skeleton with left sidebar, center terminal/workbench, and right inspector zones.

It does not complete UI parity. `UI-001..UI-009` remain `partial` until real GPUI interaction E2E and fuller visual parity checks are implemented.

## Results

| Case | Command | Result | Evidence |
|------|---------|--------|----------|
| FC-UISG-001 | `make ui-screenshot-gate` | pass | Homie and Diri structural metrics printed; visual report accepted |
| FC-UISG-002 | `python3 scripts/quality/check-ui-screenshot-evidence.py` | pass | Script prints `homie screenshot structural ok` and `diri screenshot structural ok` |
| FC-UISG-003 | `git diff --check -- scripts/quality/check-ui-screenshot-evidence.py docs/verification/diri-ui-screenshot docs/verification/diri-ui-structural-screenshot-gate prd-spec/features/diri-ui-structural-screenshot-gate openspec/changes/diri-ui-structural-screenshot-gate` | pass | No whitespace errors |
| FC-UISG-004 | `make parity-lock` | pass_with_remaining_gaps | Lock remains valid and still lists incomplete rows |
| Syntax | `python3 -m py_compile scripts/quality/check-ui-screenshot-evidence.py` | pass | Script compiles |

## Structural Metrics

| Screenshot | Size | left/center/right luma | Luma gaps | vertical edge peaks |
|------------|------|------------------------|-----------|---------------------|
| Homie window | 2336 x 1602 | 183.21 / 65.94 / 148.79 | left-center 117.28, right-center 82.86 | 0.269, 0.711, 0.950 |
| Diri reference | 3456 x 2234 | 166.21 / 65.83 / 224.21 | left-center 100.37, right-center 158.38 | 0.745, 0.144, 0.425, 0.575 |

## Remaining Gaps

- Real interaction E2E for sidebar hover/drag, terminal selection, quick open, settings, worktrees, notifications, and inspector tabs is still pending.
- Pixel-perfect or perceptual visual diff is not part of this slice.
- `make parity-lock` correctly keeps UI rows partial.
