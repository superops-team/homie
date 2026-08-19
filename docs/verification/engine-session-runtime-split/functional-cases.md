# Functional Cases — engine-session-runtime-split

## 用例清单

1. **F1 本地 resume**：`resume_spec` 对带 `returnToLoginShell` 的 agent（claude-code）生成的
   pty.argv 尾部包含 resume 标志与 uuid。锚：`control::tests::resuming_an_agent_directly_executes_the_agent`。
2. **F2 远程 resume spec**：`remote_resume_spec` 对 `record.host` 存在时探测 persistence、捕获环境、
   生成 `LaunchRequest`，错误经 `io_control_error` 映射。锚：既有 remote/control 测试。
3. **F3 状态 reducer**：`feed_signal`/`claude_hook`/`observe_prompt_input`/`capture_prompt_title` 搬迁后
   status 测试全绿。锚：`status::tests::*`（14 项）。
4. **F4 屏幕/网格**：`SessionView`/`GridWake`/scrollback 搬迁后 screen 测试全绿。
   锚：`session::screen::grid_wake_tests::*`。
5. **F5 PTY I/O**：`Transport` 读写/resize 搬迁后 pty 测试全绿。
6. **F6 生命周期**：spawn/adopt/attach/resume/migrate + spec 结构搬迁后 lifecycle 测试全绿。

## 回归基线

拆分前基线（`1a299921`）与拆分后均 `303 passed`，无断言弱化。
