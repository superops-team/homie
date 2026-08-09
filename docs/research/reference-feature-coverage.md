# Homie V1 参考功能覆盖矩阵

## 1. 目的

本矩阵用于确认 Homie 第一个产品版本是否覆盖参考工程的完整功能面。文档只描述 Homie 自己的目标能力，不复制外部项目命名。

状态定义：

- `covered-v1`: Homie V1 PRD 明确覆盖。
- `covered-later`: Homie PRD 明确作为 V1 后续阶段覆盖。
- `partial`: Homie PRD 有相关边界，但功能项不完整。
- `missing`: Homie PRD 未覆盖，需要补充。
- `covered-by-reference-parity-v1`: 已由 `reference-parity-v1` PRD/OpenSpec 接管，属于 Homie Reference Parity V1 准出范围。

## 2. 功能覆盖矩阵

| 参考功能面 | Homie 目标能力 | 当前状态 | 处理 |
|------------|----------------|----------|------|
| Rust + GPUI 桌面应用 | `homie-app` + `homie-ui` + GPUI shell | covered-by-reference-parity-v1 | FR-2, FR-7, T-007, T-008 |
| 双进程 app/runtime 架构 | `homie-app` + `homie-runtime` + protocol | covered-by-reference-parity-v1 | FR-2, FR-4, T-002, T-004 |
| 后台 runtime 管理 PTY/agent process | `homie-runtime` session/PTY/process | covered-by-reference-parity-v1 | FR-4, T-004 |
| output log detach/replay | offset-addressed output log + attach/read/replay | covered-by-reference-parity-v1 | FR-4, T-004 |
| headless terminal emulator 状态检测 | runtime headless screen + status reducer | covered-by-reference-parity-v1 | FR-3, FR-4, T-003, T-004 |
| session registry/persistence | SQLite `sessions` + context + output index | covered-by-reference-parity-v1 | FR-4, FR-18, T-004, T-005 |
| holder/PTY master survives runtime crash | holder-equivalent PTY/process ownership | covered-by-reference-parity-v1 | FR-4, T-004 |
| worktrees | workspace/worktree controller and safe cleanup | covered-by-reference-parity-v1 | FR-8, T-010 |
| command palette | command palette with actions and sessions | covered-by-reference-parity-v1 | FR-7, T-009 |
| quick open | folder index and git-aware ranking | covered-by-reference-parity-v1 | FR-7, T-009 |
| session overview board/list | overview board/list with bulk close | covered-by-reference-parity-v1 | FR-7, T-009 |
| history scan and resume | transcript/history scanner + resume | covered-by-reference-parity-v1 | FR-9, T-009 |
| terminal scrollback/selection/find | terminal grid -> scrollback/selection/find | covered-by-reference-parity-v1 | FR-6, T-006 |
| sidebar sections/pinned/archive/drag reorder | advanced sidebar interactions | covered-by-reference-parity-v1 | FR-7, T-008 |
| new session popover | profile/runtime/session creation UI | covered-by-reference-parity-v1 | FR-7, T-008 |
| settings window | General/Terminal/Resources/Remote tabs | covered-by-reference-parity-v1 | FR-7, FR-13, T-008, T-014 |
| right workbench inspector | Info/Changes/Artifacts tabs and diff virtualization | covered-by-reference-parity-v1 | FR-7, FR-10, T-008, T-011 |
| menu bar extra | status rollup/menu bar | covered-by-reference-parity-v1 | FR-7, T-007 |
| notifications approve/deny | native notification actions | covered-by-reference-parity-v1 | FR-7, FR-3, T-007, T-003 |
| sounds | status sounds | covered-by-reference-parity-v1 | FR-7, T-007 |
| usage accounting | usage_records/token/cost/cache/latency | covered-by-reference-parity-v1 | FR-11, T-012 |
| update mechanism | self-updater/release feed | covered-by-reference-parity-v1 | FR-16, T-016 |
| packaging/sign/notarize | macOS packaging pipeline | covered-by-reference-parity-v1 | FR-16, T-016 |
| remote hosts/nodes | remote execution hosts and first-party node | covered-by-reference-parity-v1 | FR-13, T-014 |
| per-node accounts/usage | node account/profile/usage | covered-by-reference-parity-v1 | FR-13, FR-11, T-014, T-012 |
| move/fork/handoff | session handoff between nodes | covered-by-reference-parity-v1 | FR-13, T-014 |
| MCP server for agents to orchestrate agents | Homie MCP automation surface | covered-by-reference-parity-v1 | FR-14, T-013 |
| hook/notify forwarders | agent hook/notify ingestion | covered-by-reference-parity-v1 | FR-14, T-013 |
| status detection manifests | runtime descriptor/status rules | covered-by-reference-parity-v1 | FR-3, T-003 |
| CLI doctor/status | `homie-cli doctor`, status, session/worktree/events | covered-by-reference-parity-v1 | FR-14, T-013 |
| resource governor | runtime resource limits and active state | covered-by-reference-parity-v1 | FR-15, T-004, T-016 |
| hibernate/wake/archive/reopen | session lifecycle controls | covered-by-reference-parity-v1 | FR-4, T-004 |
| port/artifact/PR chips | session metadata surfaces | covered-by-reference-parity-v1 | FR-10, T-011 |
| performance gates | packaged perf budgets/gates | covered-by-reference-parity-v1 | FR-17, T-016 |
| auto-update trust model | update verification | covered-by-reference-parity-v1 | FR-16, T-016 |

## 3. 结论

`reference-parity-v1` PRD 已将原先缺口纳入 Homie Reference Parity V1 准出范围，但这不代表功能已经实现。当前 Diri parity 的权威锁定清单位于 `docs/research/diri-parity-lock.md`；只要 `make parity-lock` 仍列出 incomplete rows，Homie 就不得被描述为 Diri-parity-complete。

后续实现必须按 `docs/research/diri-parity-lock.md` 的行项目拆分 Beads/OpenSpec/验证 Case，更新组件 spec，并在 `docs/verification/` 写入真实验证证据后，才能逐行把状态改为 `implemented`。
