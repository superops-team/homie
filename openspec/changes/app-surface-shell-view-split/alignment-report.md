# OpenSpec 对齐校验报告 — app-surface-shell-view-split

## 1. 需求 → 任务映射

| PRD 需求项 | OpenSpec Task | 验证 Case |
|-----------|---------------|-----------|
| FR-1 应用自身设置下沉 settings_view.rs | T3 | C3 |
| FR-2 远端主机管理下沉 hosts_view.rs | T2 | C2 |
| FR-3 通用 UI 原语下沉 widgets.rs | T1 | C1 |
| FR-4 view.rs 收尾为 facade，< 800 行 | T4 | C4 |
| FR-5 行为不变（fmt/check/test 全绿） | T5 | C5/C6 |

## 2. 功能验证 Case 覆盖矩阵

| Case | 描述 | 覆盖需求 | 执行命令 |
|------|------|---------|---------|
| C1 | UI 原语迁移后编译等价 | FR-3 | `cargo check -p homie-app` |
| C2 | 远端主机管理迁移后编译等价 | FR-2 | `cargo check -p homie-app` |
| C3 | 应用设置迁移后编译等价 | FR-1 | `cargo check -p homie-app` |
| C4 | view.rs facade 收尾，行数 < 800 | FR-4 | `wc -l` |
| C5 | 行为不变（测试全绿） | FR-5 | `cargo test -p homie-app` |
| C6 | 格式 / 静态检查通过 | FR-5 | `cargo fmt --check && cargo check -p homie-app` |

## 3. 一致性结论

- 每个 Task 均有明确验收标准 + 关联验证 Case。
- 无重叠、无遗漏；FR-1~FR-5 均被 C1~C6 覆盖。
- 拆解 100% 贴合 PRD，零漏项、零错配。

## 4. 风险与缓解

- 风险：机械移动引入 `pub(super)` 可见性 / `use` 路径错误 → 缓解：T1~T4 每片 `cargo check` 即时反馈。
- 风险：`tests.rs` 对 `setting_row` 的导入路径变化 → 缓解：T1 明确改为 `use super::widgets::setting_row;`。
- 风险：跨模块方法调用（`render_settings` 被 view.rs 调用、`remote_settings` 被 settings_view.rs 调用）
  → 缓解：仅 `render_settings` / `remote_settings` 升为 `pub(super)`。
