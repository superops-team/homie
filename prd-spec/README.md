# PRD Spec 目录规范

本目录是 Homie 的需求、方案设计和验收标准入口。所有实质性功能、重构和 Bug 修复，都应先在这里形成中文设计文档，再进入组件 spec、OpenSpec 和实现阶段。

Homie 是 Rust + GPUI 桌面应用，目标是统一管理多个后台 agent、统一 LLM 配置和虚拟 key、维护全局 context/memory/task，并内置意图识别与编排器。PRD spec 负责说明每次变更为什么做、做到什么程度、如何验收；长期接口和组件合同放入 `specs/`；执行拆解放入 `openspec/changes/`；需求状态和依赖由 Beads 管理。

## 目录结构

```text
prd-spec/
├── features/       # 新功能
├── refactors/      # 重构
└── bugfixes/       # Bug 修复
```

每个子目录下按主题创建目录，目录名使用 kebab-case：

```text
prd-spec/features/
├── agent-runtime-supervisor/
│   └── 2026-08-05-agent-runtime-supervisor-design.md
├── llm-virtual-key-proxy/
│   └── 2026-08-05-llm-virtual-key-proxy-design.md
└── context-memory-task-control-plane/
    └── 2026-08-05-context-memory-task-control-plane-design.md
```

## 文件命名

```text
YYYY-MM-DD-<主题描述>.md
```

- 日期为文档创建日期。
- 主题描述使用 kebab-case，简明且可搜索。
- 同一主题的多次迭代，新建带新日期的文件，不覆盖历史设计。

## 分类规则

| 类别 | 目录 | 适用场景 |
|------|------|---------|
| 功能 | `features/` | 新增用户可感知能力、后台 agent 能力、LLM 代理能力、context/memory/task/orchestration 能力 |
| 重构 | `refactors/` | 不改变外部行为的内部结构调整、模块边界调整、性能或可维护性改造 |
| Bug 修复 | `bugfixes/` | 修复已有功能异常、回归、崩溃、数据错误、安全漏洞或体验缺陷 |

## Homie 归属规则

| 内容 | 放置位置 | 说明 |
|------|----------|------|
| 用户可见桌面能力、agent 管理能力、LLM proxy、context/memory/task/orchestrator 能力 | `prd-spec/features/` | 作为新增能力或能力演进描述 |
| Rust crate/module 边界、GPUI 状态拆分、storage 抽象调整、agent adapter 内部整理 | `prd-spec/refactors/` | 不改变外部行为时归入重构 |
| agent 启动失败、虚拟 key 泄漏风险、session 记录错误、context 错配、编排错误 | `prd-spec/bugfixes/` | 以问题和根因为核心 |
| 长期组件接口、状态机、数据模型、安全边界、恢复语义 | `specs/<component>/README.md` | 不在 PRD 中承载全部工程合同 |
| 一次变更的任务拆解、TDD 顺序、验收映射、文件级执行计划 | `openspec/changes/<change-id>/` | 必须能追溯到 PRD 和组件 spec |
| issue 状态、优先级、依赖、负责人、阻塞关系 | Beads (`bd`) | 使用 `spec-id` 或 metadata 指向 PRD/OpenSpec |

## 文档语言

所有 `prd-spec/` 正文使用中文。

## 功能文档模板

用于 `prd-spec/features/<topic>/YYYY-MM-DD-<description>.md`。

```markdown
# <功能名称>设计文档

## 1. 概述

### 1.1 背景

### 1.2 目标

### 1.3 非目标

## 2. 用户场景

### 场景 N: <场景标题>

**Given** ...
**When** ...
**Then** ...

## 3. 功能需求

### FR-N: <需求标题>

## 4. 实现边界

### 4.1 涉及模块

### 4.2 不涉及模块

## 5. 组件 spec 影响

| 组件 | 是否影响 | 原因 | 需要更新 |
|------|----------|------|----------|

## 6. 测试计划

## 7. 验收标准

## 8. Beads 追踪
```

## Bug 修复文档模板

用于 `prd-spec/bugfixes/<topic>/YYYY-MM-DD-<description>.md`。

```markdown
# <问题描述>设计文档

## 1. 概述

### 1.1 问题

### 1.2 根因

### 1.3 影响范围

## 2. 用户场景

## 3. 修复需求

## 4. 实现方案

## 5. 边界情况

## 6. 回归测试

## 7. 验收标准

## 8. Beads 追踪
```

## 重构文档模板

用于 `prd-spec/refactors/<topic>/YYYY-MM-DD-<description>.md`。

```markdown
# <重构主题>设计文档

## 1. 概述

### 1.1 问题/动机

### 1.2 目标

### 1.3 非目标

## 2. 现状分析

## 3. 方案设计

## 4. 实施步骤

## 5. 涉及文件

## 6. 验证计划

## 7. Beads 追踪
```

## 原则

1. 一个文件一个完整文档，不拆成 `spec.md` 和 `plan.md`。
2. 同一主题的新版本设计用新文件，不覆盖旧版。
3. 文档必须自包含，读者不依赖聊天记录也能理解背景、目标、方案和验收方式。
4. P0/P1/P2 表示落地顺序，不表示可以默认裁剪。被写入 PRD 的需求必须被实现、验证或显式拆出后续 bead。
5. PRD 不直接替代组件 spec。涉及接口、状态机、数据模型、安全、恢复、持久化和跨模块合同的变更，必须同步评估 `specs/`。
