# Diri Design Token/Glyph 对齐设计文档

```yaml
change_id: diri-design-token-glyphs
beads: homie-8x8
target_rows:
  - UI-009
```

## 1. 概述

`UI-009` 已有基础 token，但缺少品牌标识、icon/status glyph catalog、gallery 数据，导致 app 内元素仍可能散落硬编码。

## 2. 目标

- 在 `homie-ui` 增加 brand mark 和 icon/status glyph catalog。
- 为 status glyph 提供 symbol、label、tone。
- 提供 gallery entries，作为后续 screenshot gate 的数据源。
- 用测试覆盖 token/glyph catalog。

## 3. 非目标

- 不生成 bitmap/icon asset。
- 不声明 screenshot gate 已完成。

## 4. 验收

- `cargo test -p homie-ui --tests -- --nocapture`
- `cargo clippy -p homie-ui --all-targets -- -D warnings`
- `make parity-lock`

