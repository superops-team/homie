# Functional Cases: Diri UI Structural Screenshot Gate

```yaml
change_id: diri-ui-structural-screenshot-gate
beads: homie-nnp
```

## FC-UISG-001: Make target validates structural screenshots

- Command: `make ui-screenshot-gate`
- Expected:
  - Homie screenshot decodes as PNG.
  - Diri screenshot decodes as PNG.
  - Both screenshots pass size and nonblank checks.
  - Both screenshots pass three-zone structural checks.
  - Output includes structural metrics.

## FC-UISG-002: Direct script validates and prints metrics

- Command: `python3 scripts/quality/check-ui-screenshot-evidence.py`
- Expected:
  - Exit code 0.
  - Output includes `homie screenshot structural ok`.
  - Output includes `diri screenshot structural ok`.

## FC-UISG-003: Scoped diff check

- Command: `git diff --check -- scripts/quality/check-ui-screenshot-evidence.py docs/verification/diri-ui-screenshot docs/verification/diri-ui-structural-screenshot-gate prd-spec/features/diri-ui-structural-screenshot-gate openspec/changes/diri-ui-structural-screenshot-gate`
- Expected:
  - Exit code 0.

## FC-UISG-004: Parity lock remains honest

- Command: `make parity-lock`
- Expected:
  - Exit code 0.
  - Output still lists remaining incomplete rows until real UI interaction E2E is complete.
