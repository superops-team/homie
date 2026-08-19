# Functional Verification Report — llm-gateway-daemon-embed

- Beads: `homie-6md` · change_id: `llm-gateway-daemon-embed` · 日期: 2026-08-19

## 1. 门禁结果

| 门禁 | 命令 | 结果 |
|------|------|------|
| 格式 | `cargo fmt --all --check` | ✅ 通过（0 diff） |
| gateway 单测 | `cargo test -p homie-gateway --offline` | ✅ 31 lib + 10 integration + 0 doc 全部通过 |
| engine 单测 | `cargo test -p homie-engine --lib --offline` | ✅ 299 passed, 0 failed, 3 ignored |
| gateway clippy | `cargo clippy -p homie-gateway --all-targets --offline` | ✅ 0 warning |
| engine clippy | `cargo clippy -p homie-engine --all-targets --offline` | ✅ 4 条既有 warning（非本变更引入） |
| 构建 | `cargo build --offline` | ✅ 成功，无新 `homie-gateway` bin |
| Swift CLI | `swift build` | ✅ 成功 |

> `homie-engine` 的 4 条 clippy warning（`collapsible_if` 等）为既有问题，不属于本变更范围。

## 2. 单元测试（gateway lib，31 通过）

- `route_key`：仅 `/responses → codex`，`/messages → None`（协议收敛）。
- auth/config/policy/upstream/usage/state 各模块既有测试全部保留通过。

## 3. 集成测试（tests/gateway.rs，10 通过）

- `messages_route_is_gone`：`POST /v1/messages` → 404。
- `Harness::mint_key`：in-process store 签发（替代原 `/admin/keys`）。
- `revoked_key_returns_unauthorized`：store.delete 后 401。
- 其余（usage、policy、master key bypass、model rewrite、rate/quota）保留通过。

## 4. 验收标准核对（PRD）

1. ✅ gateway 降级为库：`main.rs`/`[[bin]]`/`inject.rs` 已删，lib 保留九模块。
2. ✅ daemon 内嵌 listener：`homied-rs.rs::start_gateway()` 构造 `GatewayIssuer` + tokio runtime 线程 `axum::serve`。
3. ✅ 协议收敛：`/v1/messages`、`handle_messages`、`route_key` claude 分支、`claude_gateway_env`、`InjectionSpec.claude_gateway` 全部删除。
4. ✅ virtual key 内聚：`GatewayIssuer::mint` 签发；`/admin/keys`、`require_master` 已删。
5. ✅ 文档收敛：`specs/llm-gateway.md`、README、`specs/homie-cli-config-ops.md` 已更新，Swift CLI 已去掉对已删二进制的依赖。

## 5. 端到端（人工核对）

- Codex 经 daemon 内嵌 proxy 转发（`/v1/responses`）；Claude 不注入 `ANTHROPIC_*` env，回归原生凭证。
- `homie config agent` 子命令已删除（其依赖的 `homie-gateway inject` 二进制已不存在）。
