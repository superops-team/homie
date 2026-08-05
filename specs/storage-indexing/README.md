# Storage & Indexing 组件规格

## 1. 组件定位

`homie-storage` 是 Homie V1 的本地事实源组件，负责 SQLite 数据库创建、schema migration、关系约束、健康检查和 repository API。

## 2. 来源需求映射

- PRD: `prd-spec/features/workspace-storage-bootstrap/2026-08-05-workspace-storage-bootstrap-design.md`
- V1 架构: `prd-spec/features/homie-v1-architecture/2026-08-05-homie-v1-architecture-design.md`
- OpenSpec: `openspec/changes/workspace-storage-bootstrap/`
- 功能验证: `docs/verification/workspace-storage-bootstrap/functional-cases.md`

## 3. 上游与下游关系

| 方向 | 组件 | 关系 |
|------|------|------|
| 上游 | `homie-cli` | 调用 storage health/migration 用于 `doctor` |
| 上游 | `homie-runtime` | 后续读写 session/context |
| 上游 | `homie-llm` | 后续读写 provider/usage/virtual key |
| 下游 | SQLite | 本地数据库文件 |
| 下游 | output log files | 大流式输出文件，由 SQLite 索引 |

## 4. 职责边界

负责：

- data dir 解析后的 SQLite path 生成。
- SQLite 打开/创建。
- `PRAGMA foreign_keys = ON`。
- WAL 模式。
- forward-only migration。
- schema health check。
- schema 约束测试。

不负责：

- provider raw key 加密实现。
- runtime process 生命周期。
- LLM proxy 请求转发。
- GPUI UI。
- MCP server proxy。

## 5. 核心接口

```rust
pub struct StorageConfig {
    pub data_dir: PathBuf,
}

pub struct Storage;

pub struct MigrationReport {
    pub schema_version: i64,
    pub applied: Vec<i64>,
}

pub struct StorageHealth {
    pub database_path: PathBuf,
    pub schema_version: i64,
    pub foreign_keys: bool,
    pub journal_mode: String,
}

pub fn open_or_create(config: StorageConfig) -> Result<Storage, StorageError>;

impl Storage {
    pub fn migrate(&self) -> Result<MigrationReport, StorageError>;
    pub fn health_check(&self) -> Result<StorageHealth, StorageError>;
}
```

## 6. 数据模型

V1 schema version: `1`

核心表：

- `schema_migrations`
- `providers`
- `llm_profiles`
- `model_pricing`
- `pricing_snapshots`
- `runtime_descriptors`
- `permission_profiles`
- `agent_profiles`
- `skills`
- `agent_profile_skills`
- `mcp_servers`
- `agent_profile_mcp_servers`
- `effective_agent_configs`
- `sessions`
- `context_events`
- `virtual_keys`
- `usage_records`
- `tool_call_metrics`
- `tasks`
- `config_events`
- `metrics_write_failures`

## 7. 运行模型与状态机

```text
open_or_create
  -> create data dir
  -> open homie.sqlite
  -> set PRAGMAs
  -> migrate
  -> health_check
```

Migration 是 forward-only。不存在 downgrade、fallback 或兼容旧 schema。

## 8. 安全与权限

- SQLite 文件在用户数据目录内创建。
- raw provider key 不进入 SQLite。
- secret 只保存 `api_key_ref` 或 envelope ref。
- metrics 表不得保存 raw request/response、Authorization、cookie、完整 tool args/result。

## 9. 可观测性

`health_check` 输出：

- database path
- schema version
- foreign keys enabled
- journal mode

CLI doctor 使用该输出生成 human/json report。

## 10. 失败与恢复

| 场景 | 行为 |
|------|------|
| data dir 不存在 | 创建 |
| SQLite 无法打开 | 返回 error，不吞掉 |
| migration 中途失败 | transaction rollback |
| schema version 过新 | fail closed |
| corrupt database | 当前切片只返回 error；后续组件 spec 增加 quarantine |

## 11. 测试计划与验收引用

- FC-001: doctor 创建 SQLite。
- FC-002: doctor 幂等。
- FC-003: SQLite 关系约束。
- FC-004: usage schema 支持 token/cache/cost/latency。
- FC-005: workspace 质量门禁。
