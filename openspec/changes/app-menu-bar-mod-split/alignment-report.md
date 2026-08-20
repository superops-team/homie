# OpenSpec Alignment — app-menu-bar-mod-split

## PRD → OpenSpec 对齐

| PRD 需求 | OpenSpec 任务 | 状态 |
|---------|--------------|------|
| 数据/样式模型下沉 model.rs | T1 | ✅ |
| body 行渲染下沉 render.rs | T2 | ✅ |
| objc2 事件目标类下沉 target.rs | T3 | ✅ |
| NativeMenuBar 生命周期 facade 收尾 | T4 | ✅ |
| 公共 API 不变 | T1-T5（C2） | ✅ |
| 单文件 < 800 行 | T5（C3） | ✅ |
| 全量验证 | T5（C4/C6） | ✅ |

## 验收覆盖

- C1 目录化：T4。
- C2 引用方零改动：T5 code review。
- C3 行数阈值：T5。
- C4 编译/静态/构建：T5。
- C5 可见性管控：T1-T3（跨模块符号均 `pub(super)`，无 `pub` 泄漏）。
- C6 证据：T5。

无未映射需求，无漂移。
