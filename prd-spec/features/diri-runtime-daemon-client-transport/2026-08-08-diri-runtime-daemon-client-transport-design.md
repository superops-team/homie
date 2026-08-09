# Diri Runtime Daemon 与统一多路复用 Client Transport 设计

```yaml
change_id: diri-runtime-daemon-client-transport
status: approved
beads: homie-nep
parent_change_id: diri-7ba3407-parity-rebaseline
master_task: T-101
baseline_repository: diri/
baseline_commit: 7ba3407
```

## 1. 概述

### 1.1 背景

Homie 当前的 `HomieClient` 不是进程间 client。它直接持有 `RuntimeSupervisor`，并且依赖 `homie-runtime`、`homie-storage` 和部分 remote/runtime helper。app、CLI 和 MCP 每次调用 `HomieClient::open(data_dir)` 都可能在自己的进程内打开 runtime 和 SQLite。

这造成以下问题：

- app、CLI、MCP 不是同一个 live session registry 的消费者。
- client API 同时承担 transport、runtime dispatch、storage query、git/worktree 和 history scanning。
- runtime crash/restart、heartbeat、request correlation、event resume 和 attachment recovery 无法真实验证。
- `Method::ALL`、client dispatcher 和 MCP/CLI 能力目录不一致。
- app 首帧执行同步 storage/runtime/process 工作。
- terminal 只能通过同步 snapshot/read-output 获取数据，没有独立且可恢复的数据流。

Diri `7ba3407` 已验证的模式是独立 daemon、UDS control、请求关联、heartbeat/reconnect、event resume 和 terminal data channel。本 change 保留这些行为目标，但采用更适合 Homie 长期架构的单 UDS 统一二进制多路复用协议，不追求 Diri wire compatibility。

### 1.2 目标

1. 新增独立 `homie-runtime-daemon`，成为 live runtime/session/PTY/event 的唯一进程 owner。
2. 将 `homie-client` 改为纯 Tokio async transport client，删除对 runtime/storage 的生产依赖。
3. 使用单 UDS、统一 length-prefixed frame 和 stream ID，多路复用 control、events 和多个 terminal stream。
4. 提供显式 `RuntimeLauncher`，由 app/CLI 决定是否启动 daemon；client connect 不产生进程副作用。
5. 支持 hello/capability、request correlation、heartbeat、reconnect、event gap recovery、stream reset 和 bounded backpressure。
6. 将当前真实可执行的 client dispatcher 移入 daemon，能力目录只公布有 handler 的方法/stream。
7. 迁移 app、CLI 和 MCP，使其连接同一个 daemon。
8. 删除 production embedded runtime client、同步 facade 和 client direct-storage/runtime dispatch。

### 1.3 非目标

- 不在本 change 中实现 19-agent manifest-driven spawn；由 T-102 `diri-agent-session-runtime` 负责。
- 不关闭当前 holder adoption/PTY `detached` 回归；该回归作为 T-102 blocker 保留。
- 不在本 change 中删除 app/CLI 的所有直接 storage 使用；settings、doctor、usage 等 durable facts 由 T-103 `diri-storage-core-facts` 迁移。
- 不实现 remote TCP、Tailscale、node protocol、port forward 或 remote token。
- 不一次补齐 `Method::ALL` 中的 LLM、task、memory、browser、remote 等方法。
- 不保留 Diri NDJSON + 独立 attachment socket 的 wire compatibility。
- 不新增环境变量、production test mode 或旧 in-process fallback。

## 2. 用户场景

### 场景 1：app、CLI 和 MCP 共享同一 runtime

**Given** 一个 data dir 对应的 daemon 已启动  
**When** app、CLI 和 MCP 分别连接该 data dir 的 endpoint  
**Then** 三者通过同一 daemon 观察相同 session、event sequence 和 capability

### 场景 2：daemon 尚未启动

**Given** 用户从 app 或 CLI 首次访问 Homie runtime  
**When** 调用方使用显式 launcher 检测到 endpoint 不可达  
**Then** launcher 使用绝对 daemon 路径启动独立进程，client 在后台重连，调用方线程不等待 daemon 完成启动

### 场景 3：daemon 重启

**Given** client 已订阅 events 并打开 terminal stream  
**When** daemon 进程退出后由 launcher 或 service 重新启动  
**Then** client 失败所有旧 pending request、重新 hello、恢复 event cursor，并按各 stream 的 last confirmed offset 重开 terminal stream

### 场景 4：某个 terminal consumer 过慢

**Given** 同一连接承载多个 session stream  
**When** 一个 stream 的有界接收队列满  
**Then** 只 reset 该 stream，并返回 last confirmed offset；control、events 和其他 session stream 继续工作

### 场景 5：事件游标超出 replay ring

**Given** client 的 last event seq 早于 daemon 可回放的 oldest seq  
**When** client 重新订阅 events  
**Then** daemon 返回 event gap，client 请求 `state.snapshot` 并从最新 seq 重建 projection

### 场景 6：发现未实现方法

**Given** proto 中存在未来方法常量，但当前 daemon 没有 handler  
**When** client 完成 hello 或调用该方法  
**Then** hello capabilities 不包含该方法，直接调用返回 `method_not_found`，不得返回 generic runtime error

## 3. 功能需求

### FR-01：每 Data Dir 单实例 Runtime Paths

- endpoint、lock、boot log 和 daemon log 必须从绝对 data dir 派生。
- 固定路径：
  - `<data-dir>/runtime/daemon.sock`
  - `<data-dir>/runtime/daemon.lock`
  - `<data-dir>/runtime/daemon.boot.log`
  - `<data-dir>/runtime/daemon.log`
- runtime directory 权限必须为 `0700`。
- socket 和 lock 权限必须为 `0600`。
- 同一个 data dir 只能存在一个 daemon owner。
- 测试使用临时绝对 data dir，不读取真实用户 HOME。

### FR-02：显式 RuntimeLauncher

- `HomieClient::connect` 只连接 endpoint，不得启动进程。
- app 和 CLI 必须显式调用共享 `RuntimeLauncher::ensure_running`。
- launcher 输入必须包含：
  - absolute data dir；
  - absolute daemon executable path；
  - startup probe timeout；
  - fixed boot log path。
- daemon executable 的生产解析只允许：
  - 调用方传入的绝对路径；
  - app bundle 内固定绝对路径；
  - 当前 executable canonical sibling。
- 不允许通过环境变量覆盖 daemon、socket 或 data dir。
- launcher 成功 spawn 后立即返回；client reconnect loop 负责等待 daemon ready。
- 重复 launcher 通过 daemon singleton lock 收敛，不得抢占 live socket。
- 只有 endpoint 不存在或 connection refused 才允许 spawn；version mismatch、unauthorized、protocol error 或 executable hash 差异必须直接返回，不得自动替换或重启 live daemon。

### FR-03：Daemon 启动与关闭

- 新增 binary `homie-runtime-daemon`。
- daemon 启动顺序固定为：
  1. 规范化并校验 runtime paths；
  2. 创建 owner-only runtime directory；
  3. 获取 non-blocking singleton lock；
  4. 确认 lock owner 后删除 stale socket；
  5. 打开/migrate storage 并恢复 runtime/holder facts；
  6. 启动单 owner `RuntimeActor`；
  7. bind UDS；
  8. 发布 `runtime.ready`。
- SIGTERM、SIGINT 和 control shutdown 使用同一 drain 路径。
- `daemon.prepare_shutdown` 必须停止接收新 mutation 并 flush 当前 durable facts。
- `daemon.shutdown` 必须先返回 ACK，再停止 listener、streams 和 actor。
- shutdown 不得主动终止 holder-owned session。
- daemon startup failure 只清理本次 socket，不删除 session、holder、output 或 database。

### FR-04：统一二进制 Frame

每个连接使用：

```text
[preface][frame...]

preface:
  magic[8] = "HOMIEIPC"
  major u16 BE
  minor u16 BE

frame:
  frame_len u32 BE
  version u16 BE
  kind u8
  flags u8
  stream_id u32 BE
  message_id u64 BE
  sequence u64 BE
  payload[frame_len - 24]
```

规则：

- `frame_len` 表示 24-byte frame header 加 payload，不包含外层 4-byte length。
- `stream_id=0` 是 connection control stream。
- client 创建奇数 stream ID；server 保留偶数 stream ID。
- Wave 1A 的 `flags` 必须为 0；收到非 0 flags 返回 protocol error。
- 总 frame 上限 16 MiB。
- control JSON payload 上限 4 MiB。
- 单 Output frame payload 上限 64 KiB。
- frame parser 必须支持任意 partial read、多个 frame 合并 read 和 hostile length。
- unknown kind、invalid version、超限 length、sequence 回退和损坏 payload 必须 fail closed。

### FR-05：Frame Kind 与 Payload

Wave 1A 固定 kind：

| Value | Kind | Stream | Payload |
|-------|------|--------|---------|
| 1 | Hello | 0 | JSON `HelloRequest` |
| 2 | HelloAck | 0 | JSON `HelloResponse` |
| 3 | Request | 0 | JSON method + params |
| 4 | Response | 0 | JSON result/error |
| 5 | Event | event stream | JSON event envelope |
| 6 | StreamOpen | non-zero | JSON stream open request |
| 7 | StreamOpened | same stream | JSON open acknowledgement |
| 8 | StreamReset | same stream | JSON reason + last confirmed position |
| 9 | StreamClose | same stream | empty or JSON safe reason |
| 16 | Output | terminal stream | offset u64 BE + bytes |
| 17 | Input | terminal stream | raw input bytes |
| 18 | Resize | terminal stream | cols u16 + rows u16 |
| 19 | Grid | terminal stream | existing `GridUpdate` binary codec |
| 20 | Modes | terminal stream | bitset |
| 21 | ReplayBegin | terminal stream | offset u64 |
| 22 | ReplayEnd | terminal stream | offset u64 |
| 23 | Ping | 0 or stream | empty |
| 24 | Pong | matching stream | empty |

Control/Event/Stream metadata 使用 Serde JSON。Terminal/Grid/Input/Resize/Modes 使用 binary payload，不做 base64。

### FR-06：Hello 与 Capability Truth

`HelloRequest` 必须包含：

- wire major/minor；
- client name/version；
- client role；
- process id。

`HelloResponse` 必须包含：

- selected wire major/minor；
- daemon build/version；
- daemon pid；
- daemon instance id；
- executable hash；
- exact method capabilities；
- exact stream capabilities；
- event oldest/latest seq。

规则：

- Hello 是连接上的第一个 frame。
- wire major 不同立即拒绝。
- minor 只允许 additive negotiation，不提供 compatibility fallback。
- capability 中的每个 method 必须存在 handler。
- capability 中的每个 stream kind 必须存在 open handler。
- proto 中的常量可以超前存在，但不得自动进入 capability。

### FR-07：Request Correlation 与错误

- client 为每个 control request 分配非 0 `message_id`。
- response 必须使用相同 `message_id`。
- pending request map 必须有硬上限。
- 写队列失败、connection close 和 explicit shutdown 必须一次性失败全部 pending request。
- request timeout 只移除对应 pending request。
- caller 取消或 drop request future 时必须移除对应 pending waiter；迟到 response 只作为 unknown/tombstoned message 丢弃，不得关闭健康连接。
- 自动 reconnect 不得重放 mutation。
- stable error codes：
  - `method_not_found`
  - `bad_request`
  - `version_mismatch`
  - `unauthorized`
  - `unavailable`
  - `timeout`
  - `backpressure`
  - `resync_required`
  - `internal`
- error 不得包含 raw payload、argv、env、Authorization、cookie、secret 或 terminal bytes。

### FR-08：Event Stream 与 Gap Recovery

- event subscription 通过非 0 stream 建立。
- open payload 包含 `afterSeq` 和 event filter。
- daemon event replay ring 必须有固定容量和 oldest/latest seq。
- 每个 Event frame 的 `sequence` 等于 runtime event seq。
- event stream queue 必须有界。
- event replay ring 固定保留最近 1024 条 event。
- `afterSeq` 早于 oldest 或 queue overflow 时返回 `StreamReset`，reason 为 `event_gap`，并携带 latest seq。
- `state.snapshot` response 必须包含用于重开 event stream 的一致 cursor。
- client 收到 `event_gap` 后必须请求 `state.snapshot`，替换本地 projection，再从 snapshot cursor 重开 event stream。
- event gap 不能静默跳过。

### FR-09：Terminal Stream

- terminal stream open payload 包含：
  - session id；
  - from output offset；
  - client role；
  - last grid sequence（可选）。
- server open 成功后依次发送：
  1. `StreamOpened`；
  2. `ReplayBegin`；
  3. 0..N 个 `Output`；
  4. `ReplayEnd`；
  5. full `Grid`；
  6. `Modes`；
  7. live `Output`/`Grid`。
- Input/Resize/Scroll 只作用于该 stream 绑定的 session。
- Output 必须携带绝对 log offset。
- stream 内 `sequence` 必须单调递增。
- attachment 断开、reset 或 close 不得终止 session。
- 重开 stream 必须从 last confirmed output offset replay，并重新发送 full grid。
- Wave 1A 的 production terminal producer 以 holder output log 为权威源。
- daemon 内每个 attached session 只能有一个共享 `TerminalSource` tailer；多个 client stream 订阅同一 source，不得各自重复读取整份 log 或轮询 actor。
- source 活跃时最高 20 Hz 读取新增 bytes，idle 时退避到 250 ms，每次最多读取 64 KiB。
- actor 只验证 session 并返回 daemon-internal source descriptor；output replay/tail 和 `HeadlessScreen` 更新由 terminal stream hub 在 actor 外完成，input/resize 仍回到 actor。
- full grid 由现有 `HeadlessScreen` 可见文本生成，未提供的 style 使用协议默认值；T-202 可增加 styled diff，但不得改变本 change 的 stream/frame 合同。

### FR-10：有界队列与调度

- connection writer 使用两个有界队列：
  - high priority：hello、control request/response、stream open/opened/reset/close、input、resize、ping/pong；
  - low priority：event、output、grid、modes。
- high queue 容量 256 frames。
- 每个 event/terminal stream 的 low queue 容量 256 frames。
- 每个 connection 最多 64 个 active non-control streams。
- 每个 daemon 最多 64 个 active client connections；第 65 个连接在读取 payload 前拒绝。
- 每个 client connection 最多 1024 个 pending requests。
- writer 最多连续发送 32 个 high frames，然后必须尝试发送一个 low frame。
- low streams 使用 round-robin，单个 busy terminal 不得饿死其他 stream。
- slow stream queue 满时只 reset 该 stream，reason 为 `slow_consumer`。
- server high-priority queue 满时必须关闭该 connection；client 本地 high-priority queue 满时向调用方返回 `backpressure`。
- client decoded stream queue 容量 256；满时关闭本地 stream 并暴露 `ResyncRequired(last_confirmed_offset)`。
- server 不得建立隐藏的 unbounded output/event queue。
- `LongRunningLane` 固定 1 个 worker、32 个 pending jobs；第 33 个 job 返回 `backpressure`。
- long-running deadlines 固定为：output/artifact/status/snapshot 10s，git list/diff/locate 15s，history scan 30s，worktree create/remove 60s。

### FR-11：RuntimeActor 与 Daemon Dispatcher

- `RuntimeSupervisor`、SQLite connection 和 live registry 必须只由一个 `RuntimeActor` owner 持有。
- `RuntimeActor` 必须运行在名为 `homie-runtime-actor` 的专用 OS thread；它独占 SQLite、PTY、live registry 和 runtime mutation，不得占用 Tokio async worker。
- Tokio connection task 通过 bounded command channel + oneshot reply 调 actor。
- actor command queue 容量固定为 256。
- git、history 和 bounded output scan 必须进入独立 `LongRunningLane`，不得占用 RuntimeActor 或 Tokio worker。
- `LongRunningLane` 只能接收 actor 生成的 owned path/DTO snapshot，不得持有或调用 `Storage`、`RuntimeSupervisor` 或 live registry。
- long-running handler 使用 `actor prepare -> lane execute -> actor commit`；lane 失败、超时或取消时不得提交部分 storage state。
- 单 worker 天然串行所有 git jobs；不得为 Wave 1A 增加 repo-key coordinator 或多 pool。
- git hard deadline 必须终止并回收独立 process group；queued job 超时或 waiter 已取消时不得启动。
- 已启动 worktree mutation 在调用方取消后继续到成功或 60s hard deadline，不自动重放。
- 当前位于 `homie-client` 的 runtime/storage/worktree/history dispatch 必须移入 daemon。
- transport/service test 允许注入 internal `RuntimeBackend` fake。
- production daemon 只能使用真实 `RuntimeSupervisor` adapter。
- 不允许 production `--test-mode`、fixture file 或 fake backend flag。
- `events.wait` 等长等待不得占用 actor；daemon event layer 在 actor 外完成 async wait。

### FR-12：纯 Async HomieClient

- `homie-client` 生产 API 使用 Tokio async。
- client 负责：
  - connection lifecycle；
  - hello/capability；
  - request correlation；
  - priority writer；
  - frame demux；
  - event/terminal stream；
  - heartbeat/reconnect；
  - stream recovery。
- client 不负责：
  - RuntimeSupervisor construction；
  - SQLite open/query；
  - git/worktree/history implementation；
  - UI projection；
  - daemon implicit spawn。
- client connection state：
  - disconnected；
  - connecting；
  - handshaking；
  - connected；
  - degraded；
  - reconnecting；
  - shutdown。
- heartbeat idle interval 25s，response timeout 10s。
- typed long-running methods 的 client wait timeout 必须为对应 server deadline + 5s；timeout 只移除 waiter，不代表已启动 worktree mutation 被取消。
- reconnect backoff 从 500ms 指数增长到 8s，成功连接后重置。

### FR-13：CLI 与 MCP 迁移

- CLI session/worktree/history/diff/events/control operations 必须 await `HomieClient`。
- `control-stdio` 必须成为 stdin/stdout 与 daemon control 的 bridge，不再在 CLI 进程内执行 runtime dispatcher；单条 stdin control JSON 上限 4 MiB。
- MCP runtime context 必须持有 async client，不再创建 embedded runtime。
- CLI/MCP 的 data-dir 参数只用于派生 endpoint/launcher paths。
- doctor、usage 等当前 direct-storage 路径允许保留到 T-103，但不得访问 live runtime state。
- CLI 作为独立入口时必须显式 ensure daemon running。

### FR-14：GPUI App 迁移

- app 创建固定 2-worker Tokio runtime，用于 client transport 和 service bridge。
- app 首帧不得同步打开 runtime、spawn shell、读 output 或等待 daemon。
- app startup 顺序：
  1. 解析绝对 data dir/daemon path；
  2. 显式 launcher ensure；
  3. 创建 async client；
  4. 启动 connection/event bridge；
  5. 通过 GPUI message/update 回主线程更新 projection。
- 本 change 必须迁移 session list/spawn/send/resize/snapshot/events 的 runtime 数据流。
- app settings direct-storage 清理由 T-103 完成，但不得再用于 live session projection。

### FR-15：生产 Shortcut 删除

本 change 准出前必须删除：

- `HomieClient { runtime: RuntimeSupervisor }`；
- `HomieClient::open(data_dir)`；
- `HomieClient::open_with_runtime`；
- client 内 `handle_request` runtime dispatcher；
- client 对 `homie-runtime`、`homie-storage` 的 Cargo 依赖；
- app/CLI/MCP 通过 client 创建 embedded runtime 的路径；
- 同步 production client facade。

测试需要 direct backend 时，只能使用 daemon server internal test seam。

### FR-16：本地 UDS 安全

- Wave 1A 只支持 local UDS。
- daemon 必须验证 peer UID 与自身 UID 相同。
- runtime directory、socket 和 lock 不得是非 owner 可写。
- stale socket 只能在持有 singleton lock 后删除。
- frame length、queue、pending request、stream count 和 per-stream memory 都必须有上限。
- logs/events/evidence 只记录 safe endpoint kind、instance id、method、stream id、sequence、duration 和 safe error code。

## 4. 架构设计

### 4.1 组件关系

```text
homie-app / homie-cli / MCP
           |
    RuntimeLauncher (explicit)
           |
      HomieClient (Tokio async)
           |
   one owner-only UDS connection
   control + events + terminal streams
           |
   homie-runtime-daemon
     |             |
 connection hub   event/stream scheduler
           \       /
          RuntimeActor
               |
      RuntimeSupervisor + SQLite
               |
        holder-owned PTYs
```

### 4.2 文件职责目标

建议文件边界：

```text
crates/homie-proto/src/
├── transport.rs       # preface/header/frame kind/codec/limits
├── control.rs         # hello/request/response/error/capability DTO
└── stream.rs          # stream open/reset/terminal metadata

crates/homie-client/src/
├── lib.rs             # public exports
├── client.rs          # typed async facade
├── connection.rs      # lifecycle/heartbeat/reconnect/demux
├── writer.rs          # priority bounded scheduler
├── events.rs          # event stream and gap recovery
├── terminal.rs        # terminal stream handle/reopen state
└── launcher.rs        # explicit daemon path/start policy

crates/homie-runtime/src/
├── daemon.rs          # daemon lifecycle/signal/drain
├── server.rs          # UDS accept/peer validation/connection hub
├── connection.rs      # frame ingest/demux
├── dispatcher.rs      # capability + control method dispatch
├── runtime_actor.rs   # single-owner blocking runtime backend
└── stream.rs          # event/terminal producers and stream reset

crates/homie-runtime/src/bin/
└── homie-runtime-daemon.rs
```

现有 `homie-client/src/lib.rs` 和 `homie-runtime/src/lib.rs` 只在本 change 需要的范围内拆分；不做无关模块重构。

### 4.3 Capability 初始范围

Wave 1A 必须承载 handshake、基础 transport 方法和当前真实 dispatcher 方法。

Handshake：

- `Hello` / `HelloAck` frame

Request methods：

- `state.snapshot`
- `events.wait`
- `daemon.prepare_shutdown`
- `daemon.shutdown`
- `session.spawn`
- `session.list`
- `session.snapshot`
- `session.status`
- `session.artifacts`
- `session.ports`
- `session.set_parent`
- `session.list_children`
- `session.parent`
- `session.history`
- `session.resume_from_history`
- `session.read_diff`
- `session.send_text`
- `session.resize`
- `session.kill`
- `host.locate_repo`
- `worktree.list`
- `worktree.create`
- `worktree.remove`
- `worktree.overview`
- `hook.report`

Stream capabilities：

- `events.v1`
- `terminal.v1`

如果实施前发现某个方法没有真实 production handler 或无法通过 E2E，必须从 capability 中移除并在 evidence 记录，不得用 unsupported placeholder 顶替。

`session.snapshot`、`session.status`、`session.artifacts`、`session.ports`、`session.set_parent`、`session.list_children` 和 `session.parent` 是现有 typed client 行为的 transport 化；如 proto 尚无同名常量/DTO，本 change 负责添加。它们不是新增产品能力。

## 5. 错误与恢复

| 场景 | 行为 |
|------|------|
| daemon 不存在 | launcher 显式 spawn；client 保持 reconnecting |
| daemon singleton 已存在 | 新 daemon 正常退出，不删除 live socket |
| hello version 不兼容 | connection fail closed，状态为 disconnected/version_mismatch |
| peer UID 不匹配 | connection 关闭并记录 unauthorized safe event |
| pending request timeout | 只失败该 request，不自动重发 |
| connection 断开 | 失败全部 pending；event/terminal stream 进入恢复 |
| event gap | state.snapshot 后重新订阅 |
| terminal queue 满 | reset 单个 stream，按 last offset 重开 |
| invalid/oversized frame | 关闭 connection；不解析后续 bytes |
| actor queue 满 | 返回 backpressure，不阻塞 socket task |
| long-running queue 满 | 返回 backpressure，不启动 job |
| long-running read job timeout/cancel | 终止 child/scan，不执行 actor commit |
| worktree mutation caller cancel | 只取消等待，server mutation 继续到完成或 60s hard deadline |
| daemon graceful shutdown | ACK、drain、flush、关闭 listener/actor，holder 保活 |
| daemon hard crash | launcher 可重启；新 runtime 从 storage/holder 事实恢复 |

## 6. 迁移步骤

1. 先增加 transport RED tests 和 frame DTO，不修改调用方。
2. 增加 daemon server library、fake backend 和 real daemon binary。
3. 增加 async client 与 launcher，使用新测试但暂不切 app/CLI。
4. 迁移 CLI 和 MCP，确认它们连接同一 real daemon。
5. 迁移 app startup/session flow，确认首帧不阻塞。
6. 删除 embedded/sync client 和 client runtime/storage dependencies。
7. 更新 package script，把 `homie-runtime-daemon` 放入 app dependency closure；签名公证仍由 T-501 准出。
8. 跑 workspace、process E2E、security 和 evidence gates。

不提供 rollback compatibility path。开发分支回滚通过 git revert；一旦合入，新旧 client/runtime 不并存。

## 7. 测试计划

### 7.1 Protocol Unit

- preface encode/decode；
- frame partial/coalesced reads；
- max lengths；
- every kind payload roundtrip；
- unknown kind/version/flags；
- stream/message/sequence invariants；
- hostile length and allocation denial。

### 7.2 Client Unit

- request correlation/out-of-order response；
- timeout and pending cleanup；
- disconnect fail-all；
- heartbeat/reconnect backoff；
- priority writer fairness；
- stream demux；
- per-stream queue overflow/reset；
- event gap snapshot recovery；
- mutation not replayed。

### 7.3 Server/Fake Backend Integration

使用 temp absolute data dir 和 internal fake backend：

- hello/capability；
- concurrent request/response；
- method-not-found；
- event replay/gap；
- multiple terminal streams；
- replay/full-grid/live order；
- input/resize routing；
- one slow stream does not block another；
- prepare-shutdown/shutdown drain。

### 7.4 Real Daemon Process

- absolute launcher path；
- owner-only paths；
- singleton race；
- stale socket recovery；
- hello/state snapshot；
- daemon restart/client reconnect；
- app/CLI/MCP connect same instance id；
- SIGTERM drain；
- no real HOME/provider key dependency。

### 7.5 Migration Regression

- CLI session/MCP focused tests；
- GPUI app compile/first-frame；
- client crate dependency scan；
- capabilities-handler equality；
- source scan confirms embedded APIs removed；
- current holder regression remains explicitly assigned to T-102。

## 8. 受影响组件规格

| Component spec | 影响 |
|----------------|------|
| `specs/runtime-client-transport/README.md` | 改为统一二进制 multiplex framing 和 explicit launcher |
| `specs/runtime-supervisor/README.md` | 新增 daemon/actor/server ownership 与 graceful shutdown |
| `specs/desktop-shell/README.md` | app Tokio bridge、非阻塞首帧、shared daemon projection |
| `specs/mcp-automation/README.md` | MCP async client 和 control-stdio bridge |
| `specs/observability/README.md` | transport safe events、backpressure/gap evidence |
| `specs/packaging-updater/README.md` | daemon binary 进入 dependency closure，最终签名延后 |

## 9. 验收标准

### 9.1 Spec Gate

- Bead `homie-nep` 指向本 PRD。
- component specs 完成影响修订。
- OpenSpec proposal/design/specs/plan/tasks/alignment 为 4/4 且 strict valid。
- 16 维 spec review 无阻断项。

### 9.2 Implementation Gate

- `homie-runtime-daemon` 可从绝对 temp data dir 启动和关闭。
- app、CLI、MCP 连接同一 daemon instance。
- client 支持 async request、events 和至少两个并发 terminal streams。
- daemon restart 后 client 重连，event gap 走 snapshot，terminal stream 按 offset 重开。
- slow terminal 只 reset 自己，不阻塞 control/其他 stream。
- hello capabilities 与真实 handler/stream opener 集合一致。
- `homie-client` Cargo 不依赖 `homie-runtime`、`homie-storage`。
- embedded/sync production client API 删除。
- 不新增环境变量或 production fake/test mode。
- package script 包含 daemon binary。
- focused tests、workspace format/check/clippy 和不受既有 holder blocker影响的 suites 通过。

### 9.3 诚实准出

- 当前 holder/PTY live adoption 回归若仍存在，必须作为 T-102 blocker 报告。
- Wave 1A 可在 transport/client/daemon scope 内 pass，但不得将 RT-001、RT-006、RT-007 改回 implemented。
- API-002 只有在 app/CLI/MCP shared-daemon、reconnect、event resume 和 multiplex stream E2E 全部通过后才能改为 implemented。

## 10. Beads 追踪

| Bead | Change | 状态规则 |
|------|--------|----------|
| `homie-nep` | `diri-runtime-daemon-client-transport` | PRD/spec/OpenSpec/实现/evidence 完成后关闭 |
| `homie-t3u` | parent rebaseline | 已关闭，只提供 master requirement |
| future T-102 Bead | `diri-agent-session-runtime` | 接管 agent spawn、holder/resource/migrate 完整语义 |
| future T-103 Bead | `diri-storage-core-facts` | 接管 UI/CLI direct-storage 和 durable facts |
