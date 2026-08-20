# PRD — app-store-tests-split

## 背景

`homie/crates/homie-app/src/store/tests.rs`（1,658 行）是会话存储的单文件测试集：53 个测试用例
与 7 个辅助函数混合在一个文件里，覆盖 daemon 事件 hydrate/handle_event、会话生命周期
（close/resume/auto_resume/process exit）、sidebar 排序/pin/collapse/projection、switcher/overview、
attention/needs_input、主机/默认主机/repo targeting/migration/sync prefs、以及 `StoreRuntime` 惰性
运行时。单文件远超 800 行阈值，与 `store/` 各业务子模块（events/sessions/ordering/switcher/hosts/
runtime）无法一一对应，定位与维护成本高，违背仓库「组件模块化、关注点清晰」原则。

## 目标

将 `store/tests.rs` 机械拆分为目录化聚焦测试子模块，与 `store/` 业务子模块的职责边界对齐，
测试行为零变更（53 个测试全部保留且逐字迁移），公共 API 零变更，单文件 < 800 行。

## 非目标

- 不新增/删除/改写任何测试断言或测试场景。
- 不改动 `SessionStore`/`StoreRuntime` 任何生产代码。
- 不新增抽象、不新增配置、不做向后兼容层。
- 不合并或重命名测试辅助函数（`id`/`pid`/`session`/`project`/`hydrated`/`drain`/`rows`）。

## 用户场景

1. 开发者定位「daemon 事件 hydrate/handle_event」相关测试时，聚焦在 `tests/events.rs`。
2. 开发者定位「sidebar 排序/pin/collapse/projection」相关测试时，聚焦在 `tests/ordering.rs`。
3. 开发者定位「会话生命周期 close/resume/auto_resume/process exit」相关测试时，聚焦在
   `tests/sessions.rs`。
4. 开发者定位「主机/默认主机/repo targeting/migration/sync prefs」相关测试时，聚焦在 `tests/hosts.rs`。
5. 开发者定位「switcher/overview」相关测试时，聚焦在 `tests/switcher.rs`。
6. 开发者定位「attention/needs_input」相关测试时，聚焦在 `tests/attention.rs`。
7. 开发者定位「StoreRuntime 惰性运行时」相关测试时，聚焦在 `tests/runtime.rs`。

## 模块划分方案

```text
store/tests/
├── mod.rs         facade：imports + 共享辅助（id/pid/session/project/hydrated/drain）+ 模块声明
├── switcher.rs    2 个测试：switcher/overview 交互
├── ordering.rs    12 个测试 + rows 辅助：sidebar 排序/pin/collapse/projection
├── events.rs      12 个测试：hydrate/handle_event/select/click/mru/residency/事件发布
├── sessions.rs    13 个测试：close/resume/auto_resume/process exit/aux terminal/directory listing
├── attention.rs   2 个测试：attention rollup/needs_input 通知
├── hosts.rs       11 个测试：主机/默认主机/spawn 定位/repo targeting/migration/sync prefs/prefs round trip
└── runtime.rs     1 个测试：StoreRuntime 惰性运行时
```

## 可见性设计

- `store/mod.rs` 中 `#[cfg(test)] mod tests;` 声明不变，`crate::store::tests` 模块路径不变。
- 共享辅助（`id`/`pid`/`session`/`project`/`hydrated`/`drain`）下沉到 `tests/mod.rs`，各测试子模块
  经 `use super::*;` 访问；`rows` 辅助仅 `ordering.rs` 使用，随其下沉。
- 子模块内对 `store` 模块成员的引用（`SpawnOptions`/`WorktreeSpawn`/`RepoTarget`/`StoreRuntime`/
  `DirectoryListingState`/`AUXILIARY_TERMINAL_TITLE`）由原 `super::X`（指向 `store`）机械改写为
  `super::super::X`；`crate::sidebar::move_to_end` 由原 `super::super::sidebar::move_to_end` 改写为
  `super::super::super::sidebar::move_to_end`。
- 无生产代码可见性变更，无 `pub` 泄漏。

## 影响面

- 仅 `store/tests.rs` → `store/tests/` 目录化迁移，生产代码与其它测试零改动。
- 无持久化/schema/协议变更。

## 测试计划

- `cargo check -p homie-app --offline` 全绿。
- `cargo fmt --all --check` 通过。
- `cargo clippy -p homie-app --all-targets --offline` 0 警告。
- `cargo build -p homie-app --offline` 通过。
- `cargo test -p homie-app --offline` 全绿（沙箱内 2 个 `daemon_launch` socket bind EPERM 属预期）。
- `cargo test -p homie-app --offline store::` 53 passed / 0 failed。
- `wc -l` 每文件 < 800 行。

## 验收标准

- C1：目录化成功，旧单文件（1,658 行）拆为 8 个文件（最大 395 行）。
- C2：53 个测试全部保留且逐字迁移，测试行为零变更。
- C3：单文件 < 800 行。
- C4：编译、fmt、clippy、test 全绿。
- C5：共享辅助正确下沉，`#[test]` 属性完整保留。
- C6：release readiness 证据写入 `docs/verification/app-store-tests-split/`。

## Beads

- `homie-3n9`
