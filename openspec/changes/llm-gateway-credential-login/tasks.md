# OpenSpec Tasks — llm-gateway-credential-login

## T1: proto 类型与 RPC 方法

- 交付：`homie-proto` 增 `credential.resolve` 的 `NodeMethod` 常量 + `CredentialResolveParams`
  / `CredentialResolveResult`（`{ kind, base_url, token }`）+ `NotAuthenticated` 错误语义。
- 验收：类型可 serde round-trip；`NodeMethod` 常量不冲突。
- 关联验证 Case：FC-1。

## T2: node 侧 `credential.resolve` 解析（codex_api_key）

- 交付：`homie-node` 增 `resolve_credential(store, profile_id)`：读 `config_home/auth.json` 的
  `OPENAI_API_KEY`；缺失/坏 JSON/非 API-key 模式 → `NotAuthenticated`。
- 验收：auth.json 各形态（存在/缺失/坏 JSON/仅 ChatGPT token）单测全绿；不 panic。
- 关联验证 Case：FC-1、FC-2。

## T3: node RPC 接线 + 白名单

- 交付：`service.rs` 分发 `credential.resolve`；白名单校验，仅允许该方法，不暴露任意文件读。
- 验收：非法 method 被拒；`credential.resolve` 返回类型正确。
- 关联验证 Case：FC-1。

## T4: gateway 配置 `credential_source`

- 交付：`GatewayConfig` 增 `credential_source: Option<CredentialSource>`（`#[serde(default)]`），
  `from_file` 填充，默认 `Static`。
- 验收：缺失→`Static`；`node` 反序列化正确；camelCase 对齐。
- 关联验证 Case：FC-3。

## T5: gateway 动态凭证解析 + 回退

- 交付：`Upstream` 支持动态凭证解析器；`node` 模式经 `NodeClient` 调 `credential.resolve`，
  失败回退 `upstream.api_key`，皆空返回 `503` 配置错误 body（无密钥）。
- 验收：node 成功→用解析 token；node 失败+有静态 key→回退；皆空→503 无泄露。
- 关联验证 Case：FC-3、FC-4、FC-5。

## T6: 单测 + 集成测试 + 安全断言

- 交付：node 解析单测；gateway 配置/回退/503 单测；集成测试（wiremock/tower）；日志/SQLite
  无 token 明文断言。
- 验收：`cargo test -p homie-node -p homie-gateway --offline` 全绿。
- 关联验证 Case：FC-5、FC-6。

## T7: 门禁 + 证据 + 关闭

- 交付：`cargo fmt --check` / `clippy` / `test` 绿；spec review + functional + release readiness
  证据；Beads `homie-gmq` 关闭。
- 关联验证 Case：FC-6。
