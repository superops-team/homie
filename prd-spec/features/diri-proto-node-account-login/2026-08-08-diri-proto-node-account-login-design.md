# Diri Proto Node Account Login Fixtures 设计文档

```yaml
change_id: diri-proto-node-account-login
beads: homie-05q
target_rows:
  - API-001
  - REM-001
feature_atoms:
  - M10-F001
```

## 1. 概述

Diri node protocol 包含 provider account catalog、installation status、login challenge 和 provider call DTO。Homie 已补 node hello/usage/checkpoint/move DTO，但 account/login DTO 仍缺失。该切片只补 `homie-proto` serde DTO，不实现账号登录流程。

## 2. 目标

- 增加 account profile/catalog/upsert/default DTO。
- 增加 installation status 和 login DTO。
- 增加 provider call DTO。
- 覆盖 Diri camelCase 和 optional omission。

## 3. 非目标

- 不实现账号存储。
- 不实现登录轮询。
- 不实现 provider call runtime。

## 4. 验收标准

- `cargo test -p homie-proto node_account_login_match_diri_wire -- --nocapture`
- `cargo test -p homie-proto --tests`
- `cargo check -p homie-proto`
- `cargo clippy -p homie-proto --all-targets -- -D warnings`
- `cargo fmt --all -- --check`
- scoped `git diff --check`
- `make parity-lock`
