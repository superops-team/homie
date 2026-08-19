# Tasks: LLM proxy 内嵌 daemon + 协议收敛

## Phase 1 — 打破循环依赖
- [x] T1.1 删 `homie-gateway/src/main.rs` + `[[bin]]`，保留 lib。
- [x] T1.2 删 `homie-gateway/src/inject.rs` 与 Cargo 的 `homie-engine` 依赖。
- [x] T1.3 `homie-engine` Cargo 新增 `homie-gateway`/`tokio`/`axum`/`reqwest`/`rusqlite`。

## Phase 2 — daemon 内嵌 listener
- [x] T2.1 daemon 新增 tokio runtime 线程，`axum::serve(homie_gateway::routes::router(state))`。
- [x] T2.2 daemon 构造并持有 `AppState`（Db/Upstream/master_key/models/policy）。
- [x] T2.3 区分 control socket 与 LLM 端口的占用错误语义。

## Phase 3 — 协议收敛
- [x] T3.1 删 `/v1/messages`、`handle_messages`。
- [x] T3.2 删 `route_key` claude 分支、`models` claude 键。
- [x] T3.3 删 `claude_gateway_env`、`InjectionSpec.claude_gateway`、`gateway_env` claude 分支。

## Phase 4 — 虚拟 key 内聚 + 删 admin 面
- [x] T4.1 daemon spawn 时 `GatewayApiKeyStore.create` 签发并注入，移除 `gateway: None`。
- [x] T4.2 删 `/admin/keys` HTTP 面，明确 master key 归属（debug/doctor CLI）。

## Phase 5 — 文档 + 验证
- [x] T5.1 更新 `specs/llm-gateway.md`、README。
- [x] T5.2 单测：route_key/injection_args/models 无 claude。
- [x] T5.3 集成：daemon 起 proxy，/v1/responses 转发+用量+策略，/v1/messages 404。
- [x] T5.4 端到端：Codex 经内嵌 proxy 转发；Claude 不注入 ANTHROPIC_*。
- [x] T5.5 安全：虚拟 key 不回显/不落日志，无 /admin/keys HTTP 面。
- [x] T5.6 `cargo build` 无 homie-gateway bin，全量测试绿，tag + 关 Beads。
