# 发布就绪报告 — app-terminal-pane-module-split

- 变更 ID：`app-terminal-pane-module-split`
- Beads：`homie-ubu.4`（拆分 app terminal_pane 为子模块）
- 类型：重构（机械拆分，行为与公共 API 保持不变）
- 日期：2026-08-18

## 一、变更范围

将 `homie/crates/homie-app/src/terminal_pane.rs`（约 3,495 行）机械拆分为 `terminal_pane/` 子模块目录，按切片 S1–S5 逐段推进：

| 切片 | 内容 | 目标文件 |
|------|------|---------|
| S1 | 纯函数抽取（键位 / resize / clipboard / projection） | `keys.rs`、`policy.rs`、`projection.rs` |
| S2 | status chip 投影 + URL/PR 解析辅助 | `chip.rs` |
| S3 | attachment 生命周期 | `attachment.rs` |
| S4 | 渲染收敛（`impl Render` + `render_*` + 自由渲染辅助） | `view.rs` |
| S5 | 测试迁移 + facade 收尾 | `tests.rs`、`mod.rs` |

最终目录结构：

```text
terminal_pane/
├── mod.rs         facade（结构定义 + 事件分发 + 非渲染 impl）
├── chip.rs        纯投影 + URL/PR 辅助（ChipTint/PaneChip 保持 pub）
├── attachment.rs  attachment 生命周期
├── projection.rs  纯投影
├── policy.rs      纯决策
├── keys.rs        键位适配
├── view.rs        渲染
└── tests.rs       24 个测试原样迁移 + sorted_checks
```

## 二、验证证据

以下命令均在 `homie/` crate 目录下执行（共享 target 目录）：

### 2.1 编译检查

```text
$ cargo check -p homie-app
    Finished `dev` profile [unoptimized + debuginfo] target(s)
（0 错误，0 警告）
```

`root.rs` / `main.rs` 编译通过，证明公共 API 签名与可达性未变（C6）。

### 2.2 格式化检查

```text
$ cargo fmt --check
（无输出，格式通过）
```

### 2.3 单元测试

```text
$ cargo test -p homie-app
test result: ok. 303 passed; 0 failed; 1 ignored; 0 measured
```

行为保持证据：`terminal_pane` 相关的 24 个测试从单文件原样迁移到 `tests.rs`（仅调整 `include_str!` 相对路径为 `../../../homie-proto/...`，因模块嵌套层级 +1），测试逻辑本身未改动。

### 2.4 公共 API 兼容

| 公共项 | 原文件 | 拆分后 |
|--------|--------|--------|
| `TerminalPane` | `terminal_pane.rs` | `terminal_pane/mod.rs`（`pub`） |
| `TerminalPaneEvent` | 同上 | `terminal_pane/mod.rs`（`pub`） |
| `TerminalViewport` | 同上 | `terminal_pane/mod.rs`（`pub`） |
| `bind_terminal_keys` | 同上 | `terminal_pane/mod.rs`（`pub`） |
| `CopySelection` / `Paste` | 同上 | `terminal_pane/mod.rs`（`pub`） |
| `ChipTint` / `PaneChip` | 同上（原为 `pub`） | `chip.rs`（`pub`）+ `mod.rs` `pub use` 再导出 |

## 三、切片验收结论

| 切片 | 关联 Case | 结果 |
|------|-----------|------|
| S1 纯函数抽取 | C1 | ✅ `cargo test` 全绿 |
| S2 chip 投影 | C2 | ✅ `cargo test` 全绿 |
| S3 attachment 生命周期 | C3 | ✅ `cargo test` 全绿 |
| S4 渲染收敛 | C4 | ✅ `cargo test` 全绿 |
| S5 facade + 测试迁移 | C5 | ✅ `cargo check` + `cargo fmt --check` + `cargo test` 全通过 |
| 公共 API 不变 | C6 | ✅ `cargo check -p homie-app` 编译通过 |

## 四、已知限制 / 遗留项

无。本次为纯机械重构，不新增功能、不改变行为、不改变公共 API。

## 五、结论

变更满足 PRD 全部验收标准，测试、编译、格式化三项质量门全部通过，可发布。
