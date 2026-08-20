# OpenSpec Plan — app-navigation-module-split

## 目标

将 `navigation.rs` 机械拆分为 `mod.rs`（facade + 生命周期/排序/命令）+ `index.rs`（目录索引/扫描）
+ `render.rs`（渲染）+ `tests.rs`（测试）。公共 API 与行为零变更，引用方零改动，单文件 < 800 行。

## 拆分策略

- 依赖方向：`mod.rs`（父）→ `render.rs`（子）。父调用子的 `render_overlay`（`pub(super)`）；
  子通过 `use super::*` 访问父的私有字段/方法/类型。
- 渲染方法与渲染辅助自由函数下沉 `render.rs`；其余保留 `mod.rs`。
- 测试原样下沉 `tests.rs`。

## 交付切片

- T1：抽取渲染方法 + 渲染辅助 → `render.rs`。
- T2：抽取目录索引/扫描 → `index.rs`。
- T3：抽取测试 → `tests.rs`，`mod.rs` 收尾 facade。
- T4：全量验证 + code review + release readiness。

## 验证

见 `tasks.md`，最终证据写入 `docs/verification/app-navigation-module-split/`。
