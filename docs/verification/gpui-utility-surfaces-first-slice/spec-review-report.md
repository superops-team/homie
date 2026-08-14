# GPUI UtilitySurfaces 首个生命周期切片 Spec Review 报告

## 1. 总体结论

- 可行性：高。
- 最大风险：把 first slice 扩大为文件拆分或 UI primitives 改造，越过 `homie-yon` 边界。
- 推荐方向：只改 History/Worktrees 的 task 持有和 generation guard，不改 UI 外观、不拆 `surface_shell.rs`、不改 daemon/client API。

## 2. 问题清单

| 优先级 | 维度 | 问题 | 影响 | 处理 |
|--------|------|------|------|------|
| P1 | 范围控制 | 如果顺手拆 `surface_shell.rs` 或新增 primitives，会和 parent program 的其他 child changes 重叠 | PR 变大且与 `gpui-ui-primitives-a11y`、RootView split 冲突 | PRD 明确非目标，不拆文件、不改 primitives |
| P1 | 真实副作用取消语义 | Worktree cleanup 已发给 daemon 后不一定能被 task drop 取消 | 用户可能误以为关闭 overlay 会取消远程副作用 | PRD 明确 task 取消只保护 UI 生命周期；副作用用 generation 防止旧结果回写 |
| P2 | 测试可控性 | 现有 async 依赖 daemon client，不容易直接构造成功/失败竞态 | stale result 难以用真实 daemon 稳定复现 | 允许新增私有 apply/finish helper 以纯测试验证 generation guard |
| P2 | Surface guard | 仅检查 generation 不足以防止关闭 overlay 后回写不可见状态 | 关闭 History/Worktrees 后旧结果仍可能改变状态 | PRD 要求同时检查 generation 和当前 surface |

## 3. 整改后的完善方案

当前 spec 已收敛为最小可落地方案：

1. `UtilitySurfaces` 增加 history/worktrees generation 和 task 字段。
2. History load/resume、Worktrees refresh/cleanup 启动时递增 generation。
3. 异步结果回写前检查 generation 和当前 surface。
4. `close_surface` 清理对应 lifecycle task。
5. 通过 focused tests 验证 stale result 和 dismiss 后不回写。

## 4. 分层任务拆解

| 层级 | 任务 | 交付物 | 依赖 | 优先级 |
|------|------|--------|------|--------|
| Spec | PRD/spec 和 review | PRD + 本报告 | parent specs | P1 |
| Case | 功能验证 Case | `functional-cases.md` | PRD | P1 |
| OpenSpec | plan/tasks/alignment | `openspec/changes/gpui-utility-surfaces-first-slice/*` | Case | P1 |
| Tests | stale result / close guard tests | Rust targeted tests | OpenSpec | P1 |
| Implementation | task fields + generation guard | `surface_shell.rs` | Tests | P1 |
| Verification | Case 执行和 review | verification reports | implementation | P1 |

## 5. 测试规划

| 类型 | 覆盖点 | 关键用例 |
|------|--------|----------|
| Unit/GPUI focused | generation guard | stale history result must not overwrite newer state |
| Unit/GPUI focused | close guard | closing History/Worktrees prevents old result from writing visible state |
| Targeted Rust | existing utility surfaces tests | `cargo test -p homie-app utility_surfaces` |
| Static | formatting and diff | `cargo fmt --check`、`git diff --check` |

## 6. 待确认问题

- 无 P0/P1 待确认问题。
