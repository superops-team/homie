# 本地 Shell TERM 快捷键修复功能验证 Case

## FC-01: PRD/spec 根因和边界清晰

```bash
rg -n "TERM=xterm-256color|Ctrl\\+L|bytes=\\[12\\]|本地 shell/generic argv|非目标|旧 session" prd-spec/bugfixes/local-shell-term-shortcuts/2026-08-14-local-shell-term-shortcuts-design.md
```

## FC-02: OpenSpec 对齐

```bash
test -s openspec/changes/local-shell-term-shortcuts/plan.md
test -s openspec/changes/local-shell-term-shortcuts/tasks.md
test -s openspec/changes/local-shell-term-shortcuts/alignment-report.md
rg -n "FC-01|FC-02|FC-03|FC-04|FC-05|FC-06" openspec/changes/local-shell-term-shortcuts/tasks.md openspec/changes/local-shell-term-shortcuts/alignment-report.md
```

## FC-03: 本地 PTY 环境 helper 覆盖 TERM/NO_COLOR

```bash
cargo test --manifest-path homie/Cargo.toml -p homie-engine local_pty_environment -- --nocapture
```

## FC-04: 本地 shell spawn 具备 TERM

```bash
cargo test --manifest-path homie/Cargo.toml -p homie-engine local_shell_spawn_sets_term -- --nocapture
```

## FC-05: 真实 shell session 输出 TERM

```bash
cargo test --manifest-path homie/Cargo.toml -p homie-engine shell_session_reports_xterm_256color -- --nocapture
```

## FC-06: 静态门禁

```bash
(cd homie && cargo fmt --check)
git diff --check
git diff --name-only -- homie/crates/homie-app homie/crates/homie-ui homie/crates/homie-client
```
