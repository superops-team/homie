# Diri MCP release_agent Ancestor Guard 设计文档

```yaml
change_id: diri-mcp-release-ancestor-guard
beads: homie-4na
target_rows:
  - API-005
  - API-004
feature_atoms:
  - M13-F002
```

## 1. 概述

Homie 已支持 `release_agent` direct child 和 self guard，但 Diri 还要求拒绝释放 parent/ancestor，避免子任务杀掉等待其结果的上游会话。

## 2. 目标

- 补齐 release parent guard。
- 补齐 release ancestor guard。
- 修复当前 release 分支重复调用 terminate 的代码问题。

## 3. 非目标

- 不实现完整 permission profile。
- 不实现 recursive release allow/deny 全矩阵之外的 UI。

## 4. 验收

- `cargo test -p homie-cli --test mcp_release_ancestor_guard_cli -- --nocapture`
- `cargo check -p homie-client -p homie-cli`
- `cargo clippy -p homie-client -p homie-cli --all-targets -- -D warnings`
- scoped `git diff --check`
- `make parity-lock`

