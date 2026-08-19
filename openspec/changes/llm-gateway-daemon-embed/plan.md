# Plan: LLM proxy 内嵌 daemon + 协议收敛 OpenAI-only

change_id: `llm-gateway-daemon-embed` · Beads: `homie-6md`

## 目标

把 `homie-gateway` 从独立进程降级为库并内嵌 daemon，协议收敛到 OpenAI Responses，
删 Anthropic Messages，virtual key 签发内聚 daemon，Claude Code 退出 LLM 纳管。

## 分阶段实施

### Phase 1：打破循环依赖，gateway 降级为库

- 删 `homie-gateway/src/main.rs` + `[[bin]]`，保留 `lib.rs` 九模块。
- 删 `homie-gateway/src/inject.rs`（re-export）与 Cargo 里 `homie-engine` 依赖。
- `homie-engine` Cargo 新增 `homie-gateway`、`tokio`、`axum`、`reqwest`、`rusqlite` 依赖。

### Phase 2：daemon 内嵌 LLM proxy listener

- daemon 起 tokio runtime 线程 `axum::serve(router)`，绑 `gateway.listen`。
- daemon 构造并持有 `AppState`（Db/Upstream/master_key/models/policy）。
- 端口占用语义区分 control socket 与 LLM 端口。

### Phase 3：协议收敛 OpenAI-only

- 删 `/v1/messages`、`handle_messages`、`route_key` claude 分支、`models` claude 键。
- 删 `claude_gateway_env`、`InjectionSpec.claude_gateway`、`gateway_env` claude 分支。

### Phase 4：virtual key 签发内聚 + 删 admin HTTP 面

- daemon spawn 时 `GatewayApiKeyStore.create` 签发并注入；`gateway: None` 移除。
- 删 `/admin/keys` HTTP 面（保留 master key + 内嵌 CLI 签发或明确删除）。

### Phase 5：文档 + 验证

- 更新 `specs/llm-gateway.md`、README（进程表/模块图/图 5）。
- 单测 + 集成 + 端到端 + 安全，证据入 `docs/verification/llm-gateway-daemon-embed/`。
- tag（minor） + 关 Beads。

## 依赖

- 依赖 `homie-gateway` 库现成模块，不搬代码。
- 与 `mcp-http-transport-unified`（homie-gyj）并行、互不阻塞（不同 transport 层）。

## 回退

- 若 daemon host tokio 的线程安全/阻塞问题复杂，可暂保留 gateway 独立进程、仅做协议收敛
  （删 Anthropic），内嵌延后——但优先按内嵌推进。
