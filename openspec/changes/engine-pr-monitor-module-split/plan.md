# OpenSpec Plan — engine-pr-monitor-module-split

## 目标

将 `homie/crates/homie-engine/src/pr_monitor.rs`（822 行）按关注点拆分为 4 个聚焦子模块：
`wake.rs`（wake 失效 + 刷新节奏）、`github.rs`（gh 子进程 + JSON 解析）、`monitor.rs`（监控主循环）。
`mod.rs` 保留模块文档 + 子模块声明 + `pub use` 再导出 + 测试模块。所有函数逐字迁移，公共 API
不变，单文件 < 800 行。

## 拆分策略

- 依赖方向：`mod.rs`（facade，`pub use` 再导出 + 测试）→ 各子模块（显式 `use` 引入依赖）。
- `monitor.rs` 依赖 `wake.rs`（`initial_sweep_delay`/`poll_interest`/`PollInterest`/
  `RefreshState`/`PendingWake`/`PrMonitorWake`）与 `github.rs`（`resolve_gh`/`fetch`）。
- 公共函数保持 `pub` 并经 `pub use` 再导出；私有辅助仅在跨模块/测试需要时提升为 `pub(crate)`。
- 测试模块（9 个用例，另含 `control::tests` 中 1 个关联用例）逐字迁移至 `mod.rs`，
  `next_refresh_delay`/`initial_sweep_delay`/`poll_interest`/`PollInterest`/`RefreshState`
  以 `#[cfg(test)]` 限定再导出。
- 无生产代码语义变更，无外部 API 泄漏。

## 交付切片

- T1：函数/常量边界扫描，定位 18 函数 + 8 常量的闭合边界。
- T2：生成 `wake.rs`/`github.rs`/`monitor.rs` 子模块。
- T3：重建 `mod.rs`（文档 + 声明 + 再导出 + 测试），删除旧 `pr_monitor.rs`，编译验证。
- T4：全量验证（fmt/check/clippy/build/workspace-check/test）。
- T5：code review + release readiness 证据。

## 验证

见 `tasks.md`，最终证据写入 `docs/verification/engine-pr-monitor-module-split/`。
