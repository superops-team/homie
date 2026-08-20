# OpenSpec Alignment — app-launcher-mod-split

## PRD → OpenSpec 对齐

| PRD 需求 | OpenSpec 任务 | 状态 |
|---------|--------------|------|
| 渲染方法 + Render impl 下沉 render.rs | T1 | ✅ |
| 测试原样下沉 tests.rs | T2 | ✅ |
| 公共 API 不变 | T1-T3（C2） | ✅ |
| 单文件 < 800 行 | T3（C3） | ✅ |
| 全量验证 | T3（C4/C6） | ✅ |

## 验收覆盖

- C1 目录化：T1-T2。
- C2 引用方零改动：T3 code review。
- C3 行数阈值：T3。
- C4 编译/静态/测试：T3。
- C5 可见性管控：T1（零 `pub(super)` 新增）。
- C6 证据：T3。

无未映射需求，无漂移。
