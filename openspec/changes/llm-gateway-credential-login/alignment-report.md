# OpenSpec Alignment Report — llm-gateway-credential-login

## 1. 目的

证明 PRD FR、组件 spec（`specs/llm-gateway.md` §11 Credential Source）、OpenSpec tasks 一一对应。

## 2. PRD FR → Spec → Task 映射

| PRD 需求 | spec 合同 | OpenSpec Task |
|----------|-----------|---------------|
| FR-1 node `credential.resolve` 受限方法 | §11（node 受限解析、不返 refresh token、不暴露任意读） | T1、T2、T3 |
| FR-2 网关 `credential_source` 可选配置 | §11（`#[serde(default)]`=static） | T4 |
| FR-3 上游凭证解析与回退 | §11（node 优先、静态回退、503 无泄露） | T5 |
| FR-4 安全与审计 | §11（token 仅内存、audit 去 token） | T5、T6 |

## 3. 验收标准映射

| 验收标准 | 覆盖 Task | 验证 Case |
|----------|-----------|-----------|
| Codex API-key 登录后 node 模式端到端转发 | T2、T3、T5 | FC-1 |
| 默认 static 模式行为不变 | T4、T5 | FC-3 |
| 无凭证时明确报错且不泄露 | T5、T6 | FC-4、FC-5 |

## 4. 无漂移声明

- 每条 FR 均有对应 spec 合同与 task，无孤儿需求。
- 每条 task 可回溯到 FR 或验收标准，无孤儿任务。
- 本变更仅新增 `specs/llm-gateway.md` §11 Credential Source；§1–§10 既有语义不变。
