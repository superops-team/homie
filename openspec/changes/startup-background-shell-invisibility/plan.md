# startup-background-shell-invisibility OpenSpec Plan

## 1. 目标

解决 Homie 启动阶段后台 shell/exec 任务对用户造成体感的问题，并落实唯一后台进程管理架构：

- Rust Engine 是唯一 daemon/supervisord。
- Swift 后台 daemon/holder 不保留 legacy 或 fallback。
- 启动首帧/daemon ready 前不执行交互 login shell。
- 普通启动不触发 remote/browser/ssh/node 等用户未请求的后台任务。

## 2. 输入文档

- PRD：`prd-spec/bugfixes/startup-background-shell-invisibility/2026-08-12-startup-background-shell-invisibility-design.md`
- Spec review：`docs/verification/startup-background-shell-invisibility/spec-review-report.md`
- 功能验证 Case：`docs/verification/startup-background-shell-invisibility/functional-cases.md`
- Beads：`homie-f21`

## 3. 阶段拆解

### P0-A: Rust 启动无感

目标：Rust Engine 启动不再 eager 执行用户交互 login shell，PATH 捕获改为 lazy/cached/fallback，agent readiness 不阻塞首帧。

模块：

- Rust Engine startup：`homie/crates/homie-engine/src/bin/homied-rs.rs`
- Environment resolver：新建或拆分到 `homie/crates/homie-engine/src/`
- Agent readiness：`homie/crates/homie-engine/src/control.rs`
- App store readiness 消费：`homie/crates/homie-app/src/store/mod.rs`

关联 Case：FC-01、FC-02、FC-03、FC-04、FC-07。

### P0-B: Swift 后台 daemon 清理

目标：删除 Swift daemon/holder 运行路径和所有 fallback/legacy 引用，保留 Swift 代码时只保留仍真实使用的 protocol/core/CLI/MCP/macOS glue。

模块：

- `Package.swift`
- `Sources/homied/`
- `Sources/homied-holder/`
- `Sources/HomieDaemonKit/`
- `Sources/HomieHolderKit/`
- `Tests/HomieDaemonKitTests/`
- docs/scripts 中的 Swift daemon 描述和 fallback

关联 Case：FC-05、FC-06、FC-08。

### P1: 后台任务诊断增强

目标：在 Settings/Diagnostics 中展示后台任务最近状态。该阶段不阻塞 P0 准出，除非 P0 实现发现没有日志会影响可验证性。

关联 Case：可后续新增，不进入本次 P0 必选。

## 4. 关键设计

### 4.1 Environment resolver

定义一个可测试的环境解析边界，至少包含：

- `login_shell()`：只读用户数据库或环境变量，不执行 shell。
- `fallback_path()`：固定安全默认 PATH。
- `cached_path()`：读取上次成功 refresh 的 PATH。
- `refresh_path()`：用户触发或延迟触发的受控执行。

启动阶段只能调用 `login_shell()`、`cached_path()`、`fallback_path()`。

### 4.2 PATH refresh

默认策略：

- 禁止启动阶段执行 interactive `-i`。
- lazy refresh 默认最多尝试非交互 login shell：`shell -l -c 'printenv PATH'`。
- refresh 有 timeout，输出只提取 PATH-like 行。
- refresh 失败不阻塞 UI，只更新 readiness 为 unavailable/error。

### 4.3 readiness 状态

`agent.readiness` 不能触发启动关键路径 shell。若 PATH 还未 refresh：

- 使用 fallback/cache 返回现有判断；
- 或增加 checking/unknown 状态。

优先最小实现：保持现有 `path: Option<String>` 形态，使用 fallback/cache 结果；当 lazy refresh 完成后重新 publish readiness。若现有 UI 无法表达 checking，再最小扩展协议。

### 4.4 Swift cleanup

删除 Swift daemon/holder 不保留 fallback。保留 Swift targets 的条件：

- 仍被 CLI/MCP 或 shared protocol/core 使用；
- 不拥有后台进程生命周期；
- 不引用 daemon/holder/supervisor。

Package.swift、README、CONTRIBUTING、dev/package scripts 必须与该边界一致。

## 5. 风险控制

| 风险 | 控制 |
|------|------|
| PATH lazy 后 agent 找不到 | fallback/cache + on-demand refresh + 可操作错误 |
| Swift target 删除破坏 CLI | 先列依赖，保留 CLI/MCP/protocol/core，删除 daemon/holder |
| 启动 exec probe 不稳定 | wrapper PATH + HOMIE_APP_SUPPORT 临时目录 + 明确阶段边界 |
| P0 范围过大 | P0-A/P0-B 阶段化，P1 诊断 UI 不进入必要准出 |

## 6. 验收引用

- P0-A 必须通过 FC-01、FC-02、FC-03、FC-04、FC-07。
- P0-B 必须通过 FC-05、FC-06、FC-08。
- 完整准出必须包含 release readiness report。
