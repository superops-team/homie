# Process Tree + Governor 验证补全设计文档

## 1. 背景

`main` 分支已从 diri 移植了进程树枚举与资源 governor 两套核心机制：

- `homie/crates/homie-engine/src/holder/process_tree.rs`（381 行）：进程树枚举与信号投递，
  基于「(pid, start-time) 身份」防 PID 复用误杀；SIGSTOP 收敛、SIGCONT 自叶向根。
- `homie/crates/homie-engine/src/governor.rs`（507 行）：资源扫描（footprint、监听端口、
  artifact 发布）与三类自动休眠策略（硬内存上限、持续空闲冻结、全局预算最旧优先）。

这两套机制是 agent 生命周期安全与内存回收的核心，**属于 Tier 3 代码**（进程控制、并发、
误杀风险、数据/资源回收），但当前 `main` 上：

- `process_tree.rs` **零测试**（`enumerate`/`signal`/`kill_tree` 无任何单元或集成覆盖）。
- `governor.rs` 仅 4 个纯函数单测（`parse_lsof`、`footprint_of`、`should_scan_ports`、
  `physical_memory`），**无** `idle_since`（eligibility）、休眠策略、`sweep` 收敛性的验证。

本 PRD 以 TDD（RED→GREEN→REFACTOR）补全上述验证缺口，先证明「当前实现是否安全」，
再对暴露出的真实缺陷另行立项修复（不在本 change 内混入行为变更）。

## 2. 目标

1. 为 `process_tree.rs` 补集成测试（真实 fork 的 PTY 子进程树），覆盖：
   - `enumerate` 正确枚举后代 + 进程组成员，排除 holder 自身。
   - 身份安全：`start_time` 变化后（PID 复用）不再对该 pid 投递信号。
   - `signal(SIGSTOP)` 收敛（叶子停止）；`signal(SIGCONT)` 自叶向根恢复。
   - `kill_tree` 在 500ms 宽限后 SIGKILL 兜底，且不 `pkill`/全局信号。
2. 为 `governor.rs` 补 `idle_since` 与休眠策略的单元测试，覆盖保守 eligibility：
   - 仅 `Idle` 且 `unattached` 且 `unpinned` 且非已休眠者才可被自动休眠。
   - 硬内存上限、持续空闲、全局预算三类策略的触发边界。
3. 记录失败模型（failure model），对「PID 复用、STOP/CONT 竞态、休眠误伤活跃会话」
   做对抗性推理，证明当前实现满足安全基线或明确列出缺口。

## 3. 非目标

- **不在本 change 内修改 `process_tree.rs` / `governor.rs` 的实现**。若测试暴露真实缺陷，
  记录到 `docs/verification/<change-id>/gaps.md`，另行立项修复。
- 不做真实 Codex/Claude agent 的端到端休眠编排（依赖真实运行时，另行 PRD）。
- 不引入生产环境 env override 或 global signal（明确禁止，见 §6 安全约束）。

## 4. 需求

### FR-1 process_tree 集成测试

新增 `homie/crates/homie-engine/tests/process_tree.rs`（`#![cfg(unix)]`）：

- 用 `setsid` + PTY 或直接 `fork` 出真实子进程树（shell 起后台任务产生后代 + 进程组）。
- 断言 `enumerate(root)`：
  - 包含后代 pid；
  - 不包含 holder 自身 pid；
  - 返回的每个 `HolderProcessSample` 的 `start_sec > 0`。
- 断言 `signal(SIGSTOP)` 后子进程进入 stopped 态；`signal(SIGCONT)` 后恢复运行态。
- 断言 `kill_tree(root)` 后树内进程全部退出（`start_time` 查询返回 `None`）。

### FR-2 身份安全（PID reuse）对抗测试

- 用「spawn 短命子进程 → 退出 → 复用/伪造同一 pid 的 start_time」模拟 PID 复用场景，
  断言 `signal` 不会对 start_time 已变化的 pid 投递信号。
- 由于真实 PID 复用难以在测试中稳定复现，采用「观察 start_time → 人为令目标进程退出 →
  断言信号函数对已消失 pid 跳过」的最小对抗模型，并记录推理链。

### FR-3 governor eligibility 单元测试

- 新增 `idle_since` 相关测试（governor 内部 `#[cfg(test)]` 或集成 test）：
  - `status == Idle` + `unattached` + `unpinned` + 非已休眠 → 返回 `Some`。
  - `attached` → `None`；`pinned` → `None`；已 `hibernation` → `None`；`Working`/`NeedsInput` → `None`。
- 断言三类休眠策略只作用于 `idle_candidates`（`idle_since` 为 `Some` 者）。

### FR-4 失败模型与对抗性推理

- 编写 `docs/verification/process-tree-governor-verification/failure-model.md`，覆盖：
  - PID 复用：为何 (pid, start_time) 身份 + 投递前重新校验能防误杀。
  - STOP/CONT 竞态：SIGSTOP 收敛循环上限（6 次）与 SIGCONT newest-first 的正确性。
  - 休眠误伤：eligibility 三道闸（idle + unattached + unpinned）如何防止冻结活跃/被查看会话。

## 5. 受影响 Specs

- `specs/`：若存在 holder 进程控制 / governor 休眠契约文档，更新其「验证」小节。
- 不改历史 release 报告；本 change 独立成新证据目录
  `docs/verification/process-tree-governor-verification/`。

## 6. 安全约束（Tier 3）

- 禁止使用 `pkill`、`kill(-1, ...)` 或任何全局信号；只对枚举出的树内 pid 投递。
- 禁止生产环境 env override（如 `HOMIE_GOVERNOR_DISABLE`）绕过休眠安全闸。
- 身份校验（start_time 重查）必须发生在每次 `kill` 调用前，不得缓存过期样本。

## 7. 测试计划

- `cargo test -p homie-engine --test process_tree --offline` 全绿。
- `cargo test -p homie-engine --lib --offline`（含 governor 单测）全绿。
- `cargo test -p homie-engine --offline` 全量绿。

## 8. 验收标准

- process_tree 集成测试覆盖枚举/STOP/CONT/kill_tree，且无 flake（子进程就绪重试 + 短超时）。
- governor eligibility 测试覆盖三道闸 + 三类休眠策略边界。
- 失败模型文档完整记录对抗性推理。
- 证据齐全：`docs/verification/process-tree-governor-verification/`。

## 9. Beads 追踪

- change_id: `process-tree-governor-verification`
- 类型: feature（verification hardening）
- 优先级: P1（Tier 3 安全验证）
