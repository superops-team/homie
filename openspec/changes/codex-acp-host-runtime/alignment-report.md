# OpenSpec Alignment Report — codex-acp-host-runtime

## 1. 对齐结论

本变更的 PRD 需求项与 OpenSpec tasks、功能验证 Case 已逐项对齐，零漏项、零错配。

## 2. PRD 需求项 → Tasks → 验证 Case 映射

| PRD 需求项 | 内容 | Task | 验证 Case |
|-----------|------|------|-----------|
| FR-1 | ACP JSON-RPC 2.0 协议 DTO | T1 | FC-1 |
| FR-2 | newline-delimited framing | T2 | FC-2 |
| FR-3 | AcpHost 循环 + request/response 关联 + reader 线程 | T3 | FC-3 |
| FR-4 | AcpDriver（AgentDriverControl）能力由 initialize 填充 | T5 | FC-5 |
| FR-5 | approval 四态记忆 | T4 | FC-4 |
| FR-6 | 安全边界（不记录 secret/敏感 payload） | T1/T3 | FC-1/FC-3 |
| 验收 §8.2 | fake ACP server 集成测试端到端 | T6 | FC-6 |
| 验收 §8.6 | OpenSpec alignment + Beads 关闭 | T7 | FC-7 |

## 3. 非目标对齐确认

- 不引入 `codex-acp` crate 作为库：已在 PRD §1.3 与 plan §1 明确，tasks 无依赖引入项。
- 不实现 `fs/*` 文件代理、不接 GPUI canvas、不接 `session.spawn`：均在本 PRD §1.3 与
  plan §5 明确为后续 child Bead，tasks 无对应实现项。
- 不支持 `session/load` 恢复、rollback/fork、认证登录：protocol 仅预留 DTO，无语义实现。

## 4. 设计偏差说明（实现 vs PRD 原文）

PRD §4.2/§4.3/§4.4 描述的数据模型与实现存在**等价但更精简**的落地差异，语义不变：

| PRD 原文 | 实现 | 说明 |
|---------|------|------|
| `Transport` trait + `StdioTransport` | `AcpStream` + `AcpClient` trait | 抽象边界等价：transport 由 `from_stream` 注入，`spawn` 包装子进程 stdin/stdout |
| `AtomicU64` 递增 id | `AtomicI64` 递增 id | JSON-RPC id 为 `i64`，语义等价 |
| fake server 用 `current_exe` + `--acp-fake-server` | 同方案落地 | `harness=false` 集成测试 spawn 自身二进制 |
| `AcpDriver { host: AcpHost }` | `AcpDriver { client: Arc<dyn AcpClient> }` | 引入 trait 抽象便于单测注入 mock，能力更强 |

以上差异均为实现内聚/可测试性优化，不改变对外契约或需求语义。

## 5. 不一致项

无。

## 6. 结论

对齐 100%，可进入证据产出与关闭流程。
