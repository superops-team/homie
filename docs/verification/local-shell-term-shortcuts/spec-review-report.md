# 本地 Shell TERM 快捷键修复 Spec Review 报告

## 1. 总体结论

- 可行性：高。
- 最大风险：只修客户端 key adapter，忽略真实根因在本地 PTY child environment。
- 推荐方向：在 Engine 本地 shell/generic argv 的 `PtySpec` 环境准备阶段补齐 `TERM=xterm-256color`，并用 Engine 级测试验证。

## 2. 问题清单

| 优先级 | 维度 | 问题 | 影响 | 处理 |
|--------|------|------|------|------|
| P1 | 根因定位 | 可能误以为 Ctrl-L 没有到达 TerminalPane | 会修错层，无法解决 shell/TUI 行为 | PRD 已记录 app 与 engine trace，确认 `[12]` 已写入 held PTY |
| P1 | 验证范围 | 只测编码不测 shell child env | 无法证明 shell 快捷键完整恢复 | PRD 要求验证新 shell session 的 `$TERM` |
| P2 | 兼容性 | 修改 manifest agent env 可能影响 agent 行为 | 超出本 bugfix 范围 | PRD 明确只修 shell / explicit argv 路径 |

## 3. 结论

无 P0/P1 阻塞。可以进入功能验证 Case 和 OpenSpec 拆解。
