# Diri Host Prefs Sync 对齐设计文档

```yaml
change_id: diri-host-prefs-sync
beads: homie-cue
target_rows:
  - REM-003
feature_atoms:
  - M06-F002
```

## 1. 背景

`REM-003` 仍为 `missing`。Diri 的 `host.sync_prefs` 只把本机 agent preferences 同步到远端主机，明确排除 credentials、auth、projects/transcripts、todos、caches 等敏感或机器本地数据。

Homie 目前有 remote host validation 和 handoff exclude rules，但没有 prefs sync include-list/argv model。

## 2. 目标

- 在 `homie-remote` 中新增 prefs sync model。
- 固化 Claude/Codex include list。
- 只返回本地存在的可同步项目。
- 生成 mkdir/rsync argv，且永不包含 `--delete`。
- 识别远端 rsync 缺失并返回明确错误信息。

## 3. 非目标

- 不执行 ssh/rsync。
- 不实现 host.locate_repo。
- 不把 `REM-003` 标为 implemented；完整 parity 仍需真实 remote E2E。

## 4. 验收

- `cargo test -p homie-remote --test prefs_sync -- --nocapture`
- `cargo check -p homie-remote`
- `cargo clippy -p homie-remote --all-targets -- -D warnings`
- scoped `git diff --check`
- `make parity-lock`
