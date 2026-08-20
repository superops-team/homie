# OpenSpec Alignment — app-root-mod-split

## PRD → OpenSpec 对齐

| PRD 需求 | OpenSpec 任务 | 状态 |
|---------|--------------|------|
| 构造函数下沉 new.rs | T2 | ✅ |
| 键盘输入下沉 input.rs | T2 | ✅ |
| 会话派生下沉 sessions.rs | T2 | ✅ |
| 布局/缩放下沉 layout.rs | T2 | ✅ |
| 检查器下沉 inspector.rs | T2 | ✅ |
| 方法逐字迁移 | T1/T2（C5） | ✅ |
| 24 个私有方法提升 pub(crate)、new 保持 pub(crate) | T2（C5） | ✅ |
| 公共 API 不变 | T3（C2） | ✅ |
| 单文件 < 800 行 | T4（C3） | ✅ |
| 全量验证 | T4（C4） | ✅ |
| 证据 | T5（C6） | ✅ |

## 验收覆盖

- C1 目录化：T2（1,190 行拆为 5 子模块 + facade，最大 new.rs 390 行）。
- C2 引用方零改动：T3 code review。
- C3 行数阈值：T4。
- C4 编译/静态/测试：T4。
- C5 逐字迁移 + 可见性：T1/T2。
- C6 证据：T5。

无未映射需求，无漂移。
