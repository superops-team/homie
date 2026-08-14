# Brooks 架构审计问题治理设计文档

## 1. 概述

### 1.1 问题/动机

本设计文档记录 `brooks-lint:brooks-audit` 对 Homie 当前架构的结构级审计结论，并规划后续分阶段治理方案。审计范围为整个 Homie 项目：

- Rust workspace: `homie/crates/*`
- Swift CLI/Core/MCP: `Sources/*`
- 长期规格: `specs/`
- 开发规范和质量门禁: `docs/development/*`

审计总体结论：Homie 的主干分层方向是正确的，`homie-proto` / `homie-client` / `homie-engine` / `homie-app` 的依赖方向基本成立；但若干组合根和高 fan-out 模块仍在承载过多变化原因，后续功能继续叠加会扩大回归半径。

### 1.2 目标

1. 将 Brooks 架构审计识别的问题固化为可追踪 PRD/spec，而不是停留在聊天结论。
2. 明确每个结构问题的症状、影响、优化方向和验收方式。
3. 按风险和收益拆分治理阶段，避免一次性大重构。
4. 保持外部行为不变，以可验证的小切片逐步降低维护成本。
5. 为后续 OpenSpec 拆解和 dev-loop 提供来源文档。

### 1.2.1 本 PRD 的关闭口径

本 PRD 是 parent-level 架构治理文档。Beads `homie-om7` 的关闭范围只包含：

1. 记录 Brooks audit findings；
2. 完成本 PRD 的 spec review；
3. 产出功能验证 Case；
4. 创建 OpenSpec 对齐文档；
5. 为后续 child Beads 明确拆分顺序和验收规则。

`homie-om7` 不直接承诺完成 Phase 1-4 的代码重构。Phase 1-4 必须分别新建 child Beads、独立 PRD/OpenSpec/evidence，并按 dev-loop 单独交付。

### 1.3 非目标

- 不在本 PRD 中直接实施代码修改。
- 不用 `homie-om7` 承载 Phase 1-4 的代码落地。
- 不一次性重写 `homie-app` 或 `homie-engine`。
- 不改变现有用户可见功能、协议语义、会话数据或远程运行行为。
- 不删除 Swift CLI/Core/MCP 支持；Swift 仍保留为 CLI/protocol/core/MCP glue，直到单独 PRD/spec 明确迁移。
- 不把所有大文件按行数机械拆分；只有当拆分能降低真实变化半径、明确生命周期或提升测试 seam 时才执行。

### 1.4 基线快照

本 PRD 的审计基线为：

- branch: `main`
- baseline commit: `7e2376248e74838f53ebd6e23379837986b457d1`
- audit mode: `brooks-lint:brooks-audit`
- health score: `80/100`

后续每个 Phase 启动前必须刷新基线，包括目标文件行数、测试状态、相关 specs 和未提交工作区状态，不能只复用本文的静态行数。

### 1.5 与存量 PRD/spec 的关系

| 存量文档 | 关系 | 本 PRD 的处理 |
|----------|------|---------------|
| `prd-spec/refactors/gpui-architecture-hardening/2026-08-14-gpui-architecture-hardening-design.md` | 上游 GPUI 架构硬化总纲 | 复用其 GPUI shell 合同和 child Beads 思路；本文补 Brooks audit 的跨层统一优先级 |
| `prd-spec/refactors/gpui-large-module-test-boundaries/2026-08-13-gpui-large-module-test-boundaries-design.md` | GPUI 大模块纯逻辑测试边界 | Phase 1/2 复用其“纯逻辑优先、行为不变”的拆分原则，不重复新建路线 |
| `prd-spec/refactors/protocol-contract-golden-fixtures/2026-08-13-protocol-contract-golden-fixtures-design.md` | Swift/Rust 协议契约 fixtures | Phase 4 不重新设计协议 fixtures，只补质量门禁集成和 drift gate 关闭条件 |
| `specs/gpui-shell.md` | 长期 GPUI shell 合同 | Phase 1/2/RootView 变更必须同步评估是否更新该 spec |
| `specs/engine-session-runtime.md` | Engine runtime 合同 | Phase 3 变更必须保持 runtime authority 和 PTY 环境合同不变 |

## 2. 审计问题记录

### 2.1 GPUI feature containers 承载过多变化原因

**Symptom:**
`homie-app` 中多个文件同时承担渲染、状态转移、异步任务、投影规则和交互策略：

- `homie/crates/homie-app/src/inspector.rs`: 约 4692 行；
- `homie/crates/homie-app/src/surface_shell.rs`: 约 4362 行；
- `homie/crates/homie-app/src/sidebar/view.rs`: 约 4347 行；
- `homie/crates/homie-app/src/terminal_pane.rs`: 约 3495 行。

**Source:**
Fowler — Refactoring / Divergent Change；Ousterhout — A Philosophy of Software Design / Information Hiding。

**Consequence:**
单个 UI 政策变化可能需要同时理解 artifact projection、tab state、focus、async task、terminal render 和 store effect，容易引入跨区域回归。

**Remedy:**
继续沿用 `specs/gpui-shell.md` 的方向，把高变化子域拆成明确生命周期和输入输出的模块。优先治理：

1. Inspector artifact projection 与 artifact list render；
2. Inspector review workflow；
3. TerminalPane attachment / find / input 子域；
4. SurfaceShell history / worktrees / settings 子域。

### 2.2 `RootView` 仍不是足够窄的 shell composition root

**Symptom:**
`RootView` 当前同时拥有：

- child entity 组合；
- 全局 keyboard routing；
- sidebar / inspector slide state；
- auxiliary terminal lifecycle；
- service event bridge；
- sound / notification policy；
- window placement persistence。

**Source:**
Clean Architecture — SRP；Fowler — Refactoring / Divergent Change。

**Consequence:**
新增 app-shell 行为时容易与通知、声音、辅助终端、窗口持久化等无关逻辑耦合，导致根组件改动频繁且验证面过大。

**Remedy:**
保留 `RootView` 作为组合根和全局路由中心，但逐步把以下逻辑迁出：

- service event bridge；
- sound / notification policy；
- auxiliary terminal lifecycle；
- window placement persistence debounce。

### 2.3 `ControlServer` 同时是 RPC dispatcher 和运行时协调器

**Symptom:**
`homie-engine/src/control.rs` 约 3729 行，单个 `ControlServer` 实现覆盖：

- session spawn / read / kill / resume；
- remote spawn；
- host sync / initialize / directory listing / locate repo；
- browser call；
- worktree create / list / remove；
- migration；
- history；
- environment refresh；
- hibernate / wake；
- shutdown。

**Source:**
Clean Architecture — ISP / DIP；Fowler — Refactoring / Divergent Change。

**Consequence:**
新增或修改一个 protocol method family 时，会扩大到整个 control dispatcher 的理解和测试范围；不同 method family 共享隐式上下文，容易形成粗粒度测试 seam。

**Remedy:**
保持 wire dispatcher 对外稳定，但把 method family 实现拆到独立模块，统一通过共享 `ControlContext` 或等价上下文访问 registry、events、remote manager、injection、browser pool 等资源。

建议拆分方向：

- `session_control`
- `remote_control`
- `host_control`
- `worktree_control`
- `history_control`
- `environment_control`

### 2.4 Rust / Swift protocol 和 manifest 镜像仍是长期 drift surface

**Symptom:**
Rust 侧拥有 `homie-proto` 和 Engine manifests，Swift 侧仍有 `Sources/HomieProtocol`、`Sources/HomieCore` manifest mirror、CLI/MCP schema 和 parity tests。项目文档已明确 Rust 是权威源，但架构上仍存在两套协议/manifest 表达。

**Source:**
The Pragmatic Programmer — DRY；Ousterhout — Information Leakage；Software Engineering at Google — Hyrum's Law。

**Consequence:**
协议字段或 manifest schema 变化时，Rust 侧测试通过不代表 Swift CLI/MCP 行为正确；若未运行 drift gate 或 parity fixtures，可能产生静默兼容性问题。

**Remedy:**
把 Swift/Rust parity 从“约定”提升为“发布门禁”：

- Rust 作为唯一人工维护源；
- Swift mirror 尽量生成；
- 每次 protocol / manifest 字段变化必须更新 parity fixtures；
- CI 和本地 quality gate 明确阻断 drift。

## 3. 分阶段方案设计

### Phase 0: 架构治理基线固化

目标：先把审计问题、模块边界和治理优先级固化，避免后续实现从聊天上下文出发。

交付：

- 本 PRD/spec；
- 后续 OpenSpec change；
- 对 `specs/gpui-shell.md` 和 `specs/engine-session-runtime.md` 的影响评估；
- Brooks audit finding → OpenSpec task 映射。

验收：

- 每个 finding 都有对应任务或明确 deferred 理由；
- 不进入代码实现前完成 spec review。
- 产出 `FC-01` 到 `FC-08` 功能验证 Case；
- OpenSpec alignment 明确 parent PRD 与 child Beads 的边界。

### Phase 1: Inspector 先行切片

选择理由：

- Inspector 当前功能边界相对清晰：Info / Review / Code / Artifacts；
- 近期用户已经连续反馈 Artifacts 顶部和右侧展示清洁度问题；
- `inspector.rs` 体量最大，且同时包含 diff、review、artifact、ask、markdown、tab 等逻辑。

建议拆分：

```text
homie/crates/homie-app/src/inspector/
├── mod.rs
├── artifacts.rs
├── review.rs
├── diff_view.rs
├── ask.rs
└── tabs.rs
```

第一刀只建议抽 `artifacts.rs`：

- `artifact_count`;
- `artifact_visible`;
- `artifact_title`;
- `render_artifact_row`;
- PR status render 中与 artifact list 强相关的投影逻辑。

禁止事项：

- 不同时移动 Review / Diff / Ask 逻辑；
- 不改变 Inspector tab 顺序和偏好持久化；
- 不改变 session artifact 数据模型；
- 不改变当前 Inspector 用户可见行为，除非 child PRD 明确说明。

验收：

- 外部 UI 行为不变；
- Inspector 相关测试通过；
- `inspector.rs` 减少至少一个明确子域；
- `artifacts.rs` 有纯函数测试覆盖。
- `cargo test --manifest-path homie/Cargo.toml -p homie-app inspector -- --nocapture` 通过；
- 真实 dev app smoke 验证 Inspector Artifacts、Info、Review、Code tab 可用。

### Phase 2: TerminalPane 子域拆分

选择理由：

- TerminalPane 同时处理 attachment、grid render、find、clipboard image、keyboard encoding、resize/reflow、scrollback 和 overlay。
- 该模块是 shell 快捷键、PTY size、terminal rendering 的高风险路径。

建议拆分：

```text
homie/crates/homie-app/src/terminal_pane/
├── mod.rs
├── attachment.rs
├── find.rs
├── input.rs
├── resize.rs
└── header.rs
```

第一刀优先抽纯逻辑：

- resize / reflow planning；
- keyboard event mapping；
- find state scheduling。

禁止事项：

- 不改 `SessionAttachment` wire 调用路径；
- 不改变 terminal focus / paste / IME / resize 行为；
- 不与 UI 视觉重排混在同一 child change。

验收：

- `terminal_pane` targeted tests 通过；
- terminal real app smoke 通过；
- shell `Ctrl+L` 回归测试仍通过；
- 不改变 `SessionAttachment` 协议调用路径。
- `cargo test --manifest-path homie/Cargo.toml -p homie-app terminal_pane -- --nocapture` 通过。

### Phase 3: `ControlServer` method family 拆分

选择理由：

- Engine 是 daemon/runtime 权威，错误影响面最大；
- `ControlServer` 当前高 fan-out 且 method family 多；
- 但该路径测试覆盖较强，适合按 method family 保持行为不变地提取。

建议步骤：

1. 新增 `ControlContext`，只承载共享依赖；
2. 先抽无状态或低风险 family：`worktree_control`、`environment_control`；
3. 再抽高风险 family：`session_control`、`remote_control`；
4. 每个 family 保留原有 JSON wire shape 和 error code。

协议不变约束：

- method name 不变；
- request/response JSON shape 不变；
- error code 和常见错误消息不变，除非 child PRD 明确列出；
- event publish 顺序不变；
- `homie-client` public API 不变；
- socket owner-only、安全检查、shutdown/adoption 行为不变。

验收：

- `cargo test --manifest-path homie/Cargo.toml -p homie-engine` 通过；
- control socket integration tests 通过；
- `git diff --check`、`cargo fmt --check` 通过；
- 不改变 `homie-client` API。
- targeted tests 必须覆盖被抽取 method family，而不是只跑全包测试。

### Phase 4: Swift/Rust protocol drift gate 强化

选择理由：

- 这是长期维护风险，不一定每天触发，但一旦触发会影响 CLI/MCP 和 release。

建议步骤：

1. 复用 `protocol-contract-golden-fixtures` 方案，不重新设计 fixture 格式；
2. 盘点当前 Rust `homie-proto` 与 Swift `Sources/HomieProtocol` 的字段映射缺口；
3. 将 manifest drift check 和 protocol fixture check 纳入 quality gate；
4. 文档明确“新增字段时必须更新哪些 fixture 和命令”。

验收：

- Swift/Rust protocol fixture 能在本地单命令验证；
- manifest drift 修改会失败并给出修复命令；
- `docs/development/quality-gates.md` 明确相关命令。

## 3.5 功能验证 Case

| Case | 覆盖 finding / Phase | 验证目标 | 执行方式 | 通过标准 |
|------|----------------------|----------|----------|----------|
| FC-01 | 全部 / Phase 0 | Brooks findings 全部可追踪 | 检查 PRD、OpenSpec alignment | 每个 finding 至少映射一个 task 或 deferred reason |
| FC-02 | 全部 / Phase 0 | parent PRD 与 child Beads 边界清晰 | 检查 Beads / OpenSpec | `homie-om7` 只关闭规划；代码落地使用 child Beads |
| FC-03 | GPUI containers / Phase 1 | Inspector artifact 子域可独立测试 | `cargo test --manifest-path homie/Cargo.toml -p homie-app inspector -- --nocapture` | 测试通过，artifact 纯逻辑在子模块覆盖 |
| FC-04 | RootView / 后续 child Phase | RootView 不继续扩大职责 | review diff | 新逻辑进入 controller/entity；RootView 只保留组合和路由 |
| FC-05 | TerminalPane / Phase 2 | TerminalPane 拆分不影响输入/resize/find | `cargo test --manifest-path homie/Cargo.toml -p homie-app terminal_pane -- --nocapture` | 测试通过，shell smoke 通过 |
| FC-06 | ControlServer / Phase 3 | method family 拆分不改 wire shape | `cargo test --manifest-path homie/Cargo.toml -p homie-engine` + targeted control socket tests | 原 method response/error/event 行为不变 |
| FC-07 | Protocol parity / Phase 4 | Swift/Rust protocol drift 被阻断 | `swift test --package-path .` + `cargo test --manifest-path homie/Cargo.toml -p homie-proto` | 双端 fixtures 通过 |
| FC-08 | 每个 GPUI child change | 真实 app 可用 | `HOMIE_ENGINE_PATH=... ./scripts/dev.sh` | dev app 启动，目标 UI 路径人工验证通过 |

## 4. 实施步骤

### Step 1: Spec review

- 对本 PRD 做 review-spec / Brooks review；
- 确认是否按 Phase 1 先做 Inspector，或改为 Engine control 优先；
- 输出 `docs/verification/architecture-audit-hardening/spec-review-report.md`。

### Step 2: OpenSpec 拆解

创建：

```text
openspec/changes/architecture-audit-hardening/
├── plan.md
├── tasks.md
└── alignment-report.md
```

要求每个 task 关联至少一个 Brooks finding 和验证 case。

OpenSpec 必须包含以下映射表：

```text
Brooks finding -> Phase -> child Bead -> OpenSpec task -> FC case -> evidence path
```

### Step 3: Phase 1 实施

- 只处理 Inspector artifact 子域；
- 不改其它大模块；
- 先写纯函数/GPUI 测试；
- 再做提取。

### Step 4: Phase 2/3/4 逐期推进

每个 Phase 单独 Beads / OpenSpec / verification evidence，避免一个大分支长期悬挂。

每个 Phase 的 child Bead 命名建议：

| Phase | 建议 child Bead 标题 | change_id |
|-------|----------------------|-----------|
| Phase 1 | 抽离 Inspector artifact 子域 | `inspector-artifacts-module-extraction` |
| Phase 2 | 抽离 TerminalPane 单一纯逻辑子域 | `terminal-pane-logic-slice-extraction` |
| Phase 3 | 拆分 ControlServer 低风险 method family | `control-server-method-family-extraction` |
| Phase 4 | 强化 Swift/Rust protocol parity gate | `protocol-parity-quality-gate` |

## 5. 涉及文件

可能涉及：

- `homie/crates/homie-app/src/inspector.rs`
- `homie/crates/homie-app/src/terminal_pane.rs`
- `homie/crates/homie-app/src/root.rs`
- `homie/crates/homie-engine/src/control.rs`
- `homie/crates/homie-proto/src/*`
- `Sources/HomieProtocol/*`
- `Sources/HomieCore/*`
- `specs/gpui-shell.md`
- `specs/engine-session-runtime.md`
- `docs/development/quality-gates.md`

## 6. 验证计划

### 6.1 Phase 1 Inspector

```bash
cargo fmt --check
cargo test --manifest-path homie/Cargo.toml -p homie-app inspector -- --nocapture
cargo test --manifest-path homie/Cargo.toml -p homie-app terminal_pane -- --nocapture
HOMIE_ENGINE_PATH=<target/debug/homied-rs> homie/scripts/dev.sh
```

### 6.2 Phase 2 TerminalPane

```bash
cargo test --manifest-path homie/Cargo.toml -p homie-app terminal_pane -- --nocapture
cargo test --manifest-path homie/Cargo.toml -p homie-engine shell_session_reports_xterm_256color -- --nocapture
HOMIE_ENGINE_PATH=<target/debug/homied-rs> homie/scripts/dev.sh
```

### 6.3 Phase 3 Engine control

```bash
cargo test --manifest-path homie/Cargo.toml -p homie-engine
cargo test --manifest-path homie/Cargo.toml -p homie-engine control_socket -- --nocapture
```

### 6.4 Phase 4 Protocol parity

```bash
swift test --package-path .
cargo test --manifest-path homie/Cargo.toml -p homie-proto
scripts/check-agent-manifest-drift.sh
```

## 6.5 文档与 OpenSpec 验证

```bash
git diff --check
test -s openspec/changes/architecture-audit-hardening/plan.md
test -s openspec/changes/architecture-audit-hardening/tasks.md
test -s openspec/changes/architecture-audit-hardening/alignment-report.md
rg -n "FC-01|FC-02|FC-03|FC-04|FC-05|FC-06|FC-07|FC-08" \
  openspec/changes/architecture-audit-hardening \
  docs/verification/architecture-audit-hardening
```

## 7. 验收标准

1. 每个 Brooks finding 都有明确处理阶段、验收命令和 deferred 说明。
2. 每个 Phase 都能独立 landed，不依赖未完成的后续 Phase。
3. 不改变用户可见行为，除非对应 Phase PRD 明确说明。
4. 每次拆分后对应 targeted tests 通过，且原有 smoke path 不回退。
5. 被拆分出来的新模块有一句话职责说明、稳定输入输出和本地测试。
6. `RootView`、`Inspector`、`TerminalPane`、`ControlServer` 的后续新增功能优先落在对应子模块，而不是继续扩大组合根。
7. `homie-om7` 关闭时必须存在 spec review、functional cases、OpenSpec alignment；不得以代码重构完成度作为关闭条件。
8. 任一 child Phase 若需要改变用户可见行为，必须新建独立 PRD，不得复用本文默认“不改变行为”的前提。

## 8. Beads 追踪

- Beads: `homie-om7`
- change_id: `architecture-audit-hardening`
- source: `brooks-audit`
- 类型: refactor planning
- 优先级: P1
