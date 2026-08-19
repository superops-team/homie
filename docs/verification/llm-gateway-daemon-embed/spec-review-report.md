# Spec Review Report — llm-gateway-daemon-embed

- Beads: `homie-6md`
- change_id: `llm-gateway-daemon-embed`
- 日期: 2026-08-19

## 1. 范围

评审 `homie-gateway` 从独立进程降级为库并内嵌 daemon、协议收敛到 OpenAI Responses、
virtual key 签发内聚 daemon、Claude Code 退出 LLM 纳管（保留 hooks + MCP 编排）。

## 2. 技术选型评审（可行性）

| 决策 | 结论 | 依据 |
|------|------|------|
| gateway 降级为库、daemon 内嵌 axum listener | **采纳** | 打破 `homie-engine ⇄ homie-gateway` 循环依赖；单守护进程减少进程/通信/安装维护开销 |
| tokio runtime 放独立命名线程（1 worker） | **采纳** | daemon 主循环非 tokio；用 `std::thread::Builder` + `block_on` 隔离，bind/runtime 失败返回 None 不影响主循环 |
| 协议收敛 OpenAI-only（删 `/v1/messages`） | **采纳** | Claude Code 回归原生 Anthropic 凭证，不再经网关；保留 hooks + MCP 编排 |
| virtual key 由 daemon spawn 内嵌签发（删 `/admin/keys` HTTP 面） | **采纳** | 签发归属 daemon，减少暴露面；master key 仍由受信 CLI/doctor 使用 |
| Claude 仅保留 `--settings`/`--mcp-config`/hooks | **采纳** | 流量/配额/统一凭证退出，编排能力不变 |

## 3. 依赖评估

| 依赖 | 状态 | 理由 |
|------|------|------|
| `axum` | workspace 新增（0.8, json） | daemon 内嵌 listener |
| `tokio` | workspace 已有 | runtime 线程 |
| `homie-gateway`（lib） | engine 新增依赖 | 复用 routes/auth/config/policy/state 模块，不搬代码 |
| 删除 `homie-engine` ← `homie-gateway` 依赖 | 已移除 | 打破循环依赖 |
| `reqwest`/`rusqlite` | 已有 | 上游转发 / 用量与虚拟 key SQLite |

## 4. 组件合同评审

`specs/llm-gateway.md` 更新：§2（内嵌 daemon、无独立进程）、§3（签发内嵌、无 `/admin/keys`）、
§5/§7（删 Anthropic Messages / claude 路由）、§11（凭证源改为 daemon 内嵌）。评审结论：

- 删 Anthropic 分支后 §2/§5/§7 与 PRD FR-3 对齐，无缺口。
- §3 virtual key 模型语义不变（`sk-<uuid><uuid>`、SHA-256 落库、401 撤销），仅签发归属改 daemon。
- 用量、策略/配额、凭证解析语义不变，仅删 Anthropic 分支，无能力回退。

## 5. 边界情况核对

- gateway 配置缺失/DB 打开失败/bind 冲突/runtime 失败 → `start_gateway()` 返回 None，daemon 仍启动（LLM proxy 禁用）。
- mint 失败仅 eprintln，不中断 agent spawn。
- `/v1/messages` 返回 404（`messages_route_is_gone` 集成测试覆盖）。
- 撤销 key 后 401 不转发（`revoked_key_returns_unauthorized` 覆盖）。

## 6. 结论

评审通过，与 PRD FR-1~FR-5 及 OpenSpec 任务 T1~T5 对齐。
