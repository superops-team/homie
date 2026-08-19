# code_intelligence.rs 模块拆分设计文档

## 1. 背景

2026-08 架构审计（`architecture-audit-governance-2026-08`）的模块降熵序列已先后拆分
`surface_shell/view.rs`、`terminal_pane/mod.rs`、`sidebar`、`inspector`、`session_surfaces.rs` 等
God Module。当前 homie-app 前几大单文件：

| 文件 | 行数 |
|------|------|
| `store/mod.rs` | 2,434 |
| `terminal_pane/mod.rs` | 1,486 |
| `git_review.rs` | 1,427 |
| `code_intelligence.rs` | 1,426 |

`code_intelligence.rs`（1,426 行）是「轻量工作区智能」模块，为原生文件浏览提供索引与引用解析，
内部混装：

1. 工作区发现 + 索引构建（`discover_workspace_root`/`build_index`/`git_paths`/`bytes_to_path`/
   `filesystem_paths`/`ignored_path`，约 570–730 行）；
2. 符号提取（`index_symbols`/`symbol_name`/`is_symbol_text_file`，约 730–877 行）；
3. 引用解析（`parse_reference_candidates`/`clean_wrappers`/`parse_reference_fragment`/
   `parse_file_uri`/`parse_line_fragment`/`parse_stack_reference`/`parse_colon_target`/
   `looks_like_path`/`percent_decode`/`hex_value`，约 878–1091 行）；
4. 文本/评分辅助（`lexical_normalize`/`source_lines`/`clamp_target`/`path_score`/`fuzzy_score`/
   `subsequence_score`/`excerpt`，约 1092–1218 行）；
5. 内联测试（`mod tests`，约 1219–1426 行，7 个测试）。

该模块唯一引用方为 `code_viewer.rs`，仅依赖公共 API（`CodeIntelligence`、`CodeIntelligenceError`、
`SearchHit`、`SearchHitKind`、`SourceSnapshot`、`SourceTarget`）。拆分边界清晰、依赖单一，
是下一个机械拆分的理想切片。

## 2. 目标

- 把 `code_intelligence.rs`（1,426 行）拆为目录化聚焦子模块，**单文件 < 800 行**。
- 索引构建、引用解析、文本/评分辅助、测试各自下沉到独立子模块，`mod.rs` 收尾为 facade。
- 公共 API 与运行时行为完全不变；`code_viewer.rs` 零改动。

## 3. 非目标

- 不重设计任何索引/解析/评分算法，不改运行语义。
- 不改公共类型路径（`crate::code_intelligence::{...}` 保持可用）。
- 不合并/删除任何既有方法；纯职责搬迁。
- 不触及任何 `specs/` 合同（本次不涉及长生命周期组件接口变更）。

## 4. 需求

### FR-1: 索引构建下沉

`discover_workspace_root` / `build_index` / `git_paths` / `bytes_to_path`（unix/not-unix）/
`filesystem_paths` / `ignored_path` / `index_symbols` / `symbol_name` / `is_symbol_text_file`
及私有数据结构 `WorkspaceIndex` / `IndexedFile` / `IndexedSymbol` 下沉到
`code_intelligence/index.rs`。

### FR-2: 引用解析下沉

`parse_reference_candidates` / `clean_wrappers` / `parse_reference_fragment` / `parse_file_uri` /
`parse_line_fragment` / `parse_stack_reference` / `parse_colon_target` / `looks_like_path` /
`percent_decode` / `hex_value` 及 `ParsedReference` 下沉到 `code_intelligence/reference.rs`。

### FR-3: 文本/评分辅助下沉

`lexical_normalize` / `source_lines` / `clamp_target` / `path_score` / `fuzzy_score` /
`subsequence_score` / `excerpt` 下沉到 `code_intelligence/text.rs`。

### FR-4: 测试下沉

内联 `mod tests` 下沉到 `code_intelligence/tests.rs`，`#[cfg(test)]` 保持。

### FR-5: mod.rs 收尾为 facade

`code_intelligence/mod.rs` 保留公共常量/类型（`MAX_SOURCE_BYTES`、`CodeIntelligence`、
`ResolvedReference`、`SourceTarget`、`SourceSnapshot`、`SourceLine`、`SearchHit`、
`SearchHitKind`、`SourceLanguage`、`CodeIntelligenceError`）+ `impl CodeIntelligence` +
子模块声明，行数 < 800。

### FR-6: 行为不变

拆分后 `cargo check -p homie-app`、`cargo test -p homie-app`、`cargo fmt --check` 全绿，
索引/解析/评分行为与拆分前等价；`code_viewer.rs` 零改动。

## 5. 涉及文件

- `homie/crates/homie-app/src/code_intelligence.rs`（拆分源，转为目录）
- `homie/crates/homie-app/src/code_intelligence/mod.rs`（新增，facade）
- `homie/crates/homie-app/src/code_intelligence/index.rs`（新增，索引构建）
- `homie/crates/homie-app/src/code_intelligence/reference.rs`（新增，引用解析）
- `homie/crates/homie-app/src/code_intelligence/text.rs`（新增，文本/评分辅助）
- `homie/crates/homie-app/src/code_intelligence/tests.rs`（新增，测试）

## 6. 验证计划

```bash
cargo fmt --check
cargo check -p homie-app
cargo clippy -p homie-app --all-targets
cargo test -p homie-app
```

人工验收：

1. 索引/引用解析/评分行为与拆分前等价（既有 7 个测试原样通过）。
2. 每个新子模块与 `mod.rs` 均 < 800 行。
3. `code_viewer.rs` 零改动，`crate::code_intelligence::{...}` 路径不变。

## 7. Beads

- change_id: `app-code-intelligence-module-split`
- 类型: task（机械拆分，行为不变）
- 优先级: P1（homie-app 组合根降熵）
- 上游: `architecture-audit-governance-2026-08`（模块降熵序列延续）
