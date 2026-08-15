# Swift/Rust 协议契约 Golden Fixtures 设计文档

## 1. 概述

### 1.1 问题/动机

Homie 当前控制协议由 Swift 和 Rust 两端分别手写：

- Swift: `Sources/HomieProtocol/ControlMessage.swift`
- Rust: `homie/crates/homie-proto/src/control.rs`

两端都定义了 wire version、NDJSON envelope、`ControlError`、request/response/event 判别顺序和单行最大字节数。当前实现注释明确 Rust 是按 Swift envelope 复刻，但没有共享 schema 或 golden fixtures 作为漂移门禁。

Waku 对比 review 中，Waku 基本处于单 Rust 模型边界，天然减少跨语言 DTO 漂移。Homie 因 CLI/Swift glue 仍存在，需要通过共享契约测试降低双语言手写协议的维护风险。

### 1.2 目标

1. 为 Swift/Rust control protocol 建立共享 golden fixtures。
2. 让 Swift 和 Rust 双端测试读取同一批 fixture，并验证编码/解码一致。
3. 覆盖 request、success response、error response、event、空 params、null params、超长行边界等基础协议形态。
4. 为后续协议字段新增提供明确变更流程：先改 fixture，再改双端实现。

### 1.3 非目标

- 不立即把协议迁移为 Protobuf、Cap'n Proto 或其他二进制协议。
- 不改变当前 NDJSON wire format。
- 不引入跨进程兼容层。
- 不重写 Swift CLI。

## 2. 现状分析

| 协议点 | Swift 位置 | Rust 位置 | 风险 |
|--------|------------|-----------|------|
| wire version | `WireVersion.current` | `WIRE_VERSION` | 任一端升级遗漏 |
| max line bytes | `NDJSONBuffer.maxLineBytes` | `MAX_CONTROL_LINE_BYTES` | 解析边界不一致 |
| request 判别 | presence of `method` | presence of `method` | 判别顺序漂移 |
| event 判别 | presence of `event` | presence of `event` | event/response 误判 |
| error response | `err` | `err` | 错误结构/错误码漂移 |
| success response | `ok` or null | `ok` or null | null 语义漂移 |

当前已有协议 fixtures 位于 `homie/crates/homie-proto/tests/fixtures/`，但主要是 Rust 侧 JSON 响应样例，并非 Swift/Rust 双端共享契约。

## 3. 方案设计

### 3.1 共享 fixture 目录

新增一个语言无关目录，例如：

```text
protocol-fixtures/
├── control-message/
│   ├── request-hello.jsonl
│   ├── response-ok-null.jsonl
│   ├── response-error-bad-request.jsonl
│   ├── event-session-updated.jsonl
│   ├── roundtrip-cases.json
│   └── invalid-cases.json
└── README.md
```

要求：

- fixture 使用真实 wire JSON，不依赖聊天上下文。
- 文件名表达 case 语义。
- 每个 case 包含期望结构和规范化后的 JSON。
- fixture 只作为测试契约，不作为运行时配置或打包资源。
- fixture 不包含真实 prompt、Authorization、cookie、provider token、私有路径或用户会话内容。
- 测试从仓库根目录定位 fixture；package/dev bundle 不复制该目录。

### 3.2 双端测试

Rust:

- 新增或扩展 `homie/crates/homie-proto/tests/control_roundtrip.rs`。
- 读取 `protocol-fixtures/control-message/*`。
- 验证 decode 后结构正确，encode 后规范化 JSON 与期望一致。

Swift:

- 新增或扩展 `Tests/HomieProtocolTests/WireTests.swift`。
- 读取同一 fixture 目录。
- 验证 `JSONDecoder.homie` / `JSONEncoder.homie` 与 Rust 期望一致。

### 3.3 变更流程

协议变更必须按以下顺序：

1. 新增或更新 fixture。
2. Swift/Rust 双端测试先 RED。
3. 修改 Swift/Rust 实现。
4. 双端测试 GREEN。
5. 若是行为变更，更新 `docs/SECURITY-MODEL.md` 或协议相关文档。

### 3.4 后续可选：schema/codegen

本 PRD 不要求立即 codegen。但 golden fixtures 稳定后，可评估：

- JSON Schema 只描述 envelope；
- Rust `serde` 类型导出 schema；
- Swift DTO 由 schema 生成。

## 4. 实施步骤

0. 进入实现前先补齐：
   - `docs/verification/protocol-contract-golden-fixtures/spec-review-report.md`；
   - `openspec/changes/protocol-contract-golden-fixtures/{plan.md,tasks.md,alignment-report.md}`；
   - 对 `specs/engine-session-runtime.md` 的影响评估。若 fixture 只是固化现有 wire contract，可在 OpenSpec 中说明不改 durable spec；若发现 Swift/Rust 行为不一致，必须先更新 durable spec 再改代码。
1. 创建 `protocol-fixtures/control-message/README.md`，写明 wire envelope 规则。
2. 创建最小 roundtrip fixture 集：
   - request with params
   - request without params
   - event with params
   - event without params => params null
   - ok response with object
   - ok response null
   - err response
3. 创建 invalid fixture 集：
   - 非 object
   - 缺 id 的 request
   - 超过 `MAX_CONTROL_LINE_BYTES` 的 line 可通过生成测试构造，不必提交大文件。
4. Rust 测试读取 fixture 并验证。
5. Swift 测试读取 fixture 并验证。
6. 将测试加入 `scripts/check.sh` 和 CI 现有 Swift/Rust job。

## 5. 涉及文件

- `protocol-fixtures/control-message/*`
- `Sources/HomieProtocol/ControlMessage.swift`
- `Tests/HomieProtocolTests/WireTests.swift`
- `homie/crates/homie-proto/src/control.rs`
- `homie/crates/homie-proto/tests/control_roundtrip.rs`
- `scripts/check.sh`
- `.github/workflows/ci.yml`

## 6. 验证计划

### 6.1 Rust

```sh
cd homie
cargo test -p homie-proto --test control_roundtrip
```

### 6.2 Swift

```sh
swift test --package-path . --filter HomieProtocolTests
```

### 6.3 全量门禁

```sh
./scripts/check.sh
```

### 6.4 首阶段关闭口径

`homie-54o` 首阶段只关闭“现有 control protocol 行为被共享 fixture 守住”：

- 覆盖现有 request/response/event envelope，不新增 wire method。
- Rust/Swift 双端读取同一 fixture 目录并通过 focused tests。
- `scripts/check.sh` 或等价本地 gate 能调用该漂移检查。
- 不引入 schema/codegen，不迁移协议格式。

### 6.5 风险控制

| 风险 | 控制 |
|------|------|
| fixture 范围过大导致 P0 长期悬挂 | 首阶段只覆盖 envelope、null/empty params、error 和最大行边界 |
| Swift/Rust 行为不一致时直接改实现 | 先在 durable spec 或 OpenSpec alignment 中明确目标语义，再让一端测试 RED |
| 测试 fixture 泄漏真实会话内容 | 只使用人工构造的最小 JSON，不保存 prompt、token、cookie、私有路径 |
| CI 依赖一次性改动过大 | 先接入本地 `scripts/check.sh`，CI 可作为同一 change 的最后一步 |

## 7. 验收标准

1. Swift/Rust 双端读取同一 fixture 目录。
2. 任意一端改动 envelope 行为时，共享 fixture 测试能失败。
3. `WIRE_VERSION` / `WireVersion.current`、max line bytes 等关键常量有测试保护。
4. CI 对协议漂移有明确 gate。
5. OpenSpec alignment 明确每个 fixture case 对应的 Swift/Rust 测试和证据路径。
6. Beads `homie-54o` 更新为已验证状态后才可关闭。

## 8. Beads 追踪

- Beads: `homie-54o`
- change_id: `protocol-contract-golden-fixtures`
- 类型: refactor
- 优先级: P0
