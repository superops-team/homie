# OpenSpec Alignment — mcp-host-module-split

## PRD → OpenSpec 对齐

| PRD 需求 | OpenSpec 任务 | 状态 |
|---------|--------------|------|
| 谱系下沉 lineage.rs | T2 | ✅ |
| 注册表宿主 + 工具分发下沉 registry.rs | T2 | ✅ |
| 类型/函数/常量逐字迁移 | T1/T2（C5） | ✅ |
| 公共 API 不变（pub use 再导出） | T3（C2） | ✅ |
| Relation/relation_word/Lineage 提升 pub(crate) | T3（C5） | ✅ |
| 单文件 < 800 行 | T4（C3） | ✅ |
| 全量验证 | T4（C4） | ✅ |
| 证据 | T5（C6） | ✅ |

## 验收覆盖

- C1 目录化：T2（863 行拆为 3 子模块 + facade，最大 registry.rs 721 行）。
- C2 引用方零改动：T3 code review + `cargo check --workspace`。
- C3 行数阈值：T4。
- C4 编译/静态/测试：T4。
- C5 逐字迁移 + 可见性：T1/T2/T3。
- C6 证据：T5。

无未映射需求，无漂移。
