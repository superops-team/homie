# Diri 7ba3407 Parity Rebaseline Release Readiness Report

```yaml
change_id: diri-7ba3407-parity-rebaseline
report_type: release-readiness
status: pass
risk_tier: tier-1-documentation
beads: homie-t3u
source_prd: prd-spec/features/diri-7ba3407-parity-rebaseline/2026-08-08-diri-7ba3407-parity-rebaseline-design.md
openspec: openspec/changes/diri-7ba3407-parity-rebaseline/
overall_diri_parity: partial
product_test_baseline: fail
```

## 1. Scope

本报告只准出 Wave 0 的需求重基线、长期组件合同和开发规划。它不准出任何尚未实现的 Diri 功能，也不改变 `overall_diri_parity: partial`。

交付物：

- 中文主 PRD；
- Diri `7ba3407` 冻结能力矩阵；
- 14 个既有组件 spec 修订；
- 新增 runtime client transport 组件 spec；
- OpenSpec proposal、design、八个 capability specs、plan、tasks、alignment；
- 16 维 spec review；
- parity lock 中四个错误完成态修正。

## 2. Beads

| Field | Value |
|-------|-------|
| Bead | `homie-t3u` |
| Status during verification | `in_progress` |
| Final status | `closed` after all Wave 0 gates passed |
| Change ID | `diri-7ba3407-parity-rebaseline` |
| Spec ID | `prd-spec/features/diri-7ba3407-parity-rebaseline/2026-08-08-diri-7ba3407-parity-rebaseline-design.md` |
| Expected close condition | Wave 0 spec, alignment and evidence gates pass |

## 3. Required Gates

| Gate | Command | Exit | Result | Evidence |
|------|---------|------|--------|----------|
| OpenSpec completeness | `openspec status --change diri-7ba3407-parity-rebaseline` | 0 | pass: 4/4 | proposal/design/specs/tasks complete |
| OpenSpec strict validation | `openspec validate diri-7ba3407-parity-rebaseline --strict` | 0 | pass | change valid |
| Traceability | shell count/assert for FR, mappings, specs, requirements/scenarios | 0 | pass | 16 FR, 16 mapped, 8 specs, 34 requirements, 61 scenarios, 15 component refs |
| Spec review | review-spec 16 dimensions | 0 | pass | `spec-review-report.md` |
| Parity lock | `make parity-lock` | 0 | pass | lock valid; 40 incomplete rows correctly remain |
| Module inventory | `make module-inventory-check` | 0 | pass | existing inventory structurally valid |
| Component mapping | `make spec-diri-mapping-check` | 0 | pass | component Diri mappings valid |
| Rust format | `cargo fmt --all -- --check` | 0 | pass | no Rust format change |
| Diff whitespace | `git diff --check` | 0 | pass | no whitespace errors |
| Repository pre-commit | `.githooks/pre-commit` | 0 | pass | secret safety hook passed |
| New-doc illegal status scan | `rg` status vocabulary scan | 0 | pass | no illegal new status |
| New-doc secret literal scan | `rg` credential/private-key patterns | 0 | pass | no matches |

## 4. Product Test Baseline

Product tests are not a required pass gate for this documentation-only change because no product code was modified. The audit immediately before this change established the following current baseline and it is intentionally preserved as a blocker for child waves:

| Command | Result | Failure |
|---------|--------|---------|
| `cargo test --workspace` | fail | app source-text sidebar regression |
| `cargo test --workspace --exclude homie-app` | fail | MCP unsupported error expected `-32601`, actual `-32000` |
| `cargo test --workspace --exclude homie-app --exclude homie-cli` | fail | live PTY and holder adoption return `detached`, expected `running` |

These failures are recorded in `docs/research/diri-7ba3407-capability-matrix.md`. Wave 0 pass must not be interpreted as a green product workspace.

## 5. Dependency And Security Review

| Topic | Result |
|-------|--------|
| New Rust/Swift/npm dependencies | none |
| Database migration | none |
| Runtime behavior change | none |
| Credential handling change | contract only; no secret material added |
| External network action | none |
| Existing user changes reverted | none |
| Historical PRD/evidence overwritten | none; history retained |

## 6. Residual Risks

| Risk | Status | Owner |
|------|--------|-------|
| Runtime/holder tests fail | blocked for implementation parity | Wave 1B |
| Production client remains in-process | partial | Wave 1A |
| UI direct storage/local fake actions remain | partial | Wave 2 |
| MCP browser/test and omitted tools remain | partial/missing | Wave 3B |
| Remote node, LLM proxy, updater and signed package remain incomplete | partial | Wave 4/5 |
| Historical evidence contains illegal statuses | partial | owning child changes and final validator |

## 7. Gate Decision

Decision: pass

Reason:

- Wave 0 documentation, traceability, OpenSpec and safety gates passed.
- The report explicitly preserves product parity as partial and records current test failures.
- No implementation work is authorized without a new child Bead, Chinese PRD and OpenSpec.
