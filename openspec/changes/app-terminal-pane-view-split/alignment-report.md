# OpenSpec Alignment — app-terminal-pane-view-split

## PRD → OpenSpec 对齐

| PRD 需求 | OpenSpec 任务 | 状态 |
|---------|--------------|------|
| 自由辅助函数下沉 buttons.rs | T2 | ✅ |
| header/sidebar 揭示控件下沉 chrome.rs | T2 | ✅ |
| grid/overlay 下沉 grid.rs | T2 | ✅ |
| find bar 下沉 find_bar.rs | T2 | ✅ |
| exit/archived 状态下沉 status.rs | T2 | ✅ |
| 方法/函数逐字迁移 | T1/T2（C5） | ✅ |
| 8 渲染方法 + 4 辅助函数提升 pub(crate) | T2（C5） | ✅ |
| 公共 API 不变 | T3（C2） | ✅ |
| 单文件 < 800 行 | T4（C3） | ✅ |
| 全量验证 | T4（C4） | ✅ |
| 证据 | T5（C6） | ✅ |

## 验收覆盖

- C1 目录化：T2（863 行拆为 6 子模块 + facade，最大 grid.rs 200 行）。
- C2 引用方零改动：T3 code review。
- C3 行数阈值：T4。
- C4 编译/静态/测试：T4。
- C5 逐字迁移 + 可见性：T1/T2。
- C6 证据：T5。

无未映射需求，无漂移。
