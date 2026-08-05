# Workspace Storage Bootstrap 功能验证 Case

```yaml
change_id: workspace-storage-bootstrap
report_type: functional-cases
status: draft
beads: homie-mgl
source_prd: prd-spec/features/workspace-storage-bootstrap/2026-08-05-workspace-storage-bootstrap-design.md
```

## 1. 验证范围

本文件定义开发前置功能验证 Case。开发完成后必须逐条执行并记录到 `docs/verification/workspace-storage-bootstrap/functional-verification-report.md`。

## 2. Case 清单

### FC-001: CLI doctor 创建 SQLite

| 字段 | 内容 |
|------|------|
| 覆盖需求 | FR-3, FR-4, FR-5 |
| 前置条件 | Rust workspace 已实现；临时目录为空 |
| 命令 | `TMPDIR=$(mktemp -d) && cargo run -p homie-cli -- doctor --data-dir "$TMPDIR" --json` |
| 预期 | 命令 exit=0；JSON `status=ok`；`databasePath` 指向 `$TMPDIR/homie.sqlite`；`schemaVersion=1`；`foreignKeys=true`；`journalMode=wal` |
| 证据 | `docs/verification/workspace-storage-bootstrap/artifacts/fc-001-doctor.json` |
| 失败处理 | 回到 storage migration 或 CLI doctor 实现 |

### FC-002: CLI doctor 幂等

| 字段 | 内容 |
|------|------|
| 覆盖需求 | FR-3, FR-5 |
| 前置条件 | 使用同一个临时目录连续运行 doctor |
| 命令 | `TMPDIR=$(mktemp -d) && cargo run -p homie-cli -- doctor --data-dir "$TMPDIR" --json && cargo run -p homie-cli -- doctor --data-dir "$TMPDIR" --json` |
| 预期 | 两次 exit=0；第二次不报 migration conflict；schemaVersion 保持 1 |
| 证据 | `docs/verification/workspace-storage-bootstrap/artifacts/fc-002-doctor-idempotent.json` |
| 失败处理 | 修复 migration 幂等逻辑 |

### FC-003: SQLite 关系约束

| 字段 | 内容 |
|------|------|
| 覆盖需求 | FR-4 |
| 前置条件 | storage integration tests 已实现 |
| 命令 | `cargo test -p homie-storage sqlite_constraints -- --nocapture` |
| 预期 | 外键、profile-skill 唯一约束、profile-MCP 唯一约束、model pricing 唯一约束、enabled default profile 约束全部通过 |
| 证据 | `docs/verification/workspace-storage-bootstrap/artifacts/fc-003-sqlite-constraints.txt` |
| 失败处理 | 修复 schema/index/migration |

### FC-004: Usage schema 支持 token/cache/cost/latency

| 字段 | 内容 |
|------|------|
| 覆盖需求 | FR-4 |
| 前置条件 | storage integration tests 已实现 |
| 命令 | `cargo test -p homie-storage usage_metrics_schema -- --nocapture` |
| 预期 | 可插入 usage_records，包含 input/output/cache tokens、cache_hit_rate、pricing_snapshot_id、currency、estimated_cost、first_token_latency_ms、total_latency_ms |
| 证据 | `docs/verification/workspace-storage-bootstrap/artifacts/fc-004-usage-schema.txt` |
| 失败处理 | 修复 usage_records schema |

### FC-005: Workspace 质量门禁

| 字段 | 内容 |
|------|------|
| 覆盖需求 | FR-1, FR-6 |
| 前置条件 | Makefile 或 scripts 已实现 |
| 命令 | `make pre-commit` |
| 预期 | fmt check、clippy、cargo test、`.githooks/pre-commit` 全部 exit=0 |
| 证据 | `docs/verification/workspace-storage-bootstrap/artifacts/fc-005-pre-commit.txt` |
| 失败处理 | 修复对应质量门禁 |

### FC-006: Secret scan baseline

| 字段 | 内容 |
|------|------|
| 覆盖需求 | FR-6 |
| 前置条件 | 当前工作区无真实 secret staged |
| 命令 | `.githooks/pre-commit` |
| 预期 | exit=0 |
| 证据 | `docs/verification/workspace-storage-bootstrap/artifacts/fc-006-secret-scan.txt` |
| 失败处理 | 移除敏感内容或修复 hook 误报 |

## 3. 覆盖矩阵

| PRD 需求 | 覆盖 Case |
|----------|-----------|
| FR-1 Rust workspace | FC-005 |
| FR-2 `homie-proto` | FC-005 |
| FR-3 `homie-storage` | FC-001, FC-002, FC-003 |
| FR-4 SQLite schema 初版 | FC-003, FC-004 |
| FR-5 `homie-cli doctor` | FC-001, FC-002 |
| FR-6 质量入口 | FC-005, FC-006 |

## 4. 执行顺序

1. FC-006
2. FC-005
3. FC-001
4. FC-002
5. FC-003
6. FC-004

## 5. 准入结论

Decision: pass

Reason:

- 每个 P0 需求都有可执行 Case。
- Case 都有明确命令、预期结果、证据路径和失败处理方式。
