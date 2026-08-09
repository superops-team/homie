# Homie Reference Parity V1 Spec Review Report

```yaml
change_id: reference-parity-v1
report_type: spec-review
status: pass
beads: homie-h7n
source_prd: prd-spec/features/reference-parity-v1/2026-08-05-reference-parity-v1-design.md
openspec_plan: openspec/changes/reference-parity-v1/plan.md
openspec_tasks: openspec/changes/reference-parity-v1/tasks.md
alignment_report: openspec/changes/reference-parity-v1/alignment-report.md
reviewed_at: 2026-08-05
```

## 1. Review Scope

本次 review 只审查需求与规格，不审查实现代码。范围包括：

- Reference 参考功能盘点是否进入 Homie PRD。
- PRD 是否符合 Homie 项目规范、文档边界和 Beads/OpenSpec 工作流。
- OpenSpec 是否把 PRD 要求映射到可执行任务、组件 spec 和验证路径。
- Homie 安全、credential custody、LLM proxy、context/memory/task 自有能力是否没有被 Reference parity 复刻绕开。

## 2. Source Evidence

| Evidence | Used for |
|----------|----------|
| `/Users/bytedance/workspace/github/reference/README.md` | 产品定位、进程架构、agent 支持 |
| `/Users/bytedance/workspace/github/reference/reference/README.md` | GPUI app、remote host、sidebar preview |
| `/Users/bytedance/workspace/github/reference/reference/PLAN.md` | 完整 UI design、键盘映射、产品 surfaces、性能预算 |
| `/Users/bytedance/workspace/github/reference/reference/PORT.md` | Rust engine port、PTY、status、registry、control socket |
| `/Users/bytedance/workspace/github/reference/reference/NODE.md` | remote node、accounts、handoff、fleet usage |
| `/Users/bytedance/workspace/github/reference/reference/PACKAGING.md` | app packaging、signing、notarization |
| `/Users/bytedance/workspace/github/reference/reference/UPDATING.md` | updater trust model and old-to-new acceptance |
| `/Users/bytedance/workspace/github/reference/reference/PERF.md` | packaged perf gate |
| `/Users/bytedance/workspace/github/reference/Sources/<ReferenceProtocol>/Methods.swift` | protocol methods and events |
| `/Users/bytedance/workspace/github/reference/Sources/<ReferenceMCP>/Tools.swift` | MCP tool surface |
| `/Users/bytedance/workspace/github/reference/Sources/<reference-cli>/<ReferenceCLI>.swift` | CLI, hook/notify, mcp-stdio, doctor |
| `/Users/bytedance/workspace/github/reference/Sources/<ReferenceCore>/Resources/manifests/*.json` | agent catalog and status rules |
| `/Users/bytedance/workspace/github/reference/reference/crates/<reference-app>/src/*` | app shell, settings, terminal, sidebar, inspector, usage |
| `/Users/bytedance/workspace/github/reference/reference/crates/<reference-ui>/src/*` | design tokens, icons, brand marks, status glyph |

## 3. Checks

| Check | Result | Notes |
|-------|--------|-------|
| PRD is in Chinese and under `prd-spec/features/` | pass | `prd-spec/features/reference-parity-v1/2026-08-05-reference-parity-v1-design.md` |
| Beads stable `change_id` exists | pass | `homie-h7n`, metadata `reference-parity-v1` |
| PRD is self-contained | pass | Includes background, goals, non-goals, scenarios, FRs, component impact, test plan, acceptance |
| Reference product capabilities are represented | pass | FR-1 through FR-20 cover local, UI, automation, remote, ship, security |
| Reference design is represented | pass | FR-7 covers tokens, surfaces, glyphs, keyboard map, workbench, inspector |
| Homie security model is preserved | pass | FR-12 and FR-19 make virtual key and credential custody mandatory |
| Component spec impact is explicit | pass | PRD section 6 and alignment report section 2 list required specs |
| OpenSpec plan exists | pass | `openspec/changes/reference-parity-v1/plan.md` |
| OpenSpec tasks exist | pass | `openspec/changes/reference-parity-v1/tasks.md` |
| OpenSpec alignment exists and covers all FRs | pass | `openspec/changes/reference-parity-v1/alignment-report.md` |
| No implementation gate is falsely marked pass | pass | Implementation and release gates remain future tasks |

## 4. Issues Found

No blocking spec issues found.

Non-blocking follow-ups:

| Follow-up | Reason | Owner task |
|-----------|--------|------------|
| Split umbrella task into dependent Beads before coding | Reference parity is too large for one implementation branch | T-001 |
| Create missing component specs | Several specs are planned but not yet present as files | T-001 and affected implementation tasks |
| Define Homie remote-node credential policy before node work | Reference assumes node-local provider login; Homie requires virtual-key-safe custody | T-014 |
| Establish real signing/notarization environment | Updater release gate cannot be proven without real artifacts | T-016 |

## 5. Gate Decision

Decision: pass

Reason:

- The design document accurately captures the Reference parity objective and Homie's stricter architecture/security boundaries.
- OpenSpec artifacts provide a concrete path from requirements to implementation and verification.
- Remaining risks are implementation planning risks, not blockers for accepting the PRD/spec draft.

