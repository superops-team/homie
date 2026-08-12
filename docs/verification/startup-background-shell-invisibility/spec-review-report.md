# Spec Review Report

## 1. 总体结论

- 可行性：中高。
- 最大风险：需求同时包含启动无感优化和 Swift 后台 daemon 清理，范围较大；如果 OpenSpec 不切清楚阶段，容易把 P0 体验修复拖成大规模架构重构。
- 推荐方向：保留当前 PRD 的架构决策，但执行时分两阶段：先保证 Rust Engine 启动无交互 shell 体感，再清理 Swift daemon/holder target 和文档叙述。两个阶段必须在同一 change_id 下有明确验收边界，不做兼容 fallback。

## 2. 问题清单

| 优先级 | 维度 | 问题 | 影响 | 整改建议 |
|---|---|---|---|---|
| P0 | 架构一致性 | 原 PRD 一度把 Swift daemon 当作可能保留的 legacy 路径。 | 违反 AGENTS.md 的不保留 legacy 和不做兼容层原则，会形成双 daemon/supervisord。 | 已整改：PRD 明确 Rust Engine 是唯一 daemon/supervisord，Swift 后台 daemon/holder 必须删除或拆除。 |
| P0 | 范围边界 | PRD 同时要求启动无感、PATH lazy、任务分级、Swift daemon 删除、文档修正。 | 一次性实现可能过大，风险集中，难以 TDD。 | OpenSpec 拆成 P0-A 启动无感、P0-B Swift daemon 清理、P1 诊断面板/任务状态。P0-A/P0-B 都属于同一变更但分阶段验证。 |
| P0 | 存量代码影响 | `Package.swift`、`README.md`、`CONTRIBUTING.md` 仍声明 Swift daemon 是 engine；`homie/scripts/dev.sh` 仍找 installed `homied` fallback。 | 即使代码切到 Rust，文档和脚本仍会把后续开发拉回 Swift daemon。 | 已补进 PRD：OpenSpec 必须清理这些引用；验收要求 docs/scripts 不再声明 Swift daemon。 |
| P0 | 可测试性 | “后台 shell 不能让用户有体感”需要可执行检测，否则容易只靠主观判断。 | 没有可复现 gate，后续回归难发现。 | OpenSpec 必须定义 process-exec probe：统计启动到首帧/daemon ready 前的 shell/ssh/node/browser 执行；并设计 heavy rc 测试。 |
| P1 | 语义歧义 | “是否允许首次打开 agent picker 时执行 shell -l -c printenv PATH”仍待确认。 | 影响 PATH discovery 设计和用户显式授权 UI。 | 保留待确认；推荐默认允许非交互 `-l -c` 的 lazy refresh，禁止 `-i`，并提供手动 PATH 配置/刷新。 |
| P1 | 兼容与迁移 | 删除 Swift daemon/holder 可能影响 Swift CLI、MCP、manifest resource bundle、tests。 | 误删会导致 Swift build 或 packaging 断裂。 | OpenSpec 要先列出保留 Swift target 与删除 target 的依赖图，再删除。Swift 保留边界只限 CLI/protocol/core/MCP 或 macOS glue。 |
| P1 | 运行风险 | Rust Engine 当前启动时直接 `login_path()` 设置全局 PATH；改 lazy 后 spawn 可能找不到 agent。 | 用户首次创建 agent 失败或 readiness 误报。 | 引入 cached/fallback PATH 与 on-demand refresh；spawn 前可尝试一次 bounded refresh，失败时给可操作错误。 |

## 3. 整改后的完善方案

### 目标与范围

本次变更解决 Homie 启动后后台 shell/exec 任务造成用户可感知副作用的问题，并顺带落实唯一后台进程管理架构：Rust Engine 是唯一 daemon/supervisord，Swift 后台 daemon/holder 代码不以 legacy 或 fallback 形式保留。

### 非目标

- 不取消用户显式创建 PTY/shell/agent 会话。
- 不引入长期兼容层。
- 不把 PATH 管理做成复杂配置系统。
- 不把 Swift 完全从仓库中删除；可保留真实仍使用的 Swift protocol/core/CLI/macOS glue。

### 设计原则

- 启动关键路径只允许 silent critical 初始化。
- 用户 shell 只在用户显式触发相关能力后执行。
- 非 PTY shell/exec 必须静默、限时、可取消、可观测。
- Rust owns background truth；Swift 只能做 UI/platform glue。
- 删除旧实现，不做 fallback。

### 核心方案

1. Rust Engine 启动时不执行 `shell -i -l -c "printenv PATH"`。
2. PATH 使用 fallback/cache 起步，agent readiness 初始为 unknown/checking。
3. 用户进入 agent picker/settings 或首次 spawn 时，触发 lazy PATH refresh。
4. PATH refresh 默认使用 bounded 非交互 shell；禁止启动阶段 interactive shell。
5. 删除 Swift daemon/holder target、source、tests、docs/scripts 引用。
6. 保留 Swift target 前先明确用途：protocol/core/CLI/MCP/macOS glue；不再包含 daemon/supervisord。

### 兼容与风险控制

- 不兼容旧 Swift daemon：删除或拆除，而不是 fallback。
- PATH 发现失败时不阻塞 UI，只影响 agent 可用性状态。
- 对 heavy shell rc 做体验回归测试。
- 对 launch scripts/package scripts 做 grep gate，防止 `homied` Swift daemon 回流。

### 验收标准

- 首帧前不执行交互 login shell。
- 普通启动不触发 ssh/node/browser sidecar/remote restore。
- Rust Engine 是唯一 daemon/supervisord。
- Swift daemon/holder source/target/test/docs/scripts 引用被清理。
- `swift build` 验证保留 Swift targets，不再构建 daemon/holder。
- `cargo check --manifest-path homie/Cargo.toml --workspace` 通过。
- 启动无 shell 的单元/集成测试通过。

## 4. 分层任务拆解

| 层级 | 任务 | 交付物 | 依赖 | 优先级 |
|---|---|---|---|---|
| Research | 枚举启动阶段实际 exec/shell 来源 | startup exec inventory | 当前代码扫描 + app 启动日志 | P0 |
| Rust Engine | 拆出 environment resolver/cache/fallback | Rust module + tests | exec inventory | P0 |
| Rust Engine | 移除 daemon startup eager `login_path()` | homied-rs startup patch | environment resolver | P0 |
| Rust Engine | agent readiness 支持 unknown/checking 或非阻塞 refresh | protocol/app store 最小调整 | resolver | P0 |
| Swift cleanup | 列出 Swift target 依赖图，删除 daemon/holder targets/source/tests | Package.swift + source cleanup | Rust Engine 覆盖确认 | P0 |
| Scripts/docs | 清理 README/CONTRIBUTING/package/dev scripts 的 Swift daemon 叙述和 fallback | docs/scripts patch | Swift cleanup | P0 |
| Tests | 加启动无交互 shell gate 和 heavy rc test | unit/integration tests | resolver + cleanup | P0 |
| Evidence | 编译、扫描、体验验证报告 | docs/verification report | all above | P0 |
| Diagnostics | Settings/Diagnostics 后台任务状态 | UI/diagnostic docs | P0 完成后 | P1 |

## 5. 测试规划

| 类型 | 覆盖点 | 关键用例 | 执行阶段 |
|---|---|---|---|
| 单元测试 | Rust startup 不调用 path capture executor | fake resolver 断言 daemon init 不执行 shell | TDD 红 |
| 单元测试 | PATH fallback/cache | 无 cache 使用 fallback；有 cache 优先 cache | TDD 红 |
| 单元测试 | lazy refresh timeout | fake shell sleep 超时不阻塞 readiness | TDD 红 |
| 单元测试 | Swift target cleanup | Package.swift dump 不含 homied/homied-holder/HomieDaemonKit/HomieHolderKit | TDD 红 |
| 集成测试 | 启动无交互 shell | heavy rc shell，启动到 ready 不调用 `-i -l -c printenv PATH` | Green |
| 集成测试 | agent picker 懒加载 | 打开 picker 后 readiness 可刷新，UI 不阻塞 | Green |
| 回归测试 | 编译门禁 | `swift build`、`cargo check --workspace`、`cargo fmt --check` | Green |
| 静态扫描 | legacy 禁止 | grep 不含 Swift daemon fallback/docs 声明 | Green |

## 6. 开发排期

| 阶段 | 时间/顺序 | 工作项 | 风险与缓冲 | 验收物 |
|---|---|---|---|---|
| Phase 0 | 先行 | OpenSpec plan/tasks/alignment | 确认 PATH refresh 策略 | OpenSpec 文档 |
| Phase 1 | 第 1 批 | Rust startup resolver + no eager shell tests | PATH 发现失败风险 | Rust tests + cargo check |
| Phase 2 | 第 2 批 | agent readiness 非阻塞/unknown | UI 状态需要最小调整 | app store/UI tests |
| Phase 3 | 第 3 批 | Swift daemon/holder 删除和 docs/scripts 清理 | Swift target 依赖误删 | swift build |
| Phase 4 | 收尾 | 启动集成验证、heavy rc、release readiness | 本机 GUI/进程观测不稳定 | verification report |

## 7. 待确认问题

- PATH refresh 是否允许首次打开 agent picker 时执行非交互 `shell -l -c 'printenv PATH'`？
- 对只在 interactive rc 配 PATH 的用户，首版是显示“需要刷新/配置 PATH”，还是提供显式授权按钮？

## 8. 评审结论

当前 PRD 经整改后可进入 OpenSpec 阶段。执行时必须保持两条硬约束：

- 不做 Swift daemon legacy/fallback。
- P0 先证明启动无感，再做后台任务诊断 UI 的 P1 增量。
