# Proposal: Diri 7ba3407 Parity Rebaseline

## Why

Homie 当前存在大量以 `reference-parity-v1` 和微切片命名的局部交付，但真实产品仍缺少独立 runtime/client transport、agent-aware spawn、完整 UI/CLI/MCP、remote node、LLM proxy、updater 和发布闭环。协议常量、DTO、静态 UI、fixture 或局部测试曾被误用为功能完成证据，当前 workspace 还存在 app、MCP 和 runtime 回归。

需要把内嵌 `diri/` 的 commit `7ba3407` 固定为唯一基线，建立可执行的能力矩阵和纵向交付顺序，并让后续每个 change 同时满足仓库 SDD/TDD 工作流与 OpenSpec artifact completeness。

## What Changes

- 新增 20 模块的冻结能力矩阵和遗漏能力清单。
- 将 `reference-parity-v1`、`diri-parity-child-tasks` 和现有微切片降为历史参考，不再作为新实施入口。
- 修订 runtime、client、agent、desktop、storage、MCP、remote、LLM、credential、context、memory、task、orchestrator、observability 和 packaging 的长期合同。
- 把开发拆成 Wave 1 至 Wave 5 的纵向 change；每个 change 必须独立建立 Bead、中文 PRD、OpenSpec 和 evidence。
- 明确 advertised-but-unsupported、in-process runtime client、UI 直接写 storage、本地假状态和非法 evidence 状态都属于阻断项。
- 本 change 只交付规格和执行计划，不直接实现所有 parity 功能。

## Capabilities

### New Capabilities

- `diri-parity-governance`: 固定 Diri commit、能力状态、证据词汇、完成判定和 change 追踪规则。
- `diri-parity-delivery-waves`: 定义 runtime、产品、自动化、远端、usage、扩展和发布的依赖顺序与纵向准出。
- `runtime-client-boundary`: 从 in-process supervisor wrapper 改为独立 daemon 的 control/data client。
- `runtime-session-lifecycle`: 补齐 agent-aware spawn、holder、resource、migration 和 shutdown。
- `desktop-product-surface`: 要求所有 UI 操作走真实 client/service 并通过交互/截图 E2E。
- `automation-surface`: 补齐 CLI、MCP 精确 schema、lineage、browser/test sidecar。
- `remote-usage-release`: 补齐 node/handoff、usage/LLM proxy、updater/package/performance。
- `homie-control-plane`: 将 context、memory、task、orchestrator 从孤立模型接入真实产品路径。

## Impact

- 新增 Bead `homie-t3u` 和 change `diri-7ba3407-parity-rebaseline`。
- 新增中文主 PRD、能力矩阵、runtime client transport 组件规格和完整 OpenSpec artifacts。
- 更新 14 个长期组件规格和 `specs/README.md`。
- 修正当前 parity lock 中与测试/架构事实冲突的状态。
- 后续实现会删除 production in-process runtime shortcut 和 UI 直接 storage 依赖；不提供兼容 fallback。
- 本 proposal 不新增运行时依赖，不修改产品代码或数据库 schema。
