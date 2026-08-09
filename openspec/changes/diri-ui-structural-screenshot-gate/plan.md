# OpenSpec Plan: Diri UI Structural Screenshot Gate

> Change ID: `diri-ui-structural-screenshot-gate`  
> Beads: `homie-nnp`  
> Source PRD: `prd-spec/features/diri-ui-structural-screenshot-gate/2026-08-07-diri-ui-structural-screenshot-gate-design.md`

## 1. Scope

Upgrade the existing UI screenshot evidence gate from nonblank PNG validation to structural Homie/Diri side-by-side validation. The gate must remain lightweight and deterministic: decode PNGs locally, compute layout metrics, and fail when either screenshot does not resemble a three-zone workbench.

## 2. Modules

| Module | Change |
|--------|--------|
| `scripts/quality/check-ui-screenshot-evidence.py` | Add PNG scanline reconstruction, luma band metrics, vertical edge peaks, and structural assertions |
| `docs/verification/diri-ui-screenshot/visual-verification-report.md` | Record structural comparison gate and limitations |
| `docs/verification/diri-ui-structural-screenshot-gate/` | Store functional case, review, and release evidence |

## 3. Functional Cases

| Case | Purpose | Command |
|------|---------|---------|
| FC-UISG-001 | Structural screenshot gate passes for current Homie/Diri evidence | `make ui-screenshot-gate` |
| FC-UISG-002 | Direct script execution prints metrics and passes | `python3 scripts/quality/check-ui-screenshot-evidence.py` |
| FC-UISG-003 | Scoped whitespace/doc/code gate | `git diff --check -- scripts/quality/check-ui-screenshot-evidence.py docs/verification/diri-ui-screenshot docs/verification/diri-ui-structural-screenshot-gate prd-spec/features/diri-ui-structural-screenshot-gate openspec/changes/diri-ui-structural-screenshot-gate` |
| FC-UISG-004 | Parity lock remains honest and still partial where incomplete | `make parity-lock` |

## 4. Risks

| Risk | Mitigation |
|------|------------|
| Pixel-perfect diff is brittle across resolution/window chrome | Use structural luma/edge metrics instead of per-pixel comparison |
| Gate is too weak and accepts arbitrary nonblank images | Require both band contrast and vertical separator edge peaks |
| Gate is overclaimed as UI completion | Release evidence must keep UI rows `partial` and list interaction E2E gaps |

## 5. Acceptance

- `make ui-screenshot-gate` validates Homie and Diri screenshots with structural metrics.
- Visual report includes the structural gate and known limits.
- `make parity-lock` still reports incomplete UI rows until real interaction E2E exists.
