# Rust Workspace 与 SQLite Storage Bootstrap 设计文档

## 1. 概述

### 1.1 背景

Homie V1 架构已明确 Rust 是产品核心事实源，SQLite 是本地存储事实源，CLI 是诊断和 smoke 的第一入口。当前仓库还没有 Rust workspace、SQLite migration 或可运行的 CLI，因此无法进入 runtime、LLM proxy、agent adapter 或 GPUI UI 的实现。

本变更是 V1 的第一条最小纵向切片：初始化 Rust workspace、建立 `homie-storage` SQLite schema/migration、提供 `homie-cli doctor`，并建立项目后续门禁命令的初始入口。

### 1.2 目标

- 初始化 Rust workspace 和 toolchain。
- 建立基础 crate：`homie-proto`、`homie-storage`、`homie-cli`。
- 使用 `rusqlite` + `bundled` 创建 `homie.sqlite`。
- 提供 SQLite migration 初版，覆盖 provider、runtime descriptor、agent profile、permission profile、session、usage、tool metrics 等 V1 基础表。
- 提供 `homie-cli doctor`，可在指定数据目录初始化/检查 SQLite。
- 提供基础质量入口脚本或 Makefile targets，使 `fmt`、`lint`、`test`、`security`、`pre-commit` 可执行。

### 1.3 非目标

- 不实现 GPUI app。
- 不实现后台 runtime socket、PTY 或 agent process。
- 不实现 Codex adapter。
- 不实现 LLM proxy、virtual key 签发或 provider 转发。
- 不实现 encrypted local secret envelope，只预留 schema/secret ref 边界。
- 不实现 MCP server proxy。

## 2. 用户场景

### 场景 1: 开发者初始化本地数据目录

**Given** 开发者刚 clone 仓库。
**When** 运行 `cargo run -p homie-cli -- doctor --data-dir <tmpdir>`。
**Then** CLI 创建 `<tmpdir>/homie.sqlite`，执行 migration，并输出所有检查项通过。

### 场景 2: 重复运行 doctor

**Given** 数据目录已经有当前 schema 的 `homie.sqlite`。
**When** 再次运行 `cargo run -p homie-cli -- doctor --data-dir <tmpdir>`。
**Then** migration 幂等执行，不破坏已有数据，输出 schema version 和 ok 状态。

### 场景 3: 测试验证 SQLite 关系

**Given** storage integration test 使用临时目录创建数据库。
**When** 插入 provider、LLM profile、runtime descriptor、permission profile、agent profile、session。
**Then** 外键、唯一约束、default profile 约束和 usage/tool metrics schema 都按预期工作。

## 3. 功能需求

### FR-1: Rust workspace

新增：

```text
Cargo.toml
Cargo.lock
rust-toolchain.toml
crates/homie-proto/
crates/homie-storage/
crates/homie-cli/
```

workspace 初始依赖：

- `anyhow`
- `clap`
- `rusqlite` with `bundled`
- `serde`
- `serde_json`
- `thiserror`
- `time`
- `uuid`
- `tempfile` dev dependency

### FR-2: `homie-proto`

提供基础 ID/newtype 和通用错误模型，至少包含：

```rust
ProviderId
LlmProfileId
RuntimeId
AgentProfileId
PermissionProfileId
SessionId
VirtualKeyId
```

要求：

- ID 使用 text 存储，V1 可使用 UUID v7。
- proto crate 不依赖 storage/runtime/UI。

### FR-3: `homie-storage`

提供：

```rust
pub struct StorageConfig { data_dir: PathBuf }
pub struct Storage { ... }
pub fn open_or_create(config: StorageConfig) -> Result<Storage>
pub fn migrate(&self) -> Result<MigrationReport>
pub fn health_check(&self) -> Result<StorageHealth>
```

SQLite 要求：

- database path: `<data_dir>/homie.sqlite`
- `PRAGMA foreign_keys = ON`
- WAL 模式
- forward-only migration
- migration 记录在 `schema_migrations`
- migration 幂等

### FR-4: SQLite schema 初版

初版表至少包含：

```text
schema_migrations(version, applied_at)
providers(...)
llm_profiles(...)
model_pricing(...)
pricing_snapshots(...)
runtime_descriptors(...)
permission_profiles(...)
agent_profiles(...)
skills(...)
agent_profile_skills(...)
mcp_servers(...)
agent_profile_mcp_servers(...)
effective_agent_configs(...)
sessions(...)
context_events(...)
virtual_keys(...)
usage_records(...)
tool_call_metrics(...)
tasks(...)
config_events(...)
metrics_write_failures(...)
```

约束：

- 外键开启并测试。
- `agent_profile_skills(agent_profile_id, skill_id)` 唯一。
- `agent_profile_mcp_servers(agent_profile_id, mcp_server_id)` 唯一。
- `model_pricing(provider_id, model, effective_at)` 唯一。
- `agent_profiles.is_default` 最多一个 enabled default。SQLite 可用 partial unique index。
- usage_records 包含 cache hit rate、pricing snapshot、currency、latency 字段。

### FR-5: `homie-cli doctor`

命令：

```text
homie doctor [--data-dir <path>] [--json]
```

行为：

- 解析 data dir，默认使用平台数据目录；测试可传 `--data-dir`。
- 打开/创建 SQLite。
- 执行 migration。
- 检查 foreign_keys、journal_mode、schema version。
- 输出 human-readable summary。
- `--json` 输出稳定 JSON，便于功能验证 Case 检查。

### FR-6: 质量入口

新增 Makefile 或 scripts，使以下命令可执行：

```text
make fmt
make lint
make test
make security
make pre-commit
```

当前阶段：

- `fmt` -> `cargo fmt --all`
- `lint` -> `cargo clippy --workspace --all-targets -- -D warnings`
- `test` -> `cargo test --workspace`
- `security` -> `.githooks/pre-commit`
- `pre-commit` -> `fmt --check` + `lint` + `test` + `security`

## 4. 实现方案

### 4.1 Workspace

- root `Cargo.toml` 管理 workspace dependencies。
- `rust-toolchain.toml` pin stable Rust，并包含 `rustfmt`、`clippy`。
- `Cargo.lock` 提交。

### 4.2 Storage

- `homie-storage` 手写 SQL migration。
- migration SQL 放在 crate 内部常量或 `migrations/sqlite`，V1 选择简单常量也可接受，但必须有测试覆盖。
- repository API 初期只提供 migration/health，不实现全部 CRUD。

### 4.3 CLI

- `homie-cli` 使用 `clap` derive。
- doctor JSON 输出包含：

```json
{
  "status": "ok",
  "databasePath": "...",
  "schemaVersion": 1,
  "foreignKeys": true,
  "journalMode": "wal"
}
```

## 5. 组件 spec 影响

| 组件 | 是否影响 | 原因 | 需要更新 |
|------|----------|------|----------|
| `specs/storage-indexing/README.md` | 是 | 本变更实现 SQLite 事实源和 migration | 本变更应创建 |
| `specs/virtual-key-credentials/README.md` | 否 | 只预留 secret ref 字段，不实现 envelope | 后续 |
| `specs/agent-adapter-contract/README.md` | 否 | 只建关系表，不实现 adapter | 后续 |
| `specs/llm-proxy/README.md` | 否 | 只建 usage schema，不实现 proxy | 后续 |

## 6. 测试计划

### 6.1 单元测试

- ID 序列化/反序列化。
- Storage path resolution。
- Migration report。

### 6.2 集成测试

- 空临时目录创建 SQLite。
- migration 幂等。
- foreign key 生效。
- unique constraints 生效。
- default profile partial unique index 生效。
- usage_records 可插入 token/cache/cost/latency 字段。

### 6.3 功能验证

- `cargo run -p homie-cli -- doctor --data-dir <tmpdir> --json` 返回 ok。
- 重复执行 doctor 仍返回 ok。
- `make pre-commit` 通过。

## 7. 验收标准

- `cargo fmt --all -- --check` 通过。
- `cargo clippy --workspace --all-targets -- -D warnings` 通过。
- `cargo test --workspace` 通过。
- `.githooks/pre-commit` 通过。
- `homie-cli doctor --data-dir <tmpdir> --json` 可创建/检查 SQLite。
- `docs/verification/workspace-storage-bootstrap/` 留存功能验证和准出证据。

## 8. Beads 追踪

- Beads issue: `homie-mgl`
- change_id: `workspace-storage-bootstrap`
- spec-id: `prd-spec/features/workspace-storage-bootstrap/2026-08-05-workspace-storage-bootstrap-design.md`
