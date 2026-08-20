# OpenSpec Alignment — app-terminal-pane-mod-split

## PRD → OpenSpec 对齐

| PRD 需求 | OpenSpec 任务 | 状态 |
|---------|--------------|------|
| 事件分发/网格重排下沉 events.rs | T1 | ✅ |
| 键盘输入下沉 input.rs | T2 | ✅ |
| 查找下沉 find.rs | T3 | ✅ |
| 网格几何+缩放下沉 geometry.rs | T4 | ✅ |
| 剪贴板下沉 clipboard.rs | T5 | ✅ |
| 滚动回取+滚动下沉 scroll.rs | T6 | ✅ |
| 公共 API 不变 | T1-T7（C2） | ✅ |
| 单文件 < 800 行 | T7（C3） | ✅ |
| 全量验证 | T7（C4/C6） | ✅ |

## 验收覆盖

- C1 目录化：T1-T6。
- C2 引用方零改动：T7 code review。
- C3 行数阈值：T7。
- C4 编译/静态/测试：T7。
- C5 可见性管控：T1-T6（仅跨模块调用方法升 `pub(super)`）。
- C6 证据：T7。

无未映射需求，无漂移。
