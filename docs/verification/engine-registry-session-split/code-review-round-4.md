# Engine Registry/Session 持久化分离 S4 代码评审报告

## 1. 评审范围

`registry/flusher.rs` 新增 + `registry.rs` 抽取（`PERSIST_DEBOUNCE` / `impl Drop` /
`spawn_persist_flusher` / `load` / `persist` / `flush_dirty` / `persist_now` /
`records_for_persistence`）+ `registry/tests.rs` 测试模块整体下沉。机械搬迁，函数体一字不改。

## 2. 显性问题（语法/运行/接口/命名）

无。`load`/`persist`/`flush_dirty` 原为 `pub`，下沉到 `registry::flusher` 后仍是 `Registry`
的公开方法，公共 API 不变；`spawn_persist_flusher` 通过 `pub use flusher::spawn_persist_flusher`
保持 `crate::registry::spawn_persist_flusher` 可达（homied-rs 调用点不破坏）。

## 3. 隐性问题（边界/资源/规范）

- `flusher.rs` 作为 `registry` 子模块，`impl Registry` / `impl Drop for Registry` 可访问父模块
  私有字段（`state_file`/`dirty`/`last_persist`/`records`/`sessions`/`projects`），边界正确。
- 落盘语义（debounce 标记 dirty、`persist_now` 原子写 `.json.tmp` + rename、Drop 兜底
  `flush_dirty`）与拆分前逐字一致，磁盘 schema 不变。
- `tests.rs` 由 `mod tests` 整体搬迁为 `#[cfg(test)] mod tests;` + 独立文件；因 `load` 下沉后
  `repair_persisted_agent_title` 不再被 registry.rs 非测试代码引用，测试文件补一条显式
  `use super::persisted::repair_persisted_agent_title;`，避免测试侧符号缺失。
- 拆分后 registry.rs 的 `use persisted::{...}` 精简为 `{fold_session_view, normalize_agent_title}`，
  `fold_session_status`/`repair_persisted_agent_title` 随落盘逻辑下沉，无残留 unused import。

## 4. 功能验证覆盖审查

- FC-14 静态守卫：flusher.rs 无 live 协调方法命中。
- FC-15 `registry::tests` 18 passed / 0 failed / 1 ignored。
- FC-16 全量测试（lib 278 + 集成）全绿，行为与拆分前一致。
- FC-17 fmt clean、cargo check 0 warning。
- FC-18 验收：`registry.rs` 758 行 < 800。

## 5. 结论

无显性问题，无残留 P0/P1。四个切片全部完成，`registry.rs` 已从 1465 行收敛到 758 行，
满足 < 800 行验收标准。
