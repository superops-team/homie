# Code Review — llm-gateway-daemon-embed (Round 1)

- Beads: `homie-6md` · change_id: `llm-gateway-daemon-embed`

## 1. 审查范围

- `homie/crates/homie-gateway/`：删 `main.rs`/`inject.rs`、`[[bin]]`、`homie-engine` 依赖；routes/auth/config 协议收敛。
- `homie/crates/homie-engine/`：`inject.rs` 新增 `GatewayIssuer`；`control.rs`/`control/handlers.rs` 签发归属 daemon；
  `bin/homied-rs.rs` 内嵌 axum listener。
- `homie/Cargo.toml`：新增 `axum` workspace 依赖。
- Swift CLI + specs + README 收敛。

## 2. 发现与结论

| # | 项 | 结论 |
|---|----|------|
| 1 | 循环依赖打破（gateway 不再依赖 engine） | ✅ `homie-engine` 依赖 `homie-gateway`，方向单一 |
| 2 | daemon 内嵌 tokio 线程 | ✅ 独立命名线程 + `block_on`，失败降级 None，不阻塞主循环 |
| 3 | 协议收敛彻底（无 claude 分支残留） | ✅ grep 无 `claude_gateway`/`/v1/messages`/`/admin/keys`/`handle_messages` |
| 4 | virtual key 签发不泄漏 | ✅ `GatewayIssuer` 手动 `Debug`（避免打印 SQLite 连接），原始 key 仅返回一次 |
| 5 | mint 失败不中断 spawn | ✅ 仅 eprintln，符合「LLM proxy 禁用不阻断编排」语义 |
| 6 | Claude 退出 LLM 纳管但保留编排 | ✅ 仅删 gateway env 注入，`--settings`/`--mcp-config`/hooks 不变 |

## 3. 遗留（非阻塞）

- `homie-engine` 4 条既有 clippy warning（`collapsible_if` 等），独立于本变更。
- 共享 target 目录下存在旧的 `homie-gateway` 二进制残留（构建产物，gitignored，非本次产出）。
