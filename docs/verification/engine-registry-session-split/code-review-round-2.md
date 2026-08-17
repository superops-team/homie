# Engine Registry/Session 持久化分离 S2 代码评审报告

## 1. 评审范围

`registry/store.rs` 新增 + `registry.rs` 抽取（`PersistenceStore` trait + `JsonEnvelopeStore` +
`SplitJsonStore` + `write_json_atomic`/`invalid_data`），纯机械搬迁。

## 2. 显性问题（语法/运行/接口/命名）

| # | 问题 | 处理 |
|---|------|------|
| 1 | `migrate_envelope_to_split`（暂留 registry.rs）调用 store 私有方法 `load_state`/`projects_file`/`sessions_dir` 与 trait 方法 `save_session`，编译报 E0624/E0599 | 已修：`load_state`/`projects_file`/`sessions_dir` 提升 `pub(crate)`；registry.rs `use store::PersistenceStore` 使 trait 方法在作用域内 |
| 2 | `mod store` 为私有，`PersistenceStore` trait 方法仅被 `#[cfg(test)]` 引用，lib 构建触发 dead_code warning | 已修：`pub use store::{JsonEnvelopeStore, PersistenceStore, SplitJsonStore}` 保留公共可达性 |
| 3 | `use serde::Serialize` 随 `write_json_atomic` 下沉后残留 unused | 已修：移除（store.rs 内自有 `use serde::Serialize`） |

## 3. 隐性问题（边界/资源/规范）

- 函数体/impl 体一字未改，仅移动位置；`write_json_atomic`/`invalid_data` 提升 `pub(crate)`。
- store.rs 不含 `Registry`/`Session::spawn`/flusher/`HashMap<Session>`，符合持久化后端与 live 协调分离边界。
- 迁移逻辑（S3）与 store 后端通过 `pub(crate)` 方法交互，为下一切片 `registry/migrate.rs` 预留正确边界。

## 4. 功能验证覆盖审查

- FC-06 静态守卫：store.rs 无 live-session 依赖。
- FC-07 `split_store_*` focused tests 覆盖 dry-run 迁移不落盘、迁移备份与记录加载、corrupt session 隔离。
- FC-08 全量测试（lib 278 + 集成）全绿，磁盘 schema 与原子写语义不变。

## 5. 结论

显性问题全部修复，无残留 P0/P1。可进入 S3 切片。
