# Engine Registry/Session 持久化分离 S1 首轮代码评审报告

## 1. 评审范围

`registry/persisted.rs` 新增 + `registry.rs` 抽取（`PersistedState` + 投影折叠函数），纯机械搬迁。

## 2. 显性问题（语法/运行/接口/命名）

| # | 问题 | 处理 |
|---|------|------|
| 1 | `is_generic_terminal_title` 在 persisted.rs 保持 private，但 registry.rs 顶层 `use` 误导入导致 E0603 | 已修：该函数仅在 persisted.rs 内部使用，从 `use` 列表移除 |
| 2 | `Deserialize` 随 `PersistedState` 下沉后，registry.rs 的 `use serde::{Deserialize, Serialize}` 残留 unused `Deserialize` | 已修：改为 `use serde::Serialize;` |
| 3 | `normalize_agent_title` 被 `Registry::apply_hook_metadata` 跨模块调用，需提升可见性 | 已修：`pub(crate)`（其余投影函数同为 `pub(crate)`） |
| 4 | `PersistedState::current` 原为 private `fn`，下沉后测试/迁移/store 均需访问 | 已修：`pub(crate)` |

## 3. 隐性问题（边界/资源/规范）

- 无。函数体一字未改，仅移动位置 + 调整可见性为 `pub(crate)`，无 API 泄漏到 crate 外。
- `fold_session_view`/`repair_persisted_agent_title`/`fold_session_status`/`normalize_agent_title`/
  `is_generic_terminal_title` 保持原实现，无逻辑漂移。
- persisted.rs 不含 `HashMap<Session>`/`Session::spawn`/flusher，符合持久化投影与 live 协调分离边界。

## 4. 功能验证覆盖审查

- FC-02 静态守卫：persisted.rs 无 live-session 依赖。
- FC-03 `registry::tests` focused tests 覆盖投影折叠（PTY 标题 fallback/过滤、迁移 dry-run 等）。
- FC-04 全量测试（lib 278 + 集成）全绿，磁盘 shape 与恢复语义不变。

## 5. 结论

显性问题全部修复，无残留 P0/P1。可进入 S2 切片。
