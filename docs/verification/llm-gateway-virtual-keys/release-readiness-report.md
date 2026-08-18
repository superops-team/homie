# Release Readiness Report — llm-gateway-virtual-keys

Beads `homie-f91` · change_id `llm-gateway-virtual-keys` · feature · P0

## 1. 交付范围

首个纵向切片：新增 `homie-gateway` crate（本地 HTTP 网关 + 虚拟 key + 上游转发 + 用量），
并扩展 `homie-engine` 的 spawn 注入，使 Codex/Claude 可自动指向本地网关。打通
「虚拟 key → agent 指向本地网关 → 网关转发上游 → 记录用量」链路。

## 2. 实现清单

| Task | 内容 | 状态 |
|------|------|------|
| T1 | `homie/crates/homie-gateway` 脚手架、workspace 注册、`main.rs` | ✅ |
| T2 | `config.rs`：`homie.local.json` 加载、loopback-only、缺失凭证硬失败 | ✅ |
| T3 | `auth.rs` + `db.rs`：虚拟 key store（SHA-256 存哈希、`sk-` 生成、增删查、`last_used_at`）、SQLite 持久化 | ✅ |
| T4 | `auth.rs`：master + 虚拟 key，`Bearer`/`x-api-key` 双 header、Bearer 优先、401 脱敏 | ✅ |
| T5 | `upstream.rs`：单一 OpenAI-compatible 转发、SSE 透传、usage 提取 | ✅ |
| T6 | `routes.rs`：`/v1/responses`、`/v1/messages`、`/admin/keys`（master 保护） | ✅ |
| T7 | `usage.rs`：按虚拟 key 记录 `model`/token/时间 | ✅ |
| T8 | `homie-engine`：`InjectionSpec` 新增 `codexGateway`/`claudeGateway`；`inject.rs` 新增 `GatewayRuntime`/`codex_gateway_args`/`claude_gateway_env`/`codex_gateway_env`；manifests 声明字段 | ✅ |
| T9 | `tests/gateway.rs`：端到端集成测试（虚拟 key → 转发 → 用量） | ✅ |
| T10 | spec/PRD/文档同步、依赖登记、证据 | ✅ |

## 3. 验证证据

### 3.1 单元测试

```text
cargo test -p homie-gateway --offline
  11 unit tests passed (config/auth/routes/upstream/usage)

cargo test -p homie-engine --lib --offline
  300 tests passed，含 4 个网关注入新测试
```

### 3.2 集成测试

`homie/crates/homie-gateway/tests/gateway.rs` 7 个端到端用例（wiremock 模拟上游）：

- `responses_slice_records_usage_per_key`：虚拟 key → `/v1/responses` → 转发 → 用量落库
- `messages_slice_uses_same_virtual_key`：同一虚拟 key → `/v1/messages`
- `bad_key_is_rejected_and_never_forwarded`：坏 key 在鉴权层 401，不触达上游
- `revoked_key_returns_unauthorized`：撤销后 401
- `master_key_is_accepted_but_not_usage_recorded`：master 不落用量
- `admin_requires_master_key`：无 master key 时 admin 403
- `virtual_key_cannot_admin`：虚拟 key 不能触达 admin 面

```text
cargo test -p homie-gateway --offline
  7 integration tests passed
```

### 3.3 注入形状

`homie/crates/homie-engine/src/inject.rs`：

- Codex argv：`-c model_provider="homie" -c model_providers.homie.base_url="<gateway>/v1"
  -c model_providers.homie.wire_api="responses" -c model_providers.homie.env_key="HOMIE_CODEX_GATEWAY_KEY"`
- Claude env：`ANTHROPIC_BASE_URL=<gateway>`、`ANTHROPIC_AUTH_TOKEN=<virtual-key>`
- Codex env：`HOMIE_CODEX_GATEWAY_KEY=<virtual-key>`

## 4. 安全评审

- 虚拟 key 仅存 SHA-256 哈希，明文仅签发时返回一次，`list()` 永不返回明文。
- 上游真实 key 仅服务端附加，调用方不可见、不进日志。
- `homie.local.json` 与 `gateway.sqlite3` 均已被 `.gitignore`（`*.local.json`/`homie.local.*`/
  `*.sqlite3`）覆盖。
- 网关强制 loopback-only bind；无 master key + 非回环 bind 硬失败。
- 常量时间比较用于 master key / key hash 匹配。

## 5. 已知限制与后续

- **虚拟 key 发放接线（未闭环）**：`InjectionSpec`/`inject.rs` 的注入机制已落地并通过
  形状测试，但 `homied-rs` 的 `InjectionConfig.gateway` 当前为 `None`——即 daemon 尚未在
  spawn 时向网关申请虚拟 key 并注入。这需要 engine 经共享 SQLite 或网关 `/admin/keys`
  管理接口发放 key 的接线，属下一增量切片。manifests 中 `codexGateway`/`claudeGateway`
  当前声明为 `false`，待接线后翻转为 `true`。
- 上游转发为单一 OpenAI-compatible provider；`/v1/messages` 原样透传到同一上游（未做
  Anthropic↔OpenAI 协议映射），属 `llm-gateway-provider-expansion` child。
- 用量为估算（从响应 `usage` 对象提取，缺失为 0），非权威计费。

## 6. 结论

T1–T10 全部交付，编译清洁（0 warning），单元 + 集成测试绿。虚拟 key 签发/鉴权/转发/用量
端到端闭环已证明。spawn 侧虚拟 key 发放接线与协议映射留待后续 child Bead，不阻塞本切片合入。
