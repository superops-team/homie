# Release Readiness Report: process-tree-governor-verification

## 概述

为 `main` 上已移植的进程树枚举/信号（`process_tree.rs`）与资源 governor
（`governor.rs`）补 TDD 验证。二者属 Tier 3 代码（进程控制、误杀风险、内存回收）。

## 交付物

- `homie/crates/homie-engine/tests/process_tree.rs`（4 个集成测试，真实 fork 子进程树）
- `governor.rs` 新增 `eligibility_tests` 模块（2 个单测，覆盖三道闸 + 非 Idle 状态）
- `docs/verification/process-tree-governor-verification/failure-model.md`（失败模型）

## 测试结果

| 套件 | 命令 | 结果 |
|------|------|------|
| process_tree 集成 | `cargo test -p homie-engine --test process_tree --offline` | 4/4 绿 |
| governor 单测 | `cargo test -p homie-engine --lib governor --offline` | 6/6 绿（含新增 2） |

## 覆盖矩阵

| 关注点 | 覆盖方式 |
|--------|----------|
| 枚举后代 + 进程组，排除 holder | `enumerates_a_live_tree_excluding_the_holder` |
| SIGSTOP 收敛 / SIGCONT 恢复 | `sigstop_converges_and_sigcont_resumes` |
| kill_tree 全树回收 | `kill_tree_reaps_the_whole_tree` |
| PID 复用身份安全 | `signal_skips_a_recycled_pid` |
| governor eligibility 三道闸 | `idle_session_is_eligible_only_when_unattached_unpinned_unhibernated` |
| 非 Idle 状态不可休眠 | `non_idle_statuses_are_never_eligible` |

## 已知边界（记录于 failure-model.md）

- `signal()` 中 `signal_group(root, signal)` 在身份校验前投递整组信号；「pid 复用为另一
  活进程且恰为会话组长」这一极端场景未被覆盖（真实复用难以稳定构造）。当前实现对该
  场景的暴露面极小（holder 仅在子进程存活时调用 `signal`），但若需彻底消除，另立项让
  `signal_group` 先做身份校验。
- 未覆盖真实 Codex/Claude agent 端到端休眠（依赖真实运行时，另行 PRD）。

## 结论

当前 `main` 上 `process_tree.rs` 与 `governor.rs` 实现通过本 change 新增的验证，
安全基线（身份安全、STOP/CONT 收敛、保守 eligibility、禁止全局信号）成立。
本 change 为验证加固，未修改实现。
