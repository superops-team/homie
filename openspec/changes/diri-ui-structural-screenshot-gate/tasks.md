# OpenSpec Tasks: Diri UI Structural Screenshot Gate

| Task | Description | Acceptance | Cases |
|------|-------------|------------|-------|
| T-001 | Extend PNG evidence script with reconstructed pixel metrics | Script computes width/height/nonzero bytes/luma bands/edge peaks for Homie and Diri screenshots | FC-UISG-001, FC-UISG-002 |
| T-002 | Add structural assertions | Script fails when a screenshot lacks three-zone luma contrast or separator edge peaks | FC-UISG-001, FC-UISG-002 |
| T-003 | Update visual report | Report records structural comparison gate, current metric summary, and remaining E2E limits | FC-UISG-003 |
| T-004 | Run verification and record evidence | Functional report, code review, release readiness, parity lock honesty | FC-UISG-001..004 |

## Execution Notes

- No UI implementation changes are planned in this slice.
- No parity row may be marked `implemented` by this slice.
- Failure thresholds must be based on current Diri/Homie screenshot structure and remain resolution independent.
