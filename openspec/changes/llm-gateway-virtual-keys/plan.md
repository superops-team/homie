# OpenSpec Plan — llm-gateway-virtual-keys

## 1. 变更概述

本变更是 Homie 统一 LLM 配置入口的**首个真实纵向切片**：新增 `homie-gateway` crate，落地一个
本地 HTTP 网关（对 Codex 暴露 OpenAI Responses、对 Claude Code 暴露 Anthropic Messages），签发
虚拟 key，并把 Codex/Claude 的启动配置自动注入为指向该网关。打通「虚拟 key → agent 自动指向
本地网关 → 网关转发到 OpenAI 兼容上游 → 记录用量」一条完整链路。

复用方式：vendor [litellm-rust](https://github.com/LiteLLM-Labs/litellm-rust)（MIT）的
`src/proxy/auth`（虚拟 key）与上游转发逻辑进新 crate；因 litellm-rust 无 lib.rs，不 `cargo add`，
不做子进程。[aimux](https://github.com/arcships/aimux) 不在本切片引入（属 provider 广度扩展 child）。

## 2. 模块划分与依赖

```text
homie/crates/homie-gateway/
├── Cargo.toml
├── src/
│   ├── main.rs        # 二进制入口：加载配置、启动 axum、绑 127.0.0.1
│   ├── config.rs      # 端口 / 上游 base_url / api_key / master_key（本地 ignored 文件）
│   ├── state.rs       # AppState：虚拟 key store、上游 client、用量 store
│   ├── http/          # 路由注册 + /v1/responses + /v1/messages
│   ├── auth/          # master key + 虚拟 key（vendor 自 litellm-rust）
│   ├── providers/     # 上游 OpenAI-compatible 转发 + SSE
│   └── usage.rs       # 按虚拟 key 用量（SQLite）
```

新增依赖（遵循 `docs/research/rust-package-selection.md` 依赖添加政策）：

- `axum`（HTTP server，owner crate = `homie-gateway`）
- `tokio`（workspace 已有，需补 `rt` / `rt-multi-thread` 特性，已是）
- `rusqlite`（workspace 已有，bundled）
- `serde` / `serde_json`（workspace 已有）
- `homie-usage`（workspace 已有，用量估算）

## 3. 层级关系

| 层 | 产物 |
|----|------|
| 需求 | `prd-spec/features/llm-gateway-virtual-keys/2026-08-18-llm-gateway-virtual-keys-design.md` |
| 规范 | `specs/llm-gateway.md`（虚拟 key、协议、用量、安全/恢复合同） |
| 执行 | 本 OpenSpec plan/tasks/alignment + `homie-gateway/*` + `homie-engine/src/inject.rs` |
| 证据 | `docs/verification/llm-gateway-virtual-keys/` |

## 4. 与既有权威的关系

- `homie-node` 的 `accounts.json` / `usage.sqlite3` 继续负责 provider 账号与远程节点用量；
  本网关的虚拟 key 与用量是**本地网关层**的独立 SQLite，不与远程节点耦合（合并属后续 child）。
- `homie-engine/src/inject.rs` 是 agent 配置注入的权威挂载点；本变更在现有
  `codex_mcp`/`claude_mcp`/`claude_hooks` 机制旁新增 gateway 注入，不改动 PTY/holder 生命周期
  权威（见 `specs/engine-session-runtime.md`）。
- `homie-mcp` MCP 代理与本网关无关，不改动。

## 5. 安全边界（Tier 3 域）

credential custody / virtual key issuance / LLM proxying 属 security-sensitive Tier 3 域，
`docs/development/quality-gates.md` §4.2 要求 coverage + mutation gate 强制：

- 虚拟 key、鉴权、上游转发、用量四块新增/改动行需逐行覆盖（无 llvm-cov 则手工逐行核对）。
- 手工 mutation：对鉴权、key store、转发注入 3–5 个真实 bug，套件必须全部击杀并恢复。

## 6. 后续 child Bead（本变更只声明，不实现）

- `llm-gateway-provider-expansion`：接 aimux，扩展 Anthropic 原生 / 多模态 / 329 provider。
- `llm-gateway-model-routing`：per-agent 默认模型映射 + 网关 model router。
- `llm-gateway-policy-quota`：虚拟 key 配额 / 限流 / 策略 / 审计。
- `llm-gateway-credential-login`：把 Claude/Codex 登录凭证接入网关上游。
