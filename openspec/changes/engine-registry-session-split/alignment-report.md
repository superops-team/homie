# Engine Registry/Session Persistence Split Alignment Report

## 1. Alignment Summary

Maps `engine-registry-session-split` PRD requirements to OpenSpec tasks for the
P0 child refactor of finding F7 (`registry.rs` live session + persistence dual responsibility).

## 2. Requirement Mapping

| PRD Requirement | OpenSpec Task | Status |
|-----------------|---------------|--------|
| Extract PersistedState + projections (S1) | T2 | Covered |
| Extract store backends (S2) | T3 | Covered |
| Extract migration (S3) | T4 | Covered |
| Extract flusher (S4) | T5 | Covered |
| `registry.rs` < 800 lines | T5 | Covered |
| Disk schema / migration / flush semantics unchanged | T3, T4, T6 | Covered |
| Persistence modules unit-testable without live session | T2–T5 | Covered |
| Behavior unchanged, tests green | T6 | Covered |

## 3. PRD Section Mapping

| PRD Section | Content | Task |
|-------------|---------|------|
| 3.2 目标模块拓扑 | registry/persisted.rs + store.rs + migrate.rs + flusher.rs | T2–T5 |
| 3.3 下沉映射 | 持久化成员 → 目标模块 | T2–T5 |
| 3.4 实施顺序 | S1–S4 | T2–T5 |
| 4.1 测试计划 | 迁移/投影/原子写/非法数据 | T2–T4, T6 |

## 4. Scope Boundary

- Disk schema, migration semantics, and `Registry` public API are out of scope for change.
- `session.rs` state machine rework is out of scope.

## 5. Conclusion

Tasks align with the reviewed PRD; all four slices map to executable tasks with verification.
