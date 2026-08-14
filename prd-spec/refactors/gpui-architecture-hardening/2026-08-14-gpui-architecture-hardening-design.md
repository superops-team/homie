# GPUI 架构硬化总纲与第一阶段合同基线设计文档

## 1. 概述

### 1.1 问题/动机

2026-08-14 使用 `build-gpui-apps` skill 在独立 worktree
`/Users/bytedance/workspace/github/homie-gpui-audit-20260814` 中对 Homie 当前
GPUI 应用做只读架构 review。审计基于 `HEAD 7eef934` 到主线 `71b42e4`
附近的代码状态，重点检查：

- GPUI Entity 和 Render 边界；
- window/root/workbench/sidebar/utility surface 分层；
- task/subscription 生命周期；
- render 路径性能和共享状态访问；
- 交互控件、焦点、键盘、可访问性；
- 设计系统、平台偏好与视觉验证；
- worktree 依赖和 build cache 复用约束。

审计结论是：Homie 已经具备真实 GPUI app、固定 GPUI revision、本地
`gpui_macos` patch、部分 `#[gpui::test]` 和可运行 app bundle 基线，但 UI 层仍停留在
“快速堆出工作台后的阶段”。主要风险不在单个 API 错误，而在架构边界、交互合同和验证矩阵没有固化。

### 1.2 文档定位

本文档是 `gpui-architecture-hardening` 的 program-level PRD/spec，总结 GPUI
架构硬化的完整问题域、长期目标和分阶段路线。当前 Beads `homie-4lu` 不直接承诺完成所有代码重构，而是承诺完成第一阶段：合同基线、证据 inventory、OpenSpec
拆解和后续 child beads 的实施入口。

后续涉及代码行为的改动必须拆成独立 child beads 和 OpenSpec changes。每个 child
change 只交付一个可 review 的 vertical slice，避免把 RootView、UtilitySurfaces、组件库、render 性能和视觉矩阵塞进同一个实现批次。

### 1.3 当前变更目标

1. 把 GPUI 架构 review 发现转化为可追踪的 program PRD/spec。
2. 明确 `homie-4lu` 的第一阶段可关闭范围：合同文档、证据 inventory、OpenSpec 拆解、child beads。
3. 将代码级整改分解为后续 child changes，避免一个 P1 变更横跨过多模块。
4. 固化 worktree 共享 Cargo target 的项目规则，但不把机器绝对路径当成跨机器产品契约。
5. 为可访问性、键盘路径、稳定 ID、task/subscription 生命周期建立后续实现的长期合同入口。

### 1.4 非目标

- 不在本文档中直接实现重构。
- 不重写整个 GPUI app。
- 不改变 Homie 的产品形态、Diri parity 目标或 agent runtime 行为。
- 不引入兼容层保留旧架构；后续实现应按 repo 规则删除过时路径。
- 不把 `CARGO_TARGET_DIR` 写入 tracked 仓库配置。
- 不把 `homie-4lu` 作为所有 GPUI 架构问题的代码关闭项；代码整改必须拆 child beads。

## 2. 现状分析

### 2.1 已具备的基础

| 领域 | 当前状态 |
|------|----------|
| GPUI pin | `homie/Cargo.toml` 固定 `gpui` 和 `gpui_platform` 到 Zed revision `dc2a339d5d043da448a3f7ddc7c0a85c63864aad` |
| macOS backend | 通过 `[patch]` 使用本地 `vendor/gpui_macos`，避免 Command Line Tools-only 环境下 Metal shader 构建问题 |
| App crate | `homie/crates/homie-app` 是真实 GPUI desktop entrypoint |
| 测试 | 已存在 `#[gpui::test]`、store tests、layout tests、sidebar preview fixtures |
| Workbench 布局 | `workbench.rs` 已抽成纯布局状态，是可继续推广的好模式 |
| Build cache | 当前本机已设置所有 Homie worktree 的 `homie/target` symlink 到项目共享 target |

### 2.2 主要问题

| 编号 | 问题 | 证据/范围 | 风险 |
|------|------|-----------|------|
| P1 | 规则层文档断裂 | `AGENTS.md` 要求读取 `docs/architecture/project-layout.md`、`docs/development/standards.md`、`docs/development/quality-gates.md`、`docs/research/rust-package-selection.md`，但当前主线缺少这些文件；`specs/` 目录也不存在 | 后续 GPUI 架构变更缺少长期合同，review 只能依赖单次聊天或代码注释 |
| P2 | `RootView` 成为窗口级全能实体 | `root.rs` 同时管理 sidebar、terminal、launcher、navigation、inspector、utility surfaces、menu bar、notifier、服务事件、窗口尺寸、动画和全局快捷键 | 任何窗口/布局/通知/服务变更都可能牵动整屏，难以局部测试 |
| P3 | `UtilitySurfaces` 合并过多产品面板 | `surface_shell.rs` 同时管理 history、worktrees、settings、remote host editor、host initialization、prefs、store lock、runtime、updates | settings/history/worktrees/remote host 的状态、焦点和异步任务互相干扰 |
| P4 | 异步任务生命周期不可审计 | 多处 `cx.spawn(...).detach()` 没有保存在字段中，history/worktree/resume/cleanup 等操作缺少统一 generation 或取消模型 | 用户关闭面板或重复触发时，旧请求可能回写新状态 |
| P5 | 核心订阅所有权不够明确 | `RootView` 有 `_subscriptions` 字段，但 terminal/sidebar/launcher/navigation/inspector 的部分 subscription 使用 `.detach()` | 生命周期依赖 GPUI 隐式语义，后续维护者容易把 detach 当作忽略结果 |
| P6 | render 路径承担状态变更和写锁 | `Sidebar::render` 会拿 `store.write()` 调用 projection，并重建 shortcut ranks/glyphs | render 不再是纯展示路径，可能造成输入/重绘路径锁竞争 |
| P7 | 裸 `div().on_click()` 控件过多 | New Agent、session row、close button、settings tab、launcher 等大量交互直接挂 click/mouse handler | 缺少统一 role、label、keyboard activation、focus-visible、disabled/a11y contract |
| P8 | 稳定 ID 策略不统一 | launcher/navigation/settings/host option 等存在 index-based ID | 列表重排、过滤、虚拟化后 hover/focus/a11y identity 可能漂移 |
| P9 | 设计系统 token 有基础但控件合同不足 | `homie-ui` 有 `Radius`、`Typo`、`SemanticColors`、`FloatingSurface`、`HoverMarquee`，但缺少 Button/IconButton/ListRow/Dialog/Tab/TextField 原语 | hover、pressed、selected、disabled、focus、a11y 规则分散在业务文件 |
| P10 | 平台偏好和视觉验证矩阵不完整 | 已使用 `reduce_motion()`，但未看到 reduce transparency、increase contrast、differentiate without color 的统一策略 | GPUI/macOS UI 在可访问性偏好、light/dark、高对比、真实窗口下缺少硬验收 |

### 2.3 与既有 PRD 的关系

已有 `gpui-large-module-test-boundaries` 只覆盖“大模块纯逻辑测试边界拆分”，重点是把高变更逻辑从
`sidebar/view.rs`、`terminal_pane.rs`、`inspector.rs`、`root.rs` 等大文件中抽出。

本文档范围更宽，覆盖：

- 长期工程合同和缺失 docs/specs；
- worktree build cache 规则；
- entity 分层；
- task/subscription 生命周期；
- render 路径写锁；
- 交互控件和 a11y；
- 设计系统和视觉/平台验证矩阵。

后续实现可把 `gpui-large-module-test-boundaries` 作为子任务或第一阶段之一，但不能替代本文档。

### 2.4 Review 整改原则

本 PRD 经过 spec review / brooks-review 后，按以下原则收敛：

- `homie-4lu` 只关闭 Phase 0/1 合同基线，不用“至少一个代码样例”代表整个 program 完成。
- 所有代码整改拆成 child beads，避免 Change Propagation 和大 PR review 风险。
- Worktree shared target 规则以 `AGENTS.md` 为权威；PRD 中的绝对路径只作为当前机器实例，不作为跨机器产品约束。
- 候选模块名仅是可能的落点，必须通过边界准入标准后才能新增模块。
- Phase 0 必须输出结构化 review inventory，让每个问题有 source、target contract、owner task 和 verification evidence。

## 3. 方案设计

### 3.1 长期合同补齐

新增或补齐以下文档：

```text
docs/architecture/project-layout.md
docs/development/standards.md
docs/development/quality-gates.md
docs/research/rust-package-selection.md
specs/gpui-shell.md
specs/gpui-interaction-contract.md
specs/ui-components.md
```

最低要求：

- `project-layout.md` 描述 Swift/Rust/GPUI crate、scripts、docs、worktree 布局。
- `standards.md` 描述 Rust/GPUI 模块边界、task/subscription、render 禁忌、a11y 基线。
- `quality-gates.md` 描述 Cargo/Swift/packaging/preview/visual/manual gate。
- `rust-package-selection.md` 记录 GPUI、Tokio、Alacritty terminal、rusqlite、notify 等选择和约束。
- `specs/gpui-shell.md` 定义 RootView、Workbench、Sidebar、Terminal、Inspector、UtilitySurfaces 的长期边界。
- `specs/gpui-interaction-contract.md` 定义键盘、焦点、ID、a11y、overlay dismiss、reduced motion 等行为合同。
- `specs/ui-components.md` 定义共享 UI primitives 的角色、状态、尺寸、theme token 和验证要求。

### 3.2 Worktree 共享 target 规则

`AGENTS.md` 的 `Worktree Build Cache Rules` 是 Homie worktree build cache
规则的权威入口。本文档只描述当前机器实例和验证方法，不替代 `AGENTS.md`。

当前机器实例的共享 target 目录是：

```text
/Users/bytedance/workspace/github/homie-worktrees/.shared/homie-target
```

后续实现和 agent 操作必须遵守：

- 新建 worktree 后先创建 symlink，再运行 Cargo 或 packaging scripts。
- 如果已有真实 `homie/target`，先确认是可丢弃 build output，再替换为 symlink。
- 不把 symlink 或 shared target 目录提交进仓库。
- 不在 tracked `.cargo/config.toml` 写入机器绝对路径。
- 验证时检查 active Homie worktree 的 `homie/target` realpath 是否一致，而不是只检查固定两个路径。

### 3.3 RootView 分层

目标是把 `RootView` 从“全能实体”改成窗口 shell 编排实体。

候选边界：

| 边界 | 职责 | 不应承担 |
|------|------|----------|
| `RootView` | window root、全局 action routing、顶层 entity 装配、focus fallback | 具体面板业务、remote host 操作、notification policy、复杂布局数学 |
| `WorkbenchShell` | sidebar/workbench/inspector 三栏装配、seam resize、launcher 替换主 pane | daemon/store effect、settings/history 业务 |
| `WorkbenchLayout` | 纯布局状态、pane fraction、min/max、resize math | GPUI render、store 写入 |
| `ServiceEventBridge` | store status/update/usage/watch channel 到 entity event 的桥接 | 直接渲染 UI 或持久化偏好 |
| `WindowPlacementController` | window bounds debounce、placement persist、restore | session selection 或 terminal focus |

这些名称不是强制落地目录。新增模块必须满足边界准入标准：

- 能隐藏一个明确复杂性，而不是只把调用转发给原模块；
- 有稳定输入/输出或清晰 owner lifecycle；
- 能用纯测试或 focused `#[gpui::test]` 单独验证；
- 依赖方向符合 `specs/gpui-shell.md`；
- 抽出后 RootView/调用方接口更小，而不是参数列表更长。

首批不要求大搬迁。优先把纯状态和事件桥拆出，保持 UI 行为不变。

### 3.4 Utility surfaces 拆分

可将 `UtilitySurfaces` 拆成独立拥有 task、focus、error/loading state 的实体：

```text
surface_shell/
├── mod.rs
├── history_surface.rs
├── worktrees_surface.rs
├── settings_surface.rs
├── remote_host_editor.rs
└── overlay_shell.rs
```

上述路径是候选结构。实际实现应先选择一个 vertical slice 验证边界，再决定是否创建对应文件。

最低要求：

- 每个 surface 的异步任务保存在字段中，替换时取消旧任务。
- 不可取消的远程操作使用 `operation_id` 或 generation 丢弃旧结果。
- 每个 surface 自己定义 loading/empty/error/success 状态。
- overlay shell 只处理 backdrop、focus scope、Escape/outside dismiss、focus return。

首选第一批代码切片是 `UtilitySurfaces history/worktrees task lifecycle`。理由：

- 同时覆盖 task 字段持有、operation generation、stale result 测试和 close/dismiss 后不回写；
- 比 RootView 大拆分风险低；
- 能给后续 settings、remote host editor 和 overlay shell 提供可复制模式；
- 与既有 `gpui-large-module-test-boundaries` 不冲突。

### 3.5 Task/subscription 生命周期规则

新增通用规则：

- UI 生命周期绑定的 task 必须存在字段中，例如 `history_task: Option<Task<()>>`。
- 用户可重复触发的查询/刷新/远程操作必须有 generation 或 revision。
- `.detach()` 只允许用于明确 app-lifetime/service-lifetime 工作，并且要记录错误处理策略。
- 重要 entity subscription 必须显式存到 `_subscriptions`，不要默认 detach。
- store/update/usage watch loop 应集中在 bridge/controller 模块中。

### 3.6 Render 路径纯化

目标：

- render 不做磁盘/网络/process I/O。
- render 不拿长时间写锁。
- render 不启动 task/subscription。
- render 不改变 domain state。
- render 不重建大集合派生状态。

Sidebar 优先整改：

- `SidebarProjection` 由 store change 时计算/缓存。
- `shortcut_ranks` 在 projection 更新时派生，而不是 render 每帧清空重建。
- glyph lifecycle 维护从 render 移到明确的 projection/update 阶段。

### 3.7 交互组件和可访问性原语

在 `homie-ui` 增加最小 UI primitives：

```text
Button
IconButton
ListRow
Dialog
Tab
TextField 或 TextInputAdapter
```

每个 primitive 至少定义：

- stable id；
- semantic role；
- accessible name；
- selected/expanded/disabled/loading state；
- pointer down/up/click 行为；
- keyboard activation；
- visible focus；
- disabled 后阻止 pointer、keyboard、a11y activation；
- theme token；
- `#[gpui::test]` 或纯状态测试。

业务模块不再直接用裸 `div().cursor_pointer().on_click(...)` 表达按钮/行/Tab 等语义控件，除非有注释说明为什么现有 primitive 不适用。

### 3.8 稳定 ID 规则

ID 来源优先级：

1. domain id + local role，例如 `("session-row", session.id)`；
2. stable command/action id；
3. stable enum variant；
4. index 仅允许用于不可重排、不可过滤、不可插入的静态列表。

禁止：

- 对会 reorder/filter/search 的列表使用 index id；
- 对动态用户内容使用 localized display text 作为 ID；
- 在 render 中生成随机 ID；
- 在 loop 中复用同一个 static ID。

### 3.9 平台偏好和视觉验证矩阵

建立主题/偏好映射：

| 偏好 | UI 行为 |
|------|---------|
| reduce motion | seam/overlay snap 或极短 fade；动画不持续 request frame |
| reduce transparency | 使用 opaque surface，保持层级和边界 |
| increase contrast | 提升 text/border/focus 对比 |
| differentiate without color | 用 icon、stroke、label、shape 补充颜色语义 |

每个视觉/交互变更至少记录：

- light/dark；
- active/inactive window；
- min/default/wide window；
- reduced motion；
- keyboard-only path；
- screenshot 或短录屏；
- 未验证平台清单。

## 4. 实施步骤

### Phase 0: 现状固化

1. 保留当前 `AGENTS.md` worktree build cache 规则。
2. 记录当前所有 Homie worktree 的 `homie/target` symlink 状态。
3. 新增缺失 docs/specs 骨架，不改变代码。
4. 把本次 review 的具体证据写入 `docs/verification/gpui-architecture-hardening/review-inventory.md`。

`review-inventory.md` 必须使用以下表格结构：

| finding_id | source | current_symptom | target_contract | owner_task | verification_command | evidence_path |
|------------|--------|-----------------|-----------------|------------|----------------------|---------------|

字段要求：

- `finding_id` 对应本文 P1-P10 或后续 child finding；
- `source` 使用 `file:line` 或文档路径，不能只写聊天摘要；
- `target_contract` 指向 `specs/*` 或 `docs/development/*` 中的具体合同；
- `owner_task` 指向 OpenSpec task 或 child Beads；
- `verification_command` 是可执行命令或明确的手工验证步骤；
- `evidence_path` 指向 `docs/verification/<change-id>/` 下的证据文件。

### Phase 1: 合同和质量门禁

1. 补齐 `docs/architecture/project-layout.md`。
2. 补齐 `docs/development/standards.md`。
3. 补齐 `docs/development/quality-gates.md`。
4. 补齐 `docs/research/rust-package-selection.md`。
5. 新增 `specs/gpui-shell.md`、`specs/gpui-interaction-contract.md`、`specs/ui-components.md`。
6. 创建 child beads 和对应 OpenSpec changes，至少包括：
   - `gpui-lifecycle-task-ownership`
   - `gpui-utility-surfaces-first-slice`
   - `gpui-ui-primitives-a11y`
   - `gpui-render-path-purity`
   - `gpui-visual-platform-gates`

### Phase 2: 生命周期硬化（child change）

1. 清点 `cx.spawn(...).detach()` 和 detached subscriptions。
2. 将 UI 生命周期绑定 task 改为字段持有。
3. 为 history/worktree refresh/resume/cleanup 增加 operation generation。
4. 将核心 subscriptions 显式存到 owner 字段。
5. 增加 focused tests 覆盖 stale result 和 close/dismiss 后不回写。

### Phase 3: RootView 和 UtilitySurfaces 分层（child change）

1. 抽出 `WindowPlacementController` 或纯 placement debounce 模块。
2. 抽出 `WorkbenchShell` 或至少把 seam/layout/controller 逻辑从 RootView render 中分离。
3. 拆分 `UtilitySurfaces` 的 history/worktrees/settings/remote host editor。
4. 保持现有 UI 和快捷键行为不变。

### Phase 4: 交互组件和 a11y 原语（child change）

1. 在 `homie-ui` 增加最小 Button/IconButton/ListRow/Dialog/Tab。
2. 替换 New Agent、session row、close button、settings tab、launcher submit 等高频裸控件。
3. 加入 role/name/state/keyboard/focus-visible/disabled 测试。
4. 明确 index ID 替换清单。

### Phase 5: Render 路径和性能（child change）

1. 将 sidebar projection/shortcut ranks 从 render 写锁路径移出。
2. 确保 render 不创建 task/subscription、不进行 I/O、不做大集合派生。
3. 对大列表引入或验证 `list`/`uniform_list` 策略。
4. 增加 render idle frame 和 reduced motion 测试。

## 5. 涉及文件

### 文档

- `AGENTS.md`
- `docs/architecture/project-layout.md`
- `docs/development/standards.md`
- `docs/development/quality-gates.md`
- `docs/research/rust-package-selection.md`
- `specs/gpui-shell.md`
- `specs/gpui-interaction-contract.md`
- `specs/ui-components.md`
- `docs/verification/gpui-architecture-hardening/*`
- `openspec/changes/gpui-architecture-hardening/*`

### 代码候选

- `homie/crates/homie-app/src/root.rs`
- `homie/crates/homie-app/src/surface_shell.rs`
- `homie/crates/homie-app/src/sidebar/view.rs`
- `homie/crates/homie-app/src/sidebar/state.rs`
- `homie/crates/homie-app/src/workbench.rs`
- `homie/crates/homie-app/src/launcher.rs`
- `homie/crates/homie-app/src/navigation.rs`
- `homie/crates/homie-app/src/inspector.rs`
- `homie/crates/homie-app/src/terminal_pane.rs`
- `homie/crates/homie-app/src/store/mod.rs`
- `homie/crates/homie-ui/src/components.rs`
- `homie/crates/homie-ui/src/tokens.rs`

## 6. 验证计划

### 6.1 文档验证

- `AGENTS.md` 包含 worktree shared target 规则。
- `docs/architecture/project-layout.md` 等 `AGENTS.md` 要求读取的文档存在。
- `specs/` 中有 GPUI shell、interaction、components 合同。
- OpenSpec alignment report 能追踪每个 PRD requirement 到 task 和 evidence。

### 6.2 静态验证

```bash
git diff --check
cargo fmt --check --manifest-path homie/Cargo.toml
cargo check --manifest-path homie/Cargo.toml --workspace
```

### 6.3 单元和 GPUI 测试

按阶段运行：

```bash
cargo test --manifest-path homie/Cargo.toml -p homie-app
cargo test --manifest-path homie/Cargo.toml -p homie-ui
cargo test --manifest-path homie/Cargo.toml -p homie-engine
```

重点测试：

- task generation/stale result；
- overlay dismiss/focus return；
- keyboard activation；
- disabled control activation block；
- stable ID/domain ID；
- reduced motion；
- sidebar projection update；
- workbench/seam layout。

### 6.4 运行和视觉验证

至少覆盖：

- `HOMIE_SIDEBAR_PREVIEW=1 HOMIE_SIDEBAR_SCENARIO=typical ./homie/scripts/dev.sh`
- `HOMIE_SIDEBAR_PREVIEW=1 HOMIE_SIDEBAR_SCENARIO=stress ./homie/scripts/dev.sh`
- full app launch；
- 创建 session；
- 切换 session；
- 打开/关闭 launcher；
- 打开/关闭 settings/history/worktrees；
- resize sidebar/terminal/inspector；
- keyboard-only 操作；
- light/dark；
- reduced motion。

### 6.5 Worktree build cache 验证

```bash
git worktree list --porcelain
for wt in <active-homie-worktree-list>; do realpath "$wt/homie/target"; done | sort -u
```

输出应只有一个真实 target 路径。当前机器实例应为：

```text
/Users/bytedance/workspace/github/homie-worktrees/.shared/homie-target
```

## 7. 验收标准

### 7.1 Program-level 完成条件

这些条件描述整个 `gpui-architecture-hardening` program 完成后的目标态，不能作为
`homie-4lu` 第一阶段关闭条件：

1. UI 生命周期绑定 task 已按合同收敛，关键 stale result 路径有测试。
2. 核心 entity subscription 所有权清晰，重要 subscription 显式由 owner 持有。
3. 高频裸 click 控件已迁移到 `homie-ui` semantic primitives，并覆盖 keyboard/a11y/disabled 测试。
4. 主要 render 写锁或 render 派生大状态路径已移出 render。
5. RootView 和 UtilitySurfaces 完成可持续的边界拆分，接口更小且测试更聚焦。
6. 视觉/交互验证矩阵有真实运行证据和未验证平台清单。

### 7.2 `homie-4lu` 第一阶段关闭条件

Beads `homie-4lu` 只在以下 evidence 齐备后关闭：

1. `AGENTS.md` 保留 worktree shared target 规则，且 PRD 引用该规则而不复制为跨机器产品契约。
2. `docs/architecture/project-layout.md`、`docs/development/standards.md`、`docs/development/quality-gates.md`、`docs/research/rust-package-selection.md` 存在并覆盖 GPUI 架构硬化最低要求。
3. `specs/gpui-shell.md`、`specs/gpui-interaction-contract.md`、`specs/ui-components.md` 存在并声明 entity、task/subscription、render、a11y、ID、组件 primitive 合同。
4. `docs/verification/gpui-architecture-hardening/review-inventory.md` 使用本文规定的表格 schema，P1-P10 每项都有 owner task 和 verification path。
5. `openspec/changes/gpui-architecture-hardening/plan.md`、`tasks.md`、`alignment-report.md` 将 Phase 0/1 需求映射到任务和证据。
6. 至少创建 Phase 2-5 的 child beads，并在 `alignment-report.md` 中说明它们不是 `homie-4lu` 的关闭条件。
7. active Homie worktree 的 `homie/target` realpath 一致，且没有把 symlink 或 shared target 目录加入 tracked files。

## 8. 风险与缓解

| 风险 | 影响 | 缓解 |
|------|------|------|
| 分层重构过大 | 难以 review，容易引入 UI 回归 | 每次只做一个 vertical slice；先 pure state 或 task lifecycle，再改 render |
| a11y primitive API 与当前 GPUI revision 不匹配 | 编译失败或行为不完整 | 搜索 pinned GPUI 和本 repo 现有测试，先在一个控件验证 |
| shared target symlink 被误提交 | 机器路径进入仓库 | 本地 exclude 忽略；提交前 `git status --short` 检查 |
| `CARGO_TARGET_DIR` 与 symlink 双配置冲突 | scripts 和 direct cargo 输出不一致 | 项目规则优先 symlink；不要 tracked CARGO_TARGET_DIR |
| render 路径拆分改变 UI 时序 | hover/focus/selection 行为回归 | 对变更前行为补测试和 preview 证据 |
| 视觉验证成本升高 | PR 变慢 | 将 visual matrix 分层，纯逻辑只跑 unit，交互/布局才要求 launch/screenshot |
| umbrella 与 child 变更边界混淆 | 第一阶段误关闭整个架构 program | `homie-4lu` 只验收 Phase 0/1；Phase 2-5 必须通过 child beads 和独立 evidence 关闭 |

## 9. Beads 追踪

- Beads: `homie-4lu`
- change_id: `gpui-architecture-hardening`
- 类型: refactor
- 优先级: P1
- source: `gpui-skill-architecture-review`
- 关闭范围: Phase 0/1 合同基线，不包含 Phase 2-5 的代码整改
