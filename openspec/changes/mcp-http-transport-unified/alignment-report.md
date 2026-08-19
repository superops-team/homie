# OpenSpec 对齐报告

change_id: `mcp-http-transport-unified` · Beads: `homie-gyj`

## PRD → Spec → OpenSpec 映射

| PRD 需求 | Spec 契约 | OpenSpec 任务 |
|---|---|---|
| FR-1 daemon 内嵌 endpoint | `specs/mcp-transport.md` §1 端点、§5 方法 | T1.1–T1.3、T1.7 |
| FR-2 工具收敛到 Rust | §4 工具清单、§7 服务端边界 | T1.4–T1.6 |
| FR-3 认证与会话身份 | §2 认证、§3 会话身份 | T1.5、T1.7 |
| FR-4 注入改造（HTTP） | §1 端点、§2 认证 | T2.1–T2.3 |
| FR-5 删除 stdio 资产 | §7 服务端边界 | T3.1–T3.3 |
| FR-6 安全 | §2、§6 错误码 | T4.4 |

## 对齐结论

- 每条 PRD 功能需求（FR-1~FR-6）均有对应 Spec 契约与可执行 OpenSpec 任务。
- 工具语义不变，仅 transport 与服务端语言迁移，无未覆盖的能力回退风险（以 Swift `HomieMCPTools.all`
  为金本对拍）。
- 开放风险（Claude headers 展开）在 Phase 0 显式核实，不阻塞后续任务拆分。
