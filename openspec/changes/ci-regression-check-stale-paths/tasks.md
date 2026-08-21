# OpenSpec Tasks — ci-regression-check-stale-paths

## T1 定位陈旧路径

- [x] 定位 `verify-remote-refactor.sh` 中 `grep -q ... control/handlers.rs` 两处。
- 验收：两处均为已不存在的路径。关联 FR-1/FR-2。

## T2 更新路径 + 验证

- [x] `migrate` 检查改为 `control/handlers/migrate.rs`，`resume` 检查改为 `control/handlers/resume.rs`。
- [x] `bash homie/scripts/verify-remote-refactor.sh` 全部 PASS。
- [x] `bash -n homie/scripts/*.sh` 语法通过。
- 验收：13 项检查全绿。关联 C1/C2。

## T3 code review + release readiness

- [x] code review：仅路径修正，锚点字符串与检查语义不变。
- [x] release readiness 证据写入 `docs/verification/ci-regression-check-stale-paths/`。
- 验收：通过。关联 C3。
