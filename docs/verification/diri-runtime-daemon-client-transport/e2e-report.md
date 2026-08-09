# Cross-Entry E2E 报告

```yaml
change_id: diri-runtime-daemon-client-transport
beads: homie-nep
date: 2026-08-08
status: pass
test: crates/homie-cli/tests/shared_daemon_e2e.rs
```

## 1. 执行结果

```text
cargo test -p homie-cli --test shared_daemon_e2e -- --test-threads=1
1 passed; 0 failed
```

测试使用 production `RuntimeBridge`、真实 CLI/MCP 子进程、真实 daemon/holder sibling binary、raw UDS client 和绝对 temp data directory，不使用 embedded runtime 或 production test mode。

## 2. 已验证流程

1. app bridge 与 CLI Hello 观察到同一个 `daemon_instance_id`。
2. CLI 创建的 session 同时可被 MCP 与 app projection 观察。
3. app 与 CLI 读取一致的 event cursor。
4. shutdown ACK 在连接关闭前送达，pending `events.wait` 只失败一次。
5. holder 在 daemon SIGTERM/restart 后继续存活。
6. replacement daemon 使用新的 instance ID，app 自动重连。
7. event stream 从已确认 cursor 恢复。
8. terminal stream 从最后确认的 absolute offset 重开，并重新建立 full-grid barrier。
9. restart 后 MCP 仍能通过同一 shared daemon 观察 session。

## 3. 安全与容量

同一 E2E 还验证：

- 第 65 个 active connection 在协议处理前被拒绝。
- 小于 frame header 和大于 `MAX_FRAME_LEN` 的长度被拒绝。
- 非零 flags 与未知 frame kind 被拒绝。
- hostile payload marker 不出现在 daemon logs、boot log 或 process arguments。
- app、CLI、MCP 不获得真实 provider credential。

## 4. 清理

fixture 在成功和 panic 路径都执行 session termination、daemon shutdown 和 bounded kill/reap fallback。最终独立检查：

```text
test_daemon_count=0
test_holder_count=0
```

用户真实 data directory 下既有的 packaged holder 不属于测试 fixture，未被修改或终止。

## 5. 范围边界

本报告证明 API-002 所需的 shared-daemon、reconnect、event resume 和 terminal attachment/reopen，可以将 API-002 提升为 `implemented`。T-102 的 direct supervisor holder adoption、完整 GPUI 交互、公证和 remote/node 不在本 E2E 的 pass 范围。
