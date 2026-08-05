# Homie 组件规格规划

`specs/` 是 Homie 的长期技术规格入口。它把 PRD 中的一次性需求沉淀为可维护、可实现、可测试的组件合同。

`prd-spec/` 负责说明“为什么做、做到什么程度”；`specs/` 负责说明“组件如何做、接口如何对齐、边界如何守住”；`openspec/changes/` 负责把某次变更拆成可执行任务；Beads 负责需求状态、优先级、依赖和追踪链接。

## 文档分层

| 文档层 | 位置 | 生命周期 | 职责 |
|--------|------|----------|------|
| Product README | `README.md` | 项目级 | 说明 Homie 的产品愿景、核心能力和架构方向 |
| PRD spec | `prd-spec/` | 单次需求或变更 | 说明背景、目标、非目标、用户场景、范围和验收标准 |
| 组件 spec | `specs/<component>/README.md` | 长期维护 | 固化组件边界、接口、数据模型、状态机、安全约束、恢复策略和测试计划 |
| OpenSpec change | `openspec/changes/<change-id>/` | 单次变更 | 将 PRD/spec 拆解成 SDD/TDD 任务、验收映射和执行计划 |
| Beads issue | `.beads` / `bd` | 持续维护 | 记录需求状态、优先级、依赖、负责人、阻塞关系和 spec 链接 |
| 验证证据 | `docs/verification/<change-id>/` | 单次变更 | 记录 spec review、OpenSpec 对齐、测试、E2E、review、准出结论 |

分层规则：

1. 单次需求背景、业务收益、范围裁剪写入 `prd-spec/`。
2. 长期接口、状态机、数据模型、错误码、安全边界、恢复语义写入 `specs/`。
3. 任务拆解、TDD 顺序、影响文件、验收映射写入 `openspec/changes/<change-id>/`。
4. issue 状态、依赖、阻塞、优先级和负责人写入 Beads。
5. 当 PRD、组件 spec、OpenSpec 或代码出现冲突时，先暂停实现，修正文档事实源，再继续开发。

## 架构基线

Homie 是本地优先的 Rust + GPUI 桌面应用。它管理多个后台 agent，统一 LLM 配置和虚拟 key，维护全局 context/memory/task，并通过意图识别和编排器把用户请求路由到合适的 agent 或内部工作流。

核心基线：

| 主题 | 约束 |
|------|------|
| 桌面边界 | GPUI 负责交互和呈现，业务状态通过核心域模型驱动，不把 agent 运行逻辑塞进 UI 组件 |
| Agent 管理 | Codex、Claude Code、OpenCode 等通过独立 adapter 接入，adapter 只处理各自 runtime 的启动、配置和 I/O |
| LLM 凭证 | 真实 provider key 只存在 Homie 配置和凭证系统；managed agent 只能拿到 Homie 签发的虚拟 key |
| OpenAI 代理 | agent 通过 Homie 暴露的 OpenAI-compatible proxy 发起模型请求，由 Homie 处理鉴权、路由、记录和策略 |
| Context | session、workspace、文件、prompt、tool result 和 task 事实由 Homie 统一维护 |
| Memory | 区分短期 session context 和长期 durable memory，写入必须可追踪来源和权限 |
| Task | 任务是共享状态，不归属于某一个 agent；agent 可以领取、更新、阻塞或交还任务 |
| Orchestration | 编排器先实现最小可用的意图识别和路由，再逐步扩展多 agent 协作 |
| Security | credential、virtual key、prompt、tool args、session log 和 memory 都默认按敏感数据处理 |

## 组件规划

每个组件使用一个子目录，默认文件为 `specs/<component-name>/README.md`。组件名使用 kebab-case。

| 优先级 | 组件 | 建议路径 | 主要职责 |
|--------|------|----------|----------|
| P0 | Desktop Shell | `specs/desktop-shell/README.md` | GPUI 应用壳、窗口、pane、导航、状态订阅和用户交互 |
| P0 | Runtime Supervisor | `specs/runtime-supervisor/README.md` | 后台进程启动、停止、重启、健康检查、资源限制和生命周期事件 |
| P0 | Agent Adapter Contract | `specs/agent-adapter-contract/README.md` | Codex/Claude Code/OpenCode adapter 的统一边界、能力声明、I/O 和错误模型 |
| P0 | LLM Proxy | `specs/llm-proxy/README.md` | OpenAI-compatible endpoint、provider routing、model aliases、token/cost accounting |
| P0 | Virtual Key & Credentials | `specs/virtual-key-credentials/README.md` | 真实 key 存储、虚拟 key 签发、作用域、撤销、审计和泄漏防护 |
| P0 | Session Context Store | `specs/session-context-store/README.md` | agent session、conversation、workspace context、prompt history 和 tool result 存储 |
| P1 | Memory Controller | `specs/memory-controller/README.md` | 长期记忆、检索、写入候选、来源引用、权限和回收策略 |
| P1 | Task Controller | `specs/task-controller/README.md` | 用户任务、agent 任务领取、进度、阻塞、交接和与 Beads 的边界 |
| P1 | Intent Orchestrator | `specs/intent-orchestrator/README.md` | 意图识别、agent/workflow 路由、上下文裁剪、执行监督和结果归档 |
| P1 | Storage & Indexing | `specs/storage-indexing/README.md` | 本地持久化、schema、索引、迁移策略、备份和一致性 |
| P1 | Observability | `specs/observability/README.md` | 日志、metrics、trace、usage、agent run 事件和排障入口 |
| P2 | Cross-Agent Collaboration | `specs/cross-agent-collaboration/README.md` | 多 agent 分工、依赖、冲突处理、handoff 和结果合并 |

优先级含义：

| 优先级 | 定义 |
|--------|------|
| P0 | 没有该组件 spec 就无法稳定实现最小端到端链路 |
| P1 | 支撑生产可用性、安全性、可观测性和主要用户工作流 |
| P2 | 支撑规模化、多 agent 协作或后续增强 |

## 单篇组件 Spec 标准结构

每个组件 spec 默认包含以下章节。章节可以精简，但不能省略核心边界；暂无结论时写明风险和后续验证方式。

1. `组件定位`：一句话说明组件在 Homie 中的位置。
2. `来源需求映射`：列出相关 PRD、OpenSpec change 和必须满足的验收点。
3. `上游与下游关系`：说明调用方、被调用方、同步/异步关系和所有权边界。
4. `职责边界`：明确负责事项、不负责事项、输入、输出和稳定性目标。
5. `核心接口`：给出 Rust trait、struct、event、HTTP endpoint 或内部方法签名。
6. `数据模型`：给出请求、响应、事件、存储记录、错误结构和字段语义。
7. `运行模型与状态机`：描述 process、session、turn、task、memory 或 proxy 生命周期。
8. `安全与权限`：说明 credential、virtual key、redaction、policy、越界访问和审计约束。
9. `可观测性`：定义日志、指标、trace、event、healthcheck 和排障入口。
10. `失败与恢复`：列出 retry、resume、cancel、drain、崩溃恢复、provider 不可用等处理策略。
11. `测试计划与验收引用`：列出单测、集成测试、端到端验证、故障注入和人工验证命令。

## 编写要求

| 检查项 | 要求 |
|--------|------|
| 可实现 | 不能只写抽象说明，必须包含具体接口、数据结构、状态迁移或时序 |
| 可验证 | 每个关键不变量都要对应测试、运行证据或待补验证项 |
| 边界清晰 | UI、core、agent adapter、LLM proxy、context、memory、task、storage 的职责不能混写 |
| 协议稳定 | API path、event 名称、错误码、key scope、session id 和 task id 语义必须一致 |
| 安全默认 | 未授权默认拒绝，真实 key 不进入 agent 配置、日志、事件、artifact 或报告 |
| 恢复优先 | agent 进程崩溃、proxy 超时、session 中断、context 写入失败必须有明确行为 |
| 术语统一 | 使用统一术语：agent runtime、adapter、virtual key、provider key、session context、durable memory、task、orchestration |
| 来源可追踪 | 每个组件 spec 都要指向相关 PRD 和 OpenSpec change |
| 长期纯度 | 不写临时任务清单、聊天上下文、未脱敏日志或一次性调试路径 |

## 维护流程

1. 新增组件时，先更新本 README 的组件规划表，再创建 `specs/<component-name>/README.md`。
2. 修改公共接口、event、状态机、权限策略、恢复语义、credential 处理或数据模型时，必须同步更新对应组件 spec。
3. 实现完成后，在对应组件 spec 的测试计划中回填已通过的验证命令、测试文件和残余风险。
4. 删除或合并组件时，旧组件 spec 保留并标记为 `Superseded`，指向新的 canonical 组件。

## 不纳入本目录

| 内容 | 放置位置 |
|------|----------|
| 单次需求背景、业务收益、范围裁剪 | `prd-spec/` |
| 临时开发 checklist、任务分工、TDD 顺序 | `openspec/changes/<change-id>/` 或 Beads |
| 验证报告、测试输出摘要、review finding | `docs/verification/<change-id>/` |
| 真实 API key、Authorization、cookie、raw prompt、完整工具参数 | 禁止写入仓库 |
| 大段外部产品文档复制 | 保留来源，只在组件 spec 中总结已验证模式 |
