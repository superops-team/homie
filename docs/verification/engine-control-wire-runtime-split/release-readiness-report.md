# Engine Control Wire/Runtime Split 发布就绪报告

## 1. 就绪结论

S1（wire codec）、S2（codec 投影）、S3（runtime 生命周期）、S4-a（handler 机械下沉）、
S4-b（worktree_overview + prompt-injection 领域逻辑下沉）全部切片就绪，可提交。

## 2. 交付物

- `homie/crates/homie-engine/src/control/wire.rs`（170 行）：wire 编解码 + 错误映射
- `homie/crates/homie-engine/src/control/codec.rs`（122 行）：proto↔domain 投影
- `homie/crates/homie-engine/src/control/runtime.rs`（424 行）：bind 循环、订阅句柄、
  连接守卫、空闲关停、远程恢复
- `homie/crates/homie-engine/src/control/handlers.rs`（1,582 行）：43 个 handler 方法 +
  剩余自由函数/常量
- `homie/crates/homie-engine/src/control/inject.rs`（408 行）：初始 prompt 注入领域
- `homie/crates/homie-engine/src/control/tests.rs`（886 行）：control 模块 tests 随迁
- `homie/crates/homie-engine/src/git.rs`：新增 `worktree_overview` 纯领域函数（约 110 行）
- `homie/crates/homie-engine/src/control.rs`（抽取，3,802 → 460 行）

## 3. 验证汇总

| 门禁 | 结果 |
|------|------|
| `cargo test -p homie-engine` | 278 lib（3 ignored）+ 集成全绿，0 failed |
| `cargo fmt -p homie-engine -- --check` | clean |
| `cargo check -p homie-engine` | 无 warning |
| wire.rs 无重依赖 | 通过 |
| codec.rs 无重依赖 | 通过 |
| runtime.rs 无 transport 泄漏 | 通过 |
| control.rs < 800 行 | 通过（460 行） |
| handlers.rs 无 transport 层 | 通过 |
| inject.rs 无 transport 依赖 | 通过 |
| git::worktree_overview 无 ControlServer 依赖 | 通过 |
| wire shape（method/参数/返回 JSON）不变 | 通过（协议未改） |

## 4. 范围说明

`engine-control-wire-runtime-split` 已完成 S1+S2+S3+S4-a+S4-b。S4-b 领域逻辑下沉交付两个
增量：`worktree_overview` → `crate::git`，`prompt-injection` → `control/inject.rs`。

## 5. 已知限制（残余风险，留待后续 PRD/切片）

- 未新增协议 golden fixture（属 `protocol-contract-golden-fixtures`）。
- `session_spawn` / `session_spawn_remote` / `session_migrate` / `session_resume` /
  `session_resume_from_history` / `session_reopen_last` 的 spawn/resume/migrate 领域逻辑
  仍内联在 handler，且与 `ControlServer` 字段（injection/socket_path/logs_dir/holder/remote）
  耦合较深。PRD 3.3 已规划下沉到 `registry`/`session`/`remote/manager.rs`，但验收标准
  （control.rs < 800 行、wire shape 不变、测试全绿）已满足，此项作为后续切片继续下沉，
  不阻塞本 change_id 关闭。
