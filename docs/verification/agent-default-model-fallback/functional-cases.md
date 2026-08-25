# Functional Cases — agent-default-model-fallback

- Beads: `homie-qzv`
- change_id: `agent-default-model-fallback`
- 日期: 2026-08-25

## FC-1 文档链路

命令：

```bash
test -s prd-spec/bugfixes/agent-default-model-fallback/2026-08-25-agent-default-model-fallback-design.md
test -s openspec/changes/agent-default-model-fallback/plan.md
test -s openspec/changes/agent-default-model-fallback/tasks.md
test -s openspec/changes/agent-default-model-fallback/alignment-report.md
test -s docs/verification/agent-default-model-fallback/spec-review-report.md
```

通过标准：所有文件存在，且包含 `homie-qzv` 与 `agent-default-model-fallback`。

## FC-2 GatewayConfig 过滤空白模型

命令：

```bash
cargo test --manifest-path homie/Cargo.toml -p homie-gateway --offline blank_model_overrides_are_ignored_at_config_load
```

通过标准：`models.codex = ""` 和仅空白字符串在运行时配置中被移除。

## FC-3 路由层空白模型透传

命令：

```bash
cargo test --manifest-path homie/Cargo.toml -p homie-gateway --offline apply_model_route_passes_through_blank_targets
```

通过标准：空白 override 不会把请求体 `model` 改为空字符串，原 body 透传。

## FC-4 网关集成透传 agent 默认模型

命令：

```bash
cargo test --manifest-path homie/Cargo.toml -p homie-gateway --offline blank_configured_model_passes_agent_model_through
```

通过标准：`models.codex = ""` 时，经 `/v1/responses` 后 usage 记录仍是 agent 请求中的原始 model。

## FC-5 Swift CLI 默认配置

命令：

```bash
swift build
HOMIE_CONFIG=<tmp> .build/debug/homie config set --base-url https://api.example.com/v1 --model-codex ' gpt-5.2-codex '
HOMIE_CONFIG=<tmp> .build/debug/homie config set --model-codex '   '
swift test --filter ConfigOpsTests
```

通过标准：Swift 源码可编译；真实 CLI 在临时配置文件中写入非空模型、删除空白模型 override；若
本机 Swift Testing runtime 不可用，记录 `swift test` 的实际失败原因。

## FC-6 最终门禁

命令：

```bash
cargo fmt --manifest-path homie/Cargo.toml --all --check
cargo test --manifest-path homie/Cargo.toml -p homie-gateway --offline
cargo test --manifest-path homie/Cargo.toml -p homie-engine --lib --offline inject
swift test --filter ConfigOpsTests
git diff --check
```

通过标准：命令均通过；若存在环境限制或既有失败，记录实际错误和影响范围。
