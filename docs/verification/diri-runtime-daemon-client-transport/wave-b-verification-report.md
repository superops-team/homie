# Wave B 实现验证报告

## 1. 范围

- Change ID：`diri-runtime-daemon-client-transport`
- Beads：`homie-nep`
- 分支：`feature/diri-runtime-daemon-client-transport`
- 验证日期：2026-08-08
- 覆盖任务：OpenSpec Tasks 1-14

本报告只记录主线程独立复核得到的结果。Wave C 入口迁移、跨入口 E2E、全 workspace 门禁和 T-102 不属于本报告的通过范围。

## 2. 已验证能力

- `homie-proto` 提供固定大端二进制帧、JSON/raw payload、runtime DTO 和唯一 capability registry。
- `homie-client` 是 Tokio async transport client，normal dependency 不包含 `homie-runtime`、`homie-storage` 或 `homie-remote`。
- runtime control、event 和 terminal 通过同一个真实 UDS 连接工作。
- event producer 支持 replay、gap reset、snapshot recovery 和 1024 条 ring。
- terminal producer 支持 absolute offset replay、full-grid barrier、raw input、resize 和 shared source。
- writer 只在 socket 写成功后推进 delivered position，并用有界 high queue flush barrier 保证 shutdown ACK。
- daemon 实施 owner-only runtime path、singleton lock、stale socket identity-safe cleanup、可执行文件 SHA-256 和 UUID v7 instance ID。
- admin shutdown、SIGINT 和 SIGTERM 均执行 durable prepare、停止 accept、drain 已接收连接、join lane/actor，并保留 holder session。

## 3. 主线程门禁结果

| 门禁 | 结果 |
|---|---|
| `cargo test -p homie-runtime --bin homie-runtime-daemon` | 3/3 通过 |
| `cargo test -p homie-runtime daemon::tests` | 23/23 通过 |
| `cargo test -p homie-runtime runtime_actor::tests` | 6/6 通过 |
| `cargo test -p homie-runtime --test runtime_dispatcher` | 17/17 通过 |
| `cargo test -p homie-runtime --test server_control` | 35/35 通过 |
| `cargo test -p homie-runtime --test server_streams` | 18/18 通过 |
| `cargo test -p homie-runtime --test daemon_process -- --test-threads=1` | 12/12 通过 |
| `cargo fmt -p homie-runtime -p homie-client -- --check` | 通过 |
| `cargo clippy -p homie-runtime -p homie-client --all-targets -- -D warnings` | 通过 |
| `cargo check -p homie-runtime --bins` | 通过 |

此前独立 client gate 结果为 53/53 通过，且 normal dependency tree 不含 runtime、storage 或 remote implementation。

## 4. 复核中补出的边界修复

1. terminal reset 不再用 server offset 覆盖 client 已确认的连续 cursor，避免重复或跳过输出。
2. low-priority writer 只在真实写入 socket 后推进 delivered offset，避免 queue overflow 后从未发送的位置恢复。
3. shutdown response 在触发 server shutdown 前等待 high-priority flush barrier。
4. runtime actor 进入 drain 后仍允许已启动 mutation commit 和最终 shutdown request。
5. stale owner socket 的任意不可连接错误均在 inode、device、type 和 UID 二次校验后清理；`NotFound` race 视为已清理。
6. signal prepare 遇 actor queue 瞬时满时以 10 ms 异步间隔重试，不忙等、不跳过 durable prepare。

## 5. 未关闭风险

- 一次串联命令在所有目标测试通过后仍被 Trae sandbox 判为失败，因为既有 production history test 访问了沙箱外 `~/.codex/state_5.sqlite` 和 WAL。该结果未记为全量通过，后续在 Task 20 分类处理。
- 既有 T-102 holder prompt offset 和 persisted status 语义问题仍未关闭。
- CLI、MCP 和 GPUI 当前尚未完成 async client 入口迁移；same-daemon-instance 只能在 Wave C/D 后验证。
- package closure、release trust 和 notarization 尚未完成。

## 6. 结论

Wave B 的协议、runtime daemon、真实 UDS server、event/terminal producer、async client 和 daemon lifecycle 已满足 Tasks 1-14 的 focused gate。Bead `homie-nep` 必须保持 `IN_PROGRESS`，直到 Tasks 15-20 及 release-readiness evidence 完成。
