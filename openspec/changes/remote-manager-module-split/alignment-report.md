# OpenSpec Alignment — remote-manager-module-split

## PRD → OpenSpec 对齐

| PRD 需求 | OpenSpec 任务 | 状态 |
|---------|--------------|------|
| 制品目录与清单下沉 catalog.rs | T2 | ✅ |
| 远程管理器 + Helper 引导/会话管理下沉 runtime.rs | T2 | ✅ |
| 控制目录校验与规范化下沉 control_dir.rs | T2 | ✅ |
| JSON 行解析 / 随机十六进制 / 持久化分类下沉 util.rs | T2 | ✅ |
| 类型/函数/常量逐字迁移 | T1/T2（C5） | ✅ |
| 公共 API 不变（pub use 再导出） | T3（C2） | ✅ |
| 跨模块私有辅助提升 pub(crate) | T2/T3（C5） | ✅ |
| 单文件 < 800 行 | T4（C3） | ✅ |
| 全量验证 | T4（C4） | ✅ |
| 证据 | T5（C6） | ✅ |

## 验收覆盖

- C1 目录化：T2（1099 行拆为 4 子模块 + facade，最大 runtime.rs 617 行）。
- C2 引用方零改动：T3 code review + `cargo check --workspace`。
- C3 行数阈值：T4。
- C4 编译/静态/测试：T4。
- C5 逐字迁移 + 可见性：T1/T2/T3。
- C6 证据：T5。

无未映射需求，无漂移。
