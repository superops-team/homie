# GPUI Visual And Platform Gates

## 1. Purpose

GPUI changes that affect layout, focus, controls, material, motion, or window
behavior need runtime evidence. Compiler success alone is not visual proof.

## 2. Required Matrix

For visual or interaction changes, record:

- preview scenario: `typical`, `stress`, `empty`, or `artifacts`;
- appearance: `system`, `light`, or `dark`;
- reduced motion: on/off;
- keyboard-only path exercised;
- window size class: default and narrow when relevant;
- evidence path under `docs/verification/<change-id>/`.

## 3. Command Entry

Use:

```bash
homie/scripts/visual-gate.sh --dry-run
```

Remove `--dry-run` only when the local environment can launch the GPUI app.

## 4. Evidence Rules

- Screenshot or recording is required for visual layout/material changes.
- Command logs are sufficient for dry-run gate planning changes.
- Any unverified platform or preference must be listed in the verification report.
