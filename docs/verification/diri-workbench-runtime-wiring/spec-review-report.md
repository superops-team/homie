# Spec Review Report

```yaml
change_id: diri-workbench-runtime-wiring
beads: homie-3tz
status: pass
```

## 1. 总体结论

- 可行性：高。
- 最大风险：用源码字符串测试替代真实 E2E 后误标 UI parity 完成。
- 推荐方向：本轮只推进 runtime-backed UI action wiring，`UI-001`/`UI-003` 仍保持 `partial`，后续用截图/交互 E2E 收口。

## 2. 问题清单

| 优先级 | 维度 | 问题 | 影响 | 整改建议 |
|---|---|---|---|---|
| P1 | 行为真实性 | `SpawnShell` 当前只写 notice | 用户看到命令入口但没有真实 session side effect | 改为调用 `HomieClient::spawn_shell` |
| P1 | 状态来源 | sidebar 只显示一个本地 session title | 无法验证 list/select/attach workflow | 引入 session projection，渲染真实 `list_sessions` |
| P1 | runtime 接线 | terminal resize 没有调用 runtime | holder geometry 与 UI 脱节 | 添加 `sync_terminal_geometry` 调 `resize_session` |
| P2 | 验收边界 | 本轮没有截图/E2E | 不能标 UI complete | parity lock 只更新证据，状态保持 partial |

## 3. 整改后的完善方案

`homie-app` 保持只消费 `homie-client`。新增 session projection、spawn/select/resize helpers，并让 command palette 与 sidebar 调用这些 helpers。测试先以 source regression 锁定不再出现本地-only placeholder，后续再补 Playwright/真机 screenshot gate。

## 4. 分层任务拆解

| 层级 | 任务 | 交付物 | 依赖 | 优先级 |
|---|---|---|---|---|
| spec | 更新 desktop shell live-connected 合同 | `specs/desktop-shell/README.md` | PRD | P0 |
| app state | session rows + selected session | `crates/homie-app/src/main.rs` | client | P0 |
| app action | SpawnShell/select/resize helpers | `crates/homie-app/src/main.rs` | app state | P0 |
| tests | source regression gates | app tests | app action | P0 |
| evidence | verification + parity lock | docs | tests | P0 |

## 5. 测试规划

| 类型 | 覆盖点 | 关键用例 | 执行阶段 |
|---|---|---|---|
| 回归测试 | no placeholder spawn | 禁止 `SpawnShell => self.state.terminal_notice = "spawned"` | 实现后 |
| 回归测试 | sidebar selection wiring | `on_mouse_down` 调 `select_session` | 实现后 |
| 回归测试 | runtime resize | `resize_session(session_id, cols, rows)` | 实现后 |
| 集成测试 | client runtime | `cargo test -p homie-client --tests` | 收尾 |
| 门禁 | parity lock | `make parity-lock` | 收尾 |

## 6. 待确认问题

- 无阻塞问题；视觉截图和完整 UI E2E 留给后续 UI parity todo。

