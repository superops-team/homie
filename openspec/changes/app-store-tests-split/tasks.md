# OpenSpec Tasks — app-store-tests-split

## T1 编写括号扫描器解析边界

- [x] Rust 括号扫描器：跳过字符串/字符/注释/raw string，逐字解析测试函数与辅助函数边界。
- [x] `#[test]` 属性随测试函数一并保留。
- 验收：53 个测试 + 7 个辅助全部解析，无缺失/重复。关联 C2/C5。

## T2 生成 tests/ 目录

- [x] `mod.rs`：imports + 共享辅助（id/pid/session/project/hydrated/drain）+ 模块声明。
- [x] `switcher.rs`（2）、`ordering.rs`（12 + rows）、`events.rs`（12）、`sessions.rs`（13）、
  `attention.rs`（2）、`hosts.rs`（11）、`runtime.rs`（1）。
- 验收：53 个 `#[test]` 完整保留。关联 C1/C5。

## T3 修正 super:: 层级引用

- [x] `SpawnOptions`/`WorktreeSpawn`/`RepoTarget`/`StoreRuntime`/`DirectoryListingState`/
  `AUXILIARY_TERMINAL_TITLE` → `super::super::X`。
- [x] `sidebar::move_to_end` → `super::super::super::sidebar::move_to_end`。
- 验收：`cargo check -p homie-app --offline` 全绿。关联 C2/C4。

## T4 删除旧文件 + 全量验证

- [x] 删除 `store/tests.rs`。
- [x] `cargo fmt --all --check`
- [x] `cargo check -p homie-app --offline`
- [x] `cargo clippy -p homie-app --all-targets --offline`
- [x] `cargo build -p homie-app --offline`
- [x] `cargo test -p homie-app --offline`
- [x] `cargo test -p homie-app --offline store::`
- 验收：全部通过，53 store 测试全绿。关联 C3/C4。

## T5 code review + release readiness

- [x] code review：拆分边界清晰、测试逐字迁移、无行为变更、`#[test]` 完整、单文件 < 800 行。
- [x] release readiness 证据写入 `docs/verification/app-store-tests-split/`。
- 验收：通过。关联 C6。
