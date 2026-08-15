# GPUI 大模块纯逻辑测试边界拆分设计文档

## 1. 概述

### 1.1 问题/动机

Homie GPUI app 已经形成多个 3000 行以上的大模块：

- `homie/crates/homie-app/src/inspector.rs`
- `homie/crates/homie-app/src/sidebar/view.rs`
- `homie/crates/homie-app/src/surface_shell.rs`
- `homie/crates/homie-app/src/terminal_pane.rs`
- `homie/crates/homie-app/src/root.rs`

这些文件同时包含：

- GPUI render 树；
- 交互状态；
- 键盘/鼠标事件；
- store mutation；
- daemon effect dispatch；
- layout 计算；
- 文案/状态映射；
- 测试 fixture。

这会导致小交互变更需要加载超大上下文，也增加 review 和回归测试成本。Waku 也有较大的 UI 文件，但它把不少行为判断抽成纯函数，并集中在 `src/app/tests.rs` 中覆盖 composer、navigation、error compaction、model picker 等逻辑。Homie 已有 `SessionStore`、`SidebarProjection` 等纯模型方向，应继续把高变更 UI 行为从 GPUI render 中抽出。

### 1.2 目标

1. 为 GPUI 大模块建立“纯逻辑 / 渲染 / effect dispatch”边界。
2. 优先拆高变更、高风险行为，而不是做大规模文件搬迁。
3. 增加无需启动 GPUI window 的纯函数测试。
4. 保持 UI 行为和视觉结构不变。
5. 降低后续 sidebar、terminal、inspector 修改的 review 面。

### 1.3 非目标

- 不重做设计系统。
- 不整体重写 sidebar、terminal、inspector。
- 不改变 GPUI component 层级。
- 不把所有 UI 状态迁移到单一全局 store。

## 2. 现状分析

| 模块 | 当前职责 | 建议优先拆分 |
|------|----------|--------------|
| `sidebar/view.rs` | sidebar render、popover、directory picker、drag/drop、排序、快捷键、操作 dispatch | new-agent picker 选择逻辑、drag reorder、shortcut rank、popover state |
| `terminal_pane.rs` | terminal rendering、resize、input、clipboard、attachment、status chips、tests | key mapping、resize debounce、clipboard staging、status chip projection |
| `inspector.rs` | tabs、PR/review/actions、artifacts/code/change view、ask/review task | tab state、review action policy、artifact grouping、error compaction |
| `root.rs` | app root、panel seams、global shortcuts、window persistence、surface coordination | shortcut policy、seam geometry、window placement debounce |
| `surface_shell.rs` | utility surfaces and session surface composition | view-state mapping |

## 3. 方案设计

### 3.1 拆分原则

每次只拆一个行为单元，满足：

- 输入是普通 Rust struct/enum；
- 输出是普通 Rust value 或 `StoreEffect`；
- 不依赖 `Window`、`Context`、`Entity`；
- 可在普通 `cargo test` 中运行；
- render 层只负责调用和展示结果。

### 3.2 建议新增模块

```text
homie/crates/homie-app/src/sidebar/
├── view.rs
├── state.rs
├── fixture.rs
├── picker_logic.rs
├── reorder.rs
└── shortcuts.rs

homie/crates/homie-app/src/terminal/
├── input.rs
├── resize.rs
├── clipboard.rs
└── projection.rs

homie/crates/homie-app/src/inspector/
├── policy.rs
├── tabs.rs
├── artifacts.rs
└── review_actions.rs
```

实际路径可按现有目录结构调整，不要求一次性迁移。

### 3.3 首批候选拆分

优先级按 ROI 排：

1. Sidebar new-agent picker 逻辑  
   原因：目录选择、remote host、agent readiness、spawn options 混在 render 中，容易出错。

2. Terminal resize debounce/key mapping  
   原因：terminal PTY 尺寸和输入直接影响真实 agent，回归成本高。

3. Inspector review/action policy  
   原因：PR/review/ask 类操作后续变化频繁，最好纯函数化。

4. Root seam/window placement  
   原因：视觉状态容易和持久化互相影响。

首阶段建议只选择一个行为单元，优先 `Sidebar new-agent picker`。不得在同一 change 中同时拆 sidebar、terminal、inspector 和 root。若实现时发现 picker 依赖过多，应先补 characterization tests 和窄 adapter，而不是扩大为整文件搬迁。

### 3.4 与既有 GPUI 架构变更的关系

本 PRD 不替代已经完成的 `gpui-architecture-hardening` 及其 child slices：

- `gpui-render-path-purity` 已完成 sidebar shortcut rank first slice；本 PRD 首阶段不得重复改 shortcut rank。
- `gpui-utility-surfaces-first-slice` 已完成 UtilitySurfaces task/generation 生命周期切片；本 PRD 不再修改该路径。
- `gpui-ui-primitives-a11y` 已完成 Button primitive first slice；本 PRD 不新增 UI primitive。
- `gpui-lifecycle-task-ownership` 已完成 RootView subscription/task 所有权 first slice；本 PRD 不重做 RootView lifecycle。

### 3.5 首阶段关闭口径

`homie-wgv` 首阶段只关闭一个高变更 UI 行为的纯逻辑提取：

- 从 render 文件抽出一个无 `Window/Context/Entity` 依赖的逻辑模块。
- 增加 focused unit tests，覆盖 existing behavior characterization。
- 保持视觉结构、交互入口和 public API 不变。
- OpenSpec alignment 明确不与已完成 GPUI child changes 重叠。
- 若选择 Sidebar picker，必须覆盖 local/remote、agent readiness、cancel/no directory selected、spawn option 输出。

### 3.6 测试组织

新增或扩展测试模块：

```text
homie/crates/homie-app/src/sidebar/tests.rs
homie/crates/homie-app/src/terminal/tests.rs
homie/crates/homie-app/src/inspector/tests.rs
```

测试风格参考 Waku：小函数、小 fixture、直接断言行为输出。

## 4. 实施步骤

1. 统计大模块中的纯函数候选，不做行为改动。
2. 选择一个首批单元，例如 `new-agent picker`。
3. 先为现有行为补纯函数测试或 snapshot-like fixture。
4. 抽出逻辑模块。
5. render 层调用逻辑模块，保持视觉输出不变。
6. 运行 focused tests 和 app build。
7. 每个单元独立 PRD/OpenSpec task，不把全部大模块拆分塞进一个实现批次。

## 5. 涉及文件

- `homie/crates/homie-app/src/sidebar/view.rs`
- `homie/crates/homie-app/src/sidebar/state.rs`
- `homie/crates/homie-app/src/sidebar/fixture.rs`
- `homie/crates/homie-app/src/terminal_pane.rs`
- `homie/crates/homie-app/src/inspector.rs`
- `homie/crates/homie-app/src/root.rs`
- `homie/crates/homie-app/src/session_surfaces.rs`
- `homie/crates/homie-app/src/store/mod.rs`

## 6. 验证计划

### 6.1 静态验证

- 首批拆分不改变 public API。
- render 文件行数下降，但不以行数作为唯一指标。
- 抽出的逻辑模块无 GPUI `Window/Context/Entity` 依赖。
- `rg -n "Window|Context|Entity|cx\\.|div\\(" <new-logic-module>` 无命中，除非测试文件中有明确说明。
- `git diff --name-only` 不包含不相关 GPUI child slice 已完成范围。

### 6.2 单元测试

- picker logic tests。
- terminal key/resize tests。
- inspector action policy tests。
- seam/window placement tests。

### 6.3 视觉/交互回归

- sidebar preview scenarios：typical/stress/empty/artifacts。
- full dev bundle smoke。
- 手动验证：创建 session、切换 session、resize、打开 inspector、执行 close/archive/reopen。

### 6.4 风险控制

| 风险 | 控制 |
|------|------|
| 借机大规模搬迁 GPUI 文件 | 每个 change 只抽一个行为单元，OpenSpec 列明禁止改动范围 |
| 与已完成 child slices 重叠 | alignment 映射 `gpui-render-path-purity`、`gpui-utility-surfaces-first-slice` 等已完成范围 |
| 抽出逻辑后视觉/交互回退 | 先 characterization tests，再抽模块，最后跑 preview/手动交互证据 |
| 纯逻辑模块仍依赖 GPUI 上下文 | 静态检查禁止 `Window/Context/Entity/cx.` 进入逻辑模块 |
| 只追求行数下降 | 验收以行为测试和 review 面缩小为准，行数仅作辅助观察 |

## 7. 验收标准

1. 至少一个高变更 UI 行为被抽为纯逻辑模块。
2. 该模块有 focused tests，无需 GPUI window。
3. 原 UI 行为不变，现有 preview/fixture 通过。
4. 后续同类拆分有明确模板。
5. OpenSpec alignment 明确首阶段只覆盖一个行为单元，且不重复已完成 GPUI child slices。
6. Beads `homie-wgv` 更新为已验证状态后才可关闭。

## 8. Beads 追踪

- Beads: `homie-wgv`
- change_id: `gpui-large-module-test-boundaries`
- 类型: refactor
- 优先级: P2
