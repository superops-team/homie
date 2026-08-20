# OpenSpec Plan — app-store-tests-split

## 目标

将 `homie/crates/homie-app/src/store/tests.rs`（1,658 行）机械拆分为目录化聚焦测试子模块：
`switcher.rs`（2）、`ordering.rs`（12 + `rows` 辅助）、`events.rs`（12）、`sessions.rs`（13）、
`attention.rs`（2）、`hosts.rs`（11）、`runtime.rs`（1），`mod.rs` 保留 facade（imports + 共享辅助 +
模块声明）。53 个测试全部逐字迁移，测试行为零变更，单文件 < 800 行。

## 拆分策略

- 依赖方向：`mod.rs`（父，`store::tests`）→ 各测试子模块（经 `use super::*;` 访问 imports 与共享
  辅助 `id`/`pid`/`session`/`project`/`hydrated`/`drain`）。
- 共享辅助下沉到 `mod.rs`；`rows` 辅助仅 `ordering.rs` 使用，随其下沉。
- `store` 模块成员引用（`SpawnOptions`/`WorktreeSpawn`/`RepoTarget`/`StoreRuntime`/
  `DirectoryListingState`/`AUXILIARY_TERMINAL_TITLE`）由 `super::X` 改写为 `super::super::X`；
  `crate::sidebar::move_to_end` 由 `super::super::sidebar::move_to_end` 改写为
  `super::super::super::sidebar::move_to_end`。
- 逐字迁移：测试函数体、断言、`#[test]` 属性均保持不变。
- 无生产代码变更。

## 交付切片

- T1：编写 Rust 括号扫描器，逐字解析测试函数与辅助函数边界（含 `#[test]` 属性）。
- T2：生成 `tests/mod.rs` facade（imports + 共享辅助 + 模块声明）与 7 个子模块文件。
- T3：修正子模块内 `super::` 层级引用。
- T4：删除旧 `tests.rs`，全量验证。
- T5：code review + release readiness 证据。

## 验证

见 `tasks.md`，最终证据写入 `docs/verification/app-store-tests-split/`。
