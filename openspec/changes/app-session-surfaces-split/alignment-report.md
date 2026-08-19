# OpenSpec 对齐校验报告 — app-session-surfaces-split

## 1. 需求 → 任务映射

| PRD 需求项 | OpenSpec Task | 验证 Case |
|-----------|---------------|-----------|
| FR-1 overview 渲染下沉 overview.rs | T3 | C3 |
| FR-2 overview 卡片/行渲染下沉 overview_card.rs | T4 | C4 |
| FR-3 switcher 渲染下沉 switcher.rs | T2 | C2 |
| FR-4 投影自由函数下沉 projection.rs（switcher_key re-export） | T1 | C1 |
| FR-5 测试下沉 tests.rs | T5 | C5 |
| FR-6 mod.rs 收尾为 facade，< 800 行 | T5 | C5 |
| FR-7 行为不变（fmt/check/test 全绿） | T6 | C6/C7 |

## 2. 功能验证 Case 覆盖矩阵

| Case | 描述 | 覆盖需求 | 执行命令 |
|------|------|---------|---------|
| C1 | 投影函数迁移后编译等价 + switcher_key 路径不变 | FR-4 | `cargo check -p homie-app` |
| C2 | switcher 渲染迁移后编译等价 | FR-3 | `cargo check -p homie-app` |
| C3 | overview chrome 渲染迁移后编译等价 | FR-1 | `cargo check -p homie-app` |
| C4 | overview 卡片/行渲染迁移后编译等价 | FR-2 | `cargo check -p homie-app` |
| C5 | 测试下沉 + mod.rs facade 收尾，行数 < 800 | FR-5/FR-6 | `wc -l` |
| C6 | 行为不变（测试全绿） | FR-7 | `cargo test -p homie-app` |
| C7 | 格式 / 静态检查通过 | FR-7 | `cargo fmt --check && cargo clippy -p homie-app --all-targets` |

## 3. 一致性结论

- 每个 Task 均有明确验收标准 + 关联验证 Case。
- 无重叠、无遗漏；FR-1~FR-7 均被 C1~C7 覆盖。
- 拆解 100% 贴合 PRD，零漏项、零错配。

## 4. 风险与缓解

- 风险：`switcher_key` 被 `terminal_pane/mod.rs` 通过 `crate::session_surfaces::switcher_key`
  引用 → 缓解：T1 明确在 `mod.rs` re-export `pub(crate) use projection::switcher_key;`。
- 风险：机械移动引入 `use` 路径错误 → 缓解：T1~T5 每片 `cargo check` 即时反馈。
- 风险：`ui_agent_kind` 在 `terminal_pane`/`sidebar`/`inspector` 各有同名实现，
  勿误以为可全局复用 → 缓解：本变更仅在 `session_surfaces` 内部下沉，不改外部。
