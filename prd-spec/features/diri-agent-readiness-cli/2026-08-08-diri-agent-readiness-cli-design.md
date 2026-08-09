# Diri Agent Readiness CLI 对齐设计文档

```yaml
change_id: diri-agent-readiness-cli
beads: homie-8ua
target_rows:
  - AG-003
  - API-003
feature_atoms:
  - M16-F001
```

## 1. 概述

Diri 的 `agent.readiness` 会把可启动 agent binary 与 descriptor 一起投影给客户端。Homie 已有 `homie-agents` library readiness，但缺少真实 CLI/可调用入口。

## 2. 目标

- 新增 `homie agent readiness --descriptor-dir <dir> --bin-dir <dir> --json`。
- 使用 `AgentCatalog::readiness_with_resolver`，只做 PATH/stat 级别探测，不启动 agent。
- 用 fake executable fixture 验证 available/unavailable。

## 3. 非目标

- 不接 app new-agent UI。
- 不做真实 Claude/Codex 登录检测。
- 不把 `AG-003` 标为 implemented；app/readiness UI E2E 仍 pending。

## 4. 验收

- `cargo test -p homie-cli --test agent_readiness_cli -- --nocapture`
- `cargo check -p homie-agents -p homie-cli`
- `cargo clippy -p homie-agents -p homie-cli --all-targets -- -D warnings`
- scoped `git diff --check`
- `make parity-lock`

