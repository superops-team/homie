# OpenSpec Alignment Report — llm-gateway-virtual-keys

## 1. 目的

证明 PRD 功能需求（FR）、组件 spec、OpenSpec tasks 三者一一对应，无「有需求无任务」或
「有任务无需求」的漂移。

## 2. PRD FR → OpenSpec Task 映射

| PRD 需求 | spec 合同（specs/llm-gateway.md） | OpenSpec Task |
|----------|----------------------------------|---------------|
| FR-1 本地 HTTP 网关 | §2 Authority、§5 Protocol Contract | T1、T2、T6 |
| FR-2 虚拟 key 签发与鉴权 | §3 Virtual Key Model、§4 Auth Contract | T3、T4 |
| FR-3 上游 OpenAI-compatible 转发 | §6 Upstream Forwarding | T5 |
| FR-4 agent 配置自动注入 | §2 Authority（agent 指向网关） | T8 |
| FR-5 用量记录 | §7 Usage Contract | T7 |
| FR-6 安全边界 | §8 Security And Recovery | T2、T4、T5、T10 |

## 3. 验收标准映射

| 验收标准 | 覆盖 Task | 覆盖验证 Case |
|----------|-----------|---------------|
| 网关可编译启动、绑回环 | T1、T2 | FC-1、FC-2 |
| 虚拟 key 签发/校验/撤销/持久化、401 | T3、T4 | FC-3、FC-4 |
| /v1/responses + /v1/messages 转发、流式 | T5、T6 | FC-5、FC-6 |
| Codex/Claude spawn 自动注入 | T8 | FC-8 |
| 用量按虚拟 key 落库 | T7 | FC-7 |
| 上游 key 不进 git/log/agent 可见 | T5、T10 | FC-5、FC-10 |
| OpenSpec 对齐 + Beads 关闭 | T10 | FC-10 |

## 4. 无漂移声明

- 每条 PRD FR 均有对应 spec 合同与 task，无孤儿需求。
- 每条 task 均可回溯到 FR 或验收标准，无孤儿任务。
- child Bead（provider-expansion / model-routing / policy-quota / credential-login）在本变更
  中仅声明，不计入本 change 的 task 范围，避免 scope 漂移。
