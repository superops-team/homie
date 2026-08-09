# Diri Proto Node Hello Usage Fixtures 设计文档

```yaml
change_id: diri-proto-node-usage-fixtures
beads: homie-7i0
target_rows:
  - API-001
  - REM-001
  - USAGE-001
feature_atoms:
  - M10-F001
  - M19-F001
```

## 1. 概述

### 1.1 问题/背景

Diri first-party node protocol 定义 node hello/status、provider accounts 和 fleet usage DTO。Homie 目前没有 `homie-proto` 级别的 node protocol DTO，导致 API-001/REM-001/USAGE-001 的 node fixtures 仍缺失。

### 1.2 目标

- 增加 node method/capability 常量。
- 增加 provider kind、node hello/status DTO。
- 增加 usage event/query/result DTO。
- 覆盖 Diri wire：camelCase 字段、provider lowercase map key、hello 不序列化 token。

## 2. 范围

本切片只实现 node hello/status/usage DTO，不实现 account login、checkpoint、move lease、node runtime 或网络服务。

## 3. 验收标准

- `cargo test -p homie-proto node_hello_and_usage_match_diri_wire -- --nocapture`
- `cargo test -p homie-proto --tests`
- `cargo check -p homie-proto`
- `cargo clippy -p homie-proto --all-targets -- -D warnings`
- `cargo fmt --all -- --check`
- scoped `git diff --check`
- `make parity-lock`
