# GPUI 架构硬化第一阶段功能验证报告

## 1. 结论

`homie-4lu` Phase 0/1 功能验证通过。当前变更完成合同基线、OpenSpec
拆解、review inventory、child beads 和 worktree shared target 验证；未实施 Phase 2-5 的 GPUI 代码重构，符合 PRD 非目标。

## 2. Case 执行结果

| Case | 目标 | 状态 | 证据 |
|------|------|------|------|
| FC-01 | PRD/spec 已收敛为 program + Phase 0/1 关闭边界 | pass | `fc-01-prd-scope.log` |
| FC-02 | `AGENTS.md` worktree shared target 规则存在 | pass | `fc-02-agents-worktree-cache.log` |
| FC-03 | `AGENTS.md` 要求读取的 docs 已补齐 | pass | `fc-03-required-docs.log` |
| FC-04 | GPUI 长期 component specs 已补齐 | pass | `fc-04-gpui-specs.log` |
| FC-05 | review inventory 使用结构化 schema 并覆盖 P1-P10 | pass | `fc-05-review-inventory.log` |
| FC-06 | OpenSpec plan/tasks/alignment 已创建并关联功能验证 Case | pass | `fc-06-openspec-alignment.log` |
| FC-07 | 后续 child beads 已创建并链接到 `homie-4lu` | pass | `fc-07-child-beads.log` |
| FC-08 | Active Homie worktrees 共享同一个 Cargo target | pass | `fc-08-worktree-target.log` |
| FC-09 | 当前 change 未包含 Phase 2-5 GPUI 代码重构 | pass | `fc-09-no-code-scope-creep.log` |
| FC-10 | 静态文档质量检查 | pass | `fc-10-diff-check.log` |

## 3. Child Beads

| Bead | change_id | 目的 |
|------|-----------|------|
| `homie-9w2` | `gpui-lifecycle-task-ownership` | GPUI task/subscription 生命周期所有权硬化 |
| `homie-yon` | `gpui-utility-surfaces-first-slice` | UtilitySurfaces 首个生命周期切片 |
| `homie-0aj` | `gpui-ui-primitives-a11y` | 语义 UI primitives 与 a11y 合同落地 |
| `homie-4fx` | `gpui-render-path-purity` | render 路径纯化与 sidebar projection 外移 |
| `homie-mpc` | `gpui-visual-platform-gates` | 视觉与平台偏好验证门禁 |

## 4. 边界确认

- 当前变更只交付 Phase 0/1 合同基线。
- `git diff --name-only -- homie/crates/homie-app homie/crates/homie-ui` 无输出，证明未混入 GPUI app/ui 代码变更。
- 后续 Phase 2-5 必须通过 child beads 独立 PRD/OpenSpec/verification 执行。

## 5. 风险

- Child beads 已创建但尚未编写各自 PRD/spec；这是后续开发入口，不是当前闭环缺口。
- 当前验证未运行 Rust/Swift 编译，因为本变更是文档/spec-only；静态门禁使用 `git diff --check`。
