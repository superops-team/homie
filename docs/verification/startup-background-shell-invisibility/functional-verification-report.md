# 启动阶段后台 Shell 任务用户无感化功能验证报告

## 1. 总览

| Case | 状态 | 证据 |
|------|------|------|
| FC-01 Rust Engine 启动不执行交互 login shell | pass | `fc-01-02-environment-tests.log` |
| FC-02 Lazy PATH refresh 基础能力 | pass | `fc-01-02-environment-tests.log` |
| FC-03 Heavy shell rc 不影响 daemon ready | pass | `fc-03-heavy-rc-smoke.log` |
| FC-04 普通启动不触发 remote/browser/system exec | pass | `fc-04-startup-exec-probe.log`、`fc-04-exec-calls.jsonl` |
| FC-05 Swift daemon/holder target 删除且无 fallback 引用 | pass | `fc-05-swift-package.json`、`fc-05-swift-cleanup.log` |
| FC-06 保留 Swift target 仍可构建 | pass | `fc-06-swift-build.log` |
| FC-07 Rust workspace 编译与格式门禁 | pass | `fc-07-rust-gates.log` |
| FC-08 文档和代码禁止旧命名/Swift daemon legacy 回流 | pass | `fc-08-legacy-scan.log` |

## 2. 执行记录

### FC-01 / FC-02

- 命令：`cargo test --manifest-path homie/Cargo.toml -p homie-engine environment -- --nocapture`
- 结果：退出码 0。
- 说明：新增 environment resolver 单测通过，覆盖 fallback、override、cache、PATH-like 输出解析。

### FC-03

- 命令：`bash docs/verification/startup-background-shell-invisibility/run-heavy-rc-smoke.sh`
- 结果：退出码 0。
- 说明：使用 fake shell 模拟 heavy rc；daemon socket 成功创建，`rc-was-run` 未出现，boot log 无 rc 输出。

### FC-04

- 命令：`bash docs/verification/startup-background-shell-invisibility/run-startup-exec-probe.sh`
- 结果：退出码 0。
- 说明：PATH 前置 `ssh`、`rsync`、`node`、`gh`、`lsof`、`open`、`osascript` wrapper；daemon ready 前没有调用记录。

### FC-05

- 命令：
  - `swift package dump-package > fc-05-swift-package.json`
  - `rg ... fc-05-swift-package.json > fc-05-swift-cleanup.log`
- 结果：dump-package 退出码 0；扫描退出码 1 表示无命中。
- 说明：Swift package 不再包含 `homied`、`homied-holder`、`HomieDaemonKit`、`HomieHolderKit`、`HomieDetection`、`HomieClient`、`HomieGit`、`CHomiePTY`。

### FC-06

- 命令：`swift build`
- 结果：退出码 0。
- 说明：保留的 Swift CLI/protocol/core/MCP targets 构建通过。

### FC-07

- 命令：
  - `cargo fmt --manifest-path homie/Cargo.toml --all -- --check`
  - `cargo check --manifest-path homie/Cargo.toml --workspace`
- 结果：退出码 0。

### FC-08

- 命令：产品源码/文档/脚本范围 `rg` 扫描旧命名与 Swift daemon legacy 关键词。
- 结果：扫描退出码 1 表示无命中。
- 说明：PRD/OpenSpec/verification 证据目录本身描述本需求，未纳入产品残留扫描。`environment.refresh_path` 中允许保留用户触发的非交互 `shell -l -c 'printenv PATH'`，它不在 FC-08 legacy 关键词范围内。

## 3. 结论

P0 功能验证 Case 全部通过。可以进入代码评审阶段。
