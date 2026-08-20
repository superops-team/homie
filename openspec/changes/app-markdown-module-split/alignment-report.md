# OpenSpec Alignment — app-markdown-module-split

## PRD → OpenSpec 对齐

| PRD 需求 | OpenSpec 任务 | 状态 |
|---------|--------------|------|
| 块级解析下沉 block.rs | T1 | ✅ |
| 行内解析下沉 inline.rs | T2 | ✅ |
| 测试原样下沉 tests.rs | T3 | ✅ |
| 公共 API 不变 | T1-T4（C2） | ✅ |
| 单文件 < 800 行 | T4（C3） | ✅ |
| 全量验证 | T4（C4/C6） | ✅ |

## 验收覆盖

- C1 目录化：T1-T3。
- C2 引用方零改动：T4 code review。
- C3 行数阈值：T4。
- C4 编译/静态/测试：T4。
- C5 可见性管控：T1-T2（仅 `parse_blocks`/`parse_inline` 升 `pub(super)`）。
- C6 证据：T4。

无未映射需求，无漂移。
