# Release Readiness Report — llm-gateway-credential-login

Beads `homie-gmq` · change_id `llm-gateway-credential-login` · feature · P1

## 1. 交付范围

Phase 1 最小闭环：`homie-node` 新增 `credentials` 模块（库内嵌），`homie-gateway` 新增可选
`credentialSource`（`static` 默认 / `node`），使 Codex API-key 登录凭证可直接作网关上游，无需
手动配 `upstream.apiKey`。Claude OAuth / Codex ChatGPT-login token refresh 为 Phase 2，不在本切片。

## 2. 实现清单

| Task | 内容 | 状态 |
|------|------|------|
| T1 | `homie-node::credentials`：`CredentialKind`/`ResolvedCredential`/`resolve_codex_api_key`/`resolve_default_codex_credential` + `lib.rs` re-export | ✅ |
| T2 | `homie-gateway::config`：`CredentialSource`（`Static` 默认 / `Node`）、`credential_source` 字段、按模式空 key 校验 | ✅ |
| T3 | `homie-gateway`：`Upstream::new` 增 `prefer_node`、`resolve_credential()` 回退逻辑、`main.rs` 接线 | ✅ |
| T4 | 单测 + 集成测试适配新签名 + 无泄露断言 | ✅ |
| T5 | 门禁（fmt/clippy/test）绿 + 证据 + 关闭 Beads | ✅ |

## 3. 验证证据

### 3.1 单元测试

```text
cargo test -p homie-node --offline
  22 tests passed（含 credentials 6 例）

cargo test -p homie-gateway --offline
  34 lib tests + 13 integration tests passed
```

`credentials` 单测覆盖：API-key 解析、缺失文件 `NotFound`、ChatGPT-login 无 API-key `NotFound`、
坏 JSON `NotFound`、default 优先默认账户再首账户、无 Codex 账户 `NotFound`。

`config` 单测覆盖：`credential_source` 默认 `static`、`node` 模式允许空 key、`static` 模式拒绝空 key。

`upstream` 单测覆盖：static 模式解析静态 key、node 模式回退静态 key、node 模式无凭证报错（不泄露）。

### 3.2 集成测试

`homie/crates/homie-gateway/tests/gateway.rs` 13 例全部适配 `Upstream::new(..., prefer_node)` 新签名后通过。

### 3.3 门禁

```text
cargo fmt --all --check   → 0 diff
cargo clippy -p homie-gateway -p homie-node --all-targets --offline
  → homie-gateway / homie-node 无新增 warning（homie-engine 4 个 warning 为既有、非本变更）
```

## 4. 安全评审（Tier 3）

- 凭证只读 `accounts/codex/<profile_id>/auth.json` 的 `OPENAI_API_KEY` 字段，不读其他文件、不扩散。
- 解析失败仅返回 `NodeError::NotFound`，不回显文件内容；无 panic。
- token 仅存于请求内存，不落盘、不进日志、不进 SQLite。
- 无凭证时返回明确错误，不含密钥/账户数据。
- `static` 默认模式行为与 PRD1/PRD3/PRD4 完全一致，向后兼容。

## 5. 已知限制与后续

- **仅 Codex API-key 模式**：`auth.json` 为 ChatGPT-login（access/refresh token）形态时返回
  `NotFound` 并回退静态 key；其 token refresh 属 Phase 2。
- **Claude OAuth 上游凭证**：`.credentials.json` 的 OAuth token 刷新属 Phase 2，未实现。
- **多节点凭证共享**：当前库内嵌为本地单 node 场景；若需跨节点共享，接口已预留可演进为 RPC。

## 6. 结论

T1–T5 全部交付，fmt/clippy/test 绿，无新增告警，Tier 3 安全断言覆盖。Codex API-key 登录凭证
作上游的最小端到端闭环已落地，可发布。
