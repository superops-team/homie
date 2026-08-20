# OpenSpec Alignment — app-store-tests-split

## PRD → OpenSpec 对齐

| PRD 需求 | OpenSpec 任务 | 状态 |
|---------|--------------|------|
| switcher/overview 测试下沉 switcher.rs | T2 | ✅ |
| sidebar 排序/pin/collapse/projection 测试下沉 ordering.rs（含 rows） | T2 | ✅ |
| hydrate/handle_event 等测试下沉 events.rs | T2 | ✅ |
| close/resume/auto_resume 等测试下沉 sessions.rs | T2 | ✅ |
| attention/needs_input 测试下沉 attention.rs | T2 | ✅ |
| 主机/默认主机/repo targeting 等测试下沉 hosts.rs | T2 | ✅ |
| StoreRuntime 惰性运行时测试下沉 runtime.rs | T2 | ✅ |
| 共享辅助下沉 mod.rs | T2 | ✅ |
| `#[test]` 属性完整保留 | T1/T2（C5） | ✅ |
| 逐字迁移、测试行为零变更 | T3（C2） | ✅ |
| 单文件 < 800 行 | T4（C3） | ✅ |
| 全量验证 | T4（C4） | ✅ |
| 证据 | T5（C6） | ✅ |

## 验收覆盖

- C1 目录化：T2/T4（1,658 行拆为 8 文件，最大 395 行）。
- C2 逐字迁移：T3 code review。
- C3 行数阈值：T4。
- C4 编译/静态/测试：T4。
- C5 `#[test]` 完整 + 辅助下沉：T1/T2。
- C6 证据：T5。

无未映射需求，无漂移。
