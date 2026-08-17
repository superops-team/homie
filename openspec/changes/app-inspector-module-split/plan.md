# App Inspector Module Split — Plan

## 1. 目标

把 `homie/crates/homie-app/src/inspector.rs`（4,692 行，F3 Critical）拆成纯逻辑 /
render / effect dispatch 分层子模块，render 留在宿主 `view.rs`，纯状态机与投影函数下沉为
可独立单测的模块。视觉与行为不变。

## 2. 模块拓扑

```text
homie/crates/homie-app/src/inspector/
├── mod.rs        # 门面：mod + pub use 重导出，无业务逻辑
├── view.rs       # GPUI render + click handler（< 800 行）
├── state.rs      # tab 状态机、review/ask 工作流状态（纯数据类型）
├── projection.rs # artifact 分组、PR/session/URL/文案投影、error compaction
├── policy.rs     # review action 策略、快捷键决策
└── tests.rs      # 纯逻辑单测
```

## 3. 实施顺序（切片，与 PRD 2.3 一致）

- S1：下沉 tab 状态机 + review/ask 工作流状态类型到 `state.rs`。
  - `impl InspectorTab`（`ALL`/`label`/`index`/`debug_selector`）
  - `DiffContext`、`LoadState`、`ReviewLoadState`、`ReviewAction`、`AskDraft`
- S2：下沉纯投影/文案函数到 `projection.rs`（artifact 分组、PR/session/URL/文案投影、
  字节/时间/路径格式化，21 个纯函数）。
- S3：下沉 review/diff/merge 策略到 `policy.rs`。
- S4：收尾，render 留在 `view.rs`，`inspector.rs` 转为 `inspector/mod.rs` 薄门面；
  `view.rs` < 800 行。

每个切片 `cargo test -p homie-app` 全绿、`cargo fmt --check` clean、`cargo check -p homie-app`
0 warning，纯机械搬迁、函数体一字不改。

## 4. 验收标准

- `inspector/view.rs`（render 主体）< 800 行。
- 子模块（projection/state/policy）无 `Window`/`Context`/`Entity`/render 依赖，可独立单测。
- `cargo test -p homie-app` 全绿；视觉/行为不变。

## 5. 证据目录

`docs/verification/app-inspector-module-split/`
