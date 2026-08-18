# Spec Review Report — llm-gateway-virtual-keys

## 1. 范围

本报告记录 `llm-gateway-virtual-keys`（Beads `homie-f91`）在实现前的 spec 与可行性评审结论，
覆盖技术选型、组件合同、依赖评估与安全 Tier 分级。

## 2. 技术选型评审（可行性）

| 候选 | 定位 | 复用形态 | 结论 |
|------|------|----------|------|
| litellm-rust | AI 网关（axum HTTP server） | 纯 binary，vendor 源码 | **采纳**：已含虚拟 key、agent 配置注入、/v1/messages / /v1/responses |
| aimux | 统一 LLM 访问层库 | crates.io 库，可 cargo add | **本切片不采纳**：无网关/虚拟 key，留 provider 广度扩展 child |

依据：需求本质是「coding agent 网关 + 虚拟 key + agent 配置注入」，与 litellm-rust 定位一致；
aimux 是上游访问层，适合 `llm-gateway-provider-expansion` 阶段接入。

## 3. 依赖评估

| 依赖 | 状态 | 理由 |
|------|------|------|
| `axum` | 新增 | HTTP server；owner crate `homie-gateway`；符合依赖添加政策 |
| `tokio` | workspace 已有 | 复用，无需新增 |
| `rusqlite` | workspace 已有（bundled） | 虚拟 key + 用量持久化 |
| `serde`/`serde_json` | workspace 已有 | 协议序列化 |
| `homie-usage` | workspace 已有 | `openai_estimate` 用量估算 |

需在 `docs/research/rust-package-selection.md` 登记 `axum` 选择。

## 4. 组件合同评审

`specs/llm-gateway.md` 定义了虚拟 key 模型（§3）、鉴权（§4）、协议（§5）、上游转发（§6）、
用量（§7）、安全/恢复（§8）。评审结论：合同与 PRD FR 一一对应，无缺口。

## 5. 安全 Tier 分级

credential custody / virtual key issuance / LLM proxying 属 **Tier 3**（security-sensitive），
`docs/development/quality-gates.md` §4.2 强制 coverage + mutation gate。已写入
`openspec/changes/llm-gateway-virtual-keys/plan.md` §5，实现阶段需交付逐行覆盖与手工 mutation
证据。

## 6. 与既有权威的一致性

- 不改变 PTY/holder 生命周期权威（`specs/engine-session-runtime.md`）。
- 不改动 `homie-node` 远程节点与 `homie-mcp` MCP 代理。
- 网关注入是 `homie-engine/src/inject.rs` 的**新增机制**，不破坏现有 `codex_mcp`/`claude_mcp`。

## 7. 结论

方案可行，规格齐备，可进入实现阶段。
