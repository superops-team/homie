# GPUI 架构硬化 Spec Review 报告

## 1. 总体结论

- 可行性：高。
- 最大风险：把 `gpui-architecture-hardening` 当作单个大代码变更执行，导致 RootView、UtilitySurfaces、组件库、render 路径和视觉验证同时改动，形成不可 review 的大 PR。
- 推荐方向：`homie-4lu` 只关闭 Phase 0/1 合同基线；Phase 2-5 的代码整改拆成 child beads 和独立 OpenSpec changes。

## 2. 问题清单与修复记录

| 优先级 | 维度 | 问题 | 影响 | 修复状态 |
|--------|------|------|------|----------|
| P1 | 范围控制 | 原 PRD 把治理文档、RootView 分层、UtilitySurfaces 拆分、a11y 原语和 render 性能放进一个 P1 变更 | 容易形成跨模块大 PR，难以 review 和关闭 | 已修复：文档定位改为 program-level PRD，`homie-4lu` 只关闭 Phase 0/1 |
| P1 | 验收标准 | 原验收用“至少一个”样例承接整个 GPUI 架构硬化目标 | 可能做一个样例就错误关闭 umbrella issue | 已修复：拆成 Program-level 完成条件和 `homie-4lu` 第一阶段关闭条件 |
| P2 | 知识重复 | Worktree target 共享规则在 PRD 和 `AGENTS.md` 中重复表达，并固化机器绝对路径 | 路径变化后文档漂移，个人机器路径被误认为跨机器契约 | 已修复：`AGENTS.md` 是权威入口，PRD 中路径仅为当前机器实例 |
| P2 | 过度设计 | PRD 直接指定 `WorkbenchShell`、`ServiceEventBridge`、`WindowPlacementController` 等模块名 | 实施者可能机械拆文件，制造浅模块 | 已修复：模块名改为候选，并加入边界准入标准 |
| P2 | 可追溯性 | 问题清单缺少后续 inventory schema | OpenSpec 对齐时难判断每个 finding 是否被覆盖 | 已修复：Phase 0 增加 `review-inventory.md` 表格 schema |
| P2 | 执行顺序 | Phase 2-5 没有明确首选 first slice | 多个 agent 可能从不同方向开工，冲突风险高 | 已修复：指定 `UtilitySurfaces history/worktrees task lifecycle` 为首选第一批代码切片 |

## 3. 整改后的完善方案

### 3.1 当前变更边界

`homie-4lu` 的当前开发闭环只交付：

1. `AGENTS.md` worktree shared target 规则。
2. `prd-spec/refactors/gpui-architecture-hardening/2026-08-14-gpui-architecture-hardening-design.md`。
3. `docs/architecture/project-layout.md`。
4. `docs/development/standards.md`。
5. `docs/development/quality-gates.md`。
6. `docs/research/rust-package-selection.md`。
7. `specs/gpui-shell.md`。
8. `specs/gpui-interaction-contract.md`。
9. `specs/ui-components.md`。
10. `docs/verification/gpui-architecture-hardening/review-inventory.md`。
11. `openspec/changes/gpui-architecture-hardening/plan.md`、`tasks.md`、`alignment-report.md`。
12. 后续 child beads 的创建与链接。

### 3.2 当前非目标

- 不直接拆 `RootView`。
- 不直接拆 `UtilitySurfaces`。
- 不新增 `homie-ui` 交互原语实现。
- 不修改 GPUI runtime 行为。
- 不启动视觉重构。

### 3.3 Child changes

Phase 2-5 只通过 child beads 启动，至少包含：

- `gpui-lifecycle-task-ownership`
- `gpui-utility-surfaces-first-slice`
- `gpui-ui-primitives-a11y`
- `gpui-render-path-purity`
- `gpui-visual-platform-gates`

## 4. 分层任务拆解

| 层级 | 任务 | 交付物 | 依赖 | 优先级 |
|------|------|--------|------|--------|
| 流程 | 固化 PRD 修正 | 更新后的 PRD/spec | Beads `homie-4lu` | P1 |
| 验证 | 设计功能验证 Case | `functional-cases.md` | PRD | P1 |
| OpenSpec | 拆解 plan/tasks/alignment | `openspec/changes/gpui-architecture-hardening/*` | PRD + Case | P1 |
| Docs | 补齐架构、标准、质量门禁、包选型 | `docs/architecture/*`、`docs/development/*`、`docs/research/*` | OpenSpec | P1 |
| Specs | 补齐 GPUI shell、interaction、component 合同 | `specs/*.md` | OpenSpec | P1 |
| Evidence | 建立 review inventory | `review-inventory.md` | Docs + Specs | P1 |
| Beads | 创建 child beads | Beads issues + dependencies | PRD | P1 |

## 5. 测试规划

| 类型 | 覆盖点 | 关键用例 | 执行阶段 |
|------|--------|----------|----------|
| 文档一致性 | `AGENTS.md` 要求读取的 docs 存在 | 检查 4 个 docs 文件存在且有内容 | Step 7 |
| Spec 合同 | GPUI shell/interaction/components specs 存在 | 检查 3 个 specs 文件存在且覆盖 PRD 要点 | Step 7 |
| OpenSpec 对齐 | 需求、Case、Task 对齐 | 检查 alignment report 无未覆盖 P1 项 | Step 7 |
| Worktree cache | active worktrees 共享 target | `realpath` 去重只剩一个路径 | Step 7 |
| Child tracking | 后续代码重构未混入当前 change | child beads 存在且链接到 `homie-4lu` | Step 7 |
| 静态质量 | Markdown/空白 | `git diff --check` | Step 10 |

## 6. 开发排期

| 阶段 | 顺序 | 工作项 | 风险与缓冲 | 验收物 |
|------|------|--------|------------|--------|
| Step 1 | 已完成 | Spec review 与 PRD 修正 | 无 | 本报告 |
| Step 2 | 1 | 功能验证 Case 设计 | Case 不能空泛 | `functional-cases.md` |
| Step 3-4 | 2 | OpenSpec 拆解与对齐 | Task 与 Case 漏配 | `plan.md`、`tasks.md`、`alignment-report.md` |
| Step 5-6 | 3 | Docs/specs/review inventory/child beads | 文档过宽或 child 边界不清 | docs/specs/inventory/Beads |
| Step 7 | 4 | 执行功能验证 Case | Beads 命令或 realpath 结果漂移 | execution report |
| Step 8-10 | 5 | 两轮 review + 最终验证 | 文档 diff 空白/对齐问题 | review reports + release readiness |

## 7. 待确认问题

- 是否需要为每个 child bead 立即写独立 PRD/spec，还是先只创建 Beads + 在 OpenSpec 中标记为后续入口。
- 后续 Phase 2 的第一批实现是否固定选择 `UtilitySurfaces history/worktrees task lifecycle`。
