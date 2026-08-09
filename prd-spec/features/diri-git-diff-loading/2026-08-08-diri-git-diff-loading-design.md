# Diri Git Diff Loading 对齐设计文档

```yaml
change_id: diri-git-diff-loading
beads: homie-xsr
target_rows:
  - GIT-001
  - UI-004
  - API-001
feature_atoms:
  - M04-F001
  - M07-F001
  - M10-F001
```

## 1. 概述

### 1.1 问题/背景

Diri 的 inspector Changes 面板依赖 `diff.rs` / `WorktreeDiffLoader.swift` 生成统一 patch、文件/行统计和 hunk 行信息。Homie 当前 artifact inspector 已有基础，但没有 `session.read_diff` DTO、runtime diff loader、client/CLI 可验证入口。

### 1.2 目标

- 在 Homie proto 中增加 Diri-compatible `SessionDiffBase`、`SessionReadDiffRequest`、`SessionReadDiffResult`。
- `patch` 字段按 Diri wire 语义使用 base64 bytes。
- 在 runtime 增加 git diff loader，覆盖 tracked + untracked 文件、default branch / HEAD 两种 base。
- 在 client 增加 `read_diff` 和 method dispatch。
- 在 CLI 增加 `homie session diff --id <session> [--base default-branch|head]`。
- 用真实 git repo + runtime session E2E 验证。

## 2. 非目标

- 不实现 GPUI Changes 面板。
- 不做远端 daemon diff channel。
- 不把 `UI-004` / `GIT-001` 标为 implemented，直到 app inspector E2E 完成。

## 3. 验收标准

- `cargo test -p homie-proto session_read_diff_uses_diri_base64_wire -- --nocapture`
- `cargo test -p homie-runtime --test git_diff_loading -- --nocapture`
- `cargo test -p homie-cli --test session_diff_cli -- --nocapture`
- `cargo check -p homie-proto -p homie-runtime -p homie-client -p homie-cli`
- `cargo clippy -p homie-proto -p homie-runtime -p homie-client -p homie-cli --all-targets -- -D warnings`
- scoped `git diff --check`
- `make parity-lock`

