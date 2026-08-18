# OpenSpec Tasks — llm-gateway-credential-login

## T1: homie-node 库内嵌凭证解析模块

- 交付：`homie-node::credentials` 新增 `CredentialKind`、`ResolvedCredential`、
  `resolve_codex_api_key`、`resolve_default_codex_credential`；`lib.rs` 增 `pub mod credentials`
  并 re-export。
- 验收：函数签名可用；`resolve_codex_api_key` 对 auth.json 各形态（存在/缺失/坏 JSON/非 API-key）
  正确返回或 `NotFound`；不 panic。
- 关联验证 Case：FC-1、FC-2。

## T2: gateway 配置 `credential_source`

- 交付：`homie-gateway::config` 新增 `CredentialSource`（`Static` 默认 / `Node`），
  `FileConfig`/`GatewayConfig` 增 `credential_source`（`#[serde(default)]`）；`node` 模式允许空
  `upstream.apiKey`，`static` 模式仍拒绝空 key。
- 验收：缺失→`Static`；`node` 反序列化正确；camelCase 对齐；空 key 校验按模式区分。
- 关联验证 Case：FC-3。

## T3: gateway 库内嵌调用 + 回退

- 交付：`homie-gateway` 增 `homie-node` 依赖；`Upstream` 增 `prefer_node` 字段与
  `resolve_credential()`；`node` 模式先调 `resolve_default_codex_credential(&NodePaths::discover())`，
  失败回退静态 `api_key`，皆空返回 `UpstreamError`（映射为上游失败，不泄露密钥）。
- 验收：node 成功→用解析 token；node 失败+静态 key→回退；皆空→error（不泄露）。
- 关联验证 Case：FC-1、FC-3、FC-4、FC-5。

## T4: 单测 + 集成测试 + 安全断言

- 交付：node 解析单测（6 例）；gateway 配置/回退/无凭证单测；集成测试适配 `Upstream::new`
  新签名；日志/SQLite 无 token 明文断言。
- 验收：`cargo test -p homie-node -p homie-gateway --offline` 全绿。
- 关联验证 Case：FC-5、FC-6。

## T5: 门禁 + 证据 + 关闭

- 交付：`cargo fmt --check` / `clippy` / `test` 绿；spec review + functional + release readiness
  证据；Beads `homie-gmq` 关闭。
- 关联验证 Case：FC-6。
