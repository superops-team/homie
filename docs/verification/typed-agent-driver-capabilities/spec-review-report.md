# Typed Agent Driver Capability Spec Review Report

## 1. 总体结论

- 可行性：中。
- 最大风险：把 typed driver 当作替代运行时，越过现有 manifest、PTY、holder、screen reducer 和 session persistence authority。
- 推荐方向：首阶段只落 capability abstraction、fake driver 和查询链路，不接真实 provider、不新增 MCP 控制面。

## 2. 问题清单

| 优先级 | 维度 | 问题 | 影响 | 修复状态 |
|---|---|---|---|---|
| P0 | 架构边界 | 原 PRD 目标包含 steer/cancel/model/native cursor，但未限定首阶段是否接真实 provider | 可能一次性改 Engine、protocol、client、UI、MCP，形成不可 review 的大变更 | 已修复：首阶段只做 fake driver + capability 查询，真实 provider 拆 child change |
| P0 | authority | typed event 与 screen reducer/status 冲突时谁说了算不清楚 | 可见状态可能被 provider signal 覆盖，破坏现有 terminal truth | 已修复：holder/PTY/output log/screen reducer 仍是事实源，typed event 首阶段只作 signal/diagnostic |
| P1 | 安全 | driver event 没有测试化脱敏边界 | prompt、Authorization、cookie 可能进入日志或 snapshot | 已修复：要求脱敏测试和禁止记录敏感 payload |
| P1 | 协议扩面 | 原 PRD 列出 `session.steer`、`session.cancel_turn`、`agent.models`，未说明是否首阶段新增 wire method | Swift/Rust/client/MCP 可能同步扩面 | 已修复：首阶段可只在 session snapshot/model 暴露 capabilities，不新增 steer/cancel wire method |
| P2 | provider 可行性 | Codex/Claude/OpenCode native 能力差异未确认 | 抽象可能被第一个真实 provider 推翻 | 已修复：所有方法默认 unsupported，真实 provider 接入单独评审 |

## 3. 整改后的完善方案

首阶段建立 `DriverCapabilities`、默认 unsupported error、fake driver contract tests，并把 capabilities 通过 session snapshot 或等价查询暴露给 UI/CLI。无 typed driver 的 manifest agent 必须保持原 spawn/resume/status detection 行为。

真实 Codex/Claude/OpenCode driver、typed steer/cancel wire method、MCP tool surface、rollback/fork/usage event 都不属于首阶段关闭条件。

## 4. 分层任务拆解

| 层级 | 任务 | 交付物 | 依赖 | 优先级 |
|---|---|---|---|---|
| Spec | 补 OpenSpec 三件套 | `openspec/changes/typed-agent-driver-capabilities/*` | 本报告 | P0 |
| Model | 定义 capability/error/event 最小类型 | Rust types + tests | OpenSpec | P0 |
| Fake | fake driver contract | focused tests | Model | P0 |
| Session | capabilities 查询链路 | snapshot/client evidence | Fake | P0 |
| Security | 脱敏和 authority review | evidence report | Session | P1 |

## 5. 测试规划

| 类型 | 覆盖点 | 关键用例 | 执行阶段 |
|---|---|---|---|
| Unit | unsupported 默认行为 | 每个 method 返回稳定 unsupported | 开发中 |
| Unit | serialization | capability 集合稳定序列化 | 开发中 |
| Integration | fake session | capabilities 出现在 snapshot | 开发中 |
| Regression | manifest-only agent | shell/generic 不暴露 typed capabilities | 准出前 |
| Security | event 脱敏 | token/prompt/cookie 不进入 event debug | 准出前 |

## 6. 开发排期

| 阶段 | 顺序 | 工作项 | 风险与缓冲 | 验收物 |
|---|---|---|---|---|
| Phase 0 | 先 | OpenSpec 和 authority mapping | 防止扩成真实 provider 接入 | alignment report |
| Phase 1 | 次 | capability model + fake driver | provider 差异后置 | tests |
| Phase 2 | 后 | snapshot/query + regression | UI/MCP 扩面后置 | verification report |

## 7. 待确认问题

- 首阶段 capability 是放入现有 session snapshot，还是新增只读 query method。推荐优先复用现有 snapshot/model，避免协议扩面。
