# Release Readiness — app-markdown-module-split

## 变更摘要

将 `homie/crates/homie-app/src/markdown.rs`（1,105 行）机械拆分为 3 个聚焦子模块，
`mod.rs` 收尾 facade（182 行）。公共 API 与运行时行为零变更，引用方零改动。

## 拆分结果

| 文件 | 行数 | 职责 |
|------|------|------|
| `mod.rs` | 182 | 文档模型类型 + 常量 + 纯文本投影 + 模块声明 |
| `block.rs` | 299 | 块级解析（标题/列表/引用/代码块/分隔线识别） |
| `inline.rs` | 375 | 行内解析（强调/代码/链接/自动链接/转义） |
| `tests.rs` | 256 | 既有测试（原样下沉） |

全部单文件 < 800 行。

## 可见性管控

仅跨模块调用的入口函数升为 `pub(super)`，无 `pub` 可见性泄漏到 crate 外：
- `block.rs`：`parse_blocks`（mod.rs 调用）。
- `inline.rs`：`parse_inline`（mod.rs 与 block.rs 调用）。

其余辅助函数均保持私有 `fn`。`block.rs` 通过 `use super::inline::parse_inline;` 显式引用
兄弟模块入口，其余通过 `use super::*` 访问父模块私有项。

## 验证证据

| 项 | 结果 |
|----|------|
| `cargo fmt --all --check` | ✅ 通过 |
| `cargo check -p homie-app --offline` | ✅ 通过 |
| `cargo clippy -p homie-app --all-targets --offline` | ✅ 0 警告 |
| `cargo test -p homie-app --offline markdown` | ✅ 12/12 passed |
| `cargo test -p homie-app --offline` | ✅ 301 passed（沙箱内 2 个 `daemon_launch` socket bind EPERM 失败属预期） |
| `cargo test -p homie-app --offline daemon_launch::tests`（沙箱外） | ✅ 8/8 passed |
| 引用方零改动 | ✅ `git status` 仅改 `markdown/` 目录 + 文档 |

## 沙箱测试说明

沙箱内 `cargo test -p homie-app` 中 2 个 `daemon_launch` 测试因 fixture daemon 的 socket bind
返回 `PermissionDenied`（EPERM）而失败，属沙箱网络限制预期。沙箱外复测 `daemon_launch::tests`
8/8 全部通过，确认拆分未影响 daemon 启动逻辑。

## 已知限制 / 延期

- 无。`store/mod.rs`（2,434 行）为后续最重拆分目标，建议留到最后。
