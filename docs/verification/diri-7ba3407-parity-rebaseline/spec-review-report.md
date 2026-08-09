# Spec Review Report: diri-7ba3407-parity-rebaseline

```yaml
change_id: diri-7ba3407-parity-rebaseline
report_type: spec-review
status: pass
beads: homie-t3u
review_scope:
  - prd-spec/features/diri-7ba3407-parity-rebaseline/2026-08-08-diri-7ba3407-parity-rebaseline-design.md
  - docs/research/diri-7ba3407-capability-matrix.md
  - openspec/changes/diri-7ba3407-parity-rebaseline/
  - affected component specs
```

## 1. 评审概览

| 维度 | 判定 | 关键问题数 | 依据 |
|------|------|-----------|------|
| 1. 上下文逻辑连贯性 | 通过 | 0 | proposal -> design -> 8 capability specs -> 15 task contracts -> FR alignment 连贯 |
| 2. 内容空洞排查 | 通过 | 0 | 34 条 Requirement 均有可验证 Scenario，共 61 条 |
| 3. 歧义点识别 | 通过 | 0 | 基线 commit、状态词、owner、wave、完成条件均穷举 |
| 4. 大模型语义理解偏差 | 通过 | 0 | capability specs 使用 SHALL/MUST 和精确 WHEN/THEN |
| 5. SDD/TDD 开发模式适配 | 通过 | 0 | 每个 child change 强制 RED/GREEN/REFACTOR/EVIDENCE |
| 6. 最小化实现原则 | 通过 | 0 | Wave 0 仅文档；实现按纵向 change，不预建空代码/空表 |
| 7. 向下兼容隐患 | 通过 | 0 | 明确不保留错误内部边界兼容层，并给出迁移/rollback 规则 |
| 8. 存量业务影响 | 通过 | 0 | 列出 15 个组件、共享文件 owner 和 dirty worktree 约束 |
| 9. 功能失效风险预判 | 通过 | 0 | 覆盖 runtime、holder、remote、secret、update、evidence 风险 |
| 10. 落地可行性评估 | 通过 | 0 | 关键路径、并行条件、相对工作量和 child 决策点明确 |
| 11. 任务拆解与排期 | 通过 | 0 | Wave 0 为单次文档任务；后续为 program milestone，编码前必须生成 child 细粒度 tasks |
| 12. 可扩展性影响 | 通过 | 0 | 稳定 client/service 合同，Homie extension 与 Diri parity 分离 |
| 13. 过度设计警惕 | 通过 | 0 | 拒绝继续拆微切片；不在 master spec 预选低层实现 |
| 14. 小而高效改动 | 通过 | 0 | 只新增一个必要 client spec，对其他长期 spec 定向追加 |
| 15. 代码优雅性 | 通过 | 0 | 本 change 不改代码；后续要求删除 shortcut、按 owner 拆分 |
| 16. 架构统一性 | 通过 | 0 | daemon/client/service/storage/UI 分层与项目规范一致 |

总评：16/16 通过，0/16 需改进，0/16 不适用。

该结论只表示 Wave 0 规格可用于后续 child change 规划，不表示 Diri 功能实现完成。

## 2. 评审中已解决的问题

### 2.1 基线不稳定

问题：

- 历史文档使用 “Reference” 语义，没有把内嵌 Diri 精确 commit 作为唯一来源。

修正：

- PRD、能力矩阵、proposal、design 和 component specs 均固定 `diri/7ba3407`。

### 2.2 文档覆盖被误认为实现

问题：

- 旧规划允许 DTO、descriptor、静态 UI、局部 parser/test 形成完成错觉。

修正：

- capability status 明确要求真实代码、production wiring、current verification 和 evidence 四者同时成立。
- capability specs 增加 advertised-only、source-text、fixture-only 和 UI local-state 的负面 Scenario。

### 2.3 OpenSpec 产物不完整

问题：

- 历史 change 通常只有 `plan/tasks/alignment`，OpenSpec CLI 仅显示 1/4。

修正：

- 本 change 同时提供 proposal、design、八个 capability specs、tasks、plan 和 alignment。
- `openspec status` 已达到 4/4，strict validation 通过。

### 2.4 Master roadmap 与细粒度任务边界

问题：

- 完整 parity 不能在一个 change 内写成可直接执行的巨型任务。

修正：

- T-101..T-502 是 program milestones 和 child change ID，不授权直接编码。
- 每个 milestone 在编码前必须创建独立 Bead、中文 PRD、OpenSpec 和单次会话级 RED/GREEN tasks。

### 2.5 当前完成态与测试冲突

问题：

- RT-001、RT-006、RT-007 当前测试失败，API-002 仍是 in-process client，却被标记为 implemented。

修正：

- parity lock 将四项降为 partial，并在能力矩阵记录原因和重新完成门禁。

## 3. 改进后任务拆解

| 层级 | Task | 交付 |
|------|------|------|
| Wave 0 | T-000 | 重基线 PRD、matrix、component specs、OpenSpec、review/evidence |
| Foundation | T-101..T-103 | daemon/client、agent/holder、durable facts |
| Local product | T-201..T-204 | workbench/sidebar、terminal、navigation/native、inspector/artifacts |
| Automation | T-301..T-302 | CLI、MCP/lineage/browser/test |
| Remote/control | T-401..T-403 | node/handoff、usage/proxy、Homie extensions |
| Ship | T-501..T-502 | package/updater/performance、final parity gate |

每个 T-1xx 至 T-5xx milestone 必须在 child `tasks.md` 中继续拆成 2-5 分钟级 RED、执行、GREEN、refactor 和 evidence steps。

## 4. 风险清单

| 风险 | 严重度 | 缓解措施 |
|------|--------|----------|
| Runtime/client 重构影响所有入口 | High | T-101 先建跨进程 fixture，再一次性切换并删除 embedded path |
| Holder 当前已有回归 | High | T-102 从现有 detached failure 建 RED，重复运行恢复测试 |
| Secret envelope 尚未选型 | High | T-402 编码前完成安全/package research 和 threat model |
| Remote handoff 数据损坏 | High | quarantine、content hash、operation id、lease commit、source-preserving failure |
| UI monolith并行冲突 | Medium | child PRD 分配 surface module owner，避免共享区域并发修改 |
| 历史非法 evidence 状态干扰 | Medium | Wave 0 状态审计；T-502 final validator 只接受合法状态 |
| Release host/credential 不可用 | Medium | gate blocked，不允许 ad-hoc/no-run 替代 |
| Master 计划被误当直接实现授权 | Medium | tasks 和 plan 强制 child Bead/PRD/OpenSpec gate |

## 5. 验证结果

| Check | Result |
|-------|--------|
| PRD FR count | pass: 16 |
| Alignment unique FR count | pass: 16 |
| OpenSpec capability spec count | pass: 8 |
| OpenSpec requirements/scenarios | pass: 34 requirements, 61 scenarios |
| Component spec baseline references | pass: 15 |
| `openspec status --change diri-7ba3407-parity-rebaseline` | pass: 4/4 |
| `openspec validate diri-7ba3407-parity-rebaseline --strict` | pass |
| illegal new evidence status scan | pass: no matches |
| secret-shaped literal scan | pass: no matches |

## 6. Gate Decision

Decision: pass

Reason:

- 16 个评审维度无阻断问题。
- 规格已明确当前产品仍为 `not_parity_complete`。
- 后续实现只能通过独立 child change 启动。
