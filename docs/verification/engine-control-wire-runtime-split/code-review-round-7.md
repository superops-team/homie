# Engine Control Wire/Runtime Split S4-b(2) 复核报告

## 1. 结论

S4-b 第二个增量（prompt-injection → control/inject.rs）无 P0/P1 问题，质量达标。

## 2. 复核要点

- `control/inject.rs`（408 行）承接初始 prompt 注入领域：`prepare_agent_input` 及其
  全部辅助函数（Claude workspace-trust 自动确认、initial prompt 注入、echo 验证、
  就绪/屏幕稳定等待、`with_session`、`EchoOutcome`、注入窗口常量）。
- 函数体一字未改，仅 `prepare_agent_input` 提升为 `pub(super)`；
  `is_claude_workspace_trust_screen` 保持 `pub(super)` 供 tests 引用。
- `inject.rs` 仅依赖 `Arc<Mutex<Registry>>` + session id，无 ControlServer/socket/
  transport 依赖，属纯 session-input 领域。
- `session_spawn` / `session_spawn_remote` 经 `use super::inject::prepare_agent_input`
  调用，行为等价；`handlers.rs` 1,977 → 1,582 行（-395）。
- 移除 handlers.rs 残留 unused `Mutex`/`Instant` import，`cargo check` 0 warning。

## 3. 门禁

- `cargo test -p homie-engine`：278 lib（3 ignored）+ 集成全绿，0 failed。
  （首次全量跑出现 `checkpoint::adoption_seeds_from_the_checkpoint_not_the_raw_tail`
  偶发时序失败，单测重跑 3/3 通过，与本切片无关。）
- `cargo fmt --check` clean；`cargo check` 0 warning。

## 4. 结论

可提交。
