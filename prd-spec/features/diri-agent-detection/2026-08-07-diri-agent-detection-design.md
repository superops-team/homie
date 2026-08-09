# Diri Agent Catalog/Detection Parity 第一阶段设计文档

```yaml
change_id: diri-agent-detection
beads: homie-v4b
lane: lane-agent
status: draft_ready_for_openspec
source:
  - docs/research/diri-module-inventory.md
  - docs/verification/diri-module-inventory/spec-dependency-analysis.md
  - docs/verification/diri-module-inventory/bingo-component-spec-review-report.md
  - specs/agent-adapter-contract/README.md
  - diri/Sources/DirijorCore/Resources/manifests
  - diri/Tests/DirijorDetectionTests
```

## 1. 概述

### 1.1 问题/背景

Homie 的 `agent-adapter-contract` 已经定义了 agent descriptor、status authority、hook/notify parser 和 status reducer 的方向，但上一轮组件审查指出该规格仍不能直接承接 Diri 的 M16 agent detection 模块：当前合同缺少对 AgentCatalog、AgentReadiness、GoldenScreen、HookParsing redaction、resume/approve/deny 行为的逐项映射。

Diri 的做法是把 agent descriptor 与 detection rules 放在同一份 manifest 中，Catalog、Readiness、Detection Engine、Reducer 和 UI agent picker 都从同一份数据投影。Homie 第一阶段需要先把这个合同落到 `assets/agent-descriptors` 与 `crates/homie-agents`，让后续 runtime/UI lane 使用稳定数据模型，而不是继续依赖硬编码或简化 descriptor。

### 1.2 目标

- G1: `homie-agents` 可加载 19 个 Diri agent manifest，并从同一份 manifest 生成 catalog projection。
- G2: `homie-agents` 提供 Diri 风格 readiness projection：只探测有真实 binary 的 launchable agent，`shell`/`generic` 不进入 binary readiness 列表。
- G3: `homie-agents` 承接 Diri golden screen 第一阶段用例，至少覆盖 Claude Code、Codex、Cursor、Gemini 的 idle/working/blockedPermission 关键状态。
- G4: hook/notify parser 输出稳定事件和 safe summary，嵌套 secret、Authorization、Cookie、URL/token 等敏感内容在进入 reducer diagnostic 前脱敏。
- G5: `specs/agent-adapter-contract/README.md` 增加 Diri parity mapping、readiness、golden、redaction、fail-closed/fail-open 规则，作为后续 runtime lane 的长期合同。

## 2. 非目标

- NG1: 不修改 `homie-runtime`，不实现真实 PTY/process spawn，不接入实际 session lifecycle。
- NG2: 不修改 UI、CLI、MCP、storage 或 LLM proxy。
- NG3: 不声明完整 Diri parity 已完成；本阶段只完成 agent catalog/detection 的第一阶段合同与最小实现。
- NG4: 不引入新依赖，不实现实时 manifest override/hot reload。
- NG5: 不处理 macOS native notification、菜单栏、声音或 quick approve UI；本阶段仅保证 needs-input/hook redaction 数据可安全供下游消费。

## 3. 用户场景

### 场景 1: 新建 agent 菜单读取统一 catalog

**Given** Homie 启动并加载内置 agent manifests。  
**When** app/runtime 查询 `homie-agents` catalog projection。  
**Then** 返回 19 个 Diri agent descriptor，first-class、binary、resume、approve、deny、status authority 与 manifest 一致。

### 场景 2: Readiness 只检查可启动 CLI

**Given** manifest 中包含 `claude-code`、`codex`、`shell`、`generic`。  
**When** 调用 readiness projection 并传入 PATH resolver。  
**Then** 只对有 binary 的 launchable agent 产生 readiness item，缺失 binary 标记 unavailable，但不影响其他 agent；`shell` 和 `generic` 不被探测。

### 场景 3: Golden screen 状态识别

**Given** Diri golden screen 中 Claude permission dialog、Codex action required title、Cursor confirm dialog、Gemini working cancel timer。  
**When** `ManifestEngine` 用 Homie 内置 manifest 评估 screen snapshot。  
**Then** 输出与 Diri `GoldenScreenTests` 一致的 `ManifestState`、rule id、options、prompt excerpt。

### 场景 4: Hook/notify 安全解析

**Given** Claude hook payload 或 Codex notify payload 包含 nested token、authorization、cookie、URL query secret。  
**When** parser 生成 `ParsedHook`/`ParsedNotify`。  
**Then** event 类型稳定，safe summary 不包含原始 secret；subagent payload 不会改写父 session title 或 needs-input 状态。

## 4. 功能需求

| ID | 需求 | 优先级 | 来源 |
|----|------|--------|------|
| FR-1 | catalog loader 必须从 combined manifest 读取 agent descriptor 与 detection rules | P0 | M16-F001 |
| FR-2 | catalog 必须包含 19 个 Diri agent id，支持 id/shortLabel/alias 解析 | P0 | M16-F001 |
| FR-3 | descriptor 必须表达 glyph、aliases、spawnArgs、sessionIDFlag、resume style、returnToLoginShell、injection、foregroundExecNames、approve/deny | P0 | M16-F001 |
| FR-4 | readiness projection 必须只探测 launchable binary，缺失 binary fail-closed 为 unavailable，不影响 catalog 加载 | P0 | M16-F001 |
| FR-5 | full status manifest 必须至少有一条 rule；process-only manifest 可为空 | P0 | M16-F002 |
| FR-6 | golden screen tests 必须覆盖 Claude/Codex/Cursor/Gemini 的 blocked/idle/working 代表状态 | P0 | M16-F002 |
| FR-7 | hook parser 必须保留 Diri 的 Claude/Codex stable event、needs-input、first-prompt title、subagent 隔离语义 | P0 | M16-F002, M08-F001 |
| FR-8 | hook/screen/notify 摘要必须结构化脱敏，覆盖 secret key、Authorization、Cookie、password、token、API key、URL secret query | P0 | M08-F001 |
| FR-9 | unknown hook event fail-open：保留 safe summary，不阻塞 agent 运行，不产生自动 quick action | P1 | M16-F002 |
| FR-10 | spec 必须映射 Diri 测试到 Homie 验证 gate，禁止仅凭 manifest 数量宣称 parity | P1 | 组件审查 |

## 5. 实现方案

### 5.1 Manifest 数据合同

`assets/agent-descriptors/*.json` 采用 Diri combined manifest 形态：

- 顶层 detection 字段：`schemaVersion`、`id`、`version`、`statusModel`、`rules`。
- 顶层 `agent` 字段：descriptor 元数据与启动能力。
- `homie-agents::Manifest` 继续承载 detection rules，`homie-agents::AgentManifest` 承载 `agent` block。
- `load_manifest` 改为读取 combined manifest 的 `agent` block，并把 descriptor id 归一为 manifest id。

### 5.2 Catalog 与 readiness

新增纯 Rust catalog projection：

- `AgentCatalog::new(Vec<AgentManifest>)` 根据 id 去重并稳定排序。
- `descriptor(id)` 对未知 id 返回 conservative fallback：process authority、无 binary、非 first-class。
- `resolve(name)` 支持 id、short label、alias 的大小写不敏感解析。
- `launchable()` 返回有 binary 的 descriptor。
- `readiness_with_resolver` 接收 resolver 函数，返回每个 launchable agent 的 binary/path/available/descriptor。

### 5.3 Golden screen 验证

不接 runtime。测试直接用 `ManifestEngine::load_dir(assets/agent-descriptors)` 加载内置 manifests，并用 Diri golden screen 快照断言：

- Claude idle prompt box、permission dialog、working spinner、transcript viewer skip。
- Codex action required title、confirm prompt、idle prompt。
- Cursor confirm dialog、working status line、idle prompt。
- Gemini confirm dialog、working cancel timer、idle prompt。

### 5.4 Hook redaction

保持 parser public API 不变，增强 redaction：

- 字段名命中 `token`、`secret`、`password`、`authorization`、`cookie`、`api_key`、`apikey` 时整值替换。
- 字符串中的 token/secret assignments、secret-bearing request headers、URL query 中敏感参数必须脱敏。
- `tool_summary` 和 `NeedsInputDetail.summary` 不包含原始 secret。

## 6. 影响文件

- `prd-spec/features/diri-agent-detection/2026-08-07-diri-agent-detection-design.md`
- `openspec/changes/diri-agent-detection/plan.md`
- `openspec/changes/diri-agent-detection/tasks.md`
- `openspec/changes/diri-agent-detection/alignment-report.md`
- `docs/verification/diri-agent-detection/*`
- `specs/agent-adapter-contract/README.md`
- `assets/agent-descriptors/*.json`
- `crates/homie-agents/**`

## 7. 测试计划

- TDD-1: catalog/readiness tests 先失败，再实现 catalog projection。
- TDD-2: golden screen tests 先失败，再替换 combined manifest 并修正 manifest loader。
- TDD-3: hook redaction nested/URL secret tests 先失败，再增强 redaction。
- Regression: 运行 `cargo test -p homie-agents`。
- Quality: 运行 `cargo fmt --all -- --check`、`cargo check -p homie-agents`、`cargo clippy -p homie-agents --all-targets -- -D warnings`、`git diff --check`。
- Security: 运行 `.githooks/pre-commit`；若 hook 因全仓未跟踪状态不可执行，在 release report 中记录实际失败原因。

## 8. 验收标准

- AC-1: 19 个 Diri manifest 均可由 `ManifestEngine` 严格解码。
- AC-2: catalog tests 证明 first-class、authority、resume、env scrub、alias、fallback 与 Diri 行为一致。
- AC-3: readiness tests 证明 launchable binary 探测不包含 `shell`/`generic`，缺失 binary 不阻断其他 agent。
- AC-4: golden screen tests 覆盖 Claude/Codex/Cursor/Gemini 关键状态并全部通过。
- AC-5: hook parser tests 证明 nested secret/URL query/Authorization/Cookie 被脱敏，subagent 隔离仍成立。
- AC-6: `specs/agent-adapter-contract/README.md` 包含 Diri parity mapping、functional cases、failure mode 和 verification gates。
- AC-7: verification report 记录所有命令、退出码、未运行项和残余风险，不把未运行门禁写成 pass。

## 9. Beads 跟踪

- Beads: `homie-v4b`
- change_id: `diri-agent-detection`
- spec path: `prd-spec/features/diri-agent-detection/2026-08-07-diri-agent-detection-design.md`
- dependency: `diri-module-inventory`
