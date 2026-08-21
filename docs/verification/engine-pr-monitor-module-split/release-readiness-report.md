# Release Readiness — engine-pr-monitor-module-split

## 变更摘要

将 `homie/crates/homie-engine/src/pr_monitor.rs`（822 行）按关注点拆分为 4 个聚焦子模块。
18 个函数 + 8 个常量逐字迁移，公共 `pub fn` 保持 `pub` 并经 `mod.rs` 的 `pub use` 再导出
（公共 API 不变）。`resolve_gh` 因跨模块共享提升为 `pub(crate) fn`；`initial_sweep_delay`/
`poll_interest`/`PollInterest`/`RefreshState`/`PendingWake`/`next_refresh_delay` 因跨模块或测试
引用提升为 `pub(crate)`。测试模块（9 用例）逐字迁移。生产代码语义零变更。

## 拆分结果

| 文件 | 行数 | 职责 |
|------|------|------|
| `mod.rs` | 193 | facade：文档 + 子模块声明 + `pub use` 再导出 + 测试模块 |
| `wake.rs` | 160 | PrMonitorWake / PollInterest / RefreshState / PendingWake / WakeInner + 4 常量 |
| `github.rs` | 267 | resolve_gh / fetch / run_gh / pr_coordinates / parse_threads / parse / parse_github_date / now |
| `monitor.rs` | 248 | spawn_pr_monitor / sweep / next_refresh_delay / status_materially_same |

全部单文件 < 800 行（最大 `github.rs` 267 行）。

## 函数逐字迁移

- 18 个函数 + 8 个常量全部逐字迁移，PR 轮询与 GitHub 交互语义零变更。
- 公共 `pub fn`（`spawn_pr_monitor`/`fetch`/`pr_coordinates`/`parse_threads`/`parse` +
  `pub struct PrMonitorWake`）保持 `pub`，经 `pub use` 再导出。
- `resolve_gh` 原私有 `fn`，因 `monitor.rs` 跨模块共享，提升为 `pub(crate) fn`。
- `initial_sweep_delay`/`poll_interest`/`PollInterest`/`RefreshState`/`PendingWake` 因
  `monitor.rs` 跨模块使用，提升为 `pub(crate)`；`next_refresh_delay` 因测试引用，提升为
  `pub(crate)`。
- `RefreshState` 字段因测试结构体字面量 + `..Default::default()`，提升为 `pub(crate)`。
- `run_gh`/`parse_github_date`/`now`/`sweep`/`status_materially_same` 保持私有。
- 测试辅助 `next_refresh_delay`/`initial_sweep_delay`/`poll_interest`/`PollInterest`/
  `RefreshState` 以 `#[cfg(test)]` 限定再导出，不泄漏到生产编译。

## 验证证据

| 项 | 结果 |
|----|------|
| `cargo fmt --all --check` | ✅ 通过 |
| `cargo check -p homie-engine --offline` | ✅ 通过（0 警告） |
| `cargo clippy -p homie-engine --all-targets --offline` | ✅ 0 警告 |
| `cargo build -p homie-engine --offline` | ✅ 通过 |
| `cargo check --workspace --offline` | ✅ 通过 |
| `cargo test -p homie-engine --offline` | ✅ unit 303 passed / 0 failed / 3 ignored + 集成测试全绿 |
| 引用方零改动 | ✅ 仅 `pr_monitor.rs` → `pr_monitor/` 目录 + 文档 |

## 已知限制 / 延期

- 无。后续候选（按行数排序）：`remote/manager.rs`（1099 行）、`mcp/host.rs`（863 行）等。
