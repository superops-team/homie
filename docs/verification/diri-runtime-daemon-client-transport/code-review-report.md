# Code Review Report

## 1. 审查范围

- Change：`diri-runtime-daemon-client-transport`
- 重点模块：runtime connection/lifecycle、async client streams、GPUI runtime bridge、CLI/MCP、package closure
- 参考规则：PRD、OpenSpec、component specs、`AGENTS.md`
- 审查方式：第一轮调用链审查与修复；第二轮对抗式复核

## 2. 旧问题复核

Wave B/D 期间已报告的 heartbeat nonce、event recovery、terminal offset/grid、shutdown long-poll、MCP atomic spawn、app queue backpressure 和 package closure 问题均保持 `fixed`。focused tests 与 cross-entry E2E 未发现回归。

## 3. Findings

| ID | 严重度 | 类别 | 位置 | 证据与影响 | 处置 |
|---|---|---|---|---|---|
| CR-01 | high | Correctness | `crates/homie-app/src/runtime_bridge.rs` | 初始 snapshot/subscribe 失败或 event stream 结束后变为 `None`，原 select 分支永久 pending，app 不再接收 CLI/MCP runtime events | fixed：Connected 且 stream 缺失时重新 snapshot/subscribe，并加入 250ms bounded retry；mock daemon 主动 close 后 seq=9 恢复测试通过 |
| CR-02 | high | Correctness | `crates/homie-app/src/runtime_bridge.rs`, `crates/homie-app/src/main.rs` | terminal unavailable 发布 confirmed offset，但重新 `SelectSession` 固定从 0 打开，可能重复全量 replay 并再次 overflow | fixed：`RuntimeCommand::SelectSession` 携带 `output_offset`；attachment state 按 session 保存 retry offset；mock server 校验 offset=17 |
| CR-03 | high | Error Handling | `crates/homie-runtime/src/lib.rs`, `crates/homie-storage/src/lib.rs` | session INSERT 后 holder launch/readiness/status update 失败没有补偿，可能留下持久 session 或未受管 holder | fixed：readiness 失败 kill/reap child；launch/status 失败删除 session；status 失败先停止 holder；missing-holder RED/GREEN 用例通过 |
| CR-04 | medium | Error Handling | `crates/homie-app/src/main.rs` | main 丢弃 `RuntimeBridge::dispatch` 的 backpressure/unavailable，terminal attach 可永久停留 pending，其他命令静默丢失 | fixed：dispatch 返回显式 bool 并投影 notice；attach dispatch 失败清 pending；spawn/paste/resize 只在成功入队后标 queued/cached |

## 4. 对抗式复盘

第二轮复查了以下反例：

- event stream 在初次 subscribe 失败、server close、client reconnect 三条路径下是否可恢复；
- event retry 是否 busy-loop 或重复保留旧 stream；
- terminal selection change、retry offset 和 dispatch failure 是否形成 stale pending；
- holder 在 `spawn`、readiness、status persistence 三个失败点是否可能残留；
- 第一轮修复是否破坏 cross-entry restart、app queue backpressure 或 package closure。

结论：未发现新增 confirmed finding。T-102 holder adoption/prompt-offset 是已批准的 out-of-scope blocker，不作为本轮新 finding，也未通过修改产品语义掩盖。

## 5. 修复摘要

- 增加 event projection 自动重建。
- terminal reopen 使用最后确认 offset。
- app 显式消费 command queue 错误。
- spawn 失败执行 holder/session 补偿清理。
- 测试覆盖 server-close recovery、offset wire value、dispatch pending 清理和 holder launch rollback。

## 6. 验证结果

| 命令 | 结果 |
|---|---|
| `cargo test -p homie-app --tests -- --test-threads=1` | pass |
| `cargo test -p homie-cli --test shared_daemon_e2e -- --test-threads=1` | 1/1 pass |
| `cargo test -p homie-runtime --test runtime_dispatcher -- --test-threads=1` | 17/17 pass |
| `cargo test -p homie-runtime --test session_lifecycle runtime_holder_launch_failure_removes_created_session -- --exact --test-threads=1` | 1/1 pass |
| `cargo test -p homie-storage --test local_basic_v1 -- --test-threads=1` | 4/4 pass |
| `cargo clippy -p homie-app -p homie-runtime -p homie-storage -p homie-cli --all-targets -- -D warnings` | pass |
| `cargo fmt --all -- --check` | pass |

## 7. 剩余风险

- packaged GUI launch 和 notarization 仍由 package/release change 验证。
- T-102 的 3 个 holder/PTY failures 保持 `partial/blocked`。
- API-005 完整 permission profile/recursive lineage 仍是既有 parity debt，不属于本 transport change。
