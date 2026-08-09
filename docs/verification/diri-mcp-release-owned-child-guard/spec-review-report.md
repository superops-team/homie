# Spec Review Report: Diri MCP release_agent Owned-child Guard

```yaml
change_id: diri-mcp-release-owned-child-guard
beads: homie-al5
status: pass
reviewed_at: 2026-08-08
source_prd: prd-spec/features/diri-mcp-release-owned-child-guard/2026-08-08-diri-mcp-release-owned-child-guard-design.md
```

## 1. 总体结论

- 可行性：高。
- 最大风险：把 direct child release 错误地拒绝，或拒绝 sibling/unrelated 时仍误触发 terminate 副作用。
- 推荐方向：复用现有 `lineage_relation`，将 `release_agent` 收敛为 allow-list：只有 `child` 允许执行终止，其它关系全部拒绝或走既有专用 guard。

## 2. 问题清单

| 优先级 | 维度 | 问题 | 影响 | 整改建议 |
|---|---|---|---|---|
| P1 | 权限边界 | PRD 若只写拒绝 sibling，会遗漏 unrelated 和无 identity caller。 | 仍可终止无关 session。 | 验收必须覆盖 sibling、unrelated、无 session id 默认拒绝。 |
| P1 | 副作用验证 | 只断言错误码不足以证明 target 未被终止。 | 测试可能漏掉先 terminate 后返回错误的实现。 | 功能 case 必须在拒绝后读取 target snapshot，确认仍存在。 |
| P2 | 范围控制 | 容易把 recursive release 或完整 permission profile 混入本 slice。 | 扩大改动面，增加不可控风险。 | 明确非目标：descendant release/full permission matrix 后续处理。 |

## 3. 整改后的完善方案

- 目标与范围：只修改 `release_agent` 的关系 allow-list，确保只有 direct child 可释放。
- 非目标：不新增 permission profile 数据模型，不实现 recursive release，不改 UI。
- 设计原则：最小权限、无兼容层、真实 MCP stdio + runtime path 测试。
- 核心方案：`release_agent` 先判定 relation；`child` 之外拒绝，且拒绝路径在 `terminate_session` 前返回。
- 风险控制：新增拒绝后 snapshot 可读断言；保留 existing release/self/ancestor regression。

## 4. 分层任务拆解

| 层级 | 任务 | 交付物 | 依赖 | 优先级 |
|---|---|---|---|---|
| Spec | 更新 MCP automation 长期规格 | `specs/mcp-automation/README.md` | PRD | P1 |
| Test | 新增 owned-child guard CLI 测试 | `mcp_release_owned_child_guard_cli.rs` | existing MCP stdio | P1 |
| Logic | 收紧 release_agent allow-list | `crates/homie-cli/src/main.rs` | Test RED | P1 |
| Evidence | 回写功能验证、code review、release readiness | `docs/verification/...` | GREEN gates | P1 |

## 5. 测试规划

| 类型 | 覆盖点 | 关键用例 | 执行阶段 |
|---|---|---|---|
| TDD integration | sibling/unrelated deny | 新测试 RED/GREEN | 开发中 |
| Regression | direct child/self/ancestor | 既有 `mcp_release_agent_cli`、`mcp_release_ancestor_guard_cli` | 实现后 |
| Quality | lint/format/diff/parity | clippy、fmt、diff、make parity-lock | 准出 |

## 6. 开发排期

| 阶段 | 顺序 | 工作项 | 风险与缓冲 | 验收物 |
|---|---|---|---|---|
| S1 | 1 | PRD/spec/OpenSpec/case | 避免扩 scope | 文档齐备 |
| S2 | 2 | RED test | 确认现有实现可复现缺口 | failing test |
| S3 | 3 | 最小实现 | 只改 release branch | GREEN test |
| S4 | 4 | 门禁和证据 | 如有回归，回到 S3 | readiness report |

## 7. 待确认问题

- 无。本 slice 的范围和验收足以独立落地。
