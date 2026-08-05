# Beads + PRD Spec + OpenSpec 需求管理工作流

Homie 使用 Beads 管理需求状态，使用 `prd-spec/` 管理中文需求设计，使用 `specs/` 管理长期组件合同，使用 `openspec/changes/` 管理单次变更的执行计划。

## 核心原则

1. Beads 是状态系统，不是设计文档正文的替代品。
2. PRD spec 是需求事实源，说明背景、目标、非目标、范围和验收标准。
3. 组件 spec 是长期工程合同，说明接口、数据模型、状态机、安全、恢复和测试计划。
4. OpenSpec 是执行计划，说明本次变更如何按 SDD/TDD 落地。
5. 验证报告是准出证据，必须写入 `docs/verification/<change-id>/`。

## 标准链路

```text
bd issue
  -> prd-spec/<type>/<change-id>/YYYY-MM-DD-<change-id>-design.md
  -> specs/<component>/README.md
  -> openspec/changes/<change-id>/{plan.md,tasks.md,alignment-report.md}
  -> docs/verification/<change-id>/*.md
  -> bd close/update
```

## Beads 初始化

本仓库已用以下命令初始化 Beads：

```bash
bd init --non-interactive --prefix homie --skip-agents --init-if-missing
```

当前 issue prefix 为 `homie`，issue id 形如 `homie-<hash>`。

初始化时 git hooks 安装可能因为 `.git/config` 权限失败而跳过。Beads 数据库仍可使用；需要 hooks 时，修正权限后再运行：

```bash
bd hooks install --beads
```

## 创建需求

新功能：

```bash
bd create "LLM virtual key proxy" \
  --type feature \
  --priority P0 \
  --labels llm,security,proxy \
  --spec-id prd-spec/features/llm-virtual-key-proxy/2026-08-05-llm-virtual-key-proxy-design.md \
  --metadata '{"change_id":"llm-virtual-key-proxy"}'
```

Bug 修复：

```bash
bd create "Virtual key should never expose provider key" \
  --type bug \
  --priority P0 \
  --labels security,llm \
  --spec-id prd-spec/bugfixes/virtual-key-provider-key-leak/2026-08-05-virtual-key-provider-key-leak-design.md \
  --metadata '{"change_id":"virtual-key-provider-key-leak"}'
```

重构：

```bash
bd create "Split agent adapter contract from runtime supervisor" \
  --type task \
  --priority P1 \
  --labels refactor,agents,runtime \
  --spec-id prd-spec/refactors/agent-adapter-runtime-boundary/2026-08-05-agent-adapter-runtime-boundary-design.md \
  --metadata '{"change_id":"agent-adapter-runtime-boundary"}'
```

## 更新需求状态

领取需求：

```bash
bd update <bead-id> --claim
```

标记阻塞：

```bash
bd update <bead-id> --status blocked --notes "Blocked because spec review has unresolved P0 findings."
```

关闭需求：

```bash
bd close <bead-id> --reason "Implemented and verified. See docs/verification/<change-id>/release-readiness-report.md."
```

查看可做任务：

```bash
bd ready
```

查看全部状态：

```bash
bd status
bd list
bd show <bead-id> --long
```

## 依赖关系

如果需求 B 依赖需求 A，使用 `blocks` 关系：

```bash
bd link <blocked-bead-id> <blocking-bead-id>
```

例：`intent-orchestrator-mvp` 依赖 `agent-runtime-supervisor`：

```bash
bd link <intent-orchestrator-bead> <runtime-supervisor-bead>
```

## Change ID 规则

`change_id` 必须稳定，并在以下位置保持一致：

| 位置 | 规则 |
|------|------|
| Beads metadata | `{\"change_id\":\"<change-id>\"}` |
| PRD 主题目录 | `prd-spec/<type>/<change-id>/` |
| OpenSpec 目录 | `openspec/changes/<change-id>/` |
| 验证目录 | `docs/verification/<change-id>/` |

## PRD Spec 准入

进入实现前必须满足：

- Beads issue 存在，且 `spec-id` 指向 PRD 文档。
- PRD spec 自包含，写清背景、目标、非目标、需求、范围、测试计划和验收标准。
- 涉及组件合同的变更已经更新 `specs/` 或写明不影响原因。
- spec review 报告已写入 `docs/verification/<change-id>/spec-review-report.md`。

## OpenSpec 准入

进入实现前必须满足：

- `openspec/changes/<change-id>/plan.md` 存在。
- `openspec/changes/<change-id>/tasks.md` 存在。
- `openspec/changes/<change-id>/alignment-report.md` 映射所有 PRD FR。
- 每个 task 都有 RED/GREEN 验收路径。
- Beads issue 状态不是 `blocked`。

## 交付准出

关闭 bead 前必须满足：

- `docs/verification/<change-id>/test-report.md` 记录实际测试命令和结果。
- 涉及真实 agent、LLM proxy、credential、storage、context、memory、task、orchestrator 的变更有 E2E 或等价本地验证。
- 安全相关变更有 `security-review-report.md`。
- `release-readiness-report.md` 引用 PRD、spec、OpenSpec、测试、review 和残余风险。
- 未完成项已拆成新的 Beads issue，不能只留在聊天记录里。
