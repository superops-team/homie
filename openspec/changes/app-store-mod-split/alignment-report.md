# OpenSpec Alignment — app-store-mod-split

## PRD → OpenSpec 对齐

| PRD 需求 | OpenSpec 任务 | 状态 |
|---------|--------------|------|
| 构造 + 基础访问器下沉 lifecycle.rs | T1 | ✅ |
| 主机/catalog/migration/sync/repo/directory/prefs 下沉 hosts.rs | T2 | ✅ |
| sidebar 排序/pin/collapse/projection 下沉 ordering.rs | T3 | ✅ |
| hydrate/handle_event/upsert 下沉 events.rs | T4 | ✅ |
| switcher/overview/snapshot 下沉 switcher.rs | T5 | ✅ |
| 会话生命周期下沉 sessions.rs | T6 | ✅ |
| 导航 reconcile 下沉 navigation.rs | T7 | ✅ |
| StoreRuntime + run_effects 下沉 runtime.rs | T8 | ✅ |
| 公共 API 不变 | T1-T8（C2） | ✅ |
| 单文件 < 800 行 | T9（C3） | ✅ |
| 全量验证 | T9（C4/C6） | ✅ |

## 验收覆盖

- C1 目录化：T8（mod.rs 由 2,434 行瘦身为 352 行 facade）。
- C2 引用方零改动：T9 code review。
- C3 行数阈值：T9。
- C4 编译/静态/测试：T9。
- C5 可见性管控：T1-T8（跨模块符号均 `pub(super)`，无 `pub` 泄漏）。
- C6 证据：T9。

无未映射需求，无漂移。
