# Diri Inspector Artifacts 对齐设计文档

```yaml
change_id: diri-inspector-artifacts
beads: homie-3q0
parent_bead: homie-h7n.4
target_rows:
  - UI-004
  - ART-001
  - ART-002
```

## 1. 概述

### 1.1 问题/背景

Homie runtime 已有 `scan_artifacts`，可以从 session output 中识别 PR links、preview URLs、generic links 和 localhost ports。但 app inspector 仍硬编码 `Ports none`、`PRs none`，没有把 runtime 结果接入用户可见界面。

### 1.2 目标

- `homie-client` 增加 selected session artifact scan API。
- `homie-app` 在 refresh selected session 时扫描 artifacts/ports。
- inspector Artifacts tab/section 展示真实 ports、PRs、previews/links 数量。
- tests 锁定 app 不再硬编码 artifact none。

## 2. 功能需求

### FR-1: Client artifact scan

`HomieClient` 必须通过 runtime output 读取 selected session 输出并返回 `ArtifactScan`。

### FR-2: Inspector real data

`homie-app` 必须维护 artifact summary，并在 inspector 展示真实 scan 结果。

### FR-3: Evidence boundary

本轮可把 `UI-004` 和 `ART-001/ART-002` 证据推进，但保持 `partial`，直到 diff/artifact/browser/port E2E 完成。

## 3. 验收标准

- `cargo test -p homie-client --tests -- --nocapture`
- `cargo test -p homie-app --tests -- --nocapture`
- `cargo clippy -p homie-client -p homie-app --all-targets -- -D warnings`
- `make parity-lock`

