# OpenSpec Tasks — llm-gateway-model-routing

## T1: GatewayConfig 加载 models 映射

- 交付：`GatewayConfig` 增 `models: BTreeMap<String, String>`；`FileConfig.models` 去
  `#[allow(dead_code)]`；`from_file` 填充。
- 验收：`homie.local.json` 的 `models` 反序列化；缺失为空；camelCase 对齐。
- 关联验证 Case：FC-1。

## T2: AppState 携带 models

- 交付：`state.rs` `AppState` 增 `models` 字段；`main.rs` 传入。
- 验收：编译通过，`AppState::new` 签名更新且调用点一致。
- 关联验证 Case：FC-1。

## T3: 按路径改写转发 model

- 交付：`routes.rs` 增 `route_key(path)` 与 `apply_model_route(models, key, body)`；
  `forward_and_record` 在 `upstream.forward` 前改写 body。
- 验收：`/v1/responses` 用 `models.codex`、`/v1/messages` 用 `models.claude`；未配置/非 JSON/
  非字符串 model 透传。
- 关联验证 Case：FC-2、FC-3。

## T4: 用量记录使用改写后 model

- 交付：`forward_and_record` 的 `model` 取改写后值。
- 验收：用量 model 反映实际路由模型。
- 关联验证 Case：FC-4。

## T5: 单测 + 集成测试

- 交付：`route_key`/`apply_model_route`/`from_file` 单测；集成测试断言 mock upstream 收到的
  body model 被改写。
- 验收：`cargo test -p homie-gateway` 全绿。
- 关联验证 Case：FC-5。

## T6: 门禁 + 证据 + 关闭

- 交付：`cargo fmt`/`test`/`clippy` 绿；spec review + release readiness 证据；Beads
  `homie-48w` 关闭。
- 关联验证 Case：FC-6。
