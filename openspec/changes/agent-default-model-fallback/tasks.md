# OpenSpec Tasks — agent-default-model-fallback

## T1: 锁定问题与合同

- 交付：PRD、spec review、OpenSpec plan/tasks/alignment。
- 验收：文档均引用 `homie-qzv` 与 `agent-default-model-fallback`，并明确空白模型等同未配置。
- 关联验证：FC-1。

## T2: Rust 网关空白模型防御

- 交付：
  - `GatewayConfig::from_file` 过滤空白 `models` 条目；
  - `apply_model_route` 对空白 target 透传。
- 验收：
  - 空字符串、空白字符串不会覆盖请求体 model；
  - 非空模型覆盖保持不变。
- 关联验证：FC-2、FC-3。

## T3: Gateway 集成回归

- 交付：`homie-gateway/tests/gateway.rs` 增加空 `models.codex` 透传测试。
- 验收：请求经 `/v1/responses` 后，usage 记录仍是原始 agent model。
- 关联验证：FC-4。

## T4: Swift CLI 默认配置修正

- 交付：
  - `HomieConfigStore.empty.models` 改为空字典；
  - `ConfigSet` 对空白 `--model-codex` 删除 override。
- 验收：首次创建配置不写空 `models.codex`；显式非空仍写入。
- 关联验证：FC-5。

## T5: 验证、review、关闭

- 交付：功能验证报告、两轮 code review、release readiness。
- 验收：Rust/Swift 相关测试通过，`git diff --check` 通过，Beads 状态可关闭。
- 关联验证：FC-6。
