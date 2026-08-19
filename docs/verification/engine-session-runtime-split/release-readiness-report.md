# Release Readiness Report — engine-session-runtime-split

- change_id: `engine-session-runtime-split`
- Beads: `homie-sx7`
- date: 2026-08-19

## 交付内容

1. `session.rs` → `session/{mod,lifecycle,screen,pty,status,pump,remote,launch}.rs`，单文件 < 800 行。
2. `resume_spec` / `remote_resume_spec` 下沉至 `session/launch.rs`，以 `LaunchContext` 参数化；
   `control/handlers.rs` 中对应 handler 退化为薄适配。
3. `spawn_spec` / `remote_spawn_spec` 下沉至 `session/launch.rs`，返回 `SpawnPlan`；
   `session_spawn` / `session_spawn_remote` 退化为薄适配，`control/handlers.rs` 由 1,461 行降至 1,179 行。

## 质量门

| 门 | 结果 |
|----|------|
| cargo check | 0 warning / 0 error |
| cargo test（非沙箱） | 303 passed |
| cargo fmt --check | clean |
| cargo clippy --all-targets | 0 warning |

## 未完成（后续 child）

`session_migrate` 的迁移阶段（WIP commit / push / hard-sync）属编排逻辑，保留在 handler；
其 resume 调用已下沉。无其余未完成项。

## 结论

拆分 + resume/spawn spec 下沉均已完成且验证充分，code-review round 已完成并去重
（见 `code-review-report.md`）。暂不打 tag，待用户确认后按 SemVer 统一打 tag。
