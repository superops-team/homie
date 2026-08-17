# Engine Control Wire/Runtime Split S3 复核报告

## 1. 结论

`control/runtime.rs` 生命周期抽取，无 P0/P1 问题，质量达标。

## 2. 复核要点

- `bind` / `daemon_*` / `impl Drop` / `SubscriptionHandle` / `ActiveConnectionGuard` /
  `idle_shutdown_refusal` / 远程恢复函数体一字未改，仅移动 + 可见性限定。
- `SubscriptionHandle::new` 与 `ActiveConnectionGuard::new` 收敛为 `pub(super)`，
  仅 `control` 模块内构造，无跨模块泄漏。
- `daemon_prepare_shutdown`/`daemon_shutdown_if_idle`/`daemon_shutdown` 为 `pub(super)`，
  `control.rs::dispatch` 可访问，外部不可见。
- `runtime.rs` 无 `serve`/`handle_line`/`dispatch`，transport 与 lifecycle 边界清晰。
- 两个测试随迁且改用 `SubscriptionHandle::new` 构造，语义等价。
- `cargo fmt --check` / `cargo check` 干净（移除 unused `Ordering`）。

## 3. 结论

可提交。
