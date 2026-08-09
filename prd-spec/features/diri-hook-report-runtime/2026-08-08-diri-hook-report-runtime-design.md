# Diri Hook Report Runtime 持久化对齐设计文档

```yaml
change_id: diri-hook-report-runtime
beads: homie-b6o
target_rows:
  - RT-005
  - API-003
feature_atoms:
  - M12-F002
  - M14-F002
```

## 1. 概述

Homie 已有 Claude hook / Codex notify 解析与脱敏 CLI 输出，但 `hook.report` 结果没有写入 runtime/session 状态。Diri 的 hook report 会驱动 session needs-input、turn-complete 等状态，避免只在 CLI 输出里“看起来解析成功”。

## 2. 目标

- `homie hook --data-dir <dir> <event> <payload>` 在保持 JSON 输出的同时，把非 subagent 的 needs-input 写入对应 session。
- `session snapshot` 可读到 persisted `needsInput` 和 `needs_input` status。
- 无 `--data-dir` 时保持 fail-open parse-only 行为。

## 3. 非目标

- 不实现完整 hook event bus。
- 不实现所有 hook 状态迁移。
- 不把 `RT-005` 标为 implemented；runtime persisted hook report 后仍需完整 hook bus/status 矩阵。

## 4. 验收

- `cargo test -p homie-cli --test hook_report_runtime_cli -- --nocapture`
- `cargo check -p homie-storage -p homie-runtime -p homie-client -p homie-cli`
- `cargo clippy -p homie-storage -p homie-runtime -p homie-client -p homie-cli --all-targets -- -D warnings`
- scoped `git diff --check`
- `make parity-lock`

