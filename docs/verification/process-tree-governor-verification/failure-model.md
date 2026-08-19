# Failure Model: Process Tree + Governor

## 1. 身份安全 —— PID 复用

**风险**：进程退出后其 pid 可被内核回收并分配给全新进程。若缓存旧 pid 直接 `kill`，
会误杀无关进程。

**对策**：`HolderProcessSample { pid, start_sec }` 将「pid + 启动时间」作为进程身份。
`process_tree::signal` / `kill_tree` 在**每次投递信号前**都重新 `start_time(pid)` 校验
身份，只有当前观测到的 start_time 与采样时一致才投递。pid 被复用后 start_time 必然
改变，旧样本被跳过。

**残留风险**：`signal()` 中 `signal_group(root, signal)` 在身份校验**之前**就对整个
进程组 `kill(-root, signal)` 投递。若 root 的 pid 已被复用且新进程也处于同名进程组，
理论上可能误伤。当前测试 `signal_skips_a_recycled_pid` 证明「pid 已消失」时不会产生
signalled 样本，但「pid 被复用为另一活进程」这一更难场景未被覆盖（真实复用难以稳定
构造）。记录为已知边界，若需强化可另立项让 `signal_group` 也先做身份校验。

## 2. STOP/CONT 竞态

**风险**：SIGSTOP 到达前，子进程可能 fork 出新的孙进程，导致「停止」不完整；
恢复时若父先于子运行，父观测到子仍在停止态。

**对策**：
- `signal(SIGSTOP)` 采用收敛循环（最多 6 次）：反复 `enumerate` + 对仍非 stopped 的
  成员补发 SIGSTOP，直到全部观测为 stopped。
- `signal(SIGCONT)` 按 start_sec **倒序**（最新优先）排序，保证子先于父恢复运行。
- 收敛循环有界（6 次），不会无限自旋。

## 3. 休眠误伤活跃/被查看会话

**风险**：内存压力自动休眠若命中用户正在交互或正在查看的会话，会「冻结在用户眼前」。

**对策**：`governor::idle_since` 三道闸，缺一不可：
1. `status == Idle`（非 Working / NeedsInput / Starting）。
2. `attached == false`（无客户端正在查看）。
3. `pinned == false` 且 `hibernation.is_none()`（未固定、未已休眠）。

三类休眠策略（硬内存上限、持续空闲、全局预算）都只作用于 `idle_candidates`
（即 `idle_since` 返回 `Some` 者），因此共享同一 eligibility 闸门。

**残留风险**：`idle_since` 的「空闲起点」取 `last_turn_completed_at` / `updated_at` /
`last_seen_at` 的最大值，若时钟回拨或字段语义变化可能产生偏差，但当前字段语义稳定。

## 4. 禁止项（安全基线）

- 禁止 `pkill`、`kill(-1, ...)` 或任何全局信号；只对枚举出的树内 pid 投递。
- 禁止生产环境 env override 绕过休眠安全闸。
- 身份校验必须发生在每次 `kill` 前，不得缓存过期样本。
