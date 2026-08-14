# GPUI UtilitySurfaces 首个生命周期切片设计文档

## 1. 概述

### 1.1 问题/动机

`UtilitySurfaces` 当前同时承载 History、Worktrees、Settings、Remote Host
Editor 等 overlay。根据 `gpui-architecture-hardening` 总纲，首个代码切片应从
`UtilitySurfaces history/worktrees task lifecycle` 开始，因为它能以较低风险验证：

- UI 生命周期绑定 task 字段持有；
- operation generation 防止旧请求回写；
- stale result 测试；
- close/dismiss 后不回写。

当前代码中以下操作使用 `cx.spawn(...).detach()`：

- `open_history`
- `refresh_worktrees`
- `resume_history`
- `confirm_cleanup`

这些任务和当前打开的 surface、用户重复触发的操作、关闭 overlay 的行为有关，不应作为不可追踪的 fire-and-forget work。

TDD 启动时还暴露一个 clean-worktree 编译阻塞：`homie-ui` 代码使用
`include_bytes!("../assets/icons/*.svg")` 嵌入图标资产，但仓库根 `.gitignore`
中的 `Icon?` 会匹配 `icons` 目录，导致 `homie/crates/homie-ui/assets/icons/` 未被追踪。
因此本切片需要先修正 ignore 规则并追踪既有 SVG 资产，否则任何干净 worktree
都无法编译 `homie-app` 测试。

### 1.2 目标

1. 为 History 和 Worktrees 异步操作增加字段持有的 GPUI task。
2. 为 History load/resume 和 Worktrees refresh/cleanup 增加 operation generation。
3. 拒绝旧 generation 的异步结果回写当前 UI 状态。
4. 在关闭 surface 时取消与 History/Worktrees overlay 生命周期绑定的任务。
5. 增加 focused tests 覆盖 stale result 和 close/dismiss 后不回写。
6. 不改变用户可见 UI 布局、文案和操作入口。
7. 修复 clean-worktree 缺失 `homie-ui` 图标资产的前置构建阻塞。

### 1.3 非目标

- 不拆分 `surface_shell.rs` 文件结构。
- 不重构 `RootView`。
- 不新增 `homie-ui` primitives。
- 不改变 daemon/client API。
- 不改变 Worktrees 删除策略。
- 不处理 Settings/Remote Host initialization 的 task 生命周期；该路径已有 operation guard，后续可单独优化。
- 不新增或重绘图标；只追踪现有本地 SVG 资产并修正 ignore 规则。

## 2. 现状分析

| 操作 | 当前行为 | 风险 |
|------|----------|------|
| `open_history` | 设置 loading 后 detach 一个 scan task | 关闭 History 或重新打开后旧结果仍可能写回 |
| `resume_history` | 设置 loading 后 detach 一个 spawn task | 用户切换 surface 后旧 resume 结果可能清空 surface 或覆盖 error |
| `refresh_worktrees` | detach 一个 worktree overview task | 重复刷新时旧结果可能覆盖新结果 |
| `confirm_cleanup` | detach remove + overview task | 关闭 Worktrees 或重复操作后旧结果可能回写 |

## 3. 方案设计

### 3.1 新增状态

在 `UtilitySurfaces` 中新增：

```rust
history_generation: u64,
history_task: Option<Task<()>>,
worktrees_generation: u64,
worktrees_task: Option<Task<()>>,
```

### 3.2 Generation 规则

- 每次启动 History load/resume 前递增 `history_generation`。
- 每次启动 Worktrees refresh/cleanup 前递增 `worktrees_generation`。
- 异步任务完成后，只有当当前 generation 与捕获 generation 一致时才回写状态。
- 如果 surface 已不再是对应 surface，则旧任务结果不回写可见状态。

### 3.3 Task 持有规则

- 启动新 History task 时替换 `history_task`，由 drop 取消旧任务。
- 启动新 Worktrees task 时替换 `worktrees_task`，由 drop 取消旧任务。
- `close_surface` 关闭 History 时清空 `history_task`。
- `close_surface` 关闭 Worktrees 时清空 `worktrees_task`，但只取消 overlay 生命周期绑定的 refresh/cleanup UI task；已经提交给 daemon 的副作用不能保证被取消，必须通过 generation 防止旧结果回写。

### 3.4 最小实现约束

- 保持现有 UI render 结构不变。
- 保持 `WorktreesSheet` 的 pure state API 不变，除非测试需要最小 helper。
- 不引入新模块；只在当前文件中完成 first slice。
- 后续如要拆文件，必须走 `gpui-utility-surfaces-first-slice` 之后的新 child change。

## 4. 实施步骤

1. 增加字段和初始化。
2. 修正 `.gitignore` 的 `Icon?` 规则并追踪 `homie-ui` 现有 SVG assets。
3. 抽出内部 helper：`next_history_generation`、`next_worktrees_generation` 或等价私有逻辑。
4. 改造 `open_history`：持有 task + generation guard。
5. 改造 `resume_history`：持有 task + generation/surface guard。
6. 改造 `refresh_worktrees`：持有 task + generation/surface guard。
7. 改造 `confirm_cleanup`：持有 task + generation/surface guard。
8. 改造 `close_surface`：关闭 History/Worktrees 时清理对应 task。
9. 增加 focused tests。
10. 执行功能验证 Case 和 targeted Rust tests。

## 5. 涉及文件

- `homie/crates/homie-app/src/surface_shell.rs`
- `.gitignore`
- `homie/crates/homie-ui/assets/icons/*.svg`
- `docs/verification/gpui-utility-surfaces-first-slice/*`
- `openspec/changes/gpui-utility-surfaces-first-slice/*`

## 6. 验证计划

### 6.1 静态验证

```bash
git diff --check
cargo fmt --check --manifest-path homie/Cargo.toml
```

### 6.2 Targeted tests

```bash
cargo test --manifest-path homie/Cargo.toml -p homie-app utility_surfaces -- --nocapture
cargo test --manifest-path homie/Cargo.toml -p homie-app stale -- --nocapture
```

### 6.3 功能验证

功能验证 Case 见：

```text
docs/verification/gpui-utility-surfaces-first-slice/functional-cases.md
```

## 7. 验收标准

1. History 和 Worktrees 的 overlay 生命周期异步任务不再直接 detach。
2. 重复触发 History/Worktrees 操作时旧 generation 结果不会覆盖新状态。
3. 关闭 History/Worktrees surface 后旧结果不会重新写回可见 UI 状态。
4. Targeted tests 通过。
5. `git diff --check` 和 `cargo fmt --check` 通过。
6. 不修改 `homie-ui`、`RootView` 或 daemon/client API。
7. 干净 worktree 中 `homie-ui` 图标 assets 可被 Git 追踪并可被 `include_bytes!` 读取。

## 8. Beads 追踪

- Beads: `homie-yon`
- change_id: `gpui-utility-surfaces-first-slice`
- parent: `homie-4lu`
- 类型: refactor
- 优先级: P1
