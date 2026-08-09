# Spec Review Report: Diri MCP Worktree Tools

```yaml
change_id: diri-mcp-worktree-tools
beads: homie-4wg
status: pass
reviewed_at: 2026-08-08
source_prd: prd-spec/features/diri-mcp-worktree-tools/2026-08-08-diri-mcp-worktree-tools-design.md
```

## 1. 总体结论

- 可行性：高。
- 最大风险：在 MCP 层重复实现 git 命令，造成 CLI/worktree runtime 行为不一致。
- 推荐方向：MCP 只做参数解析和 client dispatch，复用既有 `HomieClient::worktree_*`。

## 2. 问题清单

| 优先级 | 维度 | 问题 | 影响 | 整改建议 |
|---|---|---|---|---|
| P1 | 架构一致性 | MCP 若直接 shell 到 git，会绕过 runtime/client 已验证逻辑。 | CLI 和 MCP 行为漂移。 | 只调用 `HomieClient::worktree_create/list/remove`。 |
| P1 | 参数合同 | Diri 使用 `repo` 和 `path`，不能只支持 Homie 内部 DTO 字段名。 | Managed agent 调用失败。 | MCP 分支解析 Diri 参数名，再构造 DTO。 |
| P1 | 真实验证 | 只测 unsupported 消失不足以证明功能可用。 | 可能返回静态 JSON。 | 用临时 git repo 做 create/list/remove E2E。 |
| P2 | 范围控制 | Worktree UI sheet、cleanup suggestion、remote repo locate 不属于本 slice。 | 扩大改动面。 | 仅实现 MCP 三工具。 |

## 3. 整改后的完善方案

- 目标与范围：接通 MCP worktree 三工具到现有 runtime/client。
- 非目标：不改 UI，不改 worktree path 算法，不新增 storage schema，不实现 cleanup suggestion。
- 设计原则：参数层轻、业务层复用、真实 git E2E。
- 核心方案：新增三个 `mcp_tool_payload` 分支，返回与 CLI JSON 一致的结构。
- 风险控制：MCP E2E 加 CLI worktree regression；缺参返回 invalid params。

## 4. 分层任务拆解

| 层级 | 任务 | 交付物 | 依赖 | 优先级 |
|---|---|---|---|---|
| Spec | 更新 MCP automation spec | `specs/mcp-automation/README.md` | PRD | P1 |
| Test | 新增 MCP worktree E2E | `mcp_worktree_tools_cli.rs` | existing CLI/runtime worktree | P1 |
| Logic | MCP payload dispatch | `crates/homie-cli/src/main.rs` | RED test | P1 |
| Evidence | 验证、review、readiness、parity lock | `docs/verification/...` | GREEN gates | P1 |

## 5. 测试规划

| 类型 | 覆盖点 | 关键用例 | 执行阶段 |
|---|---|---|---|
| TDD integration | create/list/remove | 临时 git repo MCP E2E | 开发中 |
| TDD negative | missing params | 缺 repo/path invalid params | 开发中 |
| Regression | CLI worktree | `worktree_cli` | 实现后 |
| Quality | lint/format/diff/parity | clippy、fmt、diff、make parity-lock | 准出 |

## 6. 开发排期

| 阶段 | 顺序 | 工作项 | 风险与缓冲 | 验收物 |
|---|---|---|---|---|
| S1 | 1 | PRD/spec/OpenSpec/case | 锁定 MCP 范围 | 文档齐备 |
| S2 | 2 | RED test | 确认 unsupported 缺口 | failing test |
| S3 | 3 | dispatch 实现 | 避免重写 git | GREEN test |
| S4 | 4 | 门禁与证据 | 如有回归，回到 S3 | readiness report |

## 7. 待确认问题

- 无。本 slice 可独立开发。
