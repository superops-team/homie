# Spec Review Report

```yaml
change_id: homie-v1-architecture
report_type: spec-review
status: pass
beads: homie-9c9
source_prd: prd-spec/features/homie-v1-architecture/2026-08-05-homie-v1-architecture-design.md
openspec_plan: openspec/changes/homie-v1-architecture/plan.md
openspec_tasks: openspec/changes/homie-v1-architecture/tasks.md
reviewed_at: 2026-08-05
reviewer: TRAE CLI
```

## 1. 总体结论

- 可行性：高。
- 最大风险：V1 范围覆盖 UI、runtime、SQLite、LLM proxy、agent profile、metrics、context 等多个核心面，若一次性实现会失控。
- 推荐方向：保持当前 PRD 作为目标态，实施时拆成 P0 组件 spec 和多个 OpenSpec change，先完成 SQLite schema + runtime/session + LLM proxy 的纵向闭环，再扩展 UI 和高级编排。

## 2. 问题清单

| 优先级 | 维度 | 问题 | 影响 | 整改建议 |
|---|---|---|---|---|
| P0 | 配置一致性 | 原 spec 只有 `agent.profile.activate`，未说明是默认 profile、启用状态还是影响已运行 session | profile 变更可能误改运行中 session 的 runtime/权限/LLM 配置 | 已改为 `agent.profile.set_default`，并要求 session 启动时冻结 `EffectiveAgentConfig` |
| P0 | 存储关系 | SQLite schema 初版缺少 effective config、唯一约束、default profile 约束 | 多 agent/profile/skill/MCP 关系容易重复或漂移，运行态难复现 | 已增加 `effective_agent_configs`、profile 约束和唯一性要求 |
| P0 | 成本统计 | usage 只存当前价格字段，缺少 pricing snapshot/currency | 模型价格更新后历史 cost 会被错误解释 | 已增加 `pricing_snapshots`、`pricing_snapshot_id` 和 `currency` |
| P1 | 运行边界 | 概述暗示 runtime 可独立存活，但非目标未声明 V1 是否支持 app 退出后保活 | 实现阶段可能把 holder/daemon 化提前做大 | 已明确 V1 不承诺 app 退出后 runtime 保活，V1.1 再做 |
| P1 | 可观测性 | LLM 指标缺少 cache hit rate 的聚合口径 | UI/报表可能平均百分比，导致跨请求统计失真 | 已补 `cache_read_tokens/cache_write_tokens/cache_hit_rate` 和聚合口径 |

## 3. 整改后的完善方案

目标与范围：

- V1 以 SQLite 为本地事实源，管理 provider、runtime descriptor、agent profile、skills、MCP 配置、权限、session、context、usage、tool metrics 和 task 关系。
- agent profile 是用户配置入口，runtime descriptor 是执行引擎定义；session 启动时冻结 `EffectiveAgentConfig`，运行中不受后续 profile 编辑影响。
- V1 默认真实 runtime 选择 Codex；OpenCode 和 Claude Code 后续按同一 adapter contract 接入。
- Homie LLM proxy 是统一流量入口，记录 token、cache hit rate、estimated cost、request latency、tool-call latency 和 safe error code。
- 开发前必须参考 `docs/research/rust-package-selection.md`，优先复用成熟 crate；PTy 和 secret envelope 先做 spike 决策。
- 开发前必须参考 `docs/architecture/project-layout.md`、`docs/development/standards.md`、`docs/development/quality-gates.md`，确保 Swift + Rust 混合项目结构和准出门禁先于代码落地。

非目标：

- V1 不做完整多 agent 自动协作。
- V1 不承诺 app 退出后 runtime 继续存活。
- V1 不做云同步、远端 fleet、复杂长期记忆检索。

设计原则：

- UI、runtime、protocol、LLM proxy、storage 分层清晰。
- V1 secret 存储使用 encrypted local secret envelope；所有真实 secret 只通过 secret ref 引用，禁止落入 agent env/log/context/metrics。
- SQLite 负责关系和事实，output bytes 等大流式内容走文件并由 SQLite 索引。
- 价格和成本必须历史可解释，usage 记录保存 pricing snapshot 和 currency。

验收标准：

- P0 组件 spec 完成后才能进入实现。
- Rust workspace 能构建最小 GPUI app、runtime、SQLite migration 和 fake-provider LLM proxy。
- LLM 请求必须经过 virtual key，usage/tool metrics 写入 SQLite。
- `git diff --check` 和 `.githooks/pre-commit` 通过。

## 4. 分层任务拆解

| 层级 | 任务 | 交付物 | 依赖 | 优先级 |
|---|---|---|---|---|
| 组件 spec | 编写 storage-indexing spec | SQLite schema、migration、约束、备份/损坏处理 | 当前 PRD | P0 |
| 组件 spec | 编写 virtual-key-credentials spec | provider secret ref、virtual key lifecycle、revocation | storage spec | P0 |
| 组件 spec | 编写 llm-proxy spec | OpenAI-compatible proxy、usage/cost/cache/latency/tool metrics | storage + credentials | P0 |
| 组件 spec | 编写 agent-adapter-contract spec | runtime descriptor、agent profile、EffectiveAgentConfig、env scrub | storage + llm | P0 |
| 组件 spec | 编写 runtime-supervisor spec | socket protocol、session、PTY、events、output log | proto + agents | P0 |
| 组件 spec | 编写 desktop-shell spec | GPUI shell、session sidebar、profile settings、usage summary | runtime + llm | P1 |
| 调研 | 完成依赖选型确认 | `docs/research/rust-package-selection.md` 和 spike 决策 | 当前 PRD | P0 |
| 规范 | 完成工程规范确认 | layout、development standards、quality gates | 当前 PRD | P0 |
| 实现 | Bootstrap Rust workspace | Cargo workspace、toolchain、empty crates、doctor skeleton | P0 specs | P0 |
| 实现 | SQLite storage MVP | schema migration、foreign keys、repository API | workspace | P0 |
| 实现 | Runtime/session MVP | spawn fake/shell agent、session list/input/output | storage | P0 |
| 实现 | LLM proxy MVP | virtual key、fake provider、usage records | storage + credentials | P0 |
| 实现 | GPUI V1 shell | session list、active output、profile/provider settings | runtime + proxy | P1 |

## 5. 测试规划

| 类型 | 覆盖点 | 关键用例 | 执行阶段 |
|---|---|---|---|
| 单元测试 | SQLite schema constraints | foreign key、default profile、unique bindings、migration forward-only | storage MVP |
| 单元测试 | Effective config freezing | profile 修改后 running session config 不变 | agent/runtime MVP |
| 单元测试 | pricing/cost | pricing snapshot、currency、estimated_cost 不随价格更新漂移 | llm MVP |
| 单元测试 | cache hit rate | 单请求和聚合口径正确，不平均百分比 | llm MVP |
| 集成测试 | LLM proxy | fake provider 返回 usage，写入 usage_records | llm MVP |
| 集成测试 | tool metrics | Homie 直接代理的 tool 成功、超时、permission denied 耗时记录；MCP proxy 用例后续补充 | llm/tool MVP |
| 集成测试 | runtime protocol | hello/session.spawn/list/input/read_output/events.subscribe | runtime MVP |
| E2E | 端到端 session | UI 创建 profile 和 session，agent 经 proxy 请求 fake provider，UI 展示 usage | V1 shell |
| 安全测试 | secret 边界 | agent env/log/context/metrics 不含 raw key 或 Authorization | 每阶段 |

## 6. 开发排期

| 阶段 | 时间/顺序 | 工作项 | 风险与缓冲 | 验收物 |
|---|---|---|---|---|
| Phase 1 | 先行 | P0 组件 spec + 工程规范确认：storage、credentials、llm-proxy、agent adapter、runtime、layout、quality gates | 控制范围，避免直接写实现 | spec review pass |
| Phase 2 | 第二步 | Rust workspace + SQLite migration | GPUI 依赖可后置，先保证数据层 | cargo check/test |
| Phase 3 | 第三步 | runtime/session + fake/shell agent | PTY 平台差异，先 macOS/Unix | runtime integration tests |
| Phase 4 | 第四步 | LLM proxy + virtual key + metrics | provider 差异，先 fake provider | proxy integration tests |
| Phase 5 | 第五步 | GPUI V1 shell | UI 不抢业务复杂度 | E2E smoke |

## 7. 已确认决策

- V1 首个真实 runtime 默认使用 Codex。
- V1 secret 存储使用 encrypted local secret envelope，不直接依赖 macOS Keychain。
- V1 暂不支持 MCP server proxy；当前只保留 MCP server 配置和 agent profile 绑定，实际代理执行、调用转发和 MCP tool 耗时统计后续补充开发。
