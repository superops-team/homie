# 项目研发工作流初始化设计文档

## 1. 概述

### 1.1 背景

Homie 当前处于项目初始化阶段。项目定位是 Rust + GPUI 桌面应用，负责管理多个后台 agent、统一 LLM 配置和虚拟 key、维护全局 context/memory/task，并内置意图识别与编排器。后续变更会同时涉及桌面 UI、agent 进程管理、凭证安全、OpenAI 协议代理、上下文持久化、记忆和任务编排，需要从第一天就建立清晰的需求和规格管理边界。

参考 `mpa-codex-worker` 的成熟做法，本项目需要初始化 PRD spec、组件 spec、OpenSpec、验证报告和 Beads 需求管理链路。

### 1.2 目标

- 建立 `prd-spec/` 作为中文需求设计入口。
- 建立 `specs/` 作为长期组件技术规格入口。
- 建立 `openspec/changes/` 作为单次变更执行计划入口。
- 建立 `docs/verification/` 作为验证和准出证据入口。
- 初始化 Beads，并明确 Beads 与文档体系的关系。
- 更新仓库级 `AGENTS.md`，让后续 agent 默认遵守该工作流。

### 1.3 非目标

- 不初始化 Rust crate、GPUI app 或实际业务代码。
- 不确定最终数据库、加密存储、LLM provider SDK 或 agent adapter 实现。
- 不为尚未存在的模块编写完整组件 spec。

## 2. 用户场景

### 场景 1: 新功能开发

**Given** 用户提出一个新的 Homie 能力，例如 LLM 虚拟 key 代理。
**When** agent 准备实现。
**Then** 先创建 Beads issue 和 `prd-spec/features/<change-id>/...`，评估 `specs/` 影响，再创建 OpenSpec 任务并执行。

### 场景 2: 安全敏感变更

**Given** 需求涉及真实 provider key、virtual key、代理请求或日志脱敏。
**When** agent 进入实现前评估影响。
**Then** 必须更新相关组件 spec，并在验证阶段产生安全 review 报告。

### 场景 3: 未完成项追踪

**Given** 某次变更发现 P1/P2 项暂时无法完成。
**When** 准备交付。
**Then** 不能只写在聊天记录中，必须拆出后续 Beads issue，并在 release readiness 报告里引用。

## 3. 功能需求

### FR-1: PRD spec 目录

初始化 `prd-spec/features`、`prd-spec/refactors`、`prd-spec/bugfixes`，并提供中文目录规范和模板。

### FR-2: 组件 spec 目录

初始化 `specs/README.md`，定义 Homie 的组件规划、文档分层、单篇组件 spec 标准结构和维护规则。

### FR-3: OpenSpec 目录

初始化 `openspec/README.md` 和 `openspec/templates/`，定义 `plan.md`、`tasks.md`、`alignment-report.md` 的结构和准入条件。

### FR-4: 验证报告模板

初始化 `docs/verification/report-templates/`，为 spec review、OpenSpec 对齐、SDD/TDD、测试、E2E、安全 review、code review 和 release readiness 提供统一报告入口。

### FR-5: Beads 需求管理

初始化 `.beads`，使用 `homie` issue prefix，并编写 Beads 与 PRD/spec/OpenSpec 的追踪规则。

### FR-6: Agent 工作规则

更新 `AGENTS.md`，明确实质性变更必须经过 Beads、PRD spec、组件 spec 影响分析、OpenSpec、验证报告和准出流程。

## 4. 实现边界

### 4.1 涉及模块

- `AGENTS.md`
- `README.md`
- `.beads/`
- `prd-spec/`
- `specs/`
- `openspec/`
- `docs/workflows/`
- `docs/verification/`

### 4.2 不涉及模块

- Rust workspace 初始化。
- GPUI UI 实现。
- agent runtime adapter 实现。
- LLM proxy 实现。
- context/memory/task/orchestrator 业务代码。

## 5. 组件 spec 影响

| 组件 | 是否影响 | 原因 | 需要更新 |
|------|----------|------|----------|
| `specs/README.md` | 是 | 初始化组件规格入口和规划表 | 已新增 |
| 具体组件 spec | 否 | 当前只建立规范，不实现组件合同细节 | 后续按需求创建 |

## 6. 测试计划

- 运行 `bd status --json` 验证 Beads 可用。
- 检查新增目录和模板文件存在。
- 检查 `AGENTS.md`、`README.md`、workflow 文档之间的术语一致性。
- 不运行 Rust 测试，因为本次不初始化 Rust 工程。

## 7. 验收标准

- `prd-spec/README.md` 存在，并定义 features/refactors/bugfixes 规则。
- `specs/README.md` 存在，并定义 Homie 组件规划。
- `openspec/README.md` 和模板存在。
- `docs/workflows/requirements-management.md` 存在，并给出 Beads 命令和追踪规则。
- `.beads` 初始化成功，`bd status --json` 可执行。
- `AGENTS.md` 包含本项目研发流程要求。

## 8. Beads 追踪

当前文档对应初始化工作流本身。若创建 Beads issue，建议：

```bash
bd create "Bootstrap Homie PRD/spec/OpenSpec workflow" \
  --type feature \
  --priority P0 \
  --labels workflow,prd-spec,openspec,beads \
  --spec-id prd-spec/features/project-workflow-bootstrap/2026-08-05-project-workflow-bootstrap-design.md \
  --metadata '{"change_id":"project-workflow-bootstrap"}'
```
