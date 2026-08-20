# OpenSpec Alignment — app-sidebar-popover-split

## PRD → OpenSpec 对齐

| PRD 需求 | OpenSpec 任务 | 状态 |
|---------|--------------|------|
| 弹窗入口与外壳下沉 shell.rs | T2 | ✅ |
| 新 agent 弹窗下沉 new_agent.rs | T2 | ✅ |
| 目录选择器下沉 directory_picker.rs | T2 | ✅ |
| 更新菜单行下沉 update_menu.rs | T2 | ✅ |
| 账户弹窗下沉 account.rs | T2 | ✅ |
| 项目/会话操作菜单下沉 actions.rs | T2 | ✅ |
| 方法逐字迁移、`pub(crate)` 不变 | T1/T2（C5） | ✅ |
| 公共 API 不变 | T3（C2） | ✅ |
| 单文件 < 800 行 | T4（C3） | ✅ |
| 全量验证 | T4（C4） | ✅ |
| 证据 | T5（C6） | ✅ |

## 验收覆盖

- C1 目录化：T2（1,249 行拆为 6 子模块 + facade，最大 426 行）。
- C2 引用方零改动：T3 code review。
- C3 行数阈值：T4。
- C4 编译/静态/测试：T4。
- C5 逐字迁移 + 可见性：T1/T2。
- C6 证据：T5。

无未映射需求，无漂移。
