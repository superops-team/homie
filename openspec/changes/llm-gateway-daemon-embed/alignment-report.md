# OpenSpec 对齐报告

change_id: `llm-gateway-daemon-embed` · Beads: `homie-6md`

## PRD → Spec → OpenSpec 映射

| PRD 需求 | Spec 契约 | OpenSpec 任务 |
|---|---|---|
| FR-1 gateway 降级为库 | `specs/llm-gateway.md` §2 | T1.1–T1.3 |
| FR-2 daemon 内嵌 listener | §2、§10 | T2.1–T2.3 |
| FR-3 协议收敛 OpenAI-only | §5、§7 | T3.1–T3.3 |
| FR-4 virtual key 签发内聚 | §3、§4 | T4.1–T4.2 |
| FR-5 文档与 Spec 收敛 | §2/§3/§5/§7/§11 | T5.1 |

## 对齐结论

- 每条 PRD 功能需求（FR-1~FR-5）均有对应 Spec 契约与可执行 OpenSpec 任务。
- 虚拟 key 模型、用量、策略/配额、模型路由、凭证解析的语义保持不变，仅删 Anthropic 分支与
  变更签发归属，无能力回退风险。
- 与 `mcp-http-transport-unified`（homie-gyj）并行推进，transport 层独立，不冲突。
