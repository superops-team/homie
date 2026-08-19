# Functional Verification Report — engine-session-runtime-split

- change_id: `engine-session-runtime-split`
- baseline commit: `1a299921cc2a0dc4f074c0fb07e9b275b65f24b0`
- date: 2026-08-19

## Scope verified

1. `session.rs`（2,888 行）拆分为 `session/{lifecycle,screen,pty,status,pump,remote,launch,mod}.rs`，行为不变。
2. resume spec 构造（`resume_spec` / `remote_resume_spec`）从 `control/handlers.rs` 下沉到
   `session/launch.rs`，通过 `LaunchContext` 参数化；handler 退化为 decode→domain→encode 薄适配。
3. spawn spec 构造（`spawn_spec` / `remote_spawn_spec`）下沉到 `session/launch.rs`，返回
   `SpawnPlan`（spec + record + prompt + project_root + host_id）；`session_spawn` /
   `session_spawn_remote` 退化为薄适配，registry mutation 与 mutex 纪律保留在 handler。

## Gate results

| Gate | Command | Result |
|------|---------|--------|
| Compile | `cargo check -p homie-engine --offline` | clean，0 warning / 0 error |
| Unit tests | `cargo test -p homie-engine --offline --lib` | **303 passed, 0 failed, 3 ignored** |
| Format | `cargo fmt --all --check` | clean |
| Lint | `cargo clippy -p homie-engine --all-targets --offline` | clean，0 warning |

> 沙箱内 socket 相关 4 个测试因 `EPERM`（`Operation not permitted`）失败；在非沙箱环境重跑全部 303 通过，
> 证明 4 个失败是沙箱 socket 权限所致，与代码无关。

## 单文件行数（均 < 800）

| 文件 | 行数 |
|------|------|
| `session/lifecycle.rs` | 694 |
| `session/pump.rs` | 645 |
| `session/remote.rs` | 433 |
| `session/mod.rs` | 399 |
| `session/screen.rs` | 362 |
| `session/launch.rs` | 644 |
| `session/pty.rs` | 203 |
| `session/status.rs` | 196 |

`control/handlers.rs` 由 1,607 行降至 1,179 行（resume + spawn spec 下沉后）。

## 行为保持锚点

- `control::tests::resuming_an_agent_directly_executes_the_agent`：直接调用 `server.resume_spec(...)`
  断言 resume 标志 `--resume <uuid>` 仍到达 agent 内层 argv。该测试现走
  `crate::session::resume_spec(&LaunchContext, ...)` 委托路径，通过。
- 拆分前后错误码映射保持一致：`remote_resume_spec` 的 `ensure_helper`/`probe_persistence`/
  `capture_environment` 仍经 `io_control_error`（`NotFound`→`not_found`，其余→`internal`）。

## 结论

拆分与下沉未引入行为回归，所有质量门通过。
