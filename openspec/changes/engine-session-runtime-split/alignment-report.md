# Engine Session Runtime Split Alignment Report

## 1. Alignment Summary

Maps `engine-session-runtime-split` PRD requirements to OpenSpec tasks for the P0
refactor: split `session.rs` (2,888 lines) into cohesive submodules and sink
spawn/resume/migrate handler logic into the session/registry domain.

## 2. Requirement Mapping

| PRD Requirement | OpenSpec Task | Status |
|-----------------|---------------|--------|
| Split status reducer | T2 | Done |
| Split screen/grid | T3 | Done |
| Split PTY I/O | T4 | Done |
| Split lifecycle | T5 | Done |
| `session.rs` → `session/` submodules < 800 lines | T6 | Done |
| Sink resume spec construction | T7 | Done |
| Thin spawn/resume/migrate handlers | T8 | Partial (resume done; spawn/migrate remaining) |
| Behavior unchanged, tests green | T2–T9 | Done (303 pass) |
| `specs/engine-session-runtime.md` boundary preserved | T1 | Done |
| Failure model + Tier 3 evidence (PTY lifecycle) | T9 | Done |

## 3. PRD Section Mapping

| PRD Section | Content | Task |
|-------------|---------|------|
| 3.1 拆分拓扑 | session/{lifecycle,screen,pty,status}.rs + mod.rs | T2–T6 |
| 3.2 下沉拓扑 | decode→domain→encode thin handlers | T7–T8 |
| 2.1 session.rs 职责映射 | 四类职责 → 子模块 | T2–T5 |
| 2.2 handlers 现状 | spawn/resume/migrate 下沉 | T7–T8 |
| 5 测试计划 | RED→GREEN→REFACTOR + Tier 2/3 | T1, T9 |

## 4. Scope Boundary

- Wire shape / method names / JSON semantics: out of scope (unchanged).
- `Session` public API: out of scope (unchanged).
- New persistence backends: out of scope.
- State-machine semantic rework: out of scope (verbatim move).
- Real provider typed driver: out of scope.

## 5. Conclusion

Tasks align with the reviewed PRD; all seven slices map to executable tasks with
verification and the PTY-lifecycle Tier 3 failure model is scheduled under T9.

## 6. Remaining scope (follow-up child)

`session_spawn` / `session_spawn_remote` / `session_migrate` 的 spawn-spec 组装下沉（S7）
未完成：handler 仍直接访问 `ControlServer` 私有字段组装 `SessionSpec`。本 change 已交付
session 拆分 + resume spec 下沉；spawn/migrate 下沉作为后续增量，wire 协议与行为不变。
