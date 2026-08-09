# Diri Parity Child Tasks 验证报告

```yaml
change_id: diri-parity-child-tasks
report_type: verification
status: pass
source_lock: docs/research/diri-parity-lock.md
matrix: docs/verification/diri-parity-child-tasks/child-task-matrix.md
checkpoint_commit: 48f522b
```

## 摘要

本次以 `docs/research/diri-parity-lock.md` 当前内容为唯一事实源完成治理重基线：

- lock 当前共有 39 个 non-implemented 行，状态全部为 `partial`；
- child matrix 已从 36 行同步为 39 行，补入 `RT-001`、`RT-006`、`RT-007`；
- `AG-004`、`ART-003`、`AUTO-001`、`REM-002`、`REM-003` 已从过期的 `missing` 同步为 `partial`；
- 39 行的 Current、组件 Owner 和 Required evidence 已逐项与 source lock 对齐；
- `homie-h7n.*` 仅保留为历史来源，不再作为新实施 owner。

## Beads 与依赖

| Master task | 实施 Bead | 状态 | 依赖 | 治理结论 |
|-------------|-----------|------|------|----------|
| T-101 | `homie-nep` | closed | 无未完成依赖 | master task 2.1 已完成 |
| T-102 | `homie-t3u.1` | in_progress | `homie-nep`（closed） | 由独立 child Bead 承接 |
| T-103 | `homie-t3u.2` | in_progress | `homie-nep`（closed） | 由独立 child Bead 承接 |

只读执行 `bd dep cycles --readonly` 返回 `No dependency cycles detected`。`homie-t3u.1` 与 `homie-t3u.2` 均依赖已关闭的 `homie-nep`，未形成反向依赖或环。

历史 Beads `homie-h7n.1` 至 `homie-h7n.5` 只用于追溯旧分组来源；它们不构成当前或后续编码授权。

## T-102 RED 事实

checkpoint `48f522b` 实测：

- 命令：`cargo test -p homie-runtime --test session_lifecycle -- --nocapture`
- 总计：14 tests，12 passed，2 failed；
- 失败：`runtime_reopen_can_adopt_holder_and_continue_session`，实际 `detached`、期望 `running`；
- 失败：`runtime_spawn_shell_uses_live_pty`，实际 `detached`、期望 `running`；
- 已通过：`runtime_holder_stat_tracks_resize_and_log_offsets`。

因此当前 RED 仅为 adoption 与 live PTY 两项；本报告和 child matrix 均不得继续声称存在 3 个失败。本轮为只读治理校验，未重复执行该会写入构建产物的 Rust 测试。

## 验证

| Gate | Command | Result |
|------|---------|--------|
| 39-row 覆盖与逐项一致性 | AWK 比对 lock/matrix 的 ID、Current、Owner、Required evidence，并校验必填字段 | pass，39 rows |
| Parity lock | `make parity-lock` | pass，结构合法并继续列出 39 个 `partial` |
| OpenSpec strict | `openspec validate diri-7ba3407-parity-rebaseline --strict` | pass，change valid |
| Beads 状态与依赖 | `bd show homie-nep homie-t3u.1 homie-t3u.2 --json --readonly` | pass，T-101 closed；T-102/T-103 in_progress 且依赖 closed T-101 |
| 依赖无环 | `bd dep cycles --readonly` | pass，no dependency cycles |
| 文档格式 | `git diff --check -- docs/verification/diri-parity-child-tasks/child-task-matrix.md docs/verification/diri-parity-child-tasks/verification-report.md openspec/changes/diri-7ba3407-parity-rebaseline/tasks.md` | pass |
| 修改范围 | `git diff --name-only` | pass，仅本轮授权的 3 个文件 |

## 完成规则

任何行不得因为进入 child matrix、存在历史 source Bead 或映射到 master task 而改为 `implemented`。只有 source lock 所列 required evidence 通过且证据路径回写后，才允许更新状态。

## 未授权编码结论

本次变更仅同步治理文档，未修改产品代码，也不授权从 master task 或 `homie-h7n.*` 直接编码。T-102、T-103 的实现只能分别在 `homie-t3u.1`、`homie-t3u.2` 的 child PRD/OpenSpec、评审和任务授权范围内进行。

## 门禁结论

Decision: pass

39-row 治理映射、Beads 依赖和 master task 状态与当前事实一致；Diri parity 仍为 incomplete，39 行继续保持 `partial`。
