# Release Readiness Report — llm-gateway-model-routing

- Beads: `homie-48w`
- change_id: `llm-gateway-model-routing`
- 日期: 2026-08-18

## 1. 交付内容

| 组件 | 路径 | 状态 |
|------|------|------|
| `GatewayConfig` 加载 `models` | `homie/crates/homie-gateway/src/config.rs` | 完成 |
| `AppState` 携带 `models` | `homie/crates/homie-gateway/src/state.rs` | 完成 |
| `main.rs` 传入 `models` | `homie/crates/homie-gateway/src/main.rs` | 完成 |
| 按路径改写 + 用量改写后 model | `homie/crates/homie-gateway/src/routes.rs` | 完成 |
| 组件合同 §7 Model Routing | `specs/llm-gateway.md` | 完成 |
| 集成测试（模型改写/透传） | `homie/crates/homie-gateway/tests/gateway.rs` | 完成 |

## 2. 门禁结果

| 门禁 | 命令 | 结果 |
|------|------|------|
| 格式 | `cargo fmt --all --check` | ✅ 通过 |
| 网关单测 | `cargo test -p homie-gateway --offline` | ✅ 20 lib + 10 integration + 0 doc 全部通过 |
| 网关 clippy | `cargo clippy -p homie-gateway --all-targets --offline` | ✅ 干净（0 warning） |

> `homie-engine` 的 4 条 clippy warning（`while_let_loop` / `collapsible_if`）为既有问题，
> 不属于本变更范围，未在本 PRD 中引入或修复。

## 3. 功能验证

### 3.1 单元测试（lib，20 通过）

- `route_key_maps_paths_to_agents`：`/responses`→`codex`、`/messages`→`claude`、其他→`None`。
- `apply_model_route_rewrites_when_configured`：配置存在时改写顶层 `model`。
- `apply_model_route_passes_through_when_key_missing`：缺映射透传。
- `apply_model_route_passes_through_non_json` / `_non_string_model`：非 JSON、非字符串透传。
- `retains_models_map`：`from_file` 正确反序列化 `models`，camelCase 对齐。
- `extract_model_from_body`：改写后 body 提取 model，缺失/非 JSON 回退 `unknown`。

### 3.2 集成测试（tests/gateway.rs，10 通过，新增 3）

- `codex_model_is_rewritten_before_forward_and_recorded`：配置 `models.codex` 后
  `/v1/responses` 请求，用量行 `model` 为改写后的 `gpt-5.2-codex`（非客户端传入的 `gpt-5`）。
- `claude_model_is_rewritten_before_forward_and_recorded`：配置 `models.claude` 后
  `/v1/messages` 请求，用量行 `model` 为改写后的 `claude-sonnet-4`。
- `unconfigured_model_passes_through_unchanged`：未配置时用量行 `model` 保持客户端原值
  `gpt-5`。

## 4. 验收标准核对（PRD §8）

1. ✅ `GatewayConfig` 加载 `models`，`homie.local.json` 的 `models.codex/claude` 生效
   （`retains_models_map` + `from_file` 填充）。
2. ✅ `/v1/responses` 按 `models.codex` 改写；`/v1/messages` 按 `models.claude` 改写
   （集成测试 3.2）。
3. ✅ 未配置时透传，向后兼容（`passes_through_*` + `unconfigured_...`）。
4. ✅ 用量记录为改写后 model（`forward_and_record` 取改写后 body 的 model）。
5. ✅ 无新增 key/敏感信息泄露面（改写仅触碰 `model` 字段）。

## 5. 安全验证

- 改写仅针对 JSON body 顶层 `model` 字符串，不读取/回显 `api_key`、master key、虚拟 key、
  敏感 prompt。
- 非 JSON body、缺失/非字符串 `model` 均透传且不报错，无新增错误日志泄露面。

## 6. 结论

所有验收标准满足，门禁全绿，证据齐备。可发布并打 tag `v0.4.0`。
