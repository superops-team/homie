# Spec Review Report — llm-gateway-credential-login

Beads `homie-gmq` · change_id `llm-gateway-credential-login` · feature · P1

## 1. 范围

本报告记录把 Claude Code / Codex 登录凭证接入网关上游（Phase 1：Codex API-key 模式）的
spec 与可行性评审结论，覆盖技术选型、组件合同、依赖评估与安全 Tier 分级。

## 2. 技术选型评审（可行性）

用户要求「先做库内嵌」后，经对比选定**方案 A 第一阶段：库内嵌**：

| 方案 | 做法 | 结论 |
|------|------|------|
| A1 库内嵌（采纳） | `homie-gateway` 直接以 crate 依赖调用 `homie-node::credentials` 纯函数 | **采纳**：单一凭证托管、无跨进程 RPC 复杂度、最小端到端 |
| A2 跨进程 RPC | `homie-proto` 增 `credential.resolve` + `NodeClient` 远程调用 | 本阶段不采纳：为单机本地 node 引入 RPC/通道复杂度，属过度设计 |
| B 网关直读 auth 文件 | 网关复用 `config_home` 定位直接读 provider 私有文件 | 不采纳：凭证托管职责分裂，敏感文件读取面扩大 |

依据：AGENTS.md「最小端到端优先」与「长期架构决策」——库内嵌满足当前单机本地 node 需求，
后续若需多节点凭证共享再演进为 RPC，且 `credentials` 模块接口已为此预留（纯函数、无 IO 副作用）。

## 3. 依赖评估

| 依赖 | 状态 | 理由 |
|------|------|------|
| `homie-node`（gateway 侧） | 新增 workspace 依赖 | 库内嵌调用凭证解析，符合「复用既有能力」规则 |
| `serde`/`serde_json` | workspace 已有 | `CodexAuth` 反序列化 |
| `tempfile` | workspace 已有（dev） | 单测隔离路径 |

无需新增第三方包；`docs/research/rust-package-selection.md` 无新增登记项。

## 4. 组件合同评审

`specs/llm-gateway.md` §11 Credential Source 定义了：

- `credentialSource` 可选字段（`#[serde(default)]` = `static`，向后兼容）；
- node 库函数 `resolve_default_codex_credential` / `resolve_codex_api_key` 为唯一凭证解析入口；
- 只返回短期上游 token，不返 refresh token，不暴露任意文件读取；
- Phase 1 仅 Codex API-key 模式（`OPENAI_API_KEY`）；Claude OAuth / Codex ChatGPT-login 为 Phase 2；
- 回退语义：node 失败回退静态 `upstream.apiKey`，皆空返回配置错误（不泄露）；
- 失败审计 `credential_resolve_failed`（无 token 明文）。

评审结论：合同与 PRD FR-1~FR-4 一一对应，无缺口。

## 5. 安全 Tier 分级

credential custody / LLM proxying 属 **Tier 3**（security-sensitive）。本变更：

- 凭证只读单一已知路径 `accounts/codex/<profile_id>/auth.json` 的 `OPENAI_API_KEY` 字段；
- 解析失败仅返回 `NotFound`，不回显文件内容；
- 解析出的 token 仅存于本次请求内存，不落盘、不进日志、不进 SQLite；
- 单测覆盖「无 token 泄露」形态（坏 JSON、ChatGPT-login 无 API key、缺失文件均 `NotFound`）。

## 6. 与既有权威的一致性

- 不改虚拟 key 签发、模型路由、策略/配额、注入逻辑；
- 不改变 node「凭证在使用地持有、Homie 不复制原始 token」的设计原则；
- `checkpoint.rs` 对 `auth.json` 的敏感排除不变。

## 7. 结论

方案可行、规格齐备，库内嵌实现已落地，可进入验证与发布阶段。
