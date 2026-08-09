# Wave 1A 多 TraeCLI 执行编排

```yaml
change_id: diri-runtime-daemon-client-transport
beads: homie-nep
branch: feature/diri-runtime-daemon-client-transport
execution_model: phased_parallel
max_concurrent_traecli: 4
```

## 1. 优先级

| 优先级 | 范围 | 原因 |
|--------|------|------|
| P0 | Tasks 1-14：proto、daemon、client | 所有 consumer 和 E2E 的阻塞依赖 |
| P1 | Tasks 15-17：CLI/MCP、GPUI | 产品入口必须迁移后才能删除 embedded client |
| P1 | Task 18：package closure | daemon binary 完成后可与 consumer 迁移并行 |
| P0 release gate | Tasks 19-20：E2E、review、evidence | 未通过不得关闭 Bead 或更新 parity |

P0 release gate 的优先级高于功能数量：如果 shared-daemon、reconnect、event gap 或 stream reset E2E 失败，本 change 不准出。

## 2. 依赖 DAG

```text
Wave A
  Tasks 1-3 protocol/DTO/frame
        |
        +-----------------------------+
        |                             |
Wave B |                             |
  Tasks 5-11 runtime daemon      Tasks 4,12-14 async client
        |                             |
        +---------------+-------------+
                        |
        +---------------+------------------+
        |               |                  |
Wave C |               |                  |
  Tasks 15-16       Task 17             Task 18
  CLI/MCP           GPUI bridge         package closure
        |               |                  |
        +---------------+------------------+
                        |
Wave D
  Tasks 19-20 cross-entry E2E/review/evidence
```

不能提前并行：

- runtime/client 不能在 Tasks 1-3 DTO/frame 定型前写 transport。
- CLI/MCP/app 不能在 typed async client API 定型前迁移。
- package smoke 不能在 daemon binary 和 launcher path 定型前完成。
- E2E 不能用 fake backend 代替 Wave B/C 的真实产物。

## 3. TraeCLI 分工

### Wave A

| TraeCLI | 类型 | 任务 | 写入范围 |
|---------|------|------|----------|
| `protocol_wave_a` | 编码 | Tasks 1-3 | `Cargo.toml`, `Cargo.lock`, `crates/homie-proto/**` |
| `runtime_mapping` | 只读 | 提前消除 Tasks 5-11 不确定性 | 无 |
| `consumer_mapping` | 只读 | 提前消除 Tasks 15-17 不确定性 | 无 |
| `e2e_package_mapping` | 只读 | 提前消除 Tasks 18-20 不确定性 | 无 |

### Wave B

Wave A GREEN 后启动：

| TraeCLI | 任务 | 写入范围 |
|---------|------|----------|
| `runtime_wave_b` | Tasks 5-11：RuntimeActor + single-worker LongRunningLane + daemon/streams | `crates/homie-runtime/**` |
| `client_wave_b` | Tasks 4,12-14 | `crates/homie-client/**`; `homie-proto` 只允许必要的 reviewed follow-up |

两条轨道不可互改对方 crate。协议 follow-up 由主线程串行合并，避免两个 agent 同改 proto。

### Wave C

Wave B 共同 GREEN 后启动：

| TraeCLI | 任务 | 写入范围 |
|---------|------|----------|
| `cli_mcp_wave_c` | Tasks 15-16 | `crates/homie-cli/**` |
| `app_wave_c` | Task 17 | `crates/homie-app/**` |
| `package_wave_c` | Task 18 | `scripts/package/**` |

### Wave D

| TraeCLI | 任务 | 写入范围 |
|---------|------|----------|
| `e2e_wave_d` | Task 19 | cross-entry test/support files only |
| 主线程 | Task 20 | `Makefile`、integration fixes、全量门禁、两轮 review、evidence、Bead |

Task 20 不完全委托，主线程必须独立验证 agent 报告和最终 diff。

## 4. 共享工作区规则

当前分支从含 66 项现有变更的工作区创建。所有 TraeCLI 必须遵守：

1. 不执行 `git reset`、`git checkout --`、`git clean`、`git add`、`git commit`、`git merge`。
2. 不修改 `.gitignore`；其 canonical docs ignore 风险由用户单独决策。
3. 不覆盖已有未提交改动，只做增量修改。
4. 同一时间只有一个 TraeCLI 拥有某个文件。
5. RED 测试必须先运行并确认因缺功能失败，再写 production code。
6. 每个 TraeCLI 必须报告修改文件、RED/GREEN 命令、退出码、剩余风险。
7. agent 报告“通过”不等于主线程 gate 通过；主线程重跑验证。

## 5. Wave Gate

### Wave A -> Wave B

- `cargo test -p homie-proto` 通过。
- frame hostile-input tests 通过。
- DTO/Method inventory 覆盖现存 app/CLI/MCP 调用。
- `git diff --check` 通过。

### Wave B -> Wave C

- runtime actor/server/stream focused tests 通过。
- client request/reconnect/stream tests 通过。
- `homie-client` normal dependency 不含 runtime/storage。
- exact capability == handler/opener registry。
- real daemon process Hello/snapshot/singleton tests 通过。

### Wave C -> Wave D

- CLI/MCP runtime suites 通过，method-not-found 为 `-32601`。
- app bridge/first-frame/compile gate 通过。
- package bundle 包含固定路径 daemon。
- app、CLI、MCP 不再创建 embedded runtime。

### Final Gate

- cross-entry same-instance/restart/event-gap/terminal-reopen E2E 通过。
- fmt/check/clippy/focused/workspace tests 运行并归因。
- 两轮 code review 问题修复。
- evidence 状态只使用 pass/blocked/not_run/partial/fail。
- T-102 holder blocker不被误报为 Wave 1A 完整 runtime pass。

## 6. Failure Policy

- 单 TraeCLI RED 不符合预期：暂停该轨道，修正测试，不允许进入 GREEN。
- 同一轨道连续三次遇到相同 blocker：停止并上报主线程决策。
- Wave A 失败：不启动任何写代码的 Wave B agent。
- Wave B 任一轨失败：不迁移 consumer，不删除 embedded API。
- Wave C 任一入口失败：Task 19 只能运行诊断，不得形成 pass evidence。
- 发现未规划的跨 crate API：先更新 PRD/spec/OpenSpec，再串行修改 proto。
