# OpenSpec 工作流

`openspec/` 用于把已评审的 PRD/spec 转成可执行开发计划。它不是需求入口，也不是长期组件合同；它只服务于一次明确的变更。

每个变更使用稳定的 `<change-id>`：

```text
openspec/
├── README.md
├── templates/
│   ├── plan-template.md
│   ├── tasks-template.md
│   └── alignment-report-template.md
└── changes/
    └── <change-id>/
        ├── plan.md
        ├── tasks.md
        └── alignment-report.md
```

## Change ID

`<change-id>` 默认与 PRD 主题目录、Beads issue slug 或主要能力名一致，例如：

- `agent-runtime-supervisor`
- `llm-virtual-key-proxy`
- `context-memory-task-control-plane`
- `intent-orchestrator-mvp`

同一次变更的路径应保持一致：

| 对象 | 路径/字段 |
|------|-----------|
| PRD | `prd-spec/<type>/<change-id>/YYYY-MM-DD-<change-id>-design.md` |
| 组件 spec | `specs/<component>/README.md` |
| OpenSpec | `openspec/changes/<change-id>/` |
| 验证报告 | `docs/verification/<change-id>/` |
| Beads | `bd create ... --spec-id prd-spec/...`，metadata 中记录 `change_id=<change-id>` |

## 阶段顺序

1. 在 Beads 中创建或确认需求 issue。
2. 在 `prd-spec/` 编写中文 PRD/spec。
3. 评估并更新受影响的 `specs/` 组件合同。
4. 完成 spec review，阻塞项清零。
5. 创建 `openspec/changes/<change-id>/plan.md` 和 `tasks.md`。
6. 编写或更新 `alignment-report.md`，证明每个 PRD 需求都有 OpenSpec task 和验证方式。
7. 按 `tasks.md` 执行 SDD/TDD。
8. 将测试、E2E、review、准出证据写入 `docs/verification/<change-id>/`。
9. 更新 Beads issue 状态，关闭已完成任务或拆出后续 issue。

## Plan 要求

`plan.md` 必须说明：

- source PRD；
- 受影响组件 spec；
- 目标和非目标；
- 文件级影响范围；
- 数据、状态、权限和安全影响；
- 测试和验收策略；
- 发布或准出门槛。

## Tasks 要求

`tasks.md` 必须把 PRD/spec 拆成可执行任务。每个任务至少包含：

- task id；
- 来源需求；
- 目标；
- 预计文件；
- RED 测试；
- 实现动作；
- GREEN 验收；
- 需要更新的文档或报告；
- Beads issue 链接。

## Alignment Report 要求

`alignment-report.md` 必须证明：

- 每个 PRD FR 都映射到至少一个 OpenSpec task；
- 每个 task 都有测试或验证方式；
- 每个受影响组件 spec 都已更新或说明不影响原因；
- Beads issue 的状态和文档路径一致；
- 没有未归属需求、未验证需求或未记录风险。
