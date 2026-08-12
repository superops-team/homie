# 启动阶段后台 Shell 任务用户无感化设计文档

## 1. 概述

### 1.1 问题

当前 Homie 启动后会拉起一批后台 shell/exec 任务，用户能明显感知到启动阶段有额外进程、shell 初始化副作用或后台任务噪声，整体体感差。用户期望后台 shell 任务不能让用户有体感：启动应先给出可用 UI，后台探测和维护任务应静默、延迟、可控，不应触发用户 shell rc 的重活、不应弹出交互提示、不应造成窗口/终端闪动或明显卡顿。

本 PRD 只做规划，不进入实现。正式修改需后续基于最新代码创建 OpenSpec 并按 AGENTS.md 流程执行。

### 1.2 初步定位

基于当前代码只读扫描，启动阶段潜在感知来源包括：

- Rust Engine 启动入口 `homie/crates/homie-engine/src/bin/homied-rs.rs` 在进程启动时调用 `login_path(&user_shell)`，执行用户 login shell：
  `shell -i -l -c "printenv PATH"`。
- Swift 后台 daemon 代码 `Sources/HomieDaemonKit/LoginEnvironment.swift` 也存在同类 `Process()` 调用：
  `shell -i -l -c "printenv PATH"`。
- agent readiness、browser sidecar、PR monitor、resource governor、remote restore、registry watcher 等后台服务会在启动后进入探测或轮询路径，其中部分会解析可执行文件、调用 `git`/`gh`/`lsof`/`ssh`/`node` 或创建后台线程。
- 某些 agent manifest 使用 `returnToLoginShell`，在真正 spawn agent 时需要用户 shell，但这属于用户显式创建会话后的行为，不应在 app 启动阶段提前触发。

### 1.3 根因假设

1. Homie 为了解决 launchd 环境 PATH 过窄，在 daemon 启动时主动执行交互 login shell 捕获 PATH。交互 shell 会加载用户 rc，例如 `~/.zshrc`、`config.fish`，这些文件可能启动 nvm/pyenv/starship、运行命令、打印问候、触发网络或后台任务，因此用户会感知到启动副作用。
2. 后台能力探测与 UI 首帧/daemon ready 路径耦合过近，导致启动阶段同时发生“界面启动”和“环境/工具/远程/浏览器/资源状态扫描”。
3. 后台任务缺少统一的静默执行契约：哪些任务允许启动即跑、哪些必须按需、哪些必须低优先级延迟、如何限时/取消/记录，目前分散在 app、Swift 后台 daemon 和 Rust Engine。
4. 当前仓库还保留 Swift 后台 daemon 代码，容易形成“双 daemon / 双 supervisord / 兼容 fallback”架构漂移。根据 AGENTS.md 的不保留 legacy 和不做兼容适配原则，后续必须明确：Rust daemon/supervisord 是唯一后台进程管理面，Swift 侧不保留后台 daemon legacy。

## 2. 用户场景

### 场景 1: 用户打开 Homie

**Given** 用户通过 Finder、Dock 或 `open` 启动 Homie。  
**When** Homie 首次启动到主窗口。  
**Then** 用户不应看到由后台 shell 任务引起的终端窗口、系统提示、卡顿、明显 CPU 抖动或 shell rc 输出副作用；UI 应先可用。

### 场景 2: 用户尚未创建会话

**Given** Homie 已显示主界面，但用户没有创建 agent/session。  
**When** 后台服务初始化。  
**Then** Homie 不应主动执行用户交互 shell 或远程 ssh/node/browser 任务；只允许本地纯文件/内存级初始化和必要 daemon socket 建立。

### 场景 3: 用户打开 New Agent / Settings

**Given** 用户主动打开新建会话、agent picker 或 settings。  
**When** Homie 需要知道 Claude/Codex/Node/GitHub CLI 是否可用。  
**Then** Homie 可以进行后台探测，但必须静默、限时、可取消，并在 UI 中以轻量状态呈现，不阻塞主交互。

### 场景 4: 用户创建真实 agent 会话

**Given** 用户明确点击创建 Codex/Claude/Shell 会话。  
**When** Homie 需要进入用户 login shell 或启动 PTY。  
**Then** shell 执行是用户意图的一部分，可以发生；但启动前的路径解析和准备工作仍应尽可能使用缓存/非交互 shell，避免额外 shell 副作用。

## 3. 功能需求

### FR-1: 启动首帧前禁止交互 login shell

Homie app 启动和 daemon ready 路径不得在首帧前执行 `shell -i -l -c ...`、`/bin/sh -c ...` 或其它可能加载用户 shell rc 的命令。必要环境解析必须延后或改为非交互方案。

### FR-2: PATH 捕获从 eager 改为 lazy/cached

启动时不再立刻通过用户 shell 捕获 PATH。候选策略：

- 优先读取上次成功捕获的缓存 PATH；
- 启动阶段使用保守 fallback PATH；
- 用户打开 agent picker/settings 或首次 spawn 时再触发 PATH refresh；
- PATH refresh 必须在后台静默执行，具备超时和取消；
- 捕获失败不影响主窗口可用，只影响对应 agent 的可用性状态。

### FR-3: 后台任务统一分级

将后台任务分为：

| 等级 | 示例 | 启动策略 |
|------|------|----------|
| Critical silent | daemon socket、state load、manifest load | 可启动即跑，但必须无 shell、无交互、无 UI 噪声 |
| Deferred local | agent readiness、PATH refresh、PR monitor、resource governor 补充采样 | 首帧后延迟，低优先级，限时 |
| User-triggered | browser sidecar、ssh/rsync、remote restore、agent spawn shell | 用户打开相关功能或创建会话后才跑 |

### FR-4: shell/exec 统一静默契约

所有非 PTY 的后台 shell/exec 必须满足：

- stdin 指向 `/dev/null`；
- stdout/stderr 捕获到内存或日志，不进入用户 UI；
- 不启动 Terminal.app/iTerm，也不打开可见窗口；
- 有 hard timeout；
- 可通过 task handle 取消；
- 失败以安全、简短、可诊断状态进入日志或设置页，不弹出阻塞提示；
- 对启动关键路径不得同步等待。

### FR-5: agent readiness 不阻塞启动

`agent.readiness` 不能成为主窗口首帧依赖。UI 初始可以展示“检测中/未知”，等后台检测完成后局部刷新。

### FR-6: remote/browser 相关任务按需启动

`ssh`、`rsync`、remote restore、Node/Playwright sidecar、browser pool 等不得在普通 app 启动时自动运行。只有用户打开相关入口或已有会话明确需要恢复远程能力时才允许运行，并且需要低感知策略。

### FR-7: 可观测但不打扰

新增或整理后台任务观测信息：

- debug log 中记录任务类型、耗时、退出码、是否超时；
- 默认 UI 不弹扰民提示；
- Settings/Diagnostics 可以查看后台任务最近状态；
- 日志不得包含完整 shell rc 输出、secret、Authorization、cookie。

### FR-8: Rust daemon/supervisord 是唯一后台管理面

Homie 后续架构必须明确：Rust Engine 是唯一 daemon 和 supervisord，统一管理后台进程、PTY、holder、agent session、remote、browser sidecar、resource governor、PR monitor 等生命周期。Swift 侧不得保留后台 daemon、daemon fallback、daemon compatibility adapter 或 parallel supervisor。若 Swift 代码需要保留，只能作为 macOS UI/系统集成边界，不能拥有后台进程事实源。

### FR-9: 删除 Swift 后台 daemon legacy

后续 OpenSpec 必须包含 Swift 后台 daemon 代码清理任务：

- 删除或拆除 `Sources/homied/`、`Sources/homied-holder/`、`Sources/HomieDaemonKit/`、`Sources/HomieHolderKit/` 等后台 daemon/holder 实现；
- 删除 `Package.swift` 中对应 executable/library/test target；
- 删除或改写 `README.md`、`CONTRIBUTING.md`、`Package.swift` 注释、packaging/dev scripts 中把 Swift 描述为 engine/daemon 的内容；
- 保留 Swift 代码时只能保留 `HomieCore`、`HomieProtocol`、CLI 或必要 macOS glue 中仍被真实路径使用的部分；
- 不新增“如果 Rust daemon 不可用则 fallback 到 Swift daemon”的兼容层；
- 不为历史 Swift daemon 行为维护迁移分支。

## 4. 非目标

- 不取消用户显式创建 shell/agent 会话时的 PTY 行为。
- 不改变 agent 运行时必须回到用户 shell 的产品语义。
- 不把 PATH 捕获做成复杂配置系统；优先最小方案。
- 不在本 PRD 阶段实现代码。
- 不保留 Swift daemon 作为 legacy 运行路径；后续要么彻底删除后台 daemon 职责，要么把必要平台能力重建为清晰的 Swift macOS glue。

## 5. 方案设计

### 5.1 推荐方案：启动静默基线 + 懒加载环境服务

将当前 eager shell PATH 捕获拆成 `EnvironmentResolver` / `LoginEnvironmentProvider` 一类服务：

1. daemon 启动时只确定 `loginShell`，不执行交互 shell；
2. `PATH` 初值来自：
   - `HOMIE_PATH_OVERRIDE` 或测试注入；
   - 上次缓存；
   - fallback PATH；
3. 首帧后或用户进入 agent picker 时触发一次后台 PATH refresh；
4. refresh 使用受控执行器：
   - 优先非交互 shell：`shell -l -c 'printenv PATH'`；
   - 只有用户显式允许或测试证明必要时才考虑 `-i`；
   - 设定 1-2 秒软超时和 hard timeout；
   - 捕获输出只取 PATH-like 行；
5. readiness 结果异步刷新 UI，不阻塞启动。

优点：最小化用户体感，保留工具路径发现能力。  
风险：某些用户只在 interactive rc 中配置 PATH，首次 agent picker 可能短暂显示未知或不可用，需要在 UI 上解释“检测中”。

### 5.2 备选方案：完全不执行用户 shell

只使用系统路径、Homebrew 常见路径、`~/.local/bin`、`~/.cargo/bin`、`/opt/homebrew/bin`、`/usr/local/bin` 等固定列表。

优点：启动完全无 shell 副作用。  
缺点：nvm/asdf/pyenv/fish 用户的工具发现可能失败，影响 agent 可用性。

### 5.3 备选方案：保留 shell 捕获但移出启动关键路径

仍使用 `shell -i -l -c 'printenv PATH'`，但延迟到首帧后执行，并限制超时与日志。

优点：行为变化小。  
缺点：仍可能触发用户 rc 副作用，只是延后；无法满足“后台 shell 任务不能让用户有体感”的强要求。

### 5.4 推荐结论

采用 5.1。启动阶段建立静默基线，环境捕获懒加载并弱化 interactive shell 依赖。若用户工具路径发现失败，再通过 Settings/Diagnostics 提供显式 refresh 或配置入口。

同时采用明确架构决策：Rust Engine 是唯一 daemon/supervisord；Swift 后台 daemon 不作为 legacy 保留。后续执行计划应先确认 Rust 路径覆盖后台进程管理需求，再删除 Swift daemon/holder 相关 target 与源码，避免长期双实现。

## 6. 影响范围

### 6.1 Rust Engine

- `homie/crates/homie-engine/src/bin/homied-rs.rs`
  - `login_path(&user_shell)` 从启动 eager 改为 lazy/cached。
- `homie/crates/homie-engine/src/control.rs`
  - `agent.readiness` 需要支持 unknown/checking 状态，或保证不会同步触发昂贵 shell。
- `homie/crates/homie-engine/src/agent.rs`
  - spawn spec 仍可使用 login shell，但只在用户显式 spawn 时。
- `homie/crates/homie-engine/src/browser.rs`
  - sidecar 继续 lazy，确认启动不预热 node。
- `homie/crates/homie-engine/src/pr_monitor.rs`、`governor.rs`
  - 启动后任务延迟/低优先级策略评估。

### 6.2 Swift 后台 daemon 清理

Rust Engine 是唯一后台进程管理面，因此 Swift 后台 daemon 不进入修复路径，而进入删除/拆除路径。影响范围包括：

- `Sources/homied/`
- `Sources/homied-holder/`
- `Sources/HomieDaemonKit/`
- `Sources/HomieHolderKit/`
- 相关 Swift daemon tests
- `Package.swift` 中相关 targets/products/dependencies
- 打包脚本中对 Swift daemon/holder 的引用
- `README.md`、`CONTRIBUTING.md` 中 Swift engine/daemon 叙述
- `homie/scripts/dev.sh` 中对 installed Swift `homied` 的 fallback 查找
- `homie/scripts/package.sh` 中 Swift build 仅应构建仍保留的 CLI/glue，不得构建或复制 Swift daemon/holder

清理原则：

- 不保留 legacy target；
- 不保留 compatibility fallback；
- 不保留 dead code 作为“以后可能用”；
- 必要 Swift 平台能力必须重新归入清晰的 macOS glue 边界。

### 6.3 GPUI App

- `homie/crates/homie-app/src/daemon_launch.rs`
  - daemon spawn 仍非阻塞；需要保证 app UI 不等待环境捕获。
- `homie/crates/homie-app/src/store/mod.rs`
  - agent readiness 异步刷新，初始 unknown/checking 文案。
- `homie/crates/homie-app/src/sidebar/view.rs` / launcher/settings
  - readiness 检测中的 UI 表达要轻量，不打扰。

## 7. 验证计划

### 7.1 静态验证

- 扫描启动路径中不得出现首帧前调用：
  - `shell -i -l -c`
  - `printenv PATH`
  - `Command::new(shell)` eager 调用
  - Swift `Process()` eager PATH 捕获

### 7.2 单元测试

- Rust Engine：
  - daemon startup 不调用 path capture executor；
  - fallback PATH 可用于初始 readiness；
  - lazy refresh 成功后更新 cache/readiness；
  - refresh timeout 不阻塞 startup；
  - shell 输出噪声不进入 UI payload。
- Swift cleanup：
  - `Package.swift` 不再声明 Swift daemon/holder target；
  - 当前产品启动、打包、开发脚本不引用 Swift daemon/holder；
  - Swift 保留代码不包含后台 daemon/supervisord 职责。
  - README/CONTRIBUTING 不再把 Swift 描述为 engine/daemon。

### 7.3 集成测试

- 启动 app + daemon，统计首帧前/daemon ready 前 process exec：
  - 不出现用户 shell `-i -l -c printenv PATH`；
  - 不出现 ssh/node/browser sidecar；
  - 不出现 Terminal/iTerm/open。
- 打开 agent picker 后 readiness 可异步完成。

### 7.4 体验验证

- 使用带 heavy rc 的测试 shell：
  - rc 打印内容；
  - rc sleep；
  - rc 后台启动命令；
  - rc 网络探测。
- 验证 Homie 启动无可见 shell 副作用，首帧不被 sleep 阻塞。

## 8. 验收标准

- 启动 Homie 到首帧期间不执行交互 login shell。
- 普通启动不触发 ssh/node/browser sidecar/remote restore。
- agent readiness 从启动关键路径移出，UI 初始状态可接受且不阻塞。
- 所有后台 shell/exec 具备 timeout、stdout/stderr 捕获、日志脱敏和取消策略。
- Rust Engine 是唯一 daemon/supervisord；Swift 后台 daemon/holder 代码从产品架构中删除，不保留 fallback。
- 本地验证至少包含：
  - `swift build`（仅验证仍保留的 Swift CLI/glue/protocol targets，不包含 Swift daemon/holder）
  - `cargo check --manifest-path homie/Cargo.toml --workspace`
  - `cargo fmt --manifest-path homie/Cargo.toml --all -- --check`
  - 针对启动无 shell 的单元/集成测试
- README/CONTRIBUTING/Package.swift 不再声明 Swift daemon 是产品 engine。
- Beads `homie-f21` 关闭前必须有 verification report。

## 9. Open Questions

- 是否允许在用户首次打开 agent picker 时执行 `shell -l -c 'printenv PATH'`，还是必须完全禁止任何 shell PATH 捕获？
- 对只在 interactive rc 中配置 PATH 的用户，首版是显示“需要手动刷新/配置 PATH”，还是提供一次性显式授权的 PATH 捕获按钮？

## 10. 架构决策

- 已明确：Rust Engine 是唯一 daemon/supervisord。
- 已明确：Swift 侧不能以 legacy 名义保留后台 daemon、holder、supervisor 或 fallback。
- 已明确：Homie 后续遵循 AGENTS.md 原则，不做兼容适配层，不保留过时实现；要么进入 Rust daemon 统一管理，要么删除。

## 11. Beads 跟踪

- Beads issue：`homie-f21`
- change_id：`startup-background-shell-invisibility`
- 类型：bugfix
- 优先级：P0
