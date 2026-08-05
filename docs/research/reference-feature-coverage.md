# Homie V1 参考功能覆盖矩阵

## 1. 目的

本矩阵用于确认 Homie 第一个产品版本是否覆盖参考工程的完整功能面。文档只描述 Homie 自己的目标能力，不复制外部项目命名。

状态定义：

- `covered-v1`: Homie V1 PRD 明确覆盖。
- `covered-later`: Homie PRD 明确作为 V1 后续阶段覆盖。
- `partial`: Homie PRD 有相关边界，但功能项不完整。
- `missing`: Homie PRD 未覆盖，需要补充。

## 2. 功能覆盖矩阵

| 参考功能面 | Homie 目标能力 | 当前状态 | 处理 |
|------------|----------------|----------|------|
| Rust + GPUI 桌面应用 | `homie-app` + `homie-ui` + GPUI shell | covered-v1 | 已在 V1 PRD |
| 双进程 app/runtime 架构 | `homie-app` + `homie-runtime` + protocol | covered-v1 | 已在 V1 PRD |
| 后台 runtime 管理 PTY/agent process | `homie-runtime` session/PTY/process | covered-v1 | 已在 V1 PRD |
| output log detach/replay | output log + `session.read_output` + terminal attach | partial | PRD 已有 output log/read_output，需补 offset-addressed replay 细节 |
| headless terminal emulator 状态检测 | `homie-runtime` + `homie-term` grid/screen | covered-v1 | 已在 V1 PRD |
| session registry/persistence | SQLite `sessions` + context | covered-v1 | 已在 V1 PRD |
| holder/PTY master survives runtime crash | runtime independent survival | covered-later | PRD 标为 V1.1，需显式功能项 |
| worktrees | workspace/worktree controller | missing | 需要补 `worktree.*` 能力 |
| command palette | command palette | missing | 需要补 UI surface |
| quick open | quick open | missing | 需要补 UI surface |
| session overview board/list | session overview | missing | 需要补 UI surface |
| history scan and resume | transcript/history scanner + resume | partial | PRD 有历史输出入口，不够完整 |
| terminal scrollback/selection/find | terminal grid -> scrollback/selection/find | covered-v1 | 已有演进路径 |
| sidebar sections/pinned/archive/drag reorder | session sidebar advanced interactions | partial | PRD 只有基础 sidebar |
| new session popover | profile/runtime/session creation UI | partial | PRD 有新建 session/profile，但未写 popover 行为 |
| settings window | provider/profile/permission/settings | partial | PRD 有 settings，需补设置 tabs |
| menu bar extra | status rollup/menu bar | missing | 需要补 macOS native integration |
| notifications approve/deny | native notification actions | missing | 需要补 system bridge |
| sounds | status sounds | missing | 需要补 optional UI feedback |
| usage accounting | usage_records/token/cost/cache/latency | covered-v1 | 已在 V1 PRD |
| update mechanism | self-updater/release feed | missing | 需要补 updater/packaging |
| packaging/sign/notarize | macOS packaging pipeline | missing | 需要补 packaging spec |
| remote hosts/nodes | remote execution hosts | covered-later | PRD 说不做远端 fleet，但第一个完整产品应作为 V1.x |
| per-node accounts/usage | node account/profile/usage | covered-later | 与 remote execution 绑定 |
| move/fork/handoff | session handoff between nodes | covered-later | V1.x |
| MCP server for agents to orchestrate agents | Homie MCP surface | missing | 需区别于 MCP server proxy；当前只说 MCP proxy deferred |
| hook/notify forwarders | agent hook/notify ingestion | partial | PRD 有 HookPrimary，但无 CLI/hook surface |
| status detection manifests | runtime descriptor/status rules | covered-v1 | 已在 V1 PRD |
| CLI doctor/status | `homie-cli doctor`, status | covered-v1 | 已在 V1 PRD/实现 |
| resource governor | runtime resource limits | missing | 需要补 runtime capability |
| hibernate/wake/archive/reopen | session lifecycle controls | partial | PRD 只有 terminate/history |
| port/artifact/PR chips | session metadata surfaces | missing | 需要补 artifact/port/PR monitor |
| performance gates | perf budgets/gates | partial | quality gates 有性能门禁，PRD 无具体预算 |
| auto-update trust model | update verification | missing | 需要补 updater spec |

## 3. 结论

当前 Homie V1 PRD 覆盖了核心架构、runtime、SQLite、LLM proxy、agent profile 和 usage metrics，但尚未完整覆盖参考工程的产品功能面。需要把以下能力补回 PRD：

- worktree management；
- command palette / quick open / overview / history / advanced sidebar；
- menu bar / native notification actions / sounds；
- packaging / updater / performance gate；
- resource governor；
- session lifecycle controls: archive, hibernate, wake, reopen；
- artifact/port/PR metadata surfaces；
- Homie MCP control surface；
- remote execution hosts / node accounts / handoff 作为 V1.x 阶段。
