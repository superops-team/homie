# Diri Automation Injection Plan 对齐设计文档

```yaml
change_id: diri-automation-injection-plan
beads: homie-8ib
target_rows:
  - AUTO-001
feature_atoms:
  - M12-F002
```

## 1. 背景

`AUTO-001` 仍为 `missing`。Diri 的 `InjectionBuilder.swift` 负责为 agent spawn 构造 hooks/MCP/notify 注入参数、基础环境、session id 和 return-to-login-shell 包装。Homie 当前有 agent descriptor/injection 数据和简单 intent routing，但没有自动化注入计划模型。

## 2. 目标

- 在 `homie-orchestrator` 中新增 automation spawn plan 模型。
- 支持基础 env：session id、socket、CLI path、PATH。
- 支持 Claude hooks、Claude MCP、Codex notify、Codex MCP 参数注入。
- 支持 `session_id_flag` 生成 agent session id。
- 支持 `return_to_login_shell` 包装。
- 不执行进程，不启动 MCP server。

## 3. 非目标

- 不实现完整 MCP stdio server。
- 不实现 Forward TCP port forwarding。
- 不把 `AUTO-001` 标为 implemented；完整 parity 仍需 MCP/forwarding E2E。

## 4. 验收

- `cargo test -p homie-orchestrator --test automation_injection -- --nocapture`
- `cargo check -p homie-orchestrator`
- `cargo clippy -p homie-orchestrator --all-targets -- -D warnings`
- scoped `git diff --check`
- `make parity-lock`
