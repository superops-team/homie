# Diri Transcript History Scanner 对齐设计文档

```yaml
change_id: diri-transcript-history-scanner
beads: homie-941
target_rows:
  - AG-004
feature_atoms:
  - M05-F002
```

## 1. 背景

`AG-004` 在 parity lock 中仍为 `missing`：Homie 尚未实现 Codex/Claude transcript/history scanning。此前 storage lane 已提供 `history_entries` schema 和 upsert/list/mark API，但没有真实文件扫描器把 agent transcript 转成 history entries。

Diri 的参考实现位于 `diri/diri/crates/diri-app/src/history.rs`，它只读扫描 `~/.claude/projects` 与 `~/.codex/sessions`，提取 agent session id、cwd、title、transcript path、last active，并可构造 resume 命令。

## 2. 目标

- 在 `homie-runtime` 增加 transcript history scanner。
- 支持 Claude transcript fixture：读取 cwd、首个 user prompt、尾部 ai title，优先使用最新 ai title。
- 支持 Codex transcript fixture：读取 session_meta 和首个 user_message。
- 支持 tracked ids 去重，避免已经由 live sessions 覆盖的历史重复出现。
- 支持将 scan 结果写入 `homie-storage` history API。
- 支持构造安全 resume command spec，不复制 transcript 内容。

## 3. 非目标

- 不扫描真实用户 HOME 作为测试输入。
- 不把 `AG-004` 标为 implemented，直到 app history surface 和 resume E2E 完成。
- 不读取 raw prompt 进入 storage metadata；title 只保存短摘要。
- 不启动真实 Claude/Codex 进程。

## 4. 验收

- `cargo test -p homie-runtime --test history_scanner -- --nocapture`
- `cargo check -p homie-runtime`
- `cargo clippy -p homie-runtime --all-targets -- -D warnings`
- scoped `git diff --check`
- `make parity-lock`
