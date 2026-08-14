# Code Review Report

## 1. 审查范围

- 文件/模块:
  - `homie/crates/homie-engine/src/control.rs`
  - `homie/crates/homie-engine/tests/control_socket.rs`
  - `specs/engine-session-runtime.md`
  - `prd-spec/bugfixes/local-shell-term-shortcuts/2026-08-14-local-shell-term-shortcuts-design.md`
  - `openspec/changes/local-shell-term-shortcuts/*`
- 变更类型: bugfix + regression tests + durable runtime spec。
- 调用链/数据流: `session.spawn` -> shell/generic argv `PtySpec` -> child environment -> shell reads `$TERM` -> `session.read_screen` verifies output。
- 参考规则: `AGENTS.md` workflow、PRD `local-shell-term-shortcuts`、OpenSpec tasks、`docs/development/standards.md`。

## 2. 旧问题复核

无上一轮保留 finding。

## 3. Findings

| 严重度 | 类别 | 位置 | 证据与影响 | 处置 |
|---|---|---|---|---|
| medium | Architecture | `homie/crates/homie-engine/src/control.rs` shell/generic environment handling | 初始实现只在本地路径新增 helper，远程 non-binary shell/generic 路径仍保留重复 TERM 逻辑；PRD 要求本地与远程 TERM 策略保持一致，后续默认终端类型可能漂移 | fixed: 改为 `shell_pty_environment(...)`，本地和远程 non-binary shell/generic 路径共用 |
| medium | Scope | `specs/` | 该变更影响 Engine session runtime / PTY child environment，但初始交付没有长期 component spec；不符合仓库 workflow 中“影响 runtime 行为需更新 specs”的要求 | fixed: 新增 `specs/engine-session-runtime.md`，并更新 PRD/OpenSpec 对齐 |

## 4. 对抗式复盘

- 反例/边界: inherited `TERM=dumb`、缺失 `TERM`、`NO_COLOR=1`、显式 argv shell、control socket 协议路径。
- 资源释放: 新增 shell tests 在断言后调用 `session.kill` 并关闭 socket write side，避免遗留测试进程或 server 线程。
- 范围边界: 未修改 `TerminalPane`、`homie-ui`、`homie-client`；manifest-backed agent 仍走 descriptor environment path。
- 撤回或降级: 未发现需要报告的额外 correctness/security finding。

## 5. 修复摘要

- 本地 explicit argv / shell spawn 使用 `shell_pty_environment`。
- 远程 non-binary shell/generic spawn 复用同一个 helper，行为保持为移除 `NO_COLOR`/`TERM` 后设置 `TERM=xterm-256color`。
- 新增 helper-level、engine spawn-level、socket real-control-path regression tests。
- 新增 `specs/engine-session-runtime.md` 并更新 PRD/OpenSpec 映射。

## 6. 验证结果

| 命令 | 结果 | 说明 |
|---|---|---|
| `cargo fmt --check` | 通过 | rustfmt 门禁通过 |
| `cargo test --manifest-path homie/Cargo.toml -p homie-engine local_pty_environment -- --nocapture` | 通过 | helper 行为回归 |
| `cargo test --manifest-path homie/Cargo.toml -p homie-engine local_shell_spawn_sets_term -- --nocapture` | 通过 | Engine local spawn 回归 |
| `cargo test --manifest-path homie/Cargo.toml -p homie-engine shell_session_reports_xterm_256color -- --nocapture` | 通过 | control socket real path 回归 |
| `git diff --check` | 通过 | whitespace 门禁 |
| `git diff --name-only -- homie/crates/homie-app homie/crates/homie-ui homie/crates/homie-client` | 通过 | App/UI/client 范围守卫无输出 |

## 7. 剩余风险

- 当前验证证明新建 shell/generic PTY session 可读取 `TERM=xterm-256color`；旧 session 的环境不会被 retroactively 修改，需要用户新建 shell 任务验证快捷键。
- 本次没有做 GUI 真机 `Ctrl+L` 再次截图验证，因为根因链路已由诊断 trace 证明输入字节到达 Engine，本次回归覆盖的是缺失的 child environment。
