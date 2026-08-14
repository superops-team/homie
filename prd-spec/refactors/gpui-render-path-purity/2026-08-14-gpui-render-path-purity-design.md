# GPUI render 路径纯化首个切片设计文档

## 1. 概述

### 1.1 问题

`Sidebar::render` 当前内联计算 session shortcut ranks。虽然这是纯计算，但逻辑嵌在 render 中，难以单独测试，也让后续将 projection 计算外移时缺少可复用的纯边界。

### 1.2 目标

1. 将 shortcut rank 计算抽为纯函数。
2. 为首 8 个 session 和最后一个 session 映射为 Cmd+1..9 的规则增加测试。
3. 保持 UI 行为不变。

### 1.3 非目标

- 不在本切片中移除 render 内的 `store.sidebar_projection()`。
- 不迁移 glyph lifecycle。
- 不重构 Sidebar 文件结构。

## 2. 验证

```bash
cargo test --manifest-path homie/Cargo.toml -p homie-app shortcut_rank -- --nocapture
cargo test --manifest-path homie/Cargo.toml -p homie-app sidebar -- --nocapture
(cd homie && cargo fmt --check)
git diff --check
```

## 3. Beads

- Beads: `homie-4fx`
- change_id: `gpui-render-path-purity`
