# Engine Registry/Session 持久化分离 功能验证 Case

## 1. 验证目标

面向 `engine-registry-session-split` 四个切片（S1 persisted / S2 store / S3 migrate /
S4 flusher），证明：

- 持久化职责（`PersistedState` 投影、`PersistenceStore` 后端、迁移、落盘时机）整体下沉到
  `registry/` 子模块，`Registry` 只保留 live session 协调；
- 新模块不依赖 live session map / daemon 环境，可独立单测；
- 磁盘 schema、迁移路径（envelope→split）、落盘语义与拆分前完全一致；
- 行为由普通 Rust focused tests 覆盖，全量测试全绿；
- `registry.rs` < 800 行。

## 2. Case 定义

### FC-01: 基线测试全绿

```bash
cargo test -p homie-engine --lib
```

通过标准：拆分前 278 passed / 0 failed / 3 ignored 为基线。

证据路径：`docs/verification/engine-registry-session-split/fc-01-baseline.log`

### FC-02: persisted.rs 抽取完成且无 live-session 依赖

```bash
test -s homie/crates/homie-engine/src/registry/persisted.rs
rg -n "sessions:.*HashMap|HashMap<.*Session|Session::spawn|spawn_persist_flusher" \
  homie/crates/homie-engine/src/registry/persisted.rs
```

通过标准：文件存在；`rg` 无 HashMap<Session>/Session::spawn/flusher 命中（只含投影折叠纯逻辑）。

证据路径：`docs/verification/engine-registry-session-split/fc-02-persisted-pure.log`

### FC-03: persisted 投影 focused tests

```bash
cargo test -p homie-engine registry::tests -- --nocapture
```

通过标准：`pty_titles_are_filtered_fallbacks_and_never_override_user_renames`、
`state_round_trips_through_the_swift_file_shape` 等全绿。

证据路径：`docs/verification/engine-registry-session-split/fc-03-persisted-tests.log`

### FC-04: 全量行为不变（S1）

```bash
cargo test -p homie-engine
```

通过标准：抽取后全部测试（lib 278 + 集成测试）全绿，0 failed。

证据路径：`docs/verification/engine-registry-session-split/fc-04-full-tests.log`

### FC-05: 静态门禁（S1）

```bash
cargo fmt -p homie-engine -- --check
cargo check -p homie-engine
```

通过标准：fmt 干净、无 warning。

证据路径：`docs/verification/engine-registry-session-split/fc-05-static-gates.log`

---

# S2 store 后端抽取（PersistenceStore + JsonEnvelopeStore + SplitJsonStore + write_json_atomic/invalid_data）

### FC-06: store.rs 抽取完成且无 live-session 依赖

```bash
test -s homie/crates/homie-engine/src/registry/store.rs
rg -n "Registry|Session::spawn|spawn_persist_flusher|HashMap<.*Session" \
  homie/crates/homie-engine/src/registry/store.rs
```

通过标准：文件存在；`rg` 无 Registry/Session::spawn/flusher/HashMap<Session> 命中
（store.rs 只含持久化后端 + 原子写辅助函数）。

证据路径：`docs/verification/engine-registry-session-split/fc-06-store-pure.log`

### FC-07: store focused tests

```bash
cargo test -p homie-engine registry::tests -- --nocapture
```

通过标准：`split_store_dry_run_migration_writes_nothing`、
`split_store_migration_preserves_backup_and_loads_records`、
`split_store_quarantines_one_corrupt_session_file` 全绿。

证据路径：`docs/verification/engine-registry-session-split/fc-07-store-tests.log`

### FC-08: 全量行为不变（S2）

```bash
cargo test -p homie-engine
```

通过标准：抽取后全部测试（lib 278 + 集成测试）全绿，0 failed。

证据路径：`docs/verification/engine-registry-session-split/fc-08-full-tests.log`

### FC-09: 静态门禁（S2）

```bash
cargo fmt -p homie-engine -- --check
cargo check -p homie-engine
```

通过标准：fmt 干净、无 warning（pub use 重导出保留公共可达性，无 dead_code）。

证据路径：`docs/verification/engine-registry-session-split/fc-09-static-gates.log`

---

# S3 迁移逻辑抽取（SplitMigrationReport + migrate_envelope_to_split）

### FC-10: migrate.rs 抽取完成且无 live-session 依赖

```bash
test -s homie/crates/homie-engine/src/registry/migrate.rs
rg -n "Registry|Session::spawn|spawn_persist_flusher|HashMap<.*Session" \
  homie/crates/homie-engine/src/registry/migrate.rs
```

通过标准：文件存在；`rg` 无 Registry/Session::spawn/flusher/HashMap<Session> 命中。

证据路径：`docs/verification/engine-registry-session-split/fc-10-migrate-pure.log`

### FC-11: migrate focused tests

```bash
cargo test -p homie-engine registry::tests split_store -- --nocapture
```

通过标准：`split_store_dry_run_migration_writes_nothing`、
`split_store_migration_preserves_backup_and_loads_records` 全绿。

证据路径：`docs/verification/engine-registry-session-split/fc-11-migrate-tests.log`

### FC-12: 全量行为不变（S3）

```bash
cargo test -p homie-engine
```

通过标准：全部测试（lib + 集成）全绿，0 failed。

证据路径：`docs/verification/engine-registry-session-split/fc-12-full-tests.log`

### FC-13: 静态门禁（S3）

```bash
cargo fmt -p homie-engine -- --check
cargo check -p homie-engine
```

通过标准：fmt 干净、无 warning。

证据路径：`docs/verification/engine-registry-session-split/fc-13-static-gates.log`

---

# S4 落盘时机抽取（PERSIST_DEBOUNCE + Drop + spawn_persist_flusher + load/persist/flush_dirty/persist_now/records_for_persistence）

### FC-14: flusher.rs 抽取完成且只含落盘时机

```bash
test -s homie/crates/homie-engine/src/registry/flusher.rs
rg -n "pub fn spawn\(|pub fn kill\(|pub fn restore\(|apply_hook_metadata|adopt_remote|ensure_session_awake" \
  homie/crates/homie-engine/src/registry/flusher.rs
```

通过标准：文件存在；`rg` 无 live 协调方法命中（flusher.rs 只含 debounce / Drop / flusher 线程 /
load / persist / flush_dirty / persist_now / records_for_persistence）。

证据路径：`docs/verification/engine-registry-session-split/fc-14-flusher-pure.log`

### FC-15: registry focused tests（S4）

```bash
cargo test -p homie-engine registry::tests
```

通过标准：`registry::tests` 18 passed / 0 failed / 1 ignored。

证据路径：`docs/verification/engine-registry-session-split/fc-15-flusher-tests.log`

### FC-16: 全量行为不变（S4）

```bash
cargo test -p homie-engine
```

通过标准：lib 278 passed / 0 failed / 3 ignored，全部集成测试 0 failed。

证据路径：`docs/verification/engine-registry-session-split/fc-16-full-tests.log`

### FC-17: 静态门禁（S4）

```bash
cargo fmt --check
cargo check -p homie-engine
```

通过标准：fmt 干净、check 0 warning。

证据路径：`docs/verification/engine-registry-session-split/fc-17-static-gates.log`

### FC-18: registry.rs < 800 行验收

```bash
wc -l homie/crates/homie-engine/src/registry.rs
```

通过标准：`registry.rs` 行数 < 800（实测 758）。

证据路径：`docs/verification/engine-registry-session-split/fc-18-line-budget.log`
