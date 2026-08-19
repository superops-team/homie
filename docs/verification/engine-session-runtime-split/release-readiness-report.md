# Release Readiness Report — engine-session-runtime-split

- change_id: `engine-session-runtime-split`
- Beads: `homie-sx7`
- date: 2026-08-19

## 交付内容

1. `session.rs` → `session/{mod,lifecycle,screen,pty,status,pump,remote,launch}.rs`，单文件 < 800 行。
2. `resume_spec` / `remote_resume_spec` 下沉至 `session/launch.rs`，以 `LaunchContext` 参数化；
   `control/handlers.rs` 中对应 handler 退化为薄适配。

## 质量门

| 门 | 结果 |
|----|------|
| cargo check | 0 warning / 0 error |
| cargo test（非沙箱） | 303 passed |
| cargo fmt --check | clean |
| cargo clippy --all-targets | 0 warning |

## 未完成（后续 child）

`session_spawn` / `session_spawn_remote` / `session_migrate` 的 spawn-spec 组装下沉（S7）
尚未完成，属本 change 的剩余部分，作为后续增量交付；wire 协议与行为保持不变。

## 结论

拆分 + resume spec 下沉已完成且验证充分，可提交（暂不打 tag，待 S7 完成后统一打 tag）。
