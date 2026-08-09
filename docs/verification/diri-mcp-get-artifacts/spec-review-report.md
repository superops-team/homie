# Spec Review Report: Diri MCP get_artifacts Runtime

```yaml
change_id: diri-mcp-get-artifacts
beads: homie-pyt
status: pass
reviewed_at: 2026-08-08
source_prd: prd-spec/features/diri-mcp-get-artifacts/2026-08-08-diri-mcp-get-artifacts-design.md
```

## 1. 总体结论

- 可行性：高。
- 最大风险：把 `get_artifacts` 与 PR live stats、browser pool、test_run 混成一个大改。
- 推荐方向：本 slice 只接 MCP 到现有 `HomieClient::scan_session_artifacts`，返回 artifacts 和 listeningPorts。

## 2. 问题清单

| 优先级 | 维度 | 问题 | 影响 | 整改建议 |
|---|---|---|---|---|
| P1 | 范围控制 | Diri `get_artifacts` 可附带 PR live stats，但 Homie 当前 PR monitor 是独立 partial。 | 若强行合并会扩大范围。 | 本阶段不返回 `pr` live stats，readiness 明确 scope limit。 |
| P1 | 真实路径 | 只测试静态 scanner 不足以证明 MCP 可用。 | MCP 仍可能 unsupported。 | 用真实 session 输出 + MCP stdio 测试。 |
| P1 | 参数兼容 | Diri 使用 `session_id`，Homie 其它工具常用 `sessionId`。 | agent 调用不稳定。 | 两种拼写都接受。 |
| P2 | 输出命名 | Diri 返回 `listeningPorts`，Homie runtime 类型叫 `ports`。 | 上游工具链消费失败。 | MCP 边界返回 `listeningPorts`。 |

## 3. 整改后的完善方案

- 目标与范围：runtime-backed MCP `get_artifacts`。
- 非目标：不实现 browser/test_run，不实现 PR live stats，不改 scanner 算法。
- 设计原则：复用 client scanner、真实 session output E2E、Diri 输出命名。
- 核心方案：新增 `get_artifacts` payload 分支，返回 `{sessionId, artifacts, listeningPorts}`。
- 风险控制：artifact scanner 和 ports CLI regression 一起跑。

## 4. 分层任务拆解

| 层级 | 任务 | 交付物 | 依赖 | 优先级 |
|---|---|---|---|---|
| Spec | 更新 MCP automation spec | `specs/mcp-automation/README.md` | PRD | P1 |
| Test | 新增 MCP get_artifacts E2E | `mcp_get_artifacts_cli.rs` | existing runtime scanner | P1 |
| Logic | MCP payload dispatch | `crates/homie-cli/src/main.rs` | RED test | P1 |
| Evidence | 验证、review、readiness、parity lock | `docs/verification/...` | GREEN gates | P1 |

## 5. 测试规划

| 类型 | 覆盖点 | 关键用例 | 执行阶段 |
|---|---|---|---|
| TDD integration | MCP artifact/port scan | session output -> get_artifacts | 开发中 |
| TDD negative | missing session id | invalid params | 开发中 |
| Regression | scanner/ports | `artifact_scanner`, `ports_cli` | 实现后 |
| Quality | lint/format/diff/parity | clippy、fmt、diff、make parity-lock | 准出 |

## 6. 开发排期

| 阶段 | 顺序 | 工作项 | 风险与缓冲 | 验收物 |
|---|---|---|---|---|
| S1 | 1 | PRD/spec/OpenSpec/case | 锁定 scope | 文档齐备 |
| S2 | 2 | RED test | 确认 unsupported 缺口 | failing test |
| S3 | 3 | dispatch 实现 | 不改 scanner | GREEN test |
| S4 | 4 | 门禁与证据 | 如有回归，回到 S3 | readiness report |

## 7. 待确认问题

- 无。本 slice 可独立开发。
