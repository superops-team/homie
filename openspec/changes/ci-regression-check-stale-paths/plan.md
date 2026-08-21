# OpenSpec Plan — ci-regression-check-stale-paths

## 目标

修复 `homie/scripts/verify-remote-refactor.sh` 中两处 `grep` 引用已拆分的
`crates/homie-engine/src/control/handlers.rs` 的陈旧路径，使其指向拆分后的
`control/handlers/migrate.rs` 与 `control/handlers/resume.rs`。

## 交付切片

- T1：定位两处陈旧路径。
- T2：更新路径，运行脚本验证。
- T3：code review + release readiness。
