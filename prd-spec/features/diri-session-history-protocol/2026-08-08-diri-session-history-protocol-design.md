# Diri Session History Protocol 对齐设计文档

```yaml
change_id: diri-session-history-protocol
beads: homie-ehm
target_rows:
  - AG-004
  - API-001
```

## 1. 背景

Homie 已有 transcript history scanner，但还没有把 `session.history` 暴露到 client/CLI 控制协议。

## 2. 目标

- `HomieClient` 支持 `Method::SESSION_HISTORY`。
- CLI 增加 `homie session history`。
- 测试使用 fixture roots，不扫描真实 HOME。

## 3. 非目标

- 不实现真实 resume E2E。
- 不把 AG-004/API-001 标为 implemented。

## 4. 验收

- `cargo test -p homie-client --test runtime_client history`
- `cargo test -p homie-cli --test session_history_cli -- --nocapture`
- `cargo check -p homie-client -p homie-cli`
- `make parity-lock`
