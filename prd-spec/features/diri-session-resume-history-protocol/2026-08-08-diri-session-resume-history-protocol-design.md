# Diri Session Resume From History Protocol 设计文档

```yaml
change_id: diri-session-resume-history-protocol
beads: homie-tpq
target_rows:
  - AG-004
  - API-001
```

## 1. 背景

Homie 已支持扫描 history 和 `session.history`，但还没有 `session.resume_from_history` 控制路径。

## 2. 目标

- Client 支持 `Method::SESSION_RESUME_FROM_HISTORY`。
- CLI 支持 `homie session resume-history`。
- 从 fixture history entry 生成 resume command 并创建 runtime session。
- 不启动真实 Claude/Codex resume E2E；本阶段使用 shell spawn 承载 command/title 证据。

## 3. 验收

- `cargo test -p homie-cli --test session_resume_history_cli -- --nocapture`
- `cargo check -p homie-client -p homie-cli`
- `cargo clippy -p homie-client -p homie-cli --all-targets -- -D warnings`
- scoped `git diff --check`
- `make parity-lock`
