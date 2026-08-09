# Spec Review Report

```yaml
change_id: diri-protocol-runtime-wiring
beads: homie-qci
reviewed_prd: prd-spec/features/diri-protocol-runtime-wiring/2026-08-07-diri-protocol-runtime-wiring-design.md
reviewed_openspec:
  - openspec/changes/diri-protocol-runtime-wiring/plan.md
  - openspec/changes/diri-protocol-runtime-wiring/tasks.md
  - openspec/changes/diri-protocol-runtime-wiring/alignment-report.md
status: pass
```

## 1. 总体结论

- 可行性：高。
- 最大风险：把 `API-003` 或 UI 行随 `API-002` 一起误标为完成。
- 推荐方向：限定本轮只完成 `API-002` 的 external subscription transport，并为 `API-003` 增加 runtime-backed CLI session 证据但保持 `partial`。

## 2. 问题清单

| 优先级 | 维度 | 问题 | 影响 | 整改建议 |
|---|---|---|---|---|
| P1 | 范围控制 | `API-003` 的完整范围包含 worktree/ports/MCP bridge，本轮只覆盖 session/control transport | 如果误标 implemented，会破坏 parity lock 可信度 | alignment-report 增加 Scope Guard，明确 `API-003` 保持 `partial` |
| P1 | 可验证性 | 进程内 `handle_request` 不能证明外部 transport 可用 | 仍可能被视为 facade，不满足 Diri client parity | 增加 NDJSON `ControlMessage` stream 测试与 CLI `control-stdio` 测试 |
| P2 | 错误处理 | 外部协议收到未知 method 或非 request frame 不能 panic | 会导致 managed agent 调用 Homie 时中断 | 用 safe `ErrorEnvelope` failure response |
| P2 | 最小化实现 | 引入长期 daemon/socket server 会扩大实现面 | 增加不必要复杂度 | 先用 stdio NDJSON transport，复用既有 proto envelope |

## 3. 整改后的完善方案

本轮实现只新增 transport-facing client path，不引入新 daemon。`homie-client` 负责把 `ControlMessage::Request` 转换为 runtime 调用，并以 `ControlMessage::Response` 与 `ControlMessage::Event` 输出。`homie-cli control-stdio` 仅提供外部进程可验证入口。CLI session lifecycle 改用 `HomieClient`，避免 storage-only 假 session。

## 4. 分层任务拆解

| 层级 | 任务 | 交付物 | 依赖 | 优先级 |
|---|---|---|---|---|
| 协议测试 | 固定 subscribe/wait/control stream 行为 | client tests | PRD/OpenSpec | P0 |
| client | 实现 control-message dispatcher 和 stream serving | `homie-client` | tests | P0 |
| CLI | 新增 control-stdio，session 命令切 runtime client | `homie-cli` | client | P0 |
| 证据 | 更新 lock、verification、LoopX | docs + loopx writeback | tests | P0 |

## 5. 测试规划

| 类型 | 覆盖点 | 关键用例 | 执行阶段 |
|---|---|---|---|
| 集成测试 | external subscription transport | `control_stream_subscribe_emits_event_frames_and_cursor_response` | TDD 红绿 |
| 集成测试 | timeout wait | `events.wait` 命中/超时 | TDD 红绿 |
| CLI 测试 | stdio entrypoint | `homie control-stdio` NDJSON request/response | 实现后 |
| CLI 测试 | runtime-backed session | create 后 snapshot 有 holder/live state | 实现后 |
| 门禁 | parity lock | `make parity-lock` | 收尾 |

## 6. 开发排期

| 阶段 | 顺序 | 工作项 | 风险与缓冲 | 验收物 |
|---|---|---|---|---|
| 1 | 先 | 写失败测试 | holder binary 路径需复用现有测试 helper | client/CLI tests fail as expected |
| 2 | 中 | 实现 client transport | 保持同步 I/O，避免 async 引入 | client tests pass |
| 3 | 中 | CLI 接线 | create/list/snapshot 行为改变需测试覆盖 | CLI tests pass |
| 4 | 后 | 文档与 LoopX | 不误标其他 rows | verification report + loopx writeback |

## 7. 待确认问题

- 无阻塞问题；`API-003` 剩余 worktree/ports/MCP bridge 进入后续 LoopX todo。

