# Engine Registry/Session 持久化分离设计文档

## 1. 概述

### 1.1 问题/动机

`homie/crates/homie-engine/src/registry.rs` 当前约 1,790 行，`Registry` 同时承担两类职责：

1. **持久化**：`PersistedState`（19–28 行）、`PersistenceStore` trait（38 行）、`JsonEnvelopeStore`
   （48 行）、`SplitJsonStore`（124 行）、`SplitMigrationReport`（216 行）、`migrate_envelope_to_split`
   （225 行）、`load`/`persist`/`flush_dirty`/`persist_now`/`records_for_persistence`/`spawn_persist_flusher`；
2. **live session 协调**：`spawn`/`adopt_remote`/`restore`/`get`/`views`/`records`/`terminate`/`forget`/
   `remove`/`respawn`/`wake_session`/`hibernate`/`archive`/`rename`/`apply_*` 等。

持久化（磁盘 schema、迁移、flusher 生命周期）与 live session 协调（内存状态、会话生命周期）
变化原因不同：存储 schema 演进不应触碰会话生命周期逻辑，反之亦然。二者耦合在一个 struct 里，
`Registry` 既要管内存 `HashMap<Session>` 又要管 `PersistedState` 磁盘落盘，导致「存储层」与
「运行时层」无法独立演进。

这是 2026-08 架构审计 finding **F7（Warning）**：`engine/registry.rs` 双职责（live session +
persistence），并叠加 `session.rs` 2,888 行的认知负荷。

### 1.2 目标

1. 从 `Registry` 拆出独立的持久化模块（`PersistedState` + `PersistenceStore` 实现 + 迁移逻辑），
   `Registry` 只保留 live session 协调。
2. 保持磁盘 schema、迁移路径（envelope→split）、落盘时机与语义完全不变。
3. 持久化模块可独立单测（不要求 live session 或 daemon 环境）。
4. 降低 storage/session 变更的 review 面，`registry.rs` < 800 行。
5. 遵守 `specs/engine-session-runtime.md`：会话持久化与恢复语义不变。

### 1.3 非目标

- 不改变磁盘存储格式、文件路径、原子写策略或迁移逻辑。
- 不改变 `Registry` 对外 API 语义（调用方代码无需感知内部拆分）。
- 不重写 `session.rs` 的会话状态机；本 PRD 只做 registry 持久化职责分离，session 内部深化
  留待后续 child。
- 不引入新的持久化后端（如 SQLite/rocksdb）。

### 1.4 基线快照

- branch: `main`
- baseline commit: `e4c7454`
- 目标文件：`homie/crates/homie-engine/src/registry.rs`（1,790 行）、`session.rs`（2,888 行）
- 相关测试：`homie/crates/homie-engine/tests/`、`registry.rs` 内 `#[cfg(test)]`
- 相关 spec：`specs/engine-session-runtime.md`

### 1.5 与存量 PRD 的关系

| 存量文档 | 关系 |
|----------|------|
| `architecture-audit-governance-2026-08` | 本 PRD 是其 child（homie-ubu.2），F7 |
| `architecture-audit-hardening` | Phase 3 已规划 engine 持久化/会话治理，本 PRD 落地 registry 持久化分离切片 |
| `persistence-incremental-state` | 增量持久化路线，本 PRD 不冲突，只做职责分离不改 schema |
| `specs/engine-session-runtime.md` | 持久化与恢复语义合同，拆分后保持不变 |

## 2. 现状分析

`Registry` 当前职责拆解：

| 层 | 成员 | 变化原因 |
|----|------|----------|
| 持久化 schema | `PersistedState`/`SessionRecord` 投影 | 存储结构 |
| 存储后端 | `PersistenceStore`/`JsonEnvelopeStore`/`SplitJsonStore` | 存储格式/后端 |
| 迁移 | `SplitMigrationReport`/`migrate_envelope_to_split` | schema 演进 |
| 落盘 | `load`/`persist`/`flush_dirty`/`persist_now`/`records_for_persistence`/`spawn_persist_flusher` | 持久化生命周期 |
| live 协调 | `spawn`/`adopt_remote`/`restore`/`get`/`views`/`terminate`/`forget`/`remove`/`respawn`/`wake_session`/`hibernate`/`archive`/`rename` | 会话运行时 |

关键观察：`Registry` 持有 `sessions: HashMap<String, Session>`（内存态）与 `state: PersistedState`
（磁盘态投影）两份状态，`fold_session_view`/`repair_persisted_agent_title`/`fold_session_status` 等
free function 是「内存 view → 磁盘 record」的投影折叠逻辑，本质属持久化投影，与 live 协调无关。

## 3. 方案设计

### 3.1 拆分原则

- **持久化整体下沉**：`PersistedState` + `PersistenceStore` 实现 + 迁移 + 落盘全部移入独立模块，
  作为 `Registry` 的一个字段/依赖注入。
- **投影折叠归持久化模块**：`fold_session_view`/`repair_persisted_agent_title`/`fold_session_status`
  等投影函数归入持久化模块。
- **Registry 只保留 live 协调**：内存 session map、生命周期方法、对外查询接口。
- 行为不变，`cargo test` 全绿。

### 3.2 目标模块拓扑

```text
homie/crates/homie-engine/src/
├── registry.rs                 # Registry：live session 协调（< 800 行）
├── registry/
│   ├── persisted.rs            # PersistedState + 投影折叠函数
│   ├── store.rs                # PersistenceStore trait + JsonEnvelopeStore + SplitJsonStore
│   ├── migrate.rs              # SplitMigrationReport + migrate_envelope_to_split
│   └── flusher.rs              # spawn_persist_flusher + 落盘时机逻辑
└── session.rs                  # Session 状态机（不变，留待后续 child）
```

### 3.3 下沉映射

| 现成员 | 目标模块 |
|--------|----------|
| `PersistedState`/`SessionRecord` 投影 | `registry/persisted.rs` |
| `PersistenceStore`/`JsonEnvelopeStore`/`SplitJsonStore` | `registry/store.rs` |
| `SplitMigrationReport`/`migrate_envelope_to_split` | `registry/migrate.rs` |
| `load`/`persist`/`flush_dirty`/`persist_now`/`records_for_persistence`/`spawn_persist_flusher` | `registry/flusher.rs` + `registry.rs` 薄封装 |
| `fold_session_view`/`repair_persisted_agent_title`/`fold_session_status` | `registry/persisted.rs` |
| `spawn`/`adopt_remote`/`restore`/`get`/`views`/`terminate`/`forget`/`remove`/`respawn`/`wake_session`/`hibernate`/`archive`/`rename` | `registry.rs`（保留） |

### 3.4 实施顺序（每次一个可验证切片）

1. **S1**：抽 `registry/persisted.rs`（`PersistedState` + 投影折叠函数），`Registry` 引用之。
2. **S2**：抽 `registry/store.rs`（`PersistenceStore` + 两个实现）。
3. **S3**：抽 `registry/migrate.rs`（迁移逻辑）。
4. **S4**：抽 `registry/flusher.rs`（落盘时机 + `spawn_persist_flusher`）。

每步完成后 `cargo test -p homie-engine` 全绿，再做下一步。

## 4. 测试与验收

### 4.1 测试计划

- 持久化模块单测：envelope↔split 迁移、投影折叠、原子写、非法数据。
- 集成测试：现有 `homie-engine/tests/` 的恢复/落盘行为不变。
- 迁移兼容回归：envelope→split 迁移结果与拆分前一致。

### 4.2 验收标准

1. `registry.rs` < 800 行；`registry/persisted.rs`/`store.rs`/`migrate.rs`/`flusher.rs` 内聚单一职责。
2. `cargo test -p homie-engine` 全绿，无新增失败。
3. 磁盘 schema、迁移路径、落盘语义与拆分前完全一致。
4. 持久化模块可脱离 live session/daemon 独立单测。
5. `specs/engine-session-runtime.md` 的持久化与恢复语义保持不变。

## 5. Beads 追踪

- change_id: `engine-registry-session-split`
- parent Beads: `homie-ubu`；child Beads: `homie-ubu.2`
- 类型: refactor
- 优先级: P0
- 验收证据目录: `docs/verification/engine-registry-session-split/`
