# Engine Control Wire/Runtime Split S4-a + S4-b(1) 复核报告

## 1. 结论

S4-a handler 机械下沉与 S4-b 首个增量（worktree_overview → git.rs）无 P0/P1 问题，质量达标。

## 2. 复核要点

### S4-a（handler → control/handlers.rs）

- 43 个 handler 方法 + 20 个自由函数体一字未改，仅机械搬迁 + `pub(super)` 可见性限定。
- `new_record` 经 `pub(crate) use handlers::new_record` 重导出，`mcp/host.rs` 调用不变。
- `EchoOutcome` / 注入窗口常量提升到模块顶层，`control.rs` 不再定义 handler。
- `handlers.rs` 不含 `serve`/`handle_line`/`dispatch`，transport 与 handler 边界清晰。
- `control.rs` 3,328 → 460 行（< 800 目标达成）。

### S4-b(1)（worktree_overview → crate::git）

- `crate::git::worktree_overview(records, roots)` 承接 staleness join + git subprocess
  领域逻辑（约 110 行），函数体一字未改，仅改为接收 `records`/`roots` 参数。
- handler 收敛为 lock registry → 收集 records/roots → 调用领域函数 → encode（约 12 行）。
- 领域函数无 ControlServer/socket/registry 依赖，可脱离 daemon 单测。

## 3. 门禁

- `cargo test -p homie-engine`：278 lib + 集成全绿，0 failed。
- `cargo fmt --check` clean；`cargo check` 0 warning。

## 4. 结论

可提交。
