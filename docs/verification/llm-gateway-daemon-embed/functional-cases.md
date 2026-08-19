# Functional Cases — llm-gateway-daemon-embed

- Beads: `homie-6md` · change_id: `llm-gateway-daemon-embed`

## FC-1 协议收敛 OpenAI-only

- 输入：`POST /v1/messages`（Anthropic Messages 形状）。
- 预期：路由不匹配，返回 `404 NOT_FOUND`，不转发上游。
- 覆盖：`homie-gateway/tests/gateway.rs::messages_route_is_gone`。

## FC-2 Codex Responses 转发 + 用量 + 策略

- 输入：`POST /v1/responses`，携带有效虚拟 key。
- 预期：鉴权通过 → 策略/配额 → 模型路由 → 上游转发 → SSE 回传 → 写 `gateway_usage`。
- 覆盖：`responses_slice_records_usage_per_key`、`codex_model_is_rewritten_before_forward_and_recorded`、
  `master_key_bypasses_policy`、`rate_limit_rejects_excess_requests`、`quota_rejects_when_daily_limit_exceeded`。

## FC-3 虚拟 key 签发内嵌 daemon（无 `/admin/keys`）

- 输入：daemon spawn 一个 gateway-routing 的 agent。
- 预期：`GatewayIssuer::mint` 调 `GatewayApiKeyStore.create` 签发 `sk-…`，注入 agent env；`/admin/keys` 不存在。
- 覆盖：`Harness::mint_key`（in-process store）替代原 `/admin/keys`；admin 面已从 routes 删除。

## FC-4 撤销 key 401

- 输入：已删除/撤销的虚拟 key 请求。
- 预期：`401 Unauthorized`，不转发上游。
- 覆盖：`revoked_key_returns_unauthorized`、`bad_key_is_rejected_and_never_forwarded`。

## FC-5 Claude Code 退出 LLM 纳管

- 输入：daemon spawn Claude Code。
- 预期：不注入 `ANTHROPIC_BASE_URL` / `ANTHROPIC_AUTH_TOKEN`；`--settings`/`--mcp-config`/hooks 保留。
- 覆盖：`claude_gateway` 字段与 `claude_gateway_env` 已删除；resume 路径不再重放 Claude gateway env。
