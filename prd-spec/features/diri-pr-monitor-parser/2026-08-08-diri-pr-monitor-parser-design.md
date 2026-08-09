# Diri PR Monitor Parser 对齐设计文档

```yaml
change_id: diri-pr-monitor-parser
beads: homie-jkj
target_rows:
  - ART-003
feature_atoms:
  - M04-F002
```

## 1. 背景

`ART-003` 仍为 `missing`。Diri 的 `PullRequestMonitor.swift` 通过 `gh pr view --json` 与 GraphQL 解析 PR 状态、review decision、mergeability、CI checks、评论/评审/线程统计和 +/- 行数。Homie 目前只能从输出中识别 PR URL，没有 PR status parser。

## 2. 目标

- 在 `homie-runtime` 中新增 PR monitor parser 的第一阶段纯模型。
- 解析 `gh pr view --json` fixture，生成 `PullRequestStatus`。
- 解析 GraphQL reviewThreads fixture。
- 提供 `overall` rollup 规则。
- 提供 GitHub PR URL 坐标解析。
- 不调用 `gh`、不访问网络。

## 3. 非目标

- 不实现后台轮询器。
- 不写 UI chips/popover。
- 不把 `ART-003` 标为 implemented；完整 PR monitor 仍需要 runtime polling、storage/session wiring 和 UI E2E。

## 4. 验收

- `cargo test -p homie-runtime --test pr_monitor -- --nocapture`
- `cargo check -p homie-runtime`
- `cargo clippy -p homie-runtime --all-targets -- -D warnings`
- scoped `git diff --check`
- `make parity-lock`
