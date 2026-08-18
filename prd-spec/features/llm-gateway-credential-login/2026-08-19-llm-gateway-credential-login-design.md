# Homie LLM 网关凭证接入：Claude Code / Codex 登录凭证作上游设计文档

## 1. 概述

### 1.1 问题/背景

PRD1 `llm-gateway-virtual-keys`（Beads 已关闭）交付了本地网关 + 虚拟 key 签发 + 上游转发；
上游凭证来自 `homie.local.json` 里**手动录入**的 OpenAI 兼容 `upstream.base_url/api_key`。
PRD3 `llm-gateway-model-routing`、PRD4 `llm-gateway-policy-quota` 均明确把「登录凭证接入上游」
列为 child Bead `llm-gateway-credential-login`，本 PRD 落实该目标。

当前痛点：

- 用户已用 Claude Code / Codex 登录，却还要**另找一把 OpenAI 兼容 API key** 手动配进
  `homie.local.json`，重复、易错、且与 Homie「统一凭证入口」的定位不符；
- `homie-node` 已实现 provider 登录/状态/账户管理（`AccountStore` + `ProviderManager`），
  但没有任何代码把登录后的 token 用于网关上游，登录资产被闲置；
- 网关与 node 完全独立，网关不感知账户/登录状态。

### 1.2 目标

1. 复用 `homie-node` 的账户与登录会话，把 Claude Code / Codex 的登录凭证作为网关上游的可选
   凭证源，替换/补充手动 `api_key`。
2. 网关新增「可选上游凭证源」：优先向 node 解析短期上游 token，失败/过期/未登录时回退现有
   手动 `upstream.api_key`，不破坏既有部署。
3. 凭证解析逻辑只落在 `homie-node`（单一凭证托管边界），网关不直接读取 provider 私有
   auth 文件。
4. Phase 1 只支持**最容易**的凭证形态（Codex API-key 模式的 `OPENAI_API_KEY`），先跑通
   最小端到端闭环；Claude OAuth token 与 Codex ChatGPT 登录 token 的抽取/刷新列为 Phase 2。

### 1.3 非目标

- 不接入 aimux / 多 provider 扩展（属 `llm-gateway-provider-expansion`）。
- 不做 Claude OAuth / Codex ChatGPT 登录 token 的 refresh（Phase 2）。
- 不把原始 refresh token / 长期凭证下发到 managed agent 配置（始终只下发虚拟 key）。
- 不改注入逻辑（`homie-engine::inject`）、不改虚拟 key 签发、不改模型路由/策略。
- 不做多节点凭证共享（本地单 node）。

### 1.4 可行性分析与复用评估（方案 A）

#### 复用资产（已核实）

- `homie-node::accounts::AccountStore`：账户元数据 + `config_home(profile)` 返回
  `accounts_root/{provider}/{profile_id}`（等价 `CODEX_HOME` / `CLAUDE_CONFIG_DIR` 的隔离目录）。
- `homie-node::provider::ProviderManager`：Codex 走 `codex app-server --stdio`
  （`account/read`、`account/login/start`）；Claude 走 `claude auth status/login`。
- `homie-client::NodeClient`：已有加密私有网络客户端，`provider.call` RPC
  （`ProviderCallParams { profile_id, method, params }` → `ProviderCallResult`）。
- 设计原则（`homie-node/src/lib.rs`）：「node 在凭证使用地持有凭证，Homie 不复制/托管原始
  token」；`checkpoint.rs` 已把 `auth.json`/`.credentials.json`/`credentials.json` 列为敏感
  排除项。

#### 关键技术现实

- Codex 登录有**两种模式**：① API-key 模式（`auth.json` 存 `OPENAI_API_KEY`，可直接当上游
  bearer）；② ChatGPT 登录模式（`auth.json` 存 access/refresh token，需 refresh）。
- Claude 可用上游凭证是 `.credentials.json` 的 OAuth token，**需向 Anthropic refresh**。
- `account/read` 只返回账户信息、`claude auth status --json` 也不返回可复用 token；真正可作
  上游的凭证在 provider 私有 auth 文件里，必须有人去读并刷新。
- 关键有利点：provider CLI 通过 `CODEX_HOME`/`CLAUDE_CONFIG_DIR` 环境变量，把 auth 文件写到
  Homie 隔离的 `config_home` 目录（`accounts_root` 之下），而非用户全局 `~/.codex`/`~/.claude`。
  因此 node 读自己 `config_home` 下的 auth 文件，安全边界清晰、不扩散。

#### 三方案对比

| 方案 | 做法 | 优点 | 缺点 |
|---|---|---|---|
| **A（推荐）** | node 单一托管凭证，新增受限 `credential.resolve`，网关经 node 取短期 token | 不破坏 Tier 3 单一托管；登录/刷新只实现一次；复用现有加密通道 | 网关需新增 node 客户端与降级；node 侧仍需新写 token 抽取 |
| B | 网关只复用 `config_home` 定位，直接读 provider auth 文件 | 单机 MVP 侵入最小 | 凭证托管职责分裂到网关；解析私有且易碎格式；敏感文件新增读取面 |
| C | 保留手动 key + 桥接导入 | 风险最低 | 没达到目标，体验提升有限 |

**结论：选 A**，分两阶段，避免过度设计（符合 AGENTS.md「最小端到端优先」与「长期架构决策」）。

### 1.5 关键设计决策

#### 决策 A：homie-node 以库内嵌方式暴露受限凭证解析函数（唯一凭证解析入口）

- 在 `homie-node` 新增 `credentials` 模块，暴露纯函数 `resolve_default_codex_credential(paths)`
  与 `resolve_codex_api_key(paths, profile_id)`，返回 `ResolvedCredential { kind, base_url, token }`。
  **库内嵌**：`homie-gateway` 直接以 Rust crate 依赖调用这些函数，不走跨进程 RPC。
- `kind` 区分：`codex_api_key`（Phase 1）/ `codex_oauth`（Phase 2）/ `claude_oauth`（Phase 2）。
- node **只返回短期/一次性 token**，不返回 refresh token；原始凭证永不离开 node 进程内边界。
- 解析函数只读 `NodePaths` 下 `accounts/codex/<profile_id>/auth.json` 的 `OPENAI_API_KEY` 字段，
  不暴露任意文件读取，不回显文件内容（失败仅返回 `NotFound`）。

#### 决策 B：网关新增可选上游凭证源，静态 `api_key` 降级兜底

- `GatewayConfig` 新增可选 `credential_source`：`static`（默认，读 `upstream.api_key`）或
  `node`（连 node 解析 token）。
- 转发前若 `credential_source == node`，先尝试经 node 解析短期 token；失败/过期/未登录则
  **回退** `upstream.api_key`（若已配置），否则返回明确的 `503`/配置错误。
- 短期 token 仅存内存、不落盘、不进日志、不写入 SQLite。

#### 决策 C：Phase 1 只做 Codex API-key 模式，最小闭环

- 读 `config_home/auth.json` 的 `OPENAI_API_KEY`，当作 OpenAI 兼容上游 `base_url + Bearer`。
- `base_url` 由 `credential.resolve` 返回（Phase 1 固定为 Codex 默认 OpenAI 兼容端点，或从
  `homie.local.json` 的既有 `upstream.base_url` 继承）。
- Claude OAuth / Codex ChatGPT 登录 token 的抽取与 refresh 属 Phase 2，本 PRD 只声明不实现。

## 2. 用户场景

### 场景 1：Codex API-key 登录凭证直接作上游

**Given** 用户已在 node 通过 Codex API-key 模式登录，且网关配置 `credential_source = "node"`。  
**When** managed agent 用虚拟 key 请求网关。  
**Then** 网关经 node 解析出该 profile 的 `OPENAI_API_KEY`，以其为上游 bearer 转发，用户无需
手动配 `upstream.api_key`。

### 场景 2：node 不可用/未登录时回退手动 key

**Given** 网关配置 `credential_source = "node"`，但 node 未登录或不可达，且 `upstream.api_key`
已配置。  
**When** 请求到达。  
**Then** 网关回退用 `upstream.api_key` 转发；node 恢复后自动切回 node 凭证。

### 场景 3：无任何可用凭证时明确报错

**Given** `credential_source = "node"`，node 未登录且 `upstream.api_key` 为空。  
**When** 请求到达。  
**Then** 网关返回明确错误（`503`/配置错误 body），不泄露任何密钥/账户信息。

### 场景 4：静态模式行为不变（默认）

**Given** 未配置 `credential_source`（默认 `static`）。  
**When** 任意请求。  
**Then** 网关读 `upstream.api_key` 转发，行为与 PRD1/PRD3/PRD4 完全一致。

## 3. 功能需求

### FR-1: homie-node 新增库内嵌凭证解析函数

- `homie-node::credentials` 暴露 `resolve_default_codex_credential(paths)` 与
  `resolve_codex_api_key(paths, profile_id)`，返回 `ResolvedCredential { kind, base_url, token }`
  （失败返回 `NodeError::NotFound`）。
- `kind == codex_api_key` 时，从 `accounts/codex/<profile_id>/auth.json` 解析 `OPENAI_API_KEY`；
  缺失/格式不符/非 API-key 模式返回 `NotFound`。
- 只读单一已知路径下的 `OPENAI_API_KEY` 字段，**不暴露任意文件读取**；原始 token 不回显、不落盘。

### FR-2: 网关配置新增可选 `credential_source`

- `GatewayConfig` 新增 `credential_source: Option<CredentialSource>`，`#[serde(default)]`。
- `CredentialSource::Static`（默认）/ `CredentialSource::Node`。
- 缺失 → `Static`，与既有部署零冲突。

### FR-3: 网关上游凭证解析与回退

- `Upstream` 增加 `prefer_node: bool`（由 `credential_source` 推导），`forward` 前经
  `resolve_credential()` 解析当前 token。
- `node` 模式：优先库内嵌调用 `homie_node::credentials::resolve_default_codex_credential`；
  失败回退 `upstream.api_key`；两者皆不可用返回配置错误（映射为上游 502/503，不含密钥）。
- `static` 模式：直接读 `upstream.api_key`（现状）。
- 解析出的 token 仅用于本次请求内存，不落盘、不进日志、不进 SQLite。

### FR-4: 安全与审计

- `credential.resolve` 返回的 token、node 读取的 auth 文件内容，均不进日志、不写 SQLite。
- 网关回退/解析失败事件记入 `gateway_audit`（原因：`credential_resolve_failed`），不含 token。
- node 侧解析 auth 文件失败仅返回 `NotAuthenticated`，不回显文件内容。

## 4. 受影响 Specs

- `specs/llm-gateway.md`：新增「§Credential Source」契约（可选凭证源 + 回退语义）。
- node 凭证契约（当前无独立 `specs/homie-node.md`，在 `specs/llm-gateway.md` 内以小节补充
  `credential.resolve` 协议，或新建 `specs/homie-node-credentials.md` —— 实现时定）。

## 5. 测试计划

- 单测：`credential.resolve` 对 `auth.json` 各形态（存在/缺失/坏 JSON/非 API-key 模式）的解析
  与错误映射。
- 单测：网关 `credential_source` 配置解析（默认 `static`）、回退逻辑、`503` 错误 body 无泄露。
- 集成（wiremock/tower）：`node` 模式转发用解析出的 token；回退用静态 key；无凭证时 503。
- 安全：断言日志/SQLite 不含 `OPENAI_API_KEY` 或解析出的 token 明文。
- 全量 `cargo test -p homie-gateway -p homie-node --offline` 绿。

## 6. 验收标准

- Codex API-key 登录后，`credential_source = "node"` 下 managed agent 可无需手动 `api_key` 完成
  一次转发（端到端）。
- 默认 `static` 模式下所有既有测试/行为不变。
- 无凭证可用时返回明确错误且不泄露密钥。
- 证据齐全：`docs/verification/llm-gateway-credential-login/`（spec-review / functional-cases /
  functional-verification / code-review / release-readiness）。

## 7. Beads 追踪

- change_id: `llm-gateway-credential-login`
- 类型: feature
- 优先级: P1（Phase 1 最小闭环）
