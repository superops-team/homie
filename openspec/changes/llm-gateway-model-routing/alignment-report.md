# OpenSpec Alignment Report — llm-gateway-model-routing

## 1. 目的

证明 PRD 功能需求（FR）、组件 spec（`specs/llm-gateway.md` §7）、OpenSpec tasks 一一对应，
无孤儿需求/任务。

## 2. PRD FR → OpenSpec Task 映射

| PRD 需求 | spec 合同 | OpenSpec Task |
|----------|-----------|---------------|
| FR-1 运行时加载 models | §7 Model Routing（`models` map 合同） | T1、T2 |
| FR-2 按路径改写 model | §7（route key + 覆盖语义） | T3 |
| FR-3 用量用改写后 model | §7（usage 用改写后 model） | T4 |
| FR-4 安全边界 | §9 Security And Recovery | T3、T6 |

## 3. 验收标准映射

| 验收标准 | 覆盖 Task | 覆盖验证 Case |
|----------|-----------|---------------|
| models 加载生效 | T1、T2 | FC-1 |
| /responses 按 codex、/messages 按 claude 改写 | T3 | FC-2 |
| 未配置透传 | T3 | FC-3 |
| 用量为改写后 model | T4 | FC-4 |
| 无新增泄露面 | T3、T6 | FC-6 |

## 4. 无漂移声明

- 每条 FR 均有对应 spec 合同与 task，无孤儿需求。
- 每条 task 可回溯到 FR 或验收标准，无孤儿任务。
- 本变更不改变 §5/§6 的协议/透传语义，仅新增 §7 Model Routing 合同；§8/§9 为序号后移。
