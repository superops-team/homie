# 启动阶段后台 Shell 任务用户无感化功能验证 Case

## 1. 目标

本文件是 dev-loop Step 2 的前置功能验证 Case 设计。所有 P0/P1 需求必须先有可执行、可判定、可留证的验证方案，再进入 OpenSpec 拆解和实现。

## 2. Case 清单

### FC-01: Rust Engine 启动不执行交互 login shell

- 覆盖需求：FR-1、FR-2、FR-4、FR-8
- 前置环境：
  - 新增测试 helper 或测试注入点，能替代 PATH 捕获 executor 并记录调用次数。
  - 使用临时 HOME / HOMIE_APP_SUPPORT。
- 执行命令：
  - `cargo test --manifest-path homie/Cargo.toml -p homie-engine startup_does_not_capture_interactive_login_path`
- 输入数据：
  - fake login shell path。
  - fake executor 若被调用则记录 argv。
- 预期结果：
  - daemon startup 初始化路径不会调用 `shell -i -l -c printenv PATH`。
  - 初始 PATH 来源为 fallback 或 cache。
  - 测试能断言 path capture 调用次数为 0。
- 证据路径：
  - `docs/verification/startup-background-shell-invisibility/fc-01-rust-startup-no-login-shell.log`
- 失败处理：
  - 回到环境解析设计，移除 startup eager capture。

### FC-02: Lazy PATH refresh 可静默完成并更新 readiness

- 覆盖需求：FR-2、FR-5、FR-7
- 前置环境：
  - fake executor 返回可控 PATH，例如 `/custom/bin:/usr/bin:/bin`。
  - agent readiness 可读取 refresh 后 PATH。
- 执行命令：
  - `cargo test --manifest-path homie/Cargo.toml -p homie-engine lazy_path_refresh_updates_readiness`
- 输入数据：
  - fake agent binary 位于 `/custom/bin/codex`。
  - refresh 前 fallback 不包含 `/custom/bin`。
- 预期结果：
  - 启动后 readiness 初始为 unknown 或 unavailable/checking。
  - refresh 触发后 readiness 更新为 available。
  - refresh stdout/stderr 不进入 UI payload。
- 证据路径：
  - `docs/verification/startup-background-shell-invisibility/fc-02-lazy-path-refresh.log`
- 失败处理：
  - 调整 readiness 状态模型或 refresh 触发点。

### FC-03: Heavy shell rc 不影响首帧和 daemon ready

- 覆盖需求：FR-1、FR-2、FR-4、FR-7
- 前置环境：
  - 临时 HOME，配置测试 shell rc：
    - 输出文本；
    - sleep 3；
    - 后台启动 `touch "$HOME/rc-was-run"`；
    - 尝试网络命令但被 mock/stub。
  - HOMIE_APP_SUPPORT 指向临时目录。
- 执行命令：
  - `docs/verification/startup-background-shell-invisibility/run-heavy-rc-smoke.sh`
- 输入数据：
  - 编译后的 `homied-rs` 或 `homie/scripts/dev.sh` 可启动路径。
- 预期结果：
  - 启动到 daemon socket ready 不等待 rc sleep。
  - `$HOME/rc-was-run` 不存在，证明启动阶段没有执行交互 rc。
  - daemon boot log 不包含 rc 输出文本。
- 证据路径：
  - `docs/verification/startup-background-shell-invisibility/fc-03-heavy-rc-smoke.log`
- 失败处理：
  - 回到启动路径清理，确认所有 `-i -l` 调用移出 startup。

### FC-04: 普通启动不触发 remote/browser 侧任务

- 覆盖需求：FR-3、FR-6、FR-7
- 前置环境：
  - PATH 中放置 `ssh`、`node`、`rsync`、`gh`、`lsof` wrapper，wrapper 写入调用日志后退出。
  - HOMIE_APP_SUPPORT 指向临时目录。
- 执行命令：
  - `docs/verification/startup-background-shell-invisibility/run-startup-exec-probe.sh`
- 输入数据：
  - wrapper 目录优先于系统 PATH。
- 预期结果：
  - 普通启动到首帧/daemon ready 阶段不调用 `ssh`、`node`、`rsync`、browser sidecar。
  - 若 `gh`/`lsof` 在 resource/pr monitor 中仍被调用，必须证明发生在首帧后且低优先级，或按 P0 修正。
- 证据路径：
  - `docs/verification/startup-background-shell-invisibility/fc-04-startup-exec-probe.log`
  - `docs/verification/startup-background-shell-invisibility/fc-04-exec-calls.jsonl`
- 失败处理：
  - 调整相关后台服务为 deferred/user-triggered。

### FC-05: Swift daemon/holder target 被删除且无 fallback 引用

- 覆盖需求：FR-8、FR-9
- 前置环境：
  - 当前源码树。
- 执行命令：
  - `swift package dump-package | tee docs/verification/startup-background-shell-invisibility/fc-05-swift-package.json`
  - `rg -n "HomieDaemonKit|HomieHolderKit|homied-holder|executable\\(name: \\"homied\\"|installed_daemon|Swift daemon|Swift engine" Package.swift README.md CONTRIBUTING.md homie/scripts Sources Tests`
- 输入数据：
  - 无。
- 预期结果：
  - Package.swift 不再声明 Swift daemon/holder target/product/test。
  - README/CONTRIBUTING/package/dev scripts 不再声明或查找 Swift daemon fallback。
  - 若保留 Swift target，职责只能是 protocol/core/CLI/MCP/macOS glue。
- 证据路径：
  - `docs/verification/startup-background-shell-invisibility/fc-05-swift-cleanup.log`
- 失败处理：
  - 回到 Swift cleanup task，删除残留 target/source/docs/scripts。

### FC-06: 保留 Swift target 仍可构建

- 覆盖需求：FR-9、验收标准
- 前置环境：
  - Swift daemon/holder 删除后。
- 执行命令：
  - `swift build`
- 输入数据：
  - 无。
- 预期结果：
  - 仍保留的 Swift target 构建通过。
  - 构建产物不包含 `homied` 或 `homied-holder`。
- 证据路径：
  - `docs/verification/startup-background-shell-invisibility/fc-06-swift-build.log`
- 失败处理：
  - 修正 Package.swift target 依赖或 Swift 保留边界。

### FC-07: Rust workspace 编译与格式门禁

- 覆盖需求：整体回归
- 执行命令：
  - `cargo fmt --manifest-path homie/Cargo.toml --all -- --check`
  - `cargo check --manifest-path homie/Cargo.toml --workspace`
- 预期结果：
  - 两个命令退出码为 0。
- 证据路径：
  - `docs/verification/startup-background-shell-invisibility/fc-07-rust-gates.log`

### FC-08: 文档和代码禁止 Diri/Swift daemon legacy 回流

- 覆盖需求：FR-8、FR-9
- 执行命令：
  - `rg -n "Diri|Dirijor|Swift daemon|Swift engine|HomieDaemonKit|HomieHolderKit|HomieClient|HomieDetection|HomieGit|CHomiePTY|homied-holder|HOMIED_PATH|installed_daemon|fallback to Swift|legacy daemon" README.md CONTRIBUTING.md Package.swift ROADMAP.md scripts homie/*.md homie/scripts Sources Tests homie/crates/homie-engine/src homie/crates/homie-engine/tests homie/crates/homie-proto/src homie/crates/homie-proto/tests homie/crates/homie-app/src --glob '!homie/target/**'`
- 预期结果：
  - 不出现产品代码、文档、脚本中的旧命名或 Swift daemon legacy 叙述。
  - 允许 `environment.refresh_path` 中出现非交互 `shell -l -c 'printenv PATH'`，因为该路径是用户触发的 lazy refresh，不是启动 eager shell。
- 证据路径：
  - `docs/verification/startup-background-shell-invisibility/fc-08-legacy-scan.log`

## 3. 覆盖矩阵

| 需求 | FC-01 | FC-02 | FC-03 | FC-04 | FC-05 | FC-06 | FC-07 | FC-08 |
|------|-------|-------|-------|-------|-------|-------|-------|-------|
| FR-1 启动首帧前禁止交互 login shell | x |  | x | x |  |  |  |  |
| FR-2 PATH 捕获 lazy/cached | x | x | x |  |  |  |  |  |
| FR-3 后台任务分级 |  |  |  | x |  |  |  |  |
| FR-4 shell/exec 静默契约 | x |  | x | x |  |  |  |  |
| FR-5 readiness 不阻塞启动 |  | x |  |  |  |  |  |  |
| FR-6 remote/browser 按需启动 |  |  |  | x |  |  |  |  |
| FR-7 可观测但不打扰 |  | x | x | x |  |  |  |  |
| FR-8 Rust 唯一 daemon/supervisord | x |  |  |  | x | x |  | x |
| FR-9 删除 Swift daemon legacy |  |  |  |  | x | x |  | x |

## 4. 执行顺序

1. FC-01 先红后绿，证明 startup 不做 eager shell capture。
2. FC-02 补 lazy refresh 与 readiness。
3. FC-05/FC-06 完成 Swift daemon cleanup。
4. FC-03/FC-04 做启动体验和 exec probe。
5. FC-07/FC-08 做回归和静态收口。

## 5. 证据要求

- 每个 Case 产生日志文件，记录命令、时间、退出码、关键输出。
- 对启动 exec probe 必须保留 wrapper call log。
- 对 heavy rc 必须保留临时 HOME 配置摘要和 daemon boot log 摘要。
- 不得把未执行 Case 写为 pass。
