# Failure Model — PTY Lifecycle (Tier 3)

- change_id: `engine-session-runtime-split`

本次为纯职责搬迁，PTY 进程生命周期语义不变，但按 Tier 3 要求补充失败模型，覆盖拆分与
resume spec 下沉可能引入的边界。

## 失败模式

| # | 模式 | 触发 | 影响 | 缓解/证据 |
|---|------|------|------|-----------|
| M1 | 进程残留 | spawn/adopt 时 child 已启动但 holder 未记录 marker | 孤儿 PTY 进程 | `wait_for_holder` 250×20ms 轮询 + log 证据兜底（`lifecycle.rs`，未改动） |
| M2 | 部分启动 | resume 时 agent 二进制缺失 / 不支持 resume | resume 失败，会话保持 exited | `resume_spec`/`remote_resume_spec` 在 `binary`/`resume_args` 缺失时返回 `bad_request`，不产生半启动会话 |
| M3 | 恢复竞争 | 同一 id 并发 resume / agent 已死但 registry 仍存在 | 交回尸体或重复启动 | `session_resume` 先判 liveness，exited 走 evict→respawn 路径（未改动） |
| M4 | resume 注入丢失 | 下沉后 hook/MCP 注入丢失，Claude 失去状态检测/homie 工具 | 静默降级 | `resume_spec` 的 `claude_only` InjectionSpec 与 `SESSION_ID_ENV`/`SOCKET_ENV`/`CLI_ENV`/`mcp_env` 逐项保留，F1 锚证 argv 到达 |

## 结论

拆分未新增并发/凭据/数据丢失边界；PTY 生命周期相关搬迁保持原有失败处理路径不变，
由既有 303 测试覆盖。无新增 Tier 3 风险面。
