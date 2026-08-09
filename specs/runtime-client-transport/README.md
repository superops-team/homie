# Runtime Client Transport 组件规格

## 1. 组件定位

`homie-client` 是 Homie app、CLI、MCP 和 remote control plane 访问独立 runtime daemon 的唯一生产客户端。它负责连接、统一二进制多路复用协议、请求关联、事件续传、terminal stream、重连和 backpressure，不实现 runtime、storage、launcher policy 或 UI 业务。

## 2. 来源需求映射

- PRD: `prd-spec/features/diri-7ba3407-parity-rebaseline/2026-08-08-diri-7ba3407-parity-rebaseline-design.md`
- Wave 1A PRD: `prd-spec/features/diri-runtime-daemon-client-transport/2026-08-08-diri-runtime-daemon-client-transport-design.md`
- 能力矩阵: `docs/research/diri-7ba3407-capability-matrix.md`
- OpenSpec: `openspec/changes/diri-runtime-daemon-client-transport/`
- Requirements: Wave 1A FR-01..FR-16
- Beads: `homie-nep`

## 3. 上游与下游关系

| 方向 | 组件 | 关系 |
|------|------|------|
| 上游 | `homie-app` | 订阅 projection/events，发送 typed commands |
| 上游 | `homie-cli` | 调 session/events/host/node/runtime methods |
| 上游 | MCP stdio | 执行 runtime-backed tools |
| 上游 | remote control | 访问 remote/node methods |
| 下游 | runtime daemon | 同一 UDS 上的 control/event/terminal streams |
| 下游 | `homie-proto` | request/response/event/frame DTO |

## 4. 职责边界

负责：

- endpoint 解析和连接生命周期；
- version/capability handshake；
- request id 分配、correlation、timeout 和 cancellation；
- heartbeat、disconnect detection 和 exponential backoff；
- event sequence resume 和 gap recovery；
- multiplexed terminal stream、flow control 和 frame ordering；
- typed method facade 和 stable error mapping；
- 暴露 connection state，不把 launcher 失败伪装成 transport failure。

不负责：

- 创建 `RuntimeSupervisor`；
- 直接打开 SQLite、output log 或 holder socket；
- 拼接 session 状态、权限或业务 projection；
- UI optimistic state；
- provider credential。
- 决定是否、何时或从哪个 executable 启动 daemon。

## 5. 生产接口

```rust
pub struct HomieClient {
    connection: Arc<ConnectionManager>,
}

pub struct ClientOptions {
    pub endpoint: RuntimeEndpoint,
    pub role: ClientRole,
    pub connect_timeout: Duration,
    pub request_timeout: Duration,
}

impl HomieClient {
    pub async fn connect(options: ClientOptions) -> Result<Self, ClientError>;
    pub async fn request<P, R>(&self, method: Method, params: P) -> Result<R, ClientError>
    where
        P: Serialize,
        R: DeserializeOwned;
    pub async fn subscribe_events(&self, request: EventSubscribeRequest)
        -> Result<EventStream, ClientError>;
    pub async fn open_terminal(&self, request: TerminalOpenRequest)
        -> Result<TerminalStream, ClientError>;
    pub async fn close(&self) -> Result<(), ClientError>;
}
```

生产 API 全部是 Tokio async，不提供 `open_with_runtime`、同步 facade 或接受 `RuntimeSupervisor` 的构造器。需要 direct runtime 的测试使用 daemon server internal fake backend，不能编译进 production feature。

## 6. Transport 合同

### 6.1 Connection 与 Frame

- macOS 本地默认使用 Unix Domain Socket。
- 一条 UDS 同时承载 connection control、event stream 和多个 terminal stream。
- connection 以 12-byte `HOMIEIPC` preface 开始。
- frame 是 length-prefixed binary envelope：

```text
[frame_len u32 BE]
[version u16 BE]
[kind u8]
[flags u8]
[stream_id u32 BE]
[message_id u64 BE]
[sequence u64 BE]
[payload]
```

- `stream_id=0` 为 control；client 分配奇数 stream id，server 保留偶数。
- control/event/stream metadata payload 使用 bounded Serde JSON；terminal grid/input/output 使用 binary。
- 总 frame 上限 16 MiB，control JSON 上限 4 MiB，Output chunk 上限 64 KiB。
- Wave 1A `flags` 只能为 0；unknown kind/version/flags、损坏 payload 和超限 length fail closed。
- parser 必须正确处理 partial/coalesced reads，且不能按 hostile length 预分配无界内存。

### 6.2 Hello、Control 与 Capability

- connection 的第一个 frame 必须是 `Hello`。
- wire major 不同 fail closed；minor 只允许 additive negotiation。
- `HelloAck` 返回 daemon instance/build/hash、exact methods、exact stream kinds 和 event oldest/latest seq。
- capability 只包含存在真实 handler/opener 的 method/stream。
- request 使用非 0 `message_id`，response 复用同一 id。
- 单连接 pending request 上限 1024；断线一次性失败全部 pending。
- caller 取消 request future 时移除 waiter；迟到 response 不关闭健康连接。
- 普通 request timeout 为 10s；long-running typed method 使用 server deadline + 5s。
- client timeout/cancel 只移除 waiter，不代表已启动 worktree mutation 停止。
- 自动重连不得重放 mutation。
- stable error codes 是 `method_not_found`、`bad_request`、`version_mismatch`、`unauthorized`、`unavailable`、`timeout`、`backpressure`、`resync_required`、`internal`。

### 6.3 Event 与 Terminal Stream

- 单连接最多 64 个 non-control streams。
- 单 daemon 最多 64 个 active client connections。
- event stream 使用 runtime event seq；1024-entry replay ring 不覆盖 cursor 时返回 `StreamReset(event_gap)`。
- client 在 event gap 后请求带一致 cursor 的 `state.snapshot`，替换 projection 后重新订阅。
- terminal stream 顺序是 `StreamOpened -> ReplayBegin -> Output* -> ReplayEnd -> full Grid -> Modes -> live frames`。
- Output 携带绝对 log offset；stream sequence 单调递增。
- sequence gap、slow consumer 或过旧 offset 只 reset 当前 stream。
- terminal reset 后从 last confirmed output offset 重开，并重新获取 full grid。
- attachment/stream teardown 不终止 runtime session。

### 6.4 有界调度

- writer 有 256-frame high-priority queue 和每 stream 256-frame low-priority queue。
- high priority 包含 control、stream lifecycle、input、resize 和 ping/pong。
- 最多连续发送 32 个 high frames，随后尝试一个 round-robin low frame。
- client decoded stream queue 同样为 256；满时暴露 `ResyncRequired`，不静默丢 frame。
- server high queue 满时关闭 connection；client local high queue 满时返回 `backpressure`。
- 不允许隐藏 unbounded event/output channel。

### 6.5 Reconnect

```text
disconnected
  -> connecting
  -> handshaking
  -> connected
  -> degraded
  -> reconnecting
  -> connected
```

- heartbeat timeout 进入 degraded/reconnecting。
- heartbeat idle interval 25s、timeout 10s；backoff 从 500ms 增长到 8s。
- launcher 只在 endpoint missing/refused 时 spawn；version/protocol/auth/hash 差异不得触发 live daemon 替换。
- 重连后重新 hello，先恢复 event cursor，再按 output offset 恢复 terminal stream。
- runtime 明确报告 event gap 时请求 state snapshot。
- 同一 command 不允许因自动重连被隐式重复执行；重试需要 request idempotency evidence 或调用方显式发起。

## 7. 安全

- socket path 必须位于 owner-only runtime directory。
- handshake 必须校验 peer/process ownership 或等价本地信任边界。
- remote TCP 不能复用本地无认证 transport；remote 由 node auth 合同负责。
- logs/evidence 不记录 raw control payload、terminal bytes、Authorization、cookie、virtual key 或完整 tool args/result。
- protocol decode 使用 bounded input，拒绝超限 frame、attachment 和 queue。

## 8. 失败与恢复

| 场景 | 行为 |
|------|------|
| daemon 不存在 | 返回 stable unavailable；是否启动由调用方显式 launcher 决定 |
| handshake version 不兼容 | fail closed，不启动 compatibility fallback |
| request timeout | 返回 retryable safe error；不自动重复非幂等请求 |
| event gap | 请求 full state snapshot |
| terminal gap | 丢弃 grid projection 并按 offset 重新 open |
| client queue 满 | reset 慢 stream，runtime session 和其他 stream 继续 |
| connection 断开 | 所有 stream 进入 degraded，按 cursor/offset 重建 |

## 9. 可观测性

- `client.connect_started/completed/failed`
- `client.handshake_failed`
- `client.reconnect_scheduled`
- `client.event_gap`
- `client.stream_resynced`
- `client.backpressure`

事件只记录 endpoint kind、client role、safe error code、retryable、sequence/epoch 摘要和 duration。

## 10. 测试与准出

| Gate | Required cases |
|------|----------------|
| Unit | preface/frame codec、partial read、request correlation、timeout、unknown value/error mapping |
| Integration | UDS handshake、concurrent requests、events、heartbeat、cancel |
| Recovery | daemon restart、sequence resume、event gap/full snapshot |
| Terminal | replay/full-grid/live ordering、sequence gap、backpressure、reopen |
| Cross-entry E2E | app、CLI、MCP 连接同一 daemon 并观察同一 session |
| Negative | 确认 production client 不依赖 `homie-runtime` 或 `homie-storage` |

本组件当前状态是 `partial`。只有删除生产 in-process supervisor 路径并通过以上门禁后，能力矩阵 M11 和 API-002 才能改为 `implemented`。
