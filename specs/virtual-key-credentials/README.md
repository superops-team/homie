# Virtual Key & Credentials 组件规格

## 1. 组件定位

`virtual-key-credentials` 定义 Homie 的真实 provider credential custody、secret envelope、virtual key 签发/撤销/作用域/审计和远端 node credential policy。它是 Reference parity 中所有 agent、remote、MCP、LLM proxy 的安全边界。

## 2. 来源需求映射

- PRD: `prd-spec/features/reference-parity-v1/2026-08-05-reference-parity-v1-design.md`
- OpenSpec: `openspec/changes/reference-parity-v1/`
- 功能验证: FC-013, FC-015, FC-017, FC-018

## 3. 上游与下游关系

| 方向 | 组件 | 关系 |
|------|------|------|
| 上游 | `homie-llm` | 校验 virtual key、解析 provider secret ref |
| 上游 | `homie-runtime` | session spawn 时请求 managed agent credentials |
| 上游 | `remote-node-handoff` | 定义远端执行 credential 策略 |
| 下游 | encrypted secret envelope | 保存 provider key envelope 或 secret ref |
| 下游 | `homie-storage` | 保存 metadata、audit、revocation |

## 4. 职责边界

负责：

- provider raw key 的本地 secret envelope metadata。
- virtual key 签发、scope、expiry、revocation。
- secret access audit。
- 远端/node 模式 credential policy。
- redaction policy for credential-shaped values。

不负责：

- HTTP provider 转发。
- agent process env 组装。
- UI 表单渲染。

## 5. 核心接口

```rust
pub trait CredentialService {
    fn issue_virtual_key(&self, request: VirtualKeyRequest) -> Result<VirtualKey, CredentialError>;
    fn revoke_virtual_key(&self, key_id: VirtualKeyId) -> Result<(), CredentialError>;
    fn validate_virtual_key(&self, presented: SecretString, scope: VirtualKeyScope) -> Result<VirtualKeyClaims, CredentialError>;
    fn resolve_provider_secret(&self, secret_ref: SecretRef) -> Result<SecretString, CredentialError>;
}
```

## 6. 数据模型

```rust
pub struct VirtualKeyScope {
    pub session_id: SessionId,
    pub agent_profile_id: AgentProfileId,
    pub provider_id: ProviderId,
    pub allowed_models: Vec<String>,
}

pub struct VirtualKeyClaims {
    pub key_id: VirtualKeyId,
    pub scope: VirtualKeyScope,
    pub expires_at: OffsetDateTime,
    pub revoked_at: Option<OffsetDateTime>,
}
```

## 7. 运行模型与状态机

```text
provider key configured
  -> stored as secret envelope/ref
  -> session spawn requests virtual key
  -> virtual key issued with scope/expiry
  -> proxy validates every request
  -> revoke on session end or explicit revoke
```

## 8. 安全与权限

- raw provider key 不进入 agent env/config/log/event/context/memory/metrics/report。
- secret material 使用 secrecy/zeroize 或等价模式。
- virtual key 默认短期有效且可撤销。
- node/handoff checkpoint 不包含 provider credential。
- remote node 若允许 node-local login，必须作为用户显式配置，不能从 Homie 复制 raw key。

## 9. 可观测性

审计事件：

- credential.secret_configured。
- virtual_key.issued。
- virtual_key.revoked。
- virtual_key.validation_failed。

事件中只记录 key id、scope id、错误码，不记录 secret 值。

## 10. 失败与恢复

| 场景 | 行为 |
|------|------|
| envelope 解密失败 | fail closed |
| virtual key 过期 | reject |
| revoked key 使用 | reject + audit |
| wrong provider/model scope | reject + audit |
| remote node 请求 raw key | reject |

## 11. 测试计划与验收引用

- FC-013: virtual key scope/expiry/revoke/no leak。
- FC-015: remote/node handoff 不携带 credential。
- FC-017: updater 不执行未验证 bundle。
- FC-018: full local quality gate。

## 12. Diri Behavior Parity / Homie Credential Adaptation

| Diri behavior | Homie adaptation |
|---------------|------------------|
| `host.sync_prefs` 同步 agent preferences，明确不包含 credentials、transcripts 或 caches | Homie remote prefs sync 只能同步 secretless preference/config，provider raw key 必须留在本机 credential custody |
| Diri node protocol 暴露 provider accounts、provider calls 和 usage ledger | Homie managed agents 默认不直接调用 remote provider account；它们通过 Homie local proxy 和 scoped virtual key 调用 provider |
| Diri node hello/result 不序列化 credential reference | Homie remote/node/companion hello、handoff、checkpoint、manifest 和 event payload 均不得包含 raw provider key 或 secret ref 明文 |
| Diri remote host 可作为执行位置 | Homie remote host 只能接收 virtual key/proxy config 或 node-local 用户显式登录状态，不能从 Homie 复制 provider raw key |
| Diri usage/fleet 能汇总 provider usage | Homie usage accounting 后续必须以 safe usage record 写入，不保存 raw request、raw response、Authorization 或 provider key |

## 13. Cross-Spec Mandatory Gates

任何后续 spec 或实现只要涉及 provider credential、virtual key、LLM proxy、remote/node、MCP、agent config、logs/events、context 或 memory，必须引用本组件并满足下表：

| Consumer spec/module | Mandatory gate | First-stage verification |
|----------------------|----------------|--------------------------|
| `specs/llm-proxy/README.md` / `homie-llm` | Proxy validate 前只接受 virtual key；provider raw key 只在 resolver 和 upstream request 短期内存中出现 | FC-DVKC-001, FC-DVKC-002, FC-DVKC-003, FC-DVKC-004 |
| `specs/remote-node-handoff/README.md` / `homie-remote` | Remote spawn、handoff、checkpoint、manifest、prefs sync、repo locate 不包含 raw provider key 或 secret ref 明文 | FC-DVKC-005 |
| `specs/mcp-automation/README.md` / MCP tools | Tool args/result、lineage、evidence 和 error envelope 不包含 raw provider key；需要 LLM 时只使用 virtual key/proxy config | FC-DVKC-005 |
| `specs/agent-adapter-contract/README.md` / agent config | Agent env/config 只能包含 virtual key 和 local proxy URL；manifest 不能声明 raw provider key | FC-DVKC-004, FC-DVKC-005 |
| `specs/observability/README.md` / logs/events/metrics | 事件只记录 key id、scope id、destination、error code；不记录 presented secret、raw provider key、Authorization、cookie | FC-DVKC-002, FC-DVKC-005 |
| `specs/session-context-store/README.md` and `specs/memory-controller/README.md` | Context/memory 禁止保存 raw provider key、Authorization、raw request/response 或完整 secret-bearing tool args/result | FC-DVKC-005 |

## 14. Raw Provider Key Forbidden Matrix

| Destination | Raw provider key | Virtual key | Secretless metadata |
|-------------|------------------|-------------|---------------------|
| Managed agent env/config | forbidden | allowed when scoped and short-lived | allowed |
| Remote node request/result | forbidden | allowed only as scoped proxy credential when required by local proxy flow | allowed |
| MCP tool args/result | forbidden | allowed only when tool identity and permission profile allow LLM proxy use | allowed |
| Handoff checkpoint/manifest/blob list | forbidden | forbidden unless a later spec proves a scoped handoff proxy need | allowed |
| Logs/events/metrics/evidence | forbidden | key id only; presented virtual key secret forbidden | allowed |
| Context/memory/report | forbidden | key id only; presented virtual key secret forbidden | allowed |
| Provider upstream request | allowed only in short-lived in-memory Authorization injection | not sent upstream | allowed |

## 15. First-Stage Implementation Contract

The first `diri-virtual-key-credentials` slice must provide:

- scoped virtual key issue/validate/revoke/expiry/model-denial tests;
- `ManagedLlmProxyConfig` or equivalent agent-visible config containing local proxy URL, virtual key, expiry and scope only;
- a raw-key propagation guard for remote node, MCP, managed agent config, and log/event destinations;
- secretless error messages that do not echo presented virtual keys or fake raw provider keys;
- evidence under `docs/verification/diri-virtual-key-credentials/`.

## Diri Parity Mapping

This section is mandatory for Diri-to-Homie parity planning and fixes the review gap recorded in `docs/verification/diri-module-inventory/bingo-component-spec-review-report.md`.

| Field | Value |
|-------|-------|
| Owned feature atoms | M18-F001 credential boundary, M19-F001 provider custody, Homie virtual key behavior |
| Required Diri test mapping | remote node no-raw-key, MCP no-raw-key, managed agent secretless config, issue/revoke/expired/scope-denied tests |
| Pre-implementation gaps | full proxy forwarding, secret envelope persistence, remote node and MCP spec references |

Rules:

- A PRD/OpenSpec change touching this component must cite the owned feature atom ids above.
- Implementation is not complete until the required Diri test mapping has an equivalent Homie verification gate.
- If a new Diri source/test is discovered for this component, update `docs/research/diri-module-inventory.md` and this section before coding.

## 16. Diri 7ba3407 重基线修订

权威来源：

- PRD: `prd-spec/features/diri-7ba3407-parity-rebaseline/2026-08-08-diri-7ba3407-parity-rebaseline-design.md`
- 能力矩阵: `docs/research/diri-7ba3407-capability-matrix.md`
- Requirements: FR-04, FR-12, FR-13, FR-16
- Beads: `homie-t3u`

本组件当前状态是 `partial`。in-memory issue/validate/revoke tests 不等于真实 credential custody；encrypted secret envelope、持久化 revocation/audit、proxy integration 和 remote policy 仍必须实现。

### 16.1 Secret Envelope 最小合同

- envelope 格式必须 versioned，并明确 KDF、AEAD、nonce、AAD、key source 和 rotation。
- SQLite 只保存 envelope/ref 和 secretless metadata，不保存 raw provider key。
- decrypt 只发生在 LLM proxy upstream request 的短期内存中，使用 secrecy/zeroize 或等价策略。
- decrypt、version、integrity、key unavailable 和 rotation failure 全部 fail closed。
- 不为旧明文配置提供自动 fallback；开发示例只使用 sanitised placeholder。

### 16.2 跨进程与远端传播

- agent env/config 只含 scoped virtual key 和 local proxy URL。
- runtime/client/MCP/browser sidecar/node/checkpoint/prefs sync/package/evidence 不得包含 raw provider key。
- remote node 若使用 node-local provider login，必须由用户在 node 上显式建立，不能从 Homie 复制。
- virtual key 必须有 session/profile/provider/model scope、expiry、revocation 和 audit。
- presented virtual key secret 不进入日志；只允许 key id 和 scope id。

### 16.3 完成门禁

- envelope roundtrip、tamper、wrong key/version、rotation 和 zeroization 测试通过。
- proxy、agent、remote、MCP、sidecar、checkpoint、logs/events/context/memory 的 propagation denial 测试通过。
- session end、explicit revoke、expiry 和 wrong-scope 路径通过真实 proxy E2E。
- package 和 repository secret scan 不包含真实 key 或 virtual key signing secret。
