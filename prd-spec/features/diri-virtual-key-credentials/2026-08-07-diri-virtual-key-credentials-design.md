# Diri Virtual Key Credentials 第一阶段设计

```yaml
change_id: diri-virtual-key-credentials
beads: homie-e1s
lane: lane-foundation-security
status: draft_reviewed
source_specs:
  - specs/virtual-key-credentials/README.md
  - specs/llm-proxy/README.md
  - specs/remote-node-handoff/README.md
source_research:
  - docs/research/diri-module-inventory.md
  - docs/verification/diri-module-inventory/spec-dependency-analysis.md
  - docs/verification/diri-module-inventory/bingo-component-spec-review-report.md
```

## 1. 背景

Homie 的长期安全模型与 Diri 不完全相同。Diri 的 remote/node 协议包含 provider account、provider call、usage ledger、remote spawn 和 prefs sync 等能力；其中 Diri 已明确 `host.sync_prefs` 只同步 agent preference，永不包含 credentials，`NodeHelloResult` 测试也要求 hello 结果不序列化 credential reference。

Homie 的架构要求更严格：真实 provider key 只保存在 Homie 本机配置和 secret custody 边界中。Managed agent、remote node、MCP tool 和 agent config 不直接接收真实 provider key，而是通过 Homie 的 OpenAI-compatible local proxy 使用短期 virtual key。当前 `homie-llm` 已有最小 virtual key issue/validate/revoke/expiry/model-scope 测试，但长期组件规格仍缺少强制 cross-spec gate，也缺少能让后续 remote/MCP/agent lane 复用的 raw-key 禁止传播合同。

本阶段是 foundation-security 第一阶段，不实现完整 HTTP proxy、持久化 secret envelope 或 remote node handoff；目标是把 credential parity 的安全边界规格化，并在 `homie-llm` 中落地最小可测试 contract，阻止后续 lane 把 raw provider key 写进 remote node、MCP、agent config、log 或 event。

## 2. 目标

- 更新 `specs/virtual-key-credentials/README.md`，补齐 Diri/Homie 双栏适配、cross-spec mandatory gates 和第一阶段验收测试。
- 明确 raw provider key 不得进入 remote node、MCP tool、managed agent config/env、log、event、context、memory、metrics、report、checkpoint 或 prefs sync。
- 在 `homie-llm` 提供最小公共合同，表达 managed agent 只能拿到 virtual key 和 local proxy URL，不能拿到 provider raw key。
- 增补 issue、revoke、expired、session/profile/provider/model scope denied 和 no-raw-key propagation tests。
- 以 dev-loop 产物记录 spec review、functional cases、OpenSpec plan/tasks/alignment 和验证证据。

## 3. 非目标

- 不实现 OpenAI-compatible HTTP proxy 的请求转发、streaming、provider routing 或 usage cost 写库。
- 不实现 encrypted secret envelope 的最终格式、KDF、AEAD、rotation 或 Keychain/age 集成。
- 不修改 `crates/homie-remote`、`crates/homie-cli` MCP 文件、`crates/homie-runtime` 或 `specs/remote-node-handoff/README.md`。
- 不把 Diri `diri-node` 的 provider account 协议完整迁移到 Homie。
- 不引入新依赖；本阶段使用现有 `homie-llm`、`homie-proto`、`serde`、`time` 和 `uuid` 能力。

## 4. 用户场景

### 场景 1：本地 agent 获取 LLM 配置

**Given** 用户在 Homie 中为某个 session 启动 managed agent。  
**When** runtime 向 LLM credential 边界请求 agent 可见配置。  
**Then** 返回值只能包含 local proxy base URL、virtual key、expiry 和 scope metadata，不包含真实 provider key、Authorization header 或 secret ref 明文。

### 场景 2：agent 使用错误 scope 调用 proxy

**Given** agent 持有 session A 的 virtual key。  
**When** 该 key 被用于 session B、不同 agent profile、不同 provider 或未授权模型。  
**Then** Homie fail closed，返回 scope/model denied 错误，并可记录不含 secret 的 audit/event。

### 场景 3：key 被撤销或过期

**Given** session 结束、用户显式 revoke，或 key TTL 到期。  
**When** agent 继续使用旧 virtual key。  
**Then** Homie 拒绝请求，错误中不包含 key secret 或 provider raw key。

### 场景 4：remote/MCP lane 请求 credential

**Given** 后续 remote node、MCP automation 或 agent adapter 需要 LLM 能力。  
**When** 它们尝试把 raw provider key 放入 remote payload、MCP result、agent config、log 或 event。  
**Then** 本组件规格和 `homie-llm` contract test 将该行为定义为禁止；后续 lane 必须只传 virtual key/proxy config 或 node-local 用户显式登录信息。

## 5. 功能需求

### FR-1：组件规格补强

`specs/virtual-key-credentials/README.md` 必须新增第一阶段 contract：

- Diri behavior parity / Homie credential adaptation 双栏。
- Cross-spec mandatory gates：LLM proxy、remote node handoff、MCP automation、agent adapter、observability、context/memory。
- Raw provider key 禁止传播矩阵。
- 第一阶段测试映射：issue、revoke、expired、scope-denied、model-denied、no raw key propagation。

### FR-2：Virtual key 生命周期保持 fail closed

`homie-llm` 必须通过公共接口支持并测试：

- issue 返回唯一 key id 和 virtual key secret。
- validate 要求 session/profile/provider 完全匹配。
- validate 要求 model 位于 allowed_models。
- expired key 被拒绝。
- revoked key 被拒绝。
- unknown key 被拒绝。

### FR-3：Managed agent config secretless contract

`homie-llm` 必须提供第一阶段 managed proxy config 类型或构造函数：

- 包含 `base_url`、`virtual_key`、`expires_at`、`scope`。
- 不包含 provider raw key、Authorization header、secret ref 明文或 provider request header。
- 可序列化为 agent config/event payload 时仍不泄漏 raw provider key。

### FR-4：Raw provider key 禁止传播检测

`homie-llm` 必须提供第一阶段检测/拒绝函数，用于后续 remote/MCP/agent lane 在构造 payload 前复用：

- 对 remote node payload、MCP result、agent config、log/event payload 四类 destination 识别 raw provider key。
- 检测到 raw provider key 时返回 `RawProviderKeyForbidden` 或等价错误。
- 允许 virtual key、local proxy URL 和非 secret scope metadata。

### FR-5：错误与审计保持 secretless

所有新增错误、Debug 输出、测试断言和 evidence 报告不得包含真实 key 示例。测试中使用的假 raw key 只能出现在测试输入变量中，不能出现在序列化 config、错误字符串、日志或事件输出里。

## 6. 受影响组件规格

| 组件 spec | 影响 | 本阶段动作 |
|-----------|------|------------|
| `specs/virtual-key-credentials/README.md` | 是 | 更新 cross-spec gates、Diri/Homie adaptation、测试映射 |
| `specs/llm-proxy/README.md` | 间接 | 不编辑；本阶段产物供后续 lane 引用 |
| `specs/remote-node-handoff/README.md` | 间接 | 不编辑；若需要新增 no-raw-key-to-node gate，在 final 标注 |
| `specs/mcp-automation/README.md` | 间接 | 不编辑；若需要 tool payload redaction gate，在 final 标注 |
| `specs/observability/README.md` | 间接 | 不编辑；后续应引用 safe field whitelist |

## 7. 实现方案

第一阶段在 `crates/homie-llm` 内保留现有 in-memory virtual key store，补充一个小的 credential contract 层：

- `ManagedLlmProxyConfig`：agent 可见配置，字段只允许 local proxy URL、virtual key、expiry、scope。
- `CredentialDestination`：描述 payload 目的地，如 remote node、MCP tool、agent config、log/event。
- `CredentialPropagationPolicy` 或等价函数：在 payload 构造前拒绝 raw provider key，允许 secretless payload。
- 错误类型扩展为 raw provider key forbidden，错误消息不回显 secret 值。

该实现不解决持久化和加密 envelope，只为后续 proxy/runtime/remote/MCP 提供可测试边界。

## 8. 测试计划

| 类型 | 覆盖点 | 命令 |
|------|--------|------|
| RED/GREEN 单元测试 | issue/validate/revoke/expired/scope/model denied | `cargo test -p homie-llm virtual_key` |
| Contract tests | managed config 不序列化 raw key；remote/MCP/agent/log payload 拒绝 raw key | `cargo test -p homie-llm credential_contract` |
| 格式 | Rust 格式 | `cargo fmt --all -- --check` |
| 编译 | workspace 检查 | `cargo check --workspace` |
| 安全/补丁 | trailing whitespace / diff check | `git diff --check` |

## 9. 验收标准

- PRD、spec review、functional cases、OpenSpec plan/tasks/alignment 均存在并相互引用。
- `specs/virtual-key-credentials/README.md` 包含 cross-spec mandatory gates 和第一阶段 Diri/Homie adaptation。
- `cargo test -p homie-llm` 通过，覆盖 issue、revoke、expired、scope-denied、model-denied、no raw key propagation。
- 新增 `homie-llm` 公共类型和错误不泄漏 raw provider key。
- 本次未修改 remote/MCP 文件；若发现必须改动，记录为后续 lane follow-up。

## 10. Beads 跟踪

| Field | Value |
|-------|-------|
| Beads issue | `homie-e1s` |
| Change ID | `diri-virtual-key-credentials` |
| Lane | `lane-foundation-security` |
| Priority | P0 |
| Spec path | `prd-spec/features/diri-virtual-key-credentials/2026-08-07-diri-virtual-key-credentials-design.md` |
| Evidence path | `docs/verification/diri-virtual-key-credentials/` |

