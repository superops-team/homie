# Diri Proto Host Catalog and Remote Config 设计文档

```yaml
change_id: diri-proto-host-remote-config
beads: homie-5kx
target_rows:
  - API-001
  - REM-002
  - REM-003
feature_atoms:
  - M10-F001
```

## 1. 概述

### 1.1 问题/背景

Diri protocol 定义了 `hosts.json` 和 `remote.json` 的稳定模型：`HostsConfig`、`HostEntry`、`HostNodeConfig`、`RemoteConfig`。Homie remote crate 有相近业务模型和测试，但 `homie-proto` 缺少这些 protocol DTO，导致 API-001 的 host/remote fixtures 仍未完整覆盖。

### 1.2 目标

- 在 `homie-proto` 增加 host catalog DTO。
- 在 `homie-proto` 增加 remote config DTO。
- 覆盖 camelCase、minimal schema、legacy remote config、token 字段存在但不额外泄漏。
- 不实现文件保存、权限、远程连接或 UI。

## 2. 功能需求

### FR-1：Host catalog DTO

实现：

- `HostNodeConfig`
- `HostEntry`
- `HostsConfig`

### FR-2：Remote config DTO

实现：

- `RemoteConfig`

### FR-3：Serde fixtures

测试必须覆盖 Diri documented schema、minimal host entry、empty hosts object、remote legacy file。

## 3. 非目标

- 不实现 `load/save` 文件权限逻辑。
- 不改 `homie-remote`。
- 不实现真实 remote node/SSH。

## 4. 涉及文件

- `crates/homie-proto/src/lib.rs`
- `crates/homie-proto/tests/protocol_contract.rs`
- `docs/research/diri-parity-lock.md`
- `docs/verification/diri-proto-host-remote-config/`

## 5. 验收标准

- `cargo test -p homie-proto host_catalog_and_remote_config_match_diri_wire`
- `cargo test -p homie-proto --tests`
- `cargo check -p homie-proto`
- `cargo clippy -p homie-proto --all-targets -- -D warnings`
- `cargo fmt --all -- --check`
- scoped `git diff --check`
- `make parity-lock`

