# OpenSpec Plan — engine-control-handlers-split

## 目标

将 `homie/crates/homie-engine/src/control/handlers.rs`（1173 行）的 `impl ControlServer` 方法按关注点
拆分为 10 个聚焦子模块。`mod.rs` 保留共享 imports + 子模块声明 + `new_record` 自由函数。所有方法与
自由函数逐字迁移，可见性从 `pub(super)` 统一提升为 `pub(crate)`，单文件 < 800 行。

## 拆分策略

- 依赖方向：`mod.rs`（facade，共享 imports + `new_record`）→ 各子模块（经 `use super::*;` 引入
  `ControlServer` 与共享依赖）。
- 每个子模块以 `use super::*;` + `impl ControlServer { ... }` 实现，方法体逐字迁移。
- 原 `pub(super) fn` 方法提升为 `pub(crate) fn`，以便兄弟子模块与 `control` 模块跨模块调用。
- 私有辅助 `schedule_initial_prompt` 保持 `fn`；`resume_spec` 因被 `control::tests` 引用提升为
  `pub(crate) fn`。
- 无生产代码语义变更，无外部 API 泄漏（`pub(crate)` 仍为 crate 内部）。

## 交付切片

- T1：方法边界扫描，精确定位全部 44 方法 + 1 自由函数的闭合括号。
- T2：生成 9 个业务子模块（agent/governor/handshake/host/migrate/resume/session/spawn/worktree）。
- T3：重建 `mod.rs`（共享 imports + 子模块声明 + `new_record`），删除旧 `handlers.rs`，编译验证。
- T4：全量验证（fmt/check/clippy/build/workspace-check/test）。
- T5：code review + release readiness 证据。

## 验证

见 `tasks.md`，最终证据写入 `docs/verification/engine-control-handlers-split/`。
