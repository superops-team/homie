# Release Readiness — app-store-tests-split

## 变更摘要

将 `homie/crates/homie-app/src/store/tests.rs`（1,658 行）机械拆分为 7 个聚焦测试子模块 + facade
（`mod.rs`）。53 个测试用例与 7 个辅助函数逐字迁移，测试行为零变更，生产代码零改动。

## 拆分结果

| 文件 | 行数 | 职责 |
|------|------|------|
| `mod.rs` | 94 | facade：imports + 共享辅助（id/pid/session/project/hydrated/drain）+ 模块声明 |
| `switcher.rs` | 57 | switcher/overview 交互（2 个测试） |
| `ordering.rs` | 351 | sidebar 排序/pin/collapse/projection（12 个测试 + rows 辅助） |
| `events.rs` | 297 | hydrate/handle_event/select/click/mru/residency/事件发布（12 个测试） |
| `sessions.rs` | 395 | close/resume/auto_resume/process exit/aux terminal/directory listing（13 个测试） |
| `attention.rs` | 53 | attention rollup/needs_input 通知（2 个测试） |
| `hosts.rs` | 381 | 主机/默认主机/spawn 定位/repo targeting/migration/sync prefs/prefs round trip（11 个测试） |
| `runtime.rs` | 12 | StoreRuntime 惰性运行时（1 个测试） |

全部单文件 < 800 行（最大 `sessions.rs` 395 行）。

## 测试逐字迁移

- 53 个测试函数与 7 个辅助函数（`id`/`pid`/`session`/`project`/`hydrated`/`drain`/`rows`）全部
  逐字迁移，断言与测试场景零变更。
- 53 个 `#[test]` 属性完整保留。
- `store` 模块成员引用由 `super::X` 机械改写为 `super::super::X`（`SpawnOptions`/`WorktreeSpawn`/
  `RepoTarget`/`StoreRuntime`/`DirectoryListingState`/`AUXILIARY_TERMINAL_TITLE`）；
  `crate::sidebar::move_to_end` 由 `super::super::sidebar::move_to_end` 改写为
  `super::super::super::sidebar::move_to_end`。
- 共享辅助下沉 `mod.rs`，各子模块经 `use super::*;` 访问；`rows` 随 `ordering.rs` 下沉。
- 无生产代码变更。

## 验证证据

| 项 | 结果 |
|----|------|
| `cargo fmt --all --check` | ✅ 通过 |
| `cargo check -p homie-app --offline` | ✅ 通过 |
| `cargo clippy -p homie-app --all-targets --offline` | ✅ 0 警告 |
| `cargo build -p homie-app --offline` | ✅ 通过 |
| `cargo test -p homie-app --offline` | ✅ 301 passed / 2 failed（沙箱内 2 个 `daemon_launch` socket bind EPERM 失败属预期） |
| `cargo test -p homie-app --offline store::` | ✅ 53 passed / 0 failed |
| 生产代码零改动 | ✅ `git status` 仅改 `store/tests.rs` → `store/tests/` + 文档 |

## 沙箱测试说明

沙箱内 `cargo test -p homie-app` 中 2 个 `daemon_launch` 测试因 fixture daemon 的 socket bind
返回 `PermissionDenied`（EPERM）而失败，属沙箱网络限制预期，与本次拆分无关。拆分涉及的
`store` 相关测试（53 个）全部通过。

## 已知限制 / 延期

- 无。后续候选（按行数排序）：`sidebar/popover.rs`（1,249 行）、`sidebar/sections.rs`（1,202 行）、
  `root/mod.rs`（1,190 行）、`terminal_pane/view.rs`（863 行）等。
