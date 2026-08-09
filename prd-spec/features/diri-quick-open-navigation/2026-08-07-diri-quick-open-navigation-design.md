# Diri Quick Open 导航对齐设计文档

```yaml
change_id: diri-quick-open-navigation
beads: homie-5ya
target_rows:
  - UI-005
```

## 1. 概述

`UI-005` 仍为 `partial`。Homie 已有 command palette 和 `homie-ui` fuzzy ranking/history model，但 `OpenQuickOpen` 仍只是本地 notice，未显示 Quick Open/switcher/history surface。

## 2. 目标

- `homie-app` 增加 Quick Open surface。
- Quick Open 使用 `homie-ui::rank_items` 对真实 session rows 和固定 navigation actions 排序。
- `OpenQuickOpen` 打开 surface，不再 notice-only。
- 选择 session item 时调用现有 `select_session`。

## 3. 非目标

- 不实现文件索引和 transcript scanner。
- 不把 `UI-005` 标为 implemented，直到 navigation/history E2E 完成。

## 4. 验收标准

- `cargo test -p homie-app --tests -- --nocapture`
- `cargo test -p homie-ui --tests -- --nocapture`
- `cargo clippy -p homie-app -p homie-ui --all-targets -- -D warnings`
- `make parity-lock`

