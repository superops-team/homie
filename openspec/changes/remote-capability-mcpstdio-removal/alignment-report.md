# Alignment Report — remote-capability-mcpstdio-removal

## PRD → OpenSpec 映射

| PRD 需求 | OpenSpec 任务 | 状态 |
|----------|---------------|------|
| FR-1 删除 `RemoteCapability::McpStdio` 变体 + wire_name 分支 | T1 | 完成 |
| 测试计划（proto/remote/engine 测试 + clippy） | T2 | 完成 |
| 证据 + 提交 + tag + 关 Beads | T3 | 完成（commit/tag/close 见收尾） |

## 一致性

- PRD §5 需求 FR-1 与 OpenSpec T1 一一对应。
- PRD §7 测试计划与 OpenSpec T2 对齐。
- 非目标（保留前瞻性能力变体、不 bump 协议版本、不新增兼容层）在实现中遵守。
