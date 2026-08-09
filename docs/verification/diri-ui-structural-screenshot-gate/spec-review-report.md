# Spec Review Report: Diri UI Structural Screenshot Gate

```yaml
change_id: diri-ui-structural-screenshot-gate
beads: homie-nnp
status: pass
reviewed_at: 2026-08-07
```

## Findings

| Severity | Area | Issue | Resolution |
|----------|------|-------|------------|
| P1 | Verification strength | A nonblank screenshot gate can accept static or wrong screenshots. | Added FR-2/FR-3 structural luma and separator-edge assertions. |
| P1 | Overclaim risk | A screenshot structure gate could be mistaken for completed UI parity. | PRD/OpenSpec/release evidence explicitly keep UI rows partial. |
| P2 | Fragility | Pixel-perfect diff would be unstable across window sizes and macOS chrome. | Gate uses resolution-independent luma bands and normalized edge peaks. |

## Decision

The spec is implementable and aligned with the Diri parity lock. It closes a verification gap without changing the user-facing UI or claiming full parity.
