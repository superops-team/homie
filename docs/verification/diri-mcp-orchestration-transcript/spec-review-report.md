# Spec Review Report: Diri MCP Orchestration Transcript E2E

```yaml
change_id: diri-mcp-orchestration-transcript
beads: homie-3vh
status: pass
reviewed_at: 2026-08-08
source_prd: prd-spec/features/diri-mcp-orchestration-transcript/2026-08-08-diri-mcp-orchestration-transcript-design.md
```

## 1. 总体结论

- 可行性：高。
- 最大风险：把 E2E 证据切片扩成 browser/test_run 或 UI 工作。
- 推荐方向：只新增真实 MCP stdio transcript 测试；若已实现工具能串联通过，则不改生产代码。

## 2. 问题清单

| 优先级 | 维度 | 问题 | 影响 | 整改建议 |
|---|---|---|---|---|
| P1 | 可验证性 | 分片工具各自通过，不等于 Diri orchestration flow 可用。 | parity lock 的 full transcript E2E 缺口无法关闭。 | 新增一个真实 MCP transcript 测试覆盖六个工具。 |
| P1 | 范围控制 | transcript E2E 容易被扩成 browser/test_run。 | 增加环境依赖和不稳定性。 | 明确不进入 browser sidecar。 |
| P2 | 清理 | 测试创建 runtime holder 后必须释放。 | 测试可能挂住或留下进程。 | release child，必要时 kill parent。 |

## 3. 整改后的完善方案

- 目标与范围：真实 MCP stdio 编排流 E2E。
- 非目标：不新增功能、不改 UI、不实现 browser/test_run。
- 设计原则：只走公开 CLI/MCP；每一步验证真实 output/status/artifact。
- 核心方案：新增 `mcp_orchestration_transcript_cli.rs`。
- 风险控制：显式 release child、kill parent，避免 holder 残留。

## 4. 分层任务拆解

| 层级 | 任务 | 交付物 | 依赖 | 优先级 |
|---|---|---|---|---|
| Test | 新增 transcript E2E | `mcp_orchestration_transcript_cli.rs` | existing MCP tools | P1 |
| Evidence | 验证和 parity lock | verification docs, parity lock | GREEN test | P1 |

## 5. 测试规划

| 类型 | 覆盖点 | 关键用例 | 执行阶段 |
|---|---|---|---|
| E2E | Diri MCP flow | spawn/send/wait/read/artifacts/release | 开发中 |
| Quality | lint/format/diff/parity | clippy、fmt、diff、make parity-lock | 准出 |

## 6. 开发排期

| 阶段 | 顺序 | 工作项 | 风险与缓冲 | 验收物 |
|---|---|---|---|---|
| S1 | 1 | PRD/spec/OpenSpec/case | 锁定验证范围 | 文档齐备 |
| S2 | 2 | E2E 测试 | 若失败，定位具体工具缺口 | GREEN test |
| S3 | 3 | 门禁与证据 | 不通过则回到 S2/实现修复 | readiness report |

## 7. 待确认问题

- 无。
