# OpenSpec 对齐校验报告 — app-root-store-deepening

## 1. 需求 → 任务映射

| PRD 需求项 | OpenSpec Task | 验证 Case |
|-----------|---------------|-----------|
| 抽取 shortcut 纯策略（NewSessionShortcut/new_session_shortcut/session_navigation_delta） | S1 | C1 |
| 抽取 seam 动画 + 拖拽边缘 marker | S2 | C2 |
| 抽取 auxiliary terminal 编排 | S3 | C3 |
| 抽取 render 方法 + 渲染辅助 | S4 | C4 |
| 抽取内联测试 + facade 收尾 | S5 | C5 |
| store 投影单点化评估（F8） | S6 | C6 |
| 公共 API 与视觉/行为不变 | 全 S | C7 |

## 2. 功能验证 Case 覆盖矩阵

| Case | 描述 | 覆盖需求 | 执行命令 |
|------|------|---------|---------|
| C1 | shortcut 策略抽取后行为等价 | S1 | `cargo test -p homie-app root::tests` |
| C2 | seam 动画/marker 等价 | S2 | `cargo check -p homie-app` |
| C3 | auxiliary 编排等价 | S3 | `cargo check -p homie-app` |
| C4 | render 方法等价 | S4 | `cargo check -p homie-app` |
| C5 | facade + 测试迁移 + 全量回归 | S5 | `cargo check && cargo fmt --check && cargo test` |
| C6 | store 投影单点结论 | S6 | 代码核查（engine identity vs app UI projection） |
| C7 | 公共 API 签名/可达性 | 全 S | `cargo check -p homie-app`（main.rs/store 编译通过即证） |

## 3. 一致性结论

- 每个 Task 均有明确验收标准 + 关联验证 Case。
- 无重叠、无遗漏；PRD 需求（按职责拆分、单文件不再 2,130 行、投影单点化评估、公共 API 与行为不变）均被 C1–C7 覆盖。
- 拆解 100% 贴合 PRD，零漏项、零错配。

## 4. 风险与缓解

- 风险：机械移动引入 `pub(crate)` 可见性/`use` 路径错误 → 缓解：每片 `cargo check` 即时反馈。
- 风险：`RootView` 字段跨子模块访问需可见 → 缓解：字段统一 `pub(crate)`，子模块 `use super::*` 引入。
- 风险：`seams.rs` 的 `Context/IntoElement/Render/Window` 等 gpui 导入不完整 → 缓解：编译器逐项反馈补齐。
- 风险：F8 被误判为「需大改 store」→ 缓解：PRD 明确 S6 为「评估」，结论为职责正交、无需改动。
