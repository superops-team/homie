# Diri Proto Host Sync Prefs Fixtures 设计文档

```yaml
change_id: diri-proto-host-sync-fixtures
beads: homie-vuz
target_rows:
  - API-001
  - REM-003
feature_atoms:
  - M10-F001
```

## 1. 概述

### 1.1 问题/背景

Diri protocol 中 `host.sync_prefs` 使用稳定 wire DTO：`HostSyncPrefsParams`、`PrefsSyncToolReport`、`HostSyncPrefsResult`。Homie `Method` 已包含 `host.sync_prefs`，远程 prefs sync 逻辑也已有独立测试，但 `homie-proto` 尚未定义这些 DTO，导致 protocol contract 仍缺 host sync fixture。

### 1.2 目标

- 在 `homie-proto` 增加 Diri-compatible host sync prefs DTO。
- 覆盖 Diri fixture：per-tool report、`error` 成功时省略、空 synced 表示无配置可同步。
- 不实现实际远程同步。

## 2. 功能需求

### FR-1：HostSyncPrefsParams

字段：`host: String`。

### FR-2：PrefsSyncToolReport

字段：`tool`、`ok`、`synced`、可选 `error`。序列化时 `error=None` 必须省略。

### FR-3：HostSyncPrefsResult

字段：`tools: Vec<PrefsSyncToolReport>`。

## 3. 非目标

- 不实现 host sync 执行。
- 不改 `homie-remote`。
- 不新增 remote node 连接。

## 4. 涉及文件

- `crates/homie-proto/src/lib.rs`
- `crates/homie-proto/tests/protocol_contract.rs`
- `docs/research/diri-parity-lock.md`
- `docs/verification/diri-proto-host-sync-fixtures/`

## 5. 验收标准

- `cargo test -p homie-proto host_sync_prefs_round_trips_diri_wire`
- `cargo test -p homie-proto --tests`
- `cargo check -p homie-proto`
- `cargo clippy -p homie-proto --all-targets -- -D warnings`
- `cargo fmt --all -- --check`
- scoped `git diff --check`
- `make parity-lock`

