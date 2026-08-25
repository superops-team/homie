# Code Review Round 2 — agent-default-model-fallback

- Beads: `homie-qzv`
- change_id: `agent-default-model-fallback`
- 日期: 2026-08-25

## 1. 隐性问题复盘

| 维度 | 结论 | 说明 |
|------|------|------|
| 语义偏差 | 通过 | 空白模型被统一解释为“无 override”，符合用户要求和 `specs/llm-gateway.md` |
| 历史配置 | 通过 | Rust `GatewayConfig` 加载时过滤空白值，旧配置不会继续污染运行时 |
| 非配置加载路径 | 通过 | `apply_model_route` 自身也防御空白 target，测试可捕获直接构造 `AppState` 的情况 |
| Agent 启动注入 | 通过 | `codex_gateway_args_do_not_override_the_agent_model` 锁定 spawn 注入不设置 `model` |
| Swift CLI 行为 | 通过 | 默认空配置为 `{}`；空白 `--model-codex` 删除 override；CLI 黑盒验证通过 |
| 安全 | 通过 | 未新增 secret 输出、未触碰 virtual key 生成/持久化、未改变 credential source |
| 用户已有改动 | 通过 | 未修改既有 `homied-rs.rs` / `mcp/http.rs` listener 变更，只保留其现状 |

## 2. 已知限制

- Swift 测试 target 在当前 Command Line Tools 环境下无法 import `Testing`，因此 Swift 单元测试未执行。
  该问题影响既有多个 test target，并非本次变更新引入。已用 `swift build` 与真实 CLI 临时配置行为测试补充覆盖。
- 未启动真实 GPUI app。本次变更没有 UI 渲染改动；New Agent 相关结论来自 store/spawn 路径代码审查、
  engine 注入回归测试和 gateway 行为测试。

## 3. 结论

未发现新的 P0/P1 问题。当前实现可以进入 release readiness。
