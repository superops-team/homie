# OpenSpec Plan — app-store-mod-split

## 目标

将 `homie/crates/homie-app/src/store/mod.rs`（2,434 行）机械拆分为目录化聚焦子模块：
`lifecycle.rs`（构造 + 基础访问器）、`hosts.rs`（主机/catalog/migration/sync/repo/directory/prefs）、
`ordering.rs`（sidebar 排序/pin/collapse/projection）、`events.rs`（hydrate/handle_event/upsert）、
`switcher.rs`（switcher/overview/snapshot）、`sessions.rs`（会话生命周期）、`navigation.rs`（导航
reconcile）、`runtime.rs`（StoreRuntime + run_effects）。`mod.rs` 保留 facade（prelude + 自由函数 +
StoreRuntime 再导出 + prefs_path_in_home）。公共 API 与运行时行为零变更，引用方零改动，单文件
< 800 行。

## 拆分策略

- 依赖方向：`mod.rs`（父）→ 各子模块（经 `impl super::SessionStore` / `impl super::StoreRuntime`
  跨模块实现）；`runtime.rs` 定义 `StoreRuntime` 并由 `mod.rs` 经 `pub use runtime::StoreRuntime;`
  再导出。
- 跨子模块互调采用 `pub(super)`（仅对父模块 `store` 可见），共 19 个私有方法升级为 `pub(super)`
  （见 PRD 可见性设计），`runtime.rs` 的 `tasks` 字段升级为 `pub(super)` 供 `store::tests` 访问。
- 自由函数（`attention_rank`/`reconcile_order`/`retain_live`/`toggle_vec_member`/`now_millis`）与
  常量 `UI_PUBLISH_INTERVAL` 保留在 `mod.rs`，各子模块经 `use super::*;` 访问。
- 无 `pub` 可见性泄漏到 crate 外。

## 交付切片

- T1：抽取构造 + 基础访问器 → `lifecycle.rs`。
- T2：抽取主机/catalog/migration/sync/repo/directory/prefs → `hosts.rs`。
- T3：抽取 sidebar 排序/pin/collapse/projection → `ordering.rs`。
- T4：抽取 hydrate/handle_event/upsert → `events.rs`。
- T5：抽取 switcher/overview/snapshot → `switcher.rs`。
- T6：抽取会话生命周期 → `sessions.rs`。
- T7：抽取导航 reconcile → `navigation.rs`。
- T8：抽取 StoreRuntime + run_effects → `runtime.rs`，`mod.rs` 收尾 facade。
- T9：清理跨模块可见性 + 全量验证 + code review + release readiness。

## 验证

见 `tasks.md`，最终证据写入 `docs/verification/app-store-mod-split/`。
