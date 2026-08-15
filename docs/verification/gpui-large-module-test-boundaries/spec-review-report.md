# GPUI 大模块纯逻辑测试边界 Spec Review Report

## 1. 总体结论

- 可行性：高。
- 最大风险：把“纯逻辑测试边界”执行成一次横跨 sidebar、terminal、inspector、root 的大规模文件搬迁。
- 推荐方向：首阶段只抽一个高变更 UI 行为，优先 Sidebar new-agent picker；先 characterization tests，再抽无 GPUI 上下文依赖的逻辑模块。

## 2. 问题清单

| 优先级 | 维度 | 问题 | 影响 | 修复状态 |
|---|---|---|---|---|
| P0 | 范围控制 | 原 PRD 列出多个模块和候选拆分，未强制首阶段只做一个单元 | 容易形成不可 review 的大 PR | 已修复：首阶段只选择一个行为单元，优先 Sidebar new-agent picker |
| P0 | 与既有变更重叠 | 未说明与已完成 GPUI child slices 的边界 | 可能重复修改 shortcut rank、UtilitySurfaces、Button primitive 或 RootView lifecycle | 已修复：新增与 `gpui-architecture-hardening` 及 child slices 的关系约束 |
| P1 | 可测试性 | “纯逻辑”缺少静态判定标准 | 抽出的模块可能仍依赖 `Window/Context/Entity` | 已修复：新增 `rg` 静态检查，禁止 GPUI 上下文进入逻辑模块 |
| P1 | 回归风险 | 文件搬迁可能保持编译但改变视觉/交互 | 用户可见行为回退 | 已修复：要求先 characterization tests，再抽模块，并跑 preview/手动交互证据 |
| P2 | 指标偏差 | 行数下降可能被误当主要目标 | 为降行数而增加抽象噪声 | 已修复：验收以行为测试和 review 面缩小为准，行数仅作辅助观察 |

## 3. 整改后的完善方案

首阶段从 GPUI 大模块中抽出一个行为单元，建议 `Sidebar new-agent picker`。目标是把 local/remote、agent readiness、目录选择取消、spawn option 输出等决策从 render 层挪到普通 Rust 逻辑模块，并用 focused tests 固化现有行为。

非目标：不重做设计系统，不新增 UI primitive，不整体重写 sidebar/terminal/inspector，不修改 UtilitySurfaces task lifecycle，不重做 RootView subscription/task ownership。

## 4. 分层任务拆解

| 层级 | 任务 | 交付物 | 依赖 | 优先级 |
|---|---|---|---|---|
| Spec | 补 OpenSpec alignment，声明不重叠已完成 child slices | `openspec/changes/gpui-large-module-test-boundaries/*` | 本报告 | P0 |
| Characterization | 固化选定行为的现有输入/输出 | focused tests | OpenSpec | P0 |
| Logic | 抽出无 GPUI 依赖的逻辑模块 | module + tests | Characterization | P0 |
| Render | render 层调用逻辑模块 | scoped app diff | Logic | P0 |
| Regression | preview/manual/full dev smoke evidence | verification report | Render | P1 |

## 5. 测试规划

| 类型 | 覆盖点 | 关键用例 | 执行阶段 |
|---|---|---|---|
| Static | 无 GPUI 上下文依赖 | `rg "Window|Context|Entity|cx\\.|div\\(" <logic-module>` 无命中 | 开发中 |
| Unit | picker decision | local/remote/readiness/cancel/spawn options | 开发中 |
| App focused | render 接入不回退 | sidebar tests 或 app focused tests | 准出前 |
| Visual/manual | 用户工作流 | 创建 session、切换 session、打开 inspector | 准出前 |

## 6. 开发排期

| 阶段 | 顺序 | 工作项 | 风险与缓冲 | 验收物 |
|---|---|---|---|---|
| Phase 0 | 先 | OpenSpec + overlap audit | 避免重复已完成 GPUI slices | alignment report |
| Phase 1 | 次 | characterization tests | 行为不清时先补 fixture | test logs |
| Phase 2 | 后 | 抽逻辑模块 + render 接入 | 不追求大搬迁 | verification report |

## 7. 待确认问题

- 首个行为单元是否确定为 Sidebar new-agent picker。推荐该方向，因为它 ROI 高，且与已完成 child slices 重叠最少。
