# OpenSpec Alignment — app-navigation-module-split

## PRD → OpenSpec 对齐

| PRD 需求 | OpenSpec 任务 | 状态 |
|---------|--------------|------|
| 目录化拆分 | T1 | ✅ |
| 渲染下沉 render.rs | T1 | ✅ |
| 目录索引下沉 index.rs | T2 | ✅ |
| 测试下沉 tests.rs | T3 | ✅ |
| 公共 API 不变 | T1/T2/T3/T4（C2） | ✅ |
| 单文件 < 800 行 | T3（C3） | ✅ |
| 全量验证 | T4（C4/C6） | ✅ |

## 验收覆盖

- C1 目录化：T1。
- C2 引用方零改动：T4 code review。
- C3 行数阈值：T3。
- C4 编译/静态/测试：T4。
- C5 可见性管控：T1/T2（仅 `render_overlay`/`relative_parent`/`load_cached_index`/
  `refresh_directory_index` 升 `pub(super)`）。
- C6 证据：T4。

无未映射需求，无漂移。
