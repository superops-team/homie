# OpenSpec Alignment — app-sidebar-sections-split

## PRD → OpenSpec 对齐

| PRD 需求 | OpenSpec 任务 | 状态 |
|---------|--------------|------|
| 顶栏与空状态下沉 chrome.rs | T2 | ✅ |
| 项目区块下沉 project.rs | T2 | ✅ |
| 会话行与折叠下沉 session.rs | T2 | ✅ |
| 归档下沉 archive.rs | T2 | ✅ |
| 页脚下沉 footer.rs | T2 | ✅ |
| 方法逐字迁移、`pub(crate)` 不变 | T1/T2（C5） | ✅ |
| 公共 API 不变 | T3（C2） | ✅ |
| 单文件 < 800 行 | T4（C3） | ✅ |
| 全量验证 | T4（C4） | ✅ |
| 证据 | T5（C6） | ✅ |

## 验收覆盖

- C1 目录化：T2（1,202 行拆为 5 子模块 + facade，最大 377 行）。
- C2 引用方零改动：T3 code review。
- C3 行数阈值：T4。
- C4 编译/静态/测试：T4。
- C5 逐字迁移 + 可见性：T1/T2。
- C6 证据：T5。

无未映射需求，无漂移。
