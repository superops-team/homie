# 本地 Shell TERM 快捷键修复功能验证报告

## 1. 结论

- change_id: `local-shell-term-shortcuts`
- Beads: `homie-mff`
- 执行时间: `2026-08-14 22:20:23 CST`
- 结论: FC-01 至 FC-06 全部通过。

## 2. Case 执行结果

| Case | 目标 | 结果 | 证据 |
|------|------|------|------|
| FC-01 | PRD/spec 根因和边界清晰 | 通过 | `rg -n "TERM=xterm-256color|Ctrl\\+L|bytes=\\[12\\]|本地 shell/generic argv|非目标|旧 session" prd-spec/bugfixes/local-shell-term-shortcuts/2026-08-14-local-shell-term-shortcuts-design.md` 命中 PRD 根因、输入链路和验收边界 |
| FC-02 | OpenSpec 对齐 | 通过 | `test -s openspec/changes/local-shell-term-shortcuts/{plan.md,tasks.md,alignment-report.md}` 通过；`rg -n "FC-01|FC-02|FC-03|FC-04|FC-05|FC-06" openspec/changes/local-shell-term-shortcuts/tasks.md openspec/changes/local-shell-term-shortcuts/alignment-report.md` 命中全部 case |
| FC-03 | helper 覆盖 TERM/NO_COLOR | 通过 | `cargo test --manifest-path homie/Cargo.toml -p homie-engine local_pty_environment -- --nocapture` 通过，`control::tests::local_pty_environment_sets_term_and_removes_no_color ... ok` |
| FC-04 | 本地 shell spawn 具备 TERM | 通过 | `cargo test --manifest-path homie/Cargo.toml -p homie-engine local_shell_spawn_sets_term -- --nocapture` 通过，`control::tests::local_shell_spawn_sets_term ... ok` |
| FC-05 | 真实 control socket shell session 输出 TERM | 通过 | `cargo test --manifest-path homie/Cargo.toml -p homie-engine shell_session_reports_xterm_256color -- --nocapture` 通过，`shell_session_reports_xterm_256color ... ok` |
| FC-06 | 静态门禁和范围守卫 | 通过 | `cargo fmt --check`、`git diff --check` 通过；`git diff --name-only -- homie/crates/homie-app homie/crates/homie-ui homie/crates/homie-client` 无输出 |

## 3. 关键行为证据

- 本地 shell/generic argv 路径通过共享 helper 移除继承 `TERM` 和 `NO_COLOR`，并设置 `TERM=xterm-256color`。
- 真实 control socket 测试通过 `session.spawn` 启动 `/bin/sh`，再通过 `session.read_screen` 读取 `term=xterm-256color`，覆盖客户端会使用的协议路径。
- 本次没有修改 `TerminalPane`、`homie-ui`、`homie-client`，因为诊断已证明 `Ctrl+L` 的 `[12]` 输入字节能到达 Engine。

## 4. 失败与修复记录

- 首次 `cargo fmt --check` 发现测试断言换行格式不符合 rustfmt，已运行 `cargo fmt` 修复。
- 复跑 targeted tests 和静态门禁后全部通过。
