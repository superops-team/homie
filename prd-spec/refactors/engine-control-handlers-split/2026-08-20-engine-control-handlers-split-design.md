# PRD — engine-control-handlers-split

## 背景

`homie/crates/homie-engine/src/control/handlers.rs`（1173 行）是控制通道的 God Module：单个文件同时
承载握手、会话 spawn/list/resume、宿主与 worktree 操作、hook 上报、治理（governor）、浏览器调用等
全部 `impl ControlServer` 方法，以及 `new_record` 自由函数。单文件远超 800 行阈值，阅读与变更成本
极高，违背仓库「组件模块化、关注点清晰」原则。

## 目标

将 `handlers.rs` 按关注点拆分为目录化聚焦子模块，公共 API 与运行时行为零变更，引用方零改动，
单文件 < 800 行。

## 非目标

- 不改变任何控制通道方法的运行时行为。
- 不新增抽象、不新增配置、不做向后兼容层。
- 不改动 `ControlServer` 结构体字段、任何方法签名语义。
- 不合并或重命名方法。

## 用户场景

1. 开发者定位「握手」逻辑时，聚焦在 `handshake.rs`。
2. 开发者定位「会话 spawn」逻辑时，聚焦在 `spawn.rs`。
3. 开发者定位「会话 list/resize/read/kill/rename」等时，聚焦在 `session.rs`。
4. 开发者定位「resume / reopen」逻辑时，聚焦在 `resume.rs`。
5. 开发者定位「宿主与 worktree」操作时，聚焦在 `host.rs` / `worktree.rs`。
6. 开发者定位「hook / agent readiness」时，聚焦在 `agent.rs`。
7. 开发者定位「governor / hibernation」时，聚焦在 `governor.rs`。
8. 开发者定位「migrate」时，聚焦在 `migrate.rs`。

## 模块划分方案

```text
control/handlers/
├── mod.rs        facade：共享 imports + 子模块声明 + `new_record`
├── agent.rs      hook_report / agent_readiness / environment_refresh_path / project_add / session_read_diff
├── governor.rs   governor_configure / session_hibernate / session_wake
├── handshake.rs  hello / session_capabilities
├── host.rs       host_sync_prefs / host_initialize / host_list_directories / host_locate_repo / resolve_host / hosts_file
├── migrate.rs    session_migrate
├── resume.rs     session_resume / remote_resume_spec / session_resume_from_history / resume_spec / session_reopen_last
├── session.rs    session_list / session_send_text / session_resize / session_read_screen /
│                 session_read_scrollback / session_read_scrollback_cells / session_kill /
│                 session_remove / session_rename / session_mark_seen / client_set_active /
│                 session_archive / session_unarchive / publish_updated / session_history
├── spawn.rs      session_spawn / session_spawn_remote / schedule_initial_prompt / browser_call / launch_context
└── worktree.rs   worktree_overview / worktree_create / worktree_list / worktree_remove
```

## 可见性设计

- 原 `pub(super) fn` 方法因拆分到 `control/handlers/` 子目录后需跨兄弟子模块调用，统一提升为
  `pub(crate) fn`。`pub(crate)` 仍为 crate 内部可见，无外部 API 泄漏，无生产代码语义变更。
- 私有辅助方法 `schedule_initial_prompt`、`resume_spec` 保持 `fn`（`resume_spec` 因被
  `control::tests` 引用，需提升为 `pub(crate) fn`）。
- `new_record` 自由函数保留为 `pub(crate) fn`，仍被 `control.rs` 经 `pub(crate) use
  handlers::new_record;` 重新导出。
- 各子模块以 `use super::*;` 引入 `ControlServer`、`decode`/`encode`/`poisoned`/`io_control_error`、
  `json` 等共享依赖。

## 影响面

- 仅 `control/handlers.rs` 的方法 + 自由函数拆分为 10 个聚焦子模块，生产代码与其它模块零改动。
- 无持久化/schema/协议变更。

## 测试计划

- `cargo fmt --all --check` 通过。
- `cargo check -p homie-engine --offline` 全绿。
- `cargo clippy -p homie-engine --all-targets --offline` 0 警告。
- `cargo build -p homie-engine --offline` 通过。
- `cargo check --workspace --offline` 全绿。
- `cargo test -p homie-engine --offline` 全绿（303 passed / 0 failed / 3 ignored）。
- `wc -l` 每文件 < 800 行。

## 验收标准

- C1：目录化成功，旧单文件（1173 行）拆为 10 子模块 + facade。
- C2：公共 API 不变，引用方零改动。
- C3：单文件 < 800 行。
- C4：编译、fmt、clippy、test 全绿。
- C5：方法逐字迁移，可见性统一为 `pub(crate)`（私有辅助除外）。
- C6：release readiness 证据写入 `docs/verification/engine-control-handlers-split/`。

## Beads

- `homie-f7u`
