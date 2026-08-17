# Engine Control Wire/Runtime Split Alignment Report

## 1. Alignment Summary

Maps `engine-control-wire-runtime-split` PRD requirements to OpenSpec tasks for the
P0 child refactor of finding F2 (`control.rs` dispatcher + runtime coordinator dual responsibility).

## 2. Requirement Mapping

| PRD Requirement | OpenSpec Task | Status |
|-----------------|---------------|--------|
| Extract wire codec (S1) | T2 | Covered |
| Extract proto↔domain projections (S2) | T3 | Covered |
| Extract runtime lifecycle (S3) | T4 | Covered |
| Sink business handlers, keep routing table (S4) | T5 | Covered |
| `control.rs` < 800 lines | T5 | Covered |
| Wire shape unchanged | T2, T6 | Covered |
| Pure modules unit-testable without daemon | T2, T3 | Covered |
| Behavior unchanged, tests green | T6 | Covered |

## 3. PRD Section Mapping

| PRD Section | Content | Task |
|-------------|---------|------|
| 3.2 目标模块拓扑 | control/ + control/wire.rs + codec.rs + runtime.rs | T2–T5 |
| 3.3 下沉映射 | handler → registry/session/remote | T5 |
| 3.4 实施顺序 | S1–S4 | T2–T5 |
| 4.1 测试计划 | codec/projection round-trip + integration | T2, T3, T6 |

## 4. Scope Boundary

- `homie-proto/src/control.rs` and wire shape are out of scope.
- Session state machine rework is out of scope (covered by `engine-registry-session-split`).

## 5. Conclusion

Tasks align with the reviewed PRD; all four slices map to executable tasks with verification.
