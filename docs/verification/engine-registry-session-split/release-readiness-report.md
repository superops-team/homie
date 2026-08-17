# Engine Registry/Session Persistence Split 发布就绪报告

## 1. 就绪结论

S1（PersistedState + 投影）、S2（store 后端）、S3（migrate）、S4（flusher 落盘时机 + tests 下沉）
四个切片全部就绪，`registry.rs` 从 1465 行收敛到 758 行（< 800 验收标准），可提交。

## 2. 交付物

- `homie/crates/homie-engine/src/registry/persisted.rs`（107 行）：`PersistedState` + 投影折叠函数
- `homie/crates/homie-engine/src/registry/store.rs`（201 行）：`PersistenceStore` 后端 +
  `JsonEnvelopeStore` + `SplitJsonStore` + 原子写/非法数据辅助
- `homie/crates/homie-engine/src/registry/migrate.rs`（49 行）：`SplitMigrationReport` +
  `migrate_envelope_to_split`
- `homie/crates/homie-engine/src/registry/flusher.rs`（151 行）：`PERSIST_DEBOUNCE` + `Drop` +
  `spawn_persist_flusher` + load/persist/flush_dirty/persist_now/records_for_persistence
- `homie/crates/homie-engine/src/registry/tests.rs`（568 行）：registry 测试模块随迁
- `homie/crates/homie-engine/src/registry.rs`（1465 → 758 行）：live session 协调 + 薄持久化 facade

## 3. 验证汇总

| 门禁 | 结果 |
|------|------|
| `cargo test -p homie-engine --lib` | 278 passed / 0 failed / 3 ignored |
| `cargo test -p homie-engine`（全量） | lib + 集成全绿，0 failed |
| `cargo fmt --check` | clean |
| `cargo check -p homie-engine` | 0 warning |
| persisted.rs 无 live-session 依赖 | 通过 |
| store.rs 无 Registry/live-session 依赖 | 通过 |
| migrate.rs 无 live-session 依赖 | 通过 |
| flusher.rs 无 live 协调方法 | 通过 |
| `registry.rs` < 800 行 | 通过（758 行） |
| 磁盘 schema / 迁移 / 落盘语义不变 | 通过（函数体逐字搬迁） |

## 4. 范围说明

本 change_id 仅做机械拆分，不改磁盘 schema、迁移路径（envelope→split）、原子写与落盘语义；
`Registry` 公共 API 不变（`load`/`persist`/`flush_dirty` 仍为公开方法，`spawn_persist_flusher`
经 `pub use` 保持可达）。

## 5. 已知限制（残余风险，留待后续 PRD/切片）

- `registry.rs` 仍保留约 850 行 live session 协调逻辑（spawn/kill/restore/apply_hook_metadata 等），
  已满足 < 800 行验收，但进一步的 live 协调领域拆分（如 title/PTY 领域下沉）可作后续切片继续。
