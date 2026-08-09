# Spec Review Report: Diri MCP wait_for_agent Runtime

```yaml
change_id: diri-mcp-wait-for-agent
beads: homie-trk
status: pass
reviewed_at: 2026-08-08
source_prd: prd-spec/features/diri-mcp-wait-for-agent/2026-08-08-diri-mcp-wait-for-agent-design.md
```

## 1. 总体结论

- 可行性：高。
- 最大风险：把 Diri 的事件长轮询完整机制扩成大改，或只返回 mock/unsupported 而不走真实 runtime status。
- 推荐方向：本 slice 先落 runtime-backed status wait，保持与 `wait_for_children` 同一状态判定函数；事件总线优化后续单独规划。

## 2. 问题清单

| 优先级 | 维度 | 问题 | 影响 | 整改建议 |
|---|---|---|---|---|
| P1 | 可落地性 | Diri 原实现基于 `events.wait` 长轮询，Homie 当前可以先用 status polling。 | 若强行重构事件系统，会扩大范围。 | 明确本 slice 不实现 events.wait 优化，只实现可验证 runtime status wait。 |
| P1 | 参数兼容 | Diri 使用 `session_id`、`timeout_s`，Homie 其它工具也有 `sessionId`。 | 参数不一致会让 managed agents 调用失败。 | 同时接受 snake_case 和 camelCase。 |
| P1 | 超时语义 | 如果 timeout=0 不做一次状态检查，会让已完成 session 误报超时。 | 边界行为不稳定。 | loop 内先读取状态并判定，再检查 deadline。 |
| P2 | 范围 | wait_for_agent 容易和 wait_for_children、release 权限混淆。 | 产生重复实现或权限绕过。 | 只读取目标状态，不修改 lineage 或 session。 |

## 3. 整改后的完善方案

- 目标与范围：实现 MCP `wait_for_agent` 单 session 状态等待。
- 非目标：不实现 browser/test_run，不实现 event-bus long poll，不新增 UI。
- 设计原则：真实 runtime path、结构化超时、Diri 参数拼写兼容。
- 核心方案：`wait_for_agent_payload` 复用 `child_has_reached`；返回 `settled/timedOut/sessionId/status/waitedFor`。
- 风险控制：使用 notify/kill 真实 CLI 路径制造 idle/exited 状态；保留 `wait_for_children` 回归。

## 4. 分层任务拆解

| 层级 | 任务 | 交付物 | 依赖 | 优先级 |
|---|---|---|---|---|
| Spec | 更新 MCP automation spec | `specs/mcp-automation/README.md` | PRD | P1 |
| Test | 新增 wait_for_agent CLI integration tests | `mcp_wait_for_agent_cli.rs` | existing MCP stdio | P1 |
| Logic | 实现 wait_for_agent payload | `crates/homie-cli/src/main.rs` | RED test | P1 |
| Evidence | 验证、review、readiness、parity lock | `docs/verification/...` | GREEN gates | P1 |

## 5. 测试规划

| 类型 | 覆盖点 | 关键用例 | 执行阶段 |
|---|---|---|---|
| TDD integration | done/idle wait | notify turn-complete 后等待 done | 开发中 |
| TDD integration | timeout | running session timeout_s=0 | 开发中 |
| TDD integration | exited | kill 后等待 exited | 开发中 |
| Regression | children wait | `mcp_wait_children_cli` | 实现后 |
| Quality | lint/format/diff/parity | clippy、fmt、diff、make parity-lock | 准出 |

## 6. 开发排期

| 阶段 | 顺序 | 工作项 | 风险与缓冲 | 验收物 |
|---|---|---|---|---|
| S1 | 1 | PRD/spec/OpenSpec/case | 锁定最小范围 | 文档齐备 |
| S2 | 2 | RED test | 确认 unsupported 缺口 | failing test |
| S3 | 3 | status polling 实现 | 避免改 runtime | GREEN test |
| S4 | 4 | 门禁与证据 | 如有回归，回到 S3 | readiness report |

## 7. 待确认问题

- 无。本 slice 的范围足以独立实现。
