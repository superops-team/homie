# OpenSpec Alignment Report — agent-default-model-fallback

change_id: `agent-default-model-fallback` · Beads: `homie-qzv`

## 1. PRD → Spec → OpenSpec 对齐

| PRD 需求 | 长存 spec 合同 | OpenSpec 任务 | 验证 |
|----------|----------------|---------------|------|
| FR-1 空白模型等同未配置 | `specs/llm-gateway.md` §7 Model Routing | T2 | FC-2、FC-3、FC-4 |
| FR-2 New Agent 默认自配置启动 | `specs/llm-gateway.md` §7；`specs/homie-cli-config-ops.md` §3/§4 | T2、T4 | FC-3、FC-5 |
| FR-3 CLI 不写空模型占位 | `specs/homie-cli-config-ops.md` §3/§4.2 | T4 | FC-5 |
| FR-4 非空模型路由保持 | `specs/llm-gateway.md` §7 | T2、T3 | 既有模型改写测试 + FC-6 |
| FR-5 安全边界不变 | `specs/llm-gateway.md` §10 | T2、T4、T5 | review + diff scan |

## 2. 关键一致性结论

- `models` 是可选覆盖，不是 agent 启动的必填项。
- 空白字符串没有有效模型语义，必须与缺失字段一样处理。
- spawn-time injection 继续只注入 provider/proxy/MCP 所需参数，不注入 `model`。
- 历史空配置由 Rust 加载与路由双重防御兜住；新的 CLI 默认配置不再产生空占位。

## 3. 验证映射

| Case | 覆盖点 | 证据 |
|------|--------|------|
| FC-1 | 文档链路存在且关键语义可检索 | `functional-cases.md` |
| FC-2 | `GatewayConfig` 过滤空白模型 | Rust 单测 |
| FC-3 | `apply_model_route` 空白模型透传 | Rust 单测 |
| FC-4 | Gateway 集成：空白模型不写入 usage | Rust integration test |
| FC-5 | Swift CLI 默认配置不写空模型 | Swift test |
| FC-6 | 非空模型改写不回归 + 格式/差异门禁 | 既有测试 + final gates |

## 4. 非目标确认

- 不扩展 agent manifest。
- 不引入模型选择 UI。
- 不改变 credential source 或虚拟 key 签发。
- 不改变 gateway listener 是否启动的凭证要求。
