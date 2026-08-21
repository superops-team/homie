# Release Readiness — ci-regression-check-stale-paths

## 变更摘要

修复 `homie/scripts/verify-remote-refactor.sh` 中两处 `grep` 引用已拆分的
`crates/homie-engine/src/control/handlers.rs` 的陈旧路径：

- `self.remote.is_none()` → `control/handlers/migrate.rs`
- `Evicting the corpse` → `control/handlers/resume.rs`

锚点字符串与检查语义不变，仅更新路径。此修复使 CI `engine` job 的 "Verify reviewed regressions
stay fixed" 步骤恢复绿色。

## 验证证据

| 项 | 结果 |
|----|------|
| `bash homie/scripts/verify-remote-refactor.sh` | ✅ 13 项全部 PASS |
| `bash -n homie/scripts/*.sh` | ✅ 语法通过 |

## 已知限制 / 延期

- 无。
