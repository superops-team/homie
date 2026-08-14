# 持久化从整文件 JSON 向分层/行级状态演进设计文档

## 1. 概述

### 1.1 问题/动机

Homie Engine 当前将项目和 session records 持久化为一个整体 JSON envelope：

```text
{ version, projects, sessions }
```

`Registry::persist_now()` 会序列化所有 records，并通过临时文件 + rename 原子替换整个 state file。当前已有 500ms debounce，说明频繁 mutation 已经需要合并写入。随着 Homie 管理更多长期会话、remote bindings、usage、artifacts、checkpoint、history 和未来 LLM provider 配置，整文件 JSON 会带来明显风险：

- 列表页需要加载所有 session records。
- 单个 session 的小变更会重写整个 state。
- 多 session 并发状态变化下，写放大增加。
- corrupted JSON 会影响整个状态文件。
- 未来查询、过滤、按时间排序和增量恢复难以扩展。

Waku 的持久化设计将列表字段提升为 SQLite 列，把大 transcript/details 拆分到单独表，并通过 dirty sessions 只写变更行。Homie 不必一次性全量迁移到 SQLite，但需要规划从整文件 JSON 向分层/行级状态演进。

### 1.2 目标

1. 降低 session 状态更新的写放大。
2. 降低单文件损坏影响面。
3. 为多 session、大历史、查询排序、增量恢复建立可扩展持久化边界。
4. 保留现有 holder/session restore 语义。
5. 提供可回滚、可验证的数据迁移路径。

### 1.3 非目标

- 不在第一阶段迁移所有历史数据和 logs。
- 不改变 OutputLog 的 append-only 语义。
- 不引入远端同步或云存储。
- 不改变用户可见数据位置，除非迁移报告明确说明。

## 2. 现状分析

| 模块 | 当前状态 | 问题 |
|------|----------|------|
| `Registry` | 内存 HashMap + `state.json` 整体持久化 | 小变更重写全文件 |
| `PersistedState` | `projects: Vec<Value>` + `sessions: Vec<SessionRecord>` | project fields 依赖 additive JSON，缺少结构化索引 |
| `Prefs` | app UI prefs 独立 JSON | 已有分离趋势 |
| OutputLog | session 输出日志独立文件 | 适合继续保留 |
| remote bindings | 单独 binding store | 已体现分层思想 |

Waku 参考点：

- `db/schema.ts` 中 `sessions` 行只包含列表所需字段。
- `messages`、`session_details` 拆分，避免列表页读大历史。
- `PersistedState::dirty_sessions` 明确记录哪些 session 变更。
- settings/app-managed state 分离，用户配置不混入 app 内部几何状态。

## 3. 方案设计

### 3.1 阶段 1：分文件状态，不引入 SQLite

低风险起步：

```text
~/Library/Application Support/Homie/
├── state.json                  # envelope/version/index
├── projects.json               # 项目列表
├── sessions/
│   ├── <session-id>.json
│   └── ...
└── state-migration-report.json
```

特点：

- session record 单独原子写。
- `state.json` 只保留 schema version、最近选择、全局索引或兼容标记。
- load 时枚举 `sessions/*.json`。
- 单个 session 文件损坏只 quarantine 单个文件。
- 保留旧 `state.json` 迁移入口。

适用：先解决写放大和损坏影响面，不引入 SQL 查询复杂度。

### 3.2 阶段 2：结构化索引或 SQLite

当 session 数量、查询排序、筛选、usage 聚合成为瓶颈后，引入 SQLite：

```text
app.db
sessions(id, project_id, title, status, updated_at, last_seen_at, host, archived, ...)
session_details(session_id, data_json)
projects(id, root, host, name, position, data_json)
```

要求：

- 列表页只读窄行。
- 大字段进入 details JSON。
- session mutation 只更新对应行。
- migration 有 dry-run 和 checksum。

### 3.3 推荐路线

先做阶段 1。理由：

- Homie 当前状态还处于架构迁移后稳定期。
- 分文件能快速降低写放大和 corruption blast radius。
- 不需要一次引入 SQL schema/migration 工具。
- 后续迁移 SQLite 时，分文件结构也能作为中间导入源。

## 4. 实施步骤

### 4.1 阶段 1

1. 定义 `PersistenceStore` trait：
   - `load_projects()`
   - `load_sessions()`
   - `save_project()`
   - `save_session()`
   - `delete_session()`
   - `flush()`
2. 实现 `JsonEnvelopeStore` 包装现有 `state.json`。
3. 实现 `SplitJsonStore`。
4. 增加 migration：
   - 旧 `state.json` -> split files；
   - 迁移成功后保留 backup；
   - 迁移失败不删除旧文件。
5. Registry 改为依赖 store trait，而非直接写 `state_file`。
6. dirty tracking：
   - session 级 dirty；
   - project 级 dirty；
   - global envelope dirty。
7. 测试 corrupted 单 session 文件 quarantine。

### 4.2 阶段 2 预留

1. 在 PRD 阶段只定义 SQLite 目标形态，不实现。
2. 如果阶段 1 后仍有性能/查询瓶颈，再创建独立 Beads 和 PRD。

## 5. 涉及文件

- `homie/crates/homie-engine/src/registry.rs`
- `homie/crates/homie-engine/src/directories.rs`
- `homie/crates/homie-engine/src/bin/homied-rs.rs`
- `homie/crates/homie-proto/src/model.rs`
- `homie/crates/homie-app/src/store/mod.rs`
- `homie/crates/homie-app/src/store/prefs.rs`
- `homie/crates/homie-engine/tests/*`
- `docs/GETTING_STARTED.md`

## 6. 验证计划

### 6.1 单元测试

- 旧 `state.json` 可迁移到 split store。
- split store 可加载 projects/sessions。
- 单个 session 文件损坏只 quarantine 该 session。
- session mutation 只写对应 session 文件。
- atomic rename 防止部分写入。
- dirty tracking 不丢最后一次状态。

### 6.2 集成测试

- 启动 daemon，创建 session，重启 daemon，session 仍可恢复。
- holder adoption 与 split persistence 一致。
- archive/unarchive/remove/rename 后重启仍一致。
- remote binding 不受影响。

### 6.3 性能基线

构造 100、1000、5000 session records：

- load 时间；
- 单 session mark-seen 写入字节数；
- rename 写入字节数；
- corrupted session 隔离行为。

## 7. 验收标准

1. 第一阶段实现后，单个 session 变更不再重写全部 session records。
2. 旧 `state.json` 自动、安全迁移，失败不破坏旧数据。
3. 单个 session 文件损坏不会导致全部状态丢失。
4. `Registry` 持久化逻辑通过 trait 与存储格式解耦。
5. Beads `homie-or4` 更新为已验证状态后才可关闭。

## 8. Beads 追踪

- Beads: `homie-or4`
- change_id: `persistence-incremental-state`
- 类型: refactor
- 优先级: P1
