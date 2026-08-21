# OpenSpec Tasks — engine-pr-monitor-module-split

## T1 函数/常量边界扫描

- [x] 定位 18 个函数（initial_sweep_delay/poll_interest/refresh_state_interval/record_result/
  pending_wake_is_empty/wake_session/set_foreground_active/foreground_active/wait/
  spawn_pr_monitor/sweep/next_refresh_delay/status_materially_same/resolve_gh/fetch/run_gh/
  pr_coordinates/parse_threads/parse/parse_github_date/now）+ 8 常量边界，跳过 doc 注释与多行签名。
- 验收：全部解析，无缺失/重复。关联 C2/C5。

## T2 生成子模块

- [x] `wake.rs`（PrMonitorWake + PollInterest + RefreshState + PendingWake + WakeInner + 4 常量）、
  `github.rs`（8 函数）、`monitor.rs`（4 函数 + 4 常量）。
- 验收：函数体逐字迁移，`pub` 保持 `pub`。关联 C1/C5。

## T3 重建 mod.rs + 编译验证

- [x] 移除旧 `pr_monitor.rs`，新增 `pr_monitor/mod.rs`（文档 + 声明 + `pub use` 再导出 + 测试）。
- [x] `cargo check -p homie-engine --offline` 全绿（0 警告）。
- 验收：通过。关联 C2/C4。

## T4 全量验证

- [x] `cargo fmt --all --check`
- [x] `cargo check -p homie-engine --offline`
- [x] `cargo clippy -p homie-engine --all-targets --offline`
- [x] `cargo build -p homie-engine --offline`
- [x] `cargo check --workspace --offline`
- [x] `cargo test -p homie-engine --offline`（unit 303 passed / 0 failed / 3 ignored + 集成测试全绿）
- 验收：全部通过。关联 C3/C4。

## T5 code review + release readiness

- [x] code review：拆分边界清晰、函数逐字迁移、无行为变更、可见性正确、单文件 < 800 行。
- [x] release readiness 证据写入 `docs/verification/engine-pr-monitor-module-split/`。
- 验收：通过。关联 C6。
