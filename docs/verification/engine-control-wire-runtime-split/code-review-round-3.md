# Engine Control Wire/Runtime Split S2 代码评审报告

## 1. 评审范围

`control/codec.rs` 新增 + `control.rs` 抽取 `history_entry_to_wire`/`worktree_to_wire`，
纯机械搬迁。

## 2. 显性问题

| # | 问题 | 处理 |
|---|------|------|
| 1 | `mod codec;` 声明顺序被 `cargo fmt` 重排到 `mod wire;` 之前 | 已修：`cargo fmt` 后为 `mod codec;` → `mod wire;` 顺序，fmt --check 干净 |

## 3. 隐性问题

- 无。投影函数体一字未改，仅移动位置 + `pub(super)`；可见性限定在 `control` 模块内，无 API 泄漏。
- `history_entry_to_wire` 的 `kind` match 保持穷尽（ClaudeCode/Codex 两分支），无逻辑漂移。
- `worktree_to_wire` 字段一一对应，无丢失或重排。
- 测试断言 `DateMillis(1_700_000_000_000.0)` 精确锁定毫秒换算语义，防止 `as_secs`（秒）误用回归。

## 4. 功能验证覆盖审查

- FC-07 新增 6 个 focused tests 覆盖 kind 映射、标量保留、时间戳换算、缺失字段、worktree 全字段。
- FC-08 全量测试（含集成）保障 wire shape 与 daemon 行为不变。

## 5. 结论

显性问题全部修复，无残留 P0/P1。可进入提交。
