# Diri Proto Node Checkpoint Move Fixtures 设计文档

```yaml
change_id: diri-proto-node-checkpoint-move
beads: homie-uxm
target_rows:
  - API-001
  - REM-001
feature_atoms:
  - M10-F001
```

## 1. 概述

Diri first-party node protocol 包含 checkpoint 和 move lease DTO，用于远程 session handoff。Homie 已补 node hello/status/usage DTO，但 checkpoint/move DTO 尚缺。该切片只补 `homie-proto` DTO 和 serde fixtures，不实现 node runtime。

## 2. 目标

- 增加 checkpoint prepare/manifest/blob/stage DTO。
- 增加 move commit/abort/record DTO。
- 覆盖 camelCase、optional skip、hex chunk 等 Diri wire 合同。

## 3. 非目标

- 不实现 checkpoint 文件传输。
- 不实现 move lease runtime。
- 不实现 remote spawn/handoff E2E。

## 4. 验收标准

- `cargo test -p homie-proto node_checkpoint_move_match_diri_wire -- --nocapture`
- `cargo test -p homie-proto --tests`
- `cargo check -p homie-proto`
- `cargo clippy -p homie-proto --all-targets -- -D warnings`
- `cargo fmt --all -- --check`
- scoped `git diff --check`
- `make parity-lock`
