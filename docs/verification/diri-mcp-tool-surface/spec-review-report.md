# Spec Review Report: Diri MCP Runtime-backed Tool Surface

## 1. 总体结论

- 可行性：高。
- 最大风险：把 runtime-backed 基础工具误报成完整 MCP/lineage parity。
- 推荐方向：先补真实 runtime-backed session tools，保留 API-004/API-005 partial。

## 2. 问题清单

| 优先级 | 维度 | 问题 | 影响 | 整改建议 |
|---|---|---|---|---|
| P1 | 真实性 | 当前 `mcp-stdio` 返回静态 `agents: []` | agent 无法实际编排 Homie session | 增加 `--data-dir` runtime context |
| P1 | 范围 | Diri MCP 工具很多，单 slice 全做会失控 | 质量和验证不可控 | 本 slice 只做 session runtime tools |
| P2 | 安全 | tool args/result 可能记录敏感内容 | evidence/log 泄漏 | 不记录 raw args，错误只返回 safe message |
| P2 | 兼容 | 无 `--data-dir` 时若默认打开用户 HOME 会产生副作用 | 测试污染真实本机状态 | 无 data-dir 保持 no-runtime fallback |

## 3. 整改后的完善方案

按 PRD/OpenSpec 实现 runtime-backed MCP context、基础 tool dispatch 和 CLI E2E。`list_children/wait_for_children/release_agent/worktree/browser/test_run` 后续实现前保持 unsupported safe error。

## 4. 分层任务拆解

| 层级 | 任务 | 交付物 | 依赖 | 优先级 |
|------|------|--------|------|--------|
| CLI | mcp args/context | `McpStdioArgs` | none | P1 |
| Tool dispatch | runtime-backed session tools | handler functions | HomieClient | P1 |
| Tests | real binary integration | `mcp_stdio_runtime_cli.rs` | runtime CLI | P1 |
| Evidence | reports/parity lock | docs | tests | P1 |

## 5. 测试规划

| 类型 | 覆盖点 | 关键用例 | 执行阶段 |
|------|--------|----------|----------|
| Integration | tools/list | FC-DMTS-001 | RED/GREEN |
| Integration | list/status/read | FC-DMTS-002 | GREEN |
| Integration | send/spawn | FC-DMTS-003 | GREEN |
| Regression | no-runtime mode | existing `mcp_stdio_cli` | GAUNTLET |
| Quality | check/clippy/diff/parity | FC-DMTS-005 | GAUNTLET |

## 6. 开发排期

| 阶段 | 顺序 | 工作项 | 风险与缓冲 | 验收物 |
|------|------|--------|------------|--------|
| Spec | 1 | PRD/OpenSpec/Case | 无 | docs |
| TDD | 2 | RED runtime MCP tests | 现有 CLI 无 args | failing tests |
| Impl | 3 | CLI handler | runtime availability | passing tests |
| Verify | 4 | gates/reports | broader MCP still partial | readiness |

## 7. 待确认问题

- 完整 lineage permission model、children/wait/release 和 worktree/browser/test_run 的优先级在后续 lane 决定。
