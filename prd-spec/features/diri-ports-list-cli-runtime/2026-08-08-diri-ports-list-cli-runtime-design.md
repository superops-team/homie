# Diri Ports List CLI Runtime 对齐设计文档

```yaml
change_id: diri-ports-list-cli-runtime
beads: homie-979
target_rows:
  - ART-002
  - API-003
feature_atoms:
  - M04-F002
  - M12-F001
```

## 1. 概述

### 1.1 问题/背景

Diri 提供 `dirijor ports`，从 daemon 的 session list 中汇总每个 session 产出的 listening ports，并按 port/session 排序输出。Homie 当前已有 output artifact scanner，可识别 `http://localhost:<port>`，但没有 runtime/client/CLI 层面的 Diri 风格 ports list 入口。

### 1.2 目标

- 在 Homie client 中增加跨 session port 汇总能力。
- 在 Homie CLI 中增加 `homie ports --data-dir <dir> [--json]`。
- 用真实 runtime session 输出 fixture 验证 ports list。
- 更新 parity lock，但不声明 TCP forwarding 完成。

## 2. 用户场景

**Given** 一个运行中的 session 输出了 preview URL。  
**When** 用户运行 `homie ports --json`。  
**Then** CLI 返回 port、url、session id、session title。

## 3. 功能需求

### FR-1: Port row model

Homie 必须输出 `port/url/sessionId/sessionTitle`。

### FR-2: Runtime-backed aggregation

CLI 必须通过 `HomieClient` 读取真实 session output 并使用已有 scanner 汇总。

### FR-3: Empty state

无 port 时 human 输出 `No listening ports tracked.`，JSON 输出空数组。

## 4. 非目标

- 不实现 TCP forwarding。
- 不实现远端 token / remote host ports。
- 不把 `ART-002` 标为 implemented。

## 5. 验收标准

- `cargo test -p homie-cli --test ports_cli -- --nocapture`
- `cargo check -p homie-client -p homie-cli`
- `cargo clippy -p homie-client -p homie-cli --all-targets -- -D warnings`
- scoped `git diff --check`
- `make parity-lock`

