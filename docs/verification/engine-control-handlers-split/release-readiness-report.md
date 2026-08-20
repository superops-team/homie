# Release Readiness — engine-control-handlers-split

## 变更摘要

将 `homie/crates/homie-engine/src/control/handlers.rs`（1173 行）的 `impl ControlServer` 方法按关注点
拆分为 10 个聚焦子模块。44 个方法 + `new_record` 自由函数逐字迁移，可见性从 `pub(super)` 统一提升为
`pub(crate) fn`（crate 内部，无外部泄漏）。私有辅助 `schedule_initial_prompt` 保持 `fn`；
`resume_spec` 因被 `control::tests` 引用提升为 `pub(crate) fn`。`new_record` 自由函数保留在
`mod.rs` 并继续经 `control.rs` 重新导出。生产代码语义零变更。

## 拆分结果

| 文件 | 行数 | 职责 |
|------|------|------|
| `mod.rs` | 69 | facade：共享 imports + 子模块声明 + `new_record` |
| `agent.rs` | 111 | hook_report / agent_readiness / environment_refresh_path / project_add / session_read_diff |
| `governor.rs` | 46 | governor_configure / session_hibernate / session_wake |
| `handshake.rs` | 40 | hello / session_capabilities |
| `host.rs` | 156 | host_sync_prefs / host_initialize / host_list_directories / host_locate_repo / resolve_host / hosts_file |
| `migrate.rs` | 163 | session_migrate |
| `resume.rs` | 154 | session_resume / remote_resume_spec / session_resume_from_history / resume_spec / session_reopen_last |
| `session.rs` | 229 | session_list / session_send_text / session_resize / session_read_screen / session_read_scrollback / session_read_scrollback_cells / session_kill / session_remove / session_rename / session_mark_seen / client_set_active / session_archive / session_unarchive / publish_updated / session_history |
| `spawn.rs` | 178 | session_spawn / session_spawn_remote / schedule_initial_prompt / browser_call / launch_context |
| `worktree.rs` | 64 | worktree_overview / worktree_create / worktree_list / worktree_remove |

全部单文件 < 800 行（最大 `session.rs` 229 行）。

## 方法逐字迁移

- 44 个方法 + `new_record` 自由函数全部逐字迁移，控制通道语义零变更。
- 原 `pub(super) fn` 方法因拆分到 `control/handlers/` 子目录后需跨兄弟子模块调用，统一提升为
  `pub(crate) fn`。
- 私有辅助 `schedule_initial_prompt` 保持 `fn`；`resume_spec` 因被 `control::tests` 引用提升为
  `pub(crate) fn`。
- `pub(crate)` 仍为 crate 内部可见，无外部 API 泄漏，无生产代码语义变更。
- 各子模块以 `use super::*;` 引入 `ControlServer` 与共享依赖。

## 验证证据

| 项 | 结果 |
|----|------|
| `cargo fmt --all --check` | ✅ 通过 |
| `cargo check -p homie-engine --offline` | ✅ 通过（0 警告） |
| `cargo clippy -p homie-engine --all-targets --offline` | ✅ 0 警告 |
| `cargo build -p homie-engine --offline` | ✅ 通过 |
| `cargo check --workspace --offline` | ✅ 通过（homie-engine + homie-app） |
| `cargo test -p homie-engine --offline` | ✅ 303 passed / 0 failed / 3 ignored |
| 引用方零改动 | ✅ 仅 `handlers.rs` → `handlers/` 目录 + 文档 |

## 已知限制 / 延期

- 无。后续候选（按行数排序）：`control/runtime.rs`、`registry.rs` 等其它大模块。
