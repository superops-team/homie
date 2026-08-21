# OpenSpec Alignment — engine-pr-monitor-module-split

## PRD → OpenSpec 对齐

| PRD 需求 | OpenSpec 任务 | 状态 |
|---------|--------------|------|
| wake 失效下沉 wake.rs | T2 | ✅ |
| GitHub 解析下沉 github.rs | T2 | ✅ |
| 监控主循环下沉 monitor.rs | T2 | ✅ |
| 函数逐字迁移 | T1/T2（C5） | ✅ |
| 公共 API 不变（pub use 再导出） | T3（C2） | ✅ |
| resolve_gh 提升 pub(crate) 共享 | T3（C5） | ✅ |
| 单文件 < 800 行 | T4（C3） | ✅ |
| 全量验证 | T4（C4） | ✅ |
| 证据 | T5（C6） | ✅ |

## 验收覆盖

- C1 目录化：T2（822 行拆为 4 子模块 + facade，最大 monitor.rs 248 行）。
- C2 引用方零改动：T3 code review + `cargo check --workspace`。
- C3 行数阈值：T4。
- C4 编译/静态/测试：T4。
- C5 逐字迁移 + 可见性：T1/T2/T3。
- C6 证据：T5。

无未映射需求，无漂移。
