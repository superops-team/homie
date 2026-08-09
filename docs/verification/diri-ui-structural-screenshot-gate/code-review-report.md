# Code Review Report: Diri UI Structural Screenshot Gate

```yaml
change_id: diri-ui-structural-screenshot-gate
beads: homie-nnp
status: pass
reviewed_at: 2026-08-07
```

## 1. Scope

- `scripts/quality/check-ui-screenshot-evidence.py`
- `docs/verification/diri-ui-screenshot/visual-verification-report.md`
- `prd-spec/features/diri-ui-structural-screenshot-gate/`
- `openspec/changes/diri-ui-structural-screenshot-gate/`
- `docs/verification/diri-ui-structural-screenshot-gate/`

## 2. Findings

| Severity | Category | Location | Evidence and impact | Status |
|----------|----------|----------|---------------------|--------|
| medium | Correctness | `check-ui-screenshot-evidence.py` | Initial gate only used file size/nonzero bytes, which could pass a static one-pane screenshot and fail to protect Diri workbench parity. | fixed: decode PNG scanlines, compute luma bands and separator peaks, and require structural workbench contrast. |
| low | Complexity | `check-ui-screenshot-evidence.py` | PNG filter handling can be error-prone if implemented as ad hoc byte checks. | fixed: isolate `reconstruct_png`, `paeth`, `band_luma`, `vertical_edge_peaks`, and `require_structural_workbench`. |
| low | Scope | release evidence | Structural screenshot checks can be overclaimed as UI completion. | fixed: evidence states UI rows remain partial and interaction E2E is still required. |

## 3. Brooks-Lint Review

Mode: PR Review  
Scope: focused screenshot gate script and verification docs  
Health Score: 94/100

No critical decay risk remains. The script is a small deterministic quality gate with clear helper boundaries. The remaining risk is intentional verification scope: structural screenshot metrics are stronger than nonblank screenshots but are not a substitute for real UI interaction testing.

## 4. Test Quality Review

Mode: Test Quality Review  
Scope: screenshot evidence gate

Test Suite Map:

```text
Unit tests: not applicable; this is a standalone quality script
Integration tests: make ui-screenshot-gate against real Homie/Diri PNG evidence
E2E tests: not part of this slice
```

No mock abuse or brittle test coupling was found. The quality gate exercises real screenshot files and fails with concrete metric output.

## 5. Verification

| Command | Result |
|---------|--------|
| `python3 -m py_compile scripts/quality/check-ui-screenshot-evidence.py` | pass |
| `make ui-screenshot-gate` | pass |
| scoped `git diff --check` | pass |
| `make parity-lock` | pass_with_remaining_gaps |
