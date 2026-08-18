# OpenSpec 对齐校验报告 — app-sidebar-view-module-split

## 1. 需求 → 任务映射

| PRD 需求项 | OpenSpec Task | 验证 Case |
|-----------|---------------|-----------|
| 抽取纯投影（标题/状态/排序/快捷键/路径/时长） | S1 | C1 |
| 抽取渲染辅助函数 | S2 | C2 |
| 抽取 section 渲染方法 | S3 | C3 |
| 抽取 popover 方法 | S4 | C4 |
| 抽取 dispatch + 测试迁移 + facade 收尾 | S5 | C5 |
| 公共 API 不变 | 全 S | C6 |

## 2. 功能验证 Case 覆盖矩阵

| Case | 描述 | 覆盖需求 | 执行命令 |
|------|------|---------|---------|
| C1 | 纯投影抽取后行为等价 | S1 | `cargo test -p homie-app` |
| C2 | 渲染辅助函数等价 | S2 | `cargo test -p homie-app` |
| C3 | section 渲染等价 | S3 | `cargo test -p homie-app` |
| C4 | popover 方法等价 | S4 | `cargo test -p homie-app` |
| C5 | dispatch + facade + 测试迁移 | S5 | `cargo check && cargo fmt --check && cargo test` |
| C6 | 公共 API 签名/可达性 | 全 S | `cargo check -p homie-app`（root.rs/main.rs/store 编译通过即证） |

## 3. 一致性结论

- 每个 Task 均有明确验收标准 + 关联验证 Case。
- 无重叠、无遗漏；PRD 需求（按职责拆分、单文件不再 4,310 行、公共 API 与行为不变）均被 C1–C6 覆盖。
- 拆解 100% 贴合 PRD，零漏项、零错配。

## 4. 风险与缓解

- 风险：机械移动引入 `pub(crate)` 可见性/`use` 路径错误 → 缓解：每片 `cargo check` 即时反馈。
- 风险：`DraggedSidebarItem(DragItem)` / `DragPreview` 跨子模块构造需字段可见 → 缓解：改为 `pub(crate)` 元组字段/字段，`pub(crate)` 导出。
- 风险：`sections.rs`/`popover.rs`/`commands.rs` 仍较大 → 缓解：已按渲染职责（section/popover/dispatch）归类，纯投影与渲染辅助已全部抽出，符合「逻辑与渲染分离」目标。
