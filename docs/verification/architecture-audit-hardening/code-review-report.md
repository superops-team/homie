# Brooks 架构审计治理 Code Review 报告

## 1. 审查范围

- 文件/模块：
  - `prd-spec/refactors/architecture-audit-hardening/2026-08-14-architecture-audit-hardening-design.md`
  - `docs/verification/architecture-audit-hardening/spec-review-report.md`
  - `docs/verification/architecture-audit-hardening/functional-cases.md`
  - `docs/verification/architecture-audit-hardening/functional-verification-report.md`
  - `openspec/changes/architecture-audit-hardening/plan.md`
  - `openspec/changes/architecture-audit-hardening/tasks.md`
  - `openspec/changes/architecture-audit-hardening/alignment-report.md`
- 变更类型：Phase 0 planning / documentation only。
- 调用链/数据流：Brooks audit finding → PRD requirement → functional case → OpenSpec task → verification evidence。
- 参考规则：`AGENTS.md` Required Development Workflow、`docs/development/quality-gates.md`、`review-spec` 报告。

## 2. 旧问题复核

| ID/标题 | 位置 | 状态 | 依据 |
|---|---|---|---|
| parent PRD 范围过大 | PRD 1.2.1 / 1.3 | fixed | 已明确 `homie-om7` 只关闭 Phase 0，Phase 1-4 使用 child Beads |
| 缺少存量 PRD 映射 | PRD 1.5 | fixed | 已列出 GPUI、large-module 和 protocol fixtures PRD 的关系 |
| 缺少功能验证 Case | PRD 3.5 / functional-cases | fixed | 已设计 FC-01 到 FC-08 |
| ControlServer 拆分缺少兼容约束 | PRD Phase 3 | fixed | 已新增 method name / JSON shape / error code / event order 不变约束 |

## 3. Findings

未发现需要继续修复的 P0/P1/P2 问题。

## 4. 对抗式复盘

- 反例/边界：如果执行者把 `homie-om7` 当代码重构任务，会被 PRD 关闭口径、FC-02 和 OpenSpec Phase boundary 拦住。
- 反例/边界：如果 Phase 0 混入生产代码，FC-08 的 `git diff --name-only -- homie/crates Sources Tests` 会失败。
- 反例/边界：如果 OpenSpec 忘记映射 Brooks finding 或 FC case，FC-06 会失败。
- 撤回或降级：没有需要撤回的 finding。
- 新增修复：无。

## 5. 修复摘要

- 本轮 code review 未新增修复。
- 保持 Phase 0 为 documentation/planning only。

## 6. 验证结果

| 命令 | 结果 | 说明 |
|------|------|------|
| `rg -n "2\\.1|2\\.2|2\\.3|2\\.4|Symptom|Source|Consequence|Remedy" ...` | 通过 | Brooks findings 可追踪 |
| `rg -n "关闭口径|homie-om7|不直接承诺|child Beads|Phase 1-4" ...` | 通过 | parent / child 边界明确 |
| `rg -n "gpui-architecture-hardening|gpui-large-module-test-boundaries|protocol-contract-golden-fixtures|specs/gpui-shell|specs/engine-session-runtime" ...` | 通过 | 存量 PRD/spec 关系明确 |
| `test -s openspec/changes/architecture-audit-hardening/{plan.md,tasks.md,alignment-report.md}` | 通过 | OpenSpec 结构完整 |
| `git diff --check ...` | 通过 | 文档静态门禁通过 |
| `git diff --name-only -- homie/crates Sources Tests` | 通过 | 无生产代码改动 |

## 7. 剩余风险

- Phase 1-4 尚未启动；本文只证明后续治理路线可执行。
- 后续代码重构必须新建 child Beads 和独立 verification evidence，不能复用本轮 Phase 0 结果替代实现验证。
