# Diri Parity Child Tasks 需求设计

```yaml
change_id: diri-parity-child-tasks
status: in_progress
source: docs/research/diri-parity-lock.md
beads_parent: homie-h7n
```

## 1. 背景

`docs/research/diri-parity-lock.md` 已成为 Homie 对齐 Diri 的硬锁。当前 runtime 局部能力已有真实代码与验证证据，但仍有多条 UI、protocol/client、agent detection、terminal、artifact/git/automation、remote/release 相关 row 处于 `partial` 或 `missing`。

本需求的目标不是实现所有功能，而是把每个未完成 row 转换成可执行、可验证、可追踪的 child task，避免后续继续出现“文档覆盖但实现未完成”或“无证据标 implemented”的状态漂移。

## 2. 目标

- 为 `diri-parity-lock.md` 中每个非 `implemented` row 指定 owner、Beads、OpenSpec task、functional case 和证据门禁。
- 明确哪些 row 属于 UI/product、runtime-agent、protocol-client、automation-artifact、remote-release 五类实施线。
- 后续任何 row 从 `partial/missing` 改为 `implemented` 前，必须在本 change 的矩阵中有对应 evidence。
- 保持 Diri parity 总体状态为未完成，直到 parity lock 全部 row 为 `implemented` 且 `make parity-lock` 不再列出 incomplete rows。

## 3. 非目标

- 本 change 不直接实现剩余功能。
- 不把 group Bead 完成等同于 row implemented。
- 不降低 parity lock 的证据标准。
- 不绕过现有 `reference-parity-v1` 和 `diri-engine-migration` 的已交付证据。

## 4. 功能需求

### FR-1: Row 级任务矩阵

每个非 `implemented` row 必须包含：

- lock row id；
- 当前状态；
- owner crate/module；
- execution group；
- Beads id；
- OpenSpec task；
- functional case；
- required evidence；
- complete gate。

### FR-2: Beads 分组追踪

至少创建或复用五个 group Beads：

- UI/product parity；
- runtime-agent-terminal parity；
- protocol/client/CLI/MCP parity；
- artifact/git/automation parity；
- remote/usage/update/package/perf parity。

### FR-3: OpenSpec 对齐

`openspec/changes/diri-parity-child-tasks/` 必须包含 plan、tasks、alignment-report，且能映射到矩阵。

### FR-4: 防误标门禁

矩阵必须声明：row 只有在 required evidence 真实执行通过、且 parity lock 同步更新后，才能改为 `implemented`。

## 5. 验收标准

- `docs/verification/diri-parity-child-tasks/child-task-matrix.md` 覆盖所有当前 incomplete rows。
- 每个 row 有非空 Beads、OpenSpec task、functional case、evidence gate。
- `make parity-lock` 通过且仍正确列出未完成 row。
- `loopx check --registry .loopx/registry.json --scan-root <repo>` 通过。

