# Diri Usage Summary CLI 对齐设计文档

```yaml
change_id: diri-usage-summary-cli
beads: homie-163
target_rows:
  - USAGE-001
  - API-003
feature_atoms:
  - M19-F001
```

## 1. 概述

Homie storage 已有 usage ledger 和 aggregate query，但没有用户/自动化可调用的 `usage summary` 入口。Diri usage UI/CLI 需要展示 token/cache/cost 汇总。

## 2. 目标

- 新增 `homie usage summary --data-dir <dir> --json`。
- 支持可选 `--session-id`、`--provider-id`、`--model`、`--from`、`--to`。
- 输出 events、token/cache totals、estimated/billed cost、authoritative flag。

## 3. 非目标

- 不实现 transcript watcher/parser。
- 不实现 usage UI/fleet merge。
- USAGE-001 保持 partial。

## 4. 验收

- `cargo test -p homie-cli --test usage_summary_cli -- --nocapture`
- `cargo check -p homie-storage -p homie-cli`
- `cargo clippy -p homie-storage -p homie-cli --all-targets -- -D warnings`
- scoped `git diff --check`
- `make parity-lock`

