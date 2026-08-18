# Spec Review Report — llm-gateway-model-routing

## 1. 范围

本报告记录 `llm-gateway-model-routing`（Beads `homie-48w`）实现前的 spec 与可行性评审结论，
覆盖 `models` 映射运行时加载、按路径改写转发 `model`、用量记录用改写后模型、以及安全边界。

## 2. 技术选型评审（可行性）

| 决策 | 结论 | 依据 |
|------|------|------|
| 路由键 = 请求路径（`/responses`↔codex、`/messages`↔claude） | **采纳** | PRD1 已定路径区分 agent 协议，无需给虚拟 key 增加 agent 标签 |
| 网关层改写，而非注入层注入 model | **采纳** | 网关是统一入口，改一处对全部 agent 生效；不改 `homie-engine::inject` |
| 覆盖语义 = 配置则改写、未配置透传 | **采纳** | `#[serde(default)]` 保持 MVP 向后兼容，不强制配置 |
| 仅改写 JSON 顶层 `model` 字符串，非 JSON 透传不报错 | **采纳** | 最小改动，不触碰协议语义，不做 Anthropic↔OpenAI 语义转换 |
| 无新增第三方依赖 | **采纳** | 复用 `serde_json`，符合「不新增包」 |

## 3. 依赖评估

| 依赖 | 状态 | 理由 |
|------|------|------|
| `serde_json` | workspace 已有 | body 解析与改写 |
| `std::collections::BTreeMap` | 标准库 | 稳定键序，测试可断言 |
| 上游 `llm-gateway-virtual-keys` | 已✓ | 网关/转发/用量基线 |
| 上游 `homie-cli-config-ops` | 已✓ | 提供 `models` 录入 |
| 无新增包 | — | 符合依赖添加政策 |

## 4. 组件合同评审

`specs/llm-gateway.md` 新增 §7 Model Routing，§8 Usage Contract / §9 Security And Recovery
序号后移。评审结论：

- §7 定义了 `models` map 合同（`#[serde(default)]`、缺失/部分合法）、route key、覆盖语义、
  用量用改写后模型、网关为模型路由单一事实来源。
- §7 与 PRD FR-1/FR-2/FR-3 一一对应，无缺口。
- §5 Protocol Contract / §6 Upstream Forwarding 的既有透传语义未被改动，仅新增 §7。

## 5. 边界情况核对

| 场景 | 处理 | 已验证 |
|------|------|--------|
| `models` 整体缺失 | 空映射，全透传 | `from_file` 缺失测试 |
| 仅配 `codex`、缺 `claude` | codex 改写、claude 透传 | 单测 `passes_through_when_key_missing` |
| body 非 JSON | 透传不报错 | 单测 `passes_through_non_json` |
| `model` 非字符串 | 仅字符串改写，否则透传 | 单测 `passes_through_non_string_model` |
| 网关启动配置损坏 | 沿用 PRD1 `load` 硬失败 | 未改 `load` 语义 |

## 6. 安全 Tier 分级

改写仅触碰 JSON body 顶层 `model` 字段，不涉及 `api_key`/master key/虚拟 key/敏感 prompt，
无新增泄露面。属 Tier 1（非 credential custody，但紧邻代理路径）。要求覆盖改写/透传分支的
单测 + 集成测试（见 release-readiness-report 门禁结果）。

## 7. 与既有权威的一致性

- 不改 `homie-engine::inject`（模型路由由网关统一改写，非注入层）。
- 不改 `specs/llm-gateway.md` §5/§6 的协议/透传语义。
- 不改虚拟 key / 鉴权 / 用量合同的既有语义（§3/§4/§8 仅序号后移）。

## 8. 结论

方案可行，规格齐备，路由键/覆盖语义/安全边界明确，可进入实现阶段。
