# Code Review Report — engine-session-runtime-split (spawn 下沉)

- change_id: `engine-session-runtime-split`
- Beads: `homie-sx7`
- date: 2026-08-19
- review scope: `session_spawn` / `session_spawn_remote` → `session::spawn_spec` /
  `session::remote_spawn_spec` 下沉（commit `b2d74f2`），及后续去重修复。

## 1. 审查范围

- 新增/修改：`session/launch.rs`（`spawn_spec` / `remote_spawn_spec` / `SpawnPlan` 消费）、
  `session/mod.rs`（`SpawnPlan` 定义 + re-export）、`control/handlers.rs`（handler 薄化 +
  `schedule_initial_prompt`）、`control/tests.rs`（`shell_pty_environment` 测试迁移）。
- 调用链：handler decode → `session::spawn_spec`/`remote_spawn_spec`（domain，含 manifest 查找、
  argv 组装、virtual-key mint、pty 构建）→ `SpawnPlan` → handler `ensure_session_project` /
  `spawn` / `persist` / `publish` / `schedule_initial_prompt`。
- 数据流：`SessionSpawnParams` + caller argv + worktree 解析结果 → `SpawnPlan{spec, record, prompt, project_root, host_id}`。

## 2. 行为一致性核对（第一轮：显性问题）

逐项对照旧 `session_spawn` / `session_spawn_remote`：

- 本地 argv 组装、manifest/descriptor 查找、`returnToLoginShell` shell 包裹、virtual-key mint、
  injection args、pty 构建、`project_id`/`worktree_path`/`git_branch`/`parent`/`title`/
  initial cols/rows、injection env、`agent_session_id`/`transcript_path` — 全部逐行等价。
- 远程 descriptor 由短锁内解析后按值传入，慢速远程调用（ensure_helper / probe_persistence /
  capture_environment）不持有 registry mutex，锁纪律不变。
- `project_root`（本地 `p.cwd`、远程 `captured.cwd`）与 `host_id`（本地 `None`、远程
  `Some(host.id)`）语义与旧 `ensure_session_project` 参数一致。
- `schedule_initial_prompt` 与旧内联逻辑等价（`CLAUDE_CODE_ID || prompt.is_some()`）。

**结论**：未发现行为偏差。

## 3. Findings（第二轮：对抗式复盘）

| 严重度 | 类别 | 位置 | 证据与影响 | 处置 |
|---|---|---|---|---|
| low | Complexity | `session/launch.rs` | 新增 `io_control_error` 与 `control/wire.rs` 中既有实现完全重复 | fixed：wire.rs `pub(crate)` + control.rs re-export，launch.rs 复用 |
| low | Complexity | `session/launch.rs` + `control/handlers.rs` | 新增 `resolve_host` 与 handler 的 `resolve_host` 方法逻辑重复 | fixed：统一到 `session::resolve_host(socket_path, host_id)`，handler 委托 |
| low | Naming | `session/launch.rs` | `pub fn resolve_host` 位于私有 `mod launch`，`pub` 无意义 | fixed：改 `pub(crate)` |

## 4. 对抗式复盘

- **反例 1（空 argv + 无 binary）**：本地无 binary 且 argv 为空 → 返回 `bad_request`，与旧逻辑一致。
- **反例 2（remote + new_worktree / same_repo_as）**：仍返回 `bad_request`，未下沉到 domain 误判。
- **反例 3（descriptor 生命周期）**：`remote_spawn_spec` 接收 `descriptor` 按值（`clone()` 产物），
  无 borrow 生命周期问题；锁内只解析 descriptor，锁外做慢 I/O。
- **撤回/降级**：无。

## 5. 修复摘要

- `control/wire.rs`：`io_control_error` 由 `pub(super)` 提升为 `pub(crate)`。
- `control.rs`：`pub(crate) use wire::io_control_error;`。
- `session/launch.rs`：删除本地 `io_control_error` 重复实现；`resolve_host` 签名改为
  `(socket_path: &Path, host_id: &str)` 并降为 `pub(crate)`；内部调用改为 `&ctx.socket_path`。
- `session/mod.rs`：re-export `resolve_host`。
- `control/handlers.rs`：`resolve_host` 方法委托到 `crate::session::resolve_host`。

## 6. 验证结果

- `cargo check -p homie-engine --offline` → 0 warning / 0 error
- `cargo clippy -p homie-engine --all-targets --offline` → 0 warning
- `cargo fmt --all --check` → clean
- `cargo test -p homie-engine --lib --offline`（非沙箱）→ **303 passed / 0 failed / 3 ignored**

## 7. 残余风险

- `session_migrate` 迁移阶段（WIP commit / push / hard-sync）仍为编排逻辑，保留在 handler，
  属后续 child，非本次下沉范围。
- 暂不打 tag，待统一 code-review round 后按 SemVer 打 tag。
