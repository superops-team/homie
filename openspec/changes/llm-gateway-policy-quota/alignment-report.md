# OpenSpec Alignment Report — llm-gateway-policy-quota

## 1. 目的

证明 PRD 功能需求（FR）、组件 spec（`specs/llm-gateway.md` §9）、OpenSpec tasks 一一对应，
无孤儿需求/任务。

## 2. PRD FR → OpenSpec Task 映射

| PRD 需求 | spec 合同 | OpenSpec Task |
|----------|-----------|---------------|
| FR-1 策略配置加载 | §9（`policy` 可选、0 值视为未配置） | T1、T2 |
| FR-2 每 key 速率限制 | §9（内存分钟窗口） | T3 |
| FR-3 每 key 配额 | §9（SQLite 聚合） | T4 |
| FR-4 429 结构化拒绝 | §9（429 + 不写 gateway_usage） | T5 |
| FR-5 审计事件 | §9（gateway_audit 拒绝事件） | T5 |
| FR-6 安全边界 | §10 Security And Recovery | T5、T7 |

## 3. 验收标准映射

| 验收标准 | 覆盖 Task | 覆盖验证 Case |
|----------|-----------|---------------|
| policy 可选、缺失不限制 | T1、T2 | FC-1 |
| 超 requests_per_minute 返回 429 | T3 | FC-2 |
| 超 daily_token_limit 返回 429 | T4 | FC-3 |
| 拒绝事件落库、放行用量照旧 | T5 | FC-4 |
| 无密钥/敏感 prompt 泄露 | T5、T7 | FC-6 |

## 4. 无漂移声明

- 每条 FR 均有对应 spec 合同与 task，无孤儿需求。
- 每条 task 可回溯到 FR 或验收标准，无孤儿任务。
- 本变更不改变 §3/§4/§7/§8 既有语义，仅新增 §9 Policy And Quota；§10 序号后移。
