# Diri Codex Notify Runtime 持久化对齐设计文档

```yaml
change_id: diri-notify-runtime
beads: homie-qki
target_rows:
  - RT-005
  - API-003
feature_atoms:
  - M12-F002
  - M14-F002
```

## 1. 概述

Homie 已解析 Codex notify，但 `agent-turn-complete` 不影响 runtime session 状态。Diri 使用 notify 作为 Codex turn complete 的状态信号。

## 2. 目标

- `homie notify --data-dir <dir> <payload>` 在 parse 输出之外，把 `agent-turn-complete` 对应 session 标记为 idle。
- `session snapshot` 可看到 `status.status=idle`。
- 无 `--data-dir` 时保留 parse-only fail-open 行为。

## 3. 非目标

- 不实现完整 Codex notify bus。
- 不实现所有 notify 类型。
- RT-005 保持 partial。

## 4. 验收

- `cargo test -p homie-cli --test notify_runtime_cli -- --nocapture`
- `cargo check -p homie-runtime -p homie-client -p homie-cli`
- `cargo clippy -p homie-runtime -p homie-client -p homie-cli --all-targets -- -D warnings`
- scoped `git diff --check`
- `make parity-lock`

