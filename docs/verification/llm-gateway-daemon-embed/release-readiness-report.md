# Release Readiness Report — llm-gateway-daemon-embed

- Beads: `homie-6md` · change_id: `llm-gateway-daemon-embed` · 日期: 2026-08-19

## 1. 交付内容

| 组件 | 路径 | 状态 |
|------|------|------|
| gateway 降级为库（删 bin/inject） | `homie/crates/homie-gateway/` | 完成 |
| daemon 内嵌 axum listener | `homie/crates/homie-engine/src/bin/homied-rs.rs` | 完成 |
| `GatewayIssuer` 签发内聚 | `homie/crates/homie-engine/src/inject.rs` | 完成 |
| 协议收敛 OpenAI-only | routes/agent/inject 多处 | 完成 |
| 组件合同收敛 | `specs/llm-gateway.md`、`specs/homie-cli-config-ops.md` | 完成 |
| 架构文档收敛 | `README.md` | 完成 |
| Swift CLI 去掉已删二进制依赖 | `Sources/homie-cli/` | 完成 |
| 验证证据 | `docs/verification/llm-gateway-daemon-embed/` | 完成 |

## 2. 门禁结果

| 门禁 | 结果 |
|------|------|
| `cargo fmt --all --check` | ✅ |
| `cargo test -p homie-gateway --offline` | ✅ 31 + 10 |
| `cargo test -p homie-engine --lib --offline` | ✅ 299 |
| `cargo clippy -p homie-gateway --all-targets` | ✅ 0 warning |
| `cargo build --offline` | ✅ 无新 homie-gateway bin |
| `swift build` | ✅ |

## 3. 验收标准

1. ✅ gateway 降级为库，循环依赖打破。
2. ✅ daemon 内嵌 OpenAI-only proxy listener，失败降级不阻断编排。
3. ✅ `/v1/messages` 删除（404），`claude_gateway`/`claude_gateway_env` 删除。
4. ✅ virtual key 由 daemon spawn 内嵌签发，`/admin/keys` 删除。
5. ✅ Claude Code 回归原生 Anthropic 凭证，保留 hooks + MCP 编排。
6. ✅ 文档/spec/README/Swift CLI 全部收敛，测试绿。

## 4. 安全

- 无真实 provider key 进 git；虚拟 key 仅落 SHA-256，原始 key 只返回一次。
- 无 `/admin/keys` HTTP 暴露面；master key 仅受信 CLI/doctor 使用。
- 拒绝/审计响应不含 key/model/prompt。

## 5. 结论

满足发布就绪标准，可 tag（minor）并关闭 Beads `homie-6md`。
