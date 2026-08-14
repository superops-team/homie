# 本地 Shell TERM 快捷键修复 Release Readiness

## 1. 结论

- change_id: `local-shell-term-shortcuts`
- Beads: `homie-mff`
- 状态: Ready to merge
- 范围: Engine shell/generic PTY environment、Engine regression tests、runtime contract/spec evidence

## 2. 交付内容

- 本地 shell/generic argv spawn 路径通过 `shell_pty_environment` 统一处理 PTY child environment。
- 远程 non-binary shell/generic spawn 路径复用同一 helper，避免本地/远程 TERM 策略漂移。
- 新增 helper-level、Engine spawn-level、control socket real-path 回归测试。
- 新增 `specs/engine-session-runtime.md` 固化 Engine session runtime 的 shell/generic PTY 环境合同。
- PRD、OpenSpec、verification evidence 已更新。

## 3. 验证结果

| 命令 | 结果 |
|------|------|
| `cargo test --manifest-path homie/Cargo.toml -p homie-engine local_pty_environment -- --nocapture` | 通过 |
| `cargo test --manifest-path homie/Cargo.toml -p homie-engine local_shell_spawn_sets_term -- --nocapture` | 通过 |
| `cargo test --manifest-path homie/Cargo.toml -p homie-engine shell_session_reports_xterm_256color -- --nocapture` | 通过 |
| `cargo test --manifest-path homie/Cargo.toml -p homie-engine` | 通过，258 passed, 0 failed, 3 ignored |
| `cargo fmt --check` | 通过 |
| `git diff --check` | 通过 |
| `git diff --name-only -- homie/crates/homie-app homie/crates/homie-ui homie/crates/homie-client` | 通过，无输出 |

## 4. 风险与说明

- 旧 shell session 不会 retroactively 获得新环境；用户需要新建 shell 任务验证 `Ctrl+L`。
- 完整 `homie-engine` 测试中出现 `/bin/sh: ... Terminated: 15 sleep 30` 输出，来自测试用例主动 `session.kill` 清理 `sleep 30`，最终测试结果为通过。
- 本次没有提交诊断 worktree 中的 `HOMIE_KEY_TRACE` 临时 trace。
