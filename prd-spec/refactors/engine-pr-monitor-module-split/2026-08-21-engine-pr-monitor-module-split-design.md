# PRD — engine-pr-monitor-module-split

## 背景

`homie/crates/homie-engine/src/pr_monitor.rs`（822 行）是 PR 轮询监控的单文件模块，同时承载三类
关注点：wake 失效与刷新节奏（`PrMonitorWake`/`PollInterest`/`RefreshState`/`PendingWake`/
`initial_sweep_delay`/`poll_interest`）、GitHub 交互与解析（`resolve_gh`/`fetch`/`run_gh`/
`pr_coordinates`/`parse_threads`/`parse`/`parse_github_date`/`now`）、以及监控主循环
（`spawn_pr_monitor`/`sweep`/`next_refresh_delay`/`status_materially_same`）。
单文件超过 800 行阈值，且三类关注点彼此独立，阅读与变更成本高，违背仓库「组件模块化、关注点清晰」
原则。

## 目标

将 `pr_monitor.rs` 按关注点拆分为目录化聚焦子模块，公共 API 与运行时行为零变更，引用方零改动，
单文件 < 800 行。

## 非目标

- 不改变任何 PR 轮询、GitHub 请求、wake 失效的运行时行为。
- 不新增抽象、不新增配置、不做向后兼容层。
- 不改动任何函数签名语义。
- 不合并或重命名函数。

## 用户场景

1. 开发者定位「wake 失效与前后台刷新节奏」时，聚焦在 `wake.rs`。
2. 开发者定位「GitHub 子进程与 JSON 解析」时，聚焦在 `github.rs`。
3. 开发者定位「监控主循环与批处理上限」时，聚焦在 `monitor.rs`。

## 模块划分方案

```text
pr_monitor/
├── mod.rs      facade：模块文档 + 子模块声明 + `pub use` 再导出 + 测试
├── wake.rs     PrMonitorWake / PollInterest / RefreshState / PendingWake / WakeInner /
│               initial_sweep_delay / poll_interest（前后台刷新相关常量）
├── github.rs   resolve_gh / fetch / run_gh / pr_coordinates / parse_threads / parse /
│               parse_github_date / now
└── monitor.rs  spawn_pr_monitor / sweep / next_refresh_delay / status_materially_same
```

## 可见性设计

- 所有原 `pub fn`（`spawn_pr_monitor`/`fetch`/`pr_coordinates`/`parse_threads`/`parse`）及
  `pub struct PrMonitorWake` 保持 `pub`，经 `mod.rs` 的 `pub use` 再导出，公共 API 不变。
- `resolve_gh` 原为私有 `fn`，因被 `monitor.rs` 跨模块调用，提升为 `pub(crate) fn`。
- `initial_sweep_delay`/`poll_interest`/`PollInterest`/`RefreshState`/`PendingWake` 被
  `monitor.rs` 跨模块使用，提升为 `pub(crate)`；`next_refresh_delay` 被测试引用，提升为
  `pub(crate)`。
- `RefreshState` 两个字段因测试使用结构体字面量 + `..Default::default()`，提升为 `pub(crate)`。
- `run_gh`/`parse_github_date`/`now`/`sweep`/`status_materially_same` 仅模块内部使用，保持私有。
- 测试专用符号（`next_refresh_delay`/`initial_sweep_delay`/`poll_interest`/`PollInterest`/
  `RefreshState`）在 `mod.rs` 以 `#[cfg(test)]` 限定再导出（仅测试可见，不泄漏到生产编译）。
- 各子模块以显式 `use` 引入所需依赖。

## 影响面

- 仅 `pr_monitor.rs` 的函数/常量拆分为 4 个聚焦子模块，生产代码与其它模块零改动。
- 无持久化/schema/协议变更。

## 测试计划

- `cargo fmt --all --check` 通过。
- `cargo check -p homie-engine --offline` 全绿。
- `cargo clippy -p homie-engine --all-targets --offline` 0 警告。
- `cargo build -p homie-engine --offline` 通过。
- `cargo check --workspace --offline` 全绿。
- `cargo test -p homie-engine --offline` 全绿（unit 303 passed / 0 failed / 3 ignored + 集成测试全绿）。
- `wc -l` 每文件 < 800 行。

## 验收标准

- C1：目录化成功，旧单文件（822 行）拆为 4 子模块 + facade。
- C2：公共 API 不变，引用方零改动。
- C3：单文件 < 800 行。
- C4：编译、fmt、clippy、test 全绿。
- C5：函数逐字迁移，公共可见性不变，私有辅助仅内部提升为 `pub(crate)`。
- C6：release readiness 证据写入 `docs/verification/engine-pr-monitor-module-split/`。

## Beads

- `homie-bd2`
