# Spec Review Report

```yaml
change_id: diri-agent-detection
beads: homie-v4b
reviewed_spec:
  - prd-spec/features/diri-agent-detection/2026-08-07-diri-agent-detection-design.md
  - specs/agent-adapter-contract/README.md
review_skill: review-spec
status: p0_p1_actions_defined
```

## 1. 总体结论

- 可行性：高。
- 最大风险：如果只保留当前简化 descriptor schema，Homie 会有 19 个 agent 名称，但没有 Diri 的 combined manifest、readiness、resume style、golden screen 和 hook redaction 语义，后续 runtime/UI 会在错误合同上继续开发。
- 推荐方向：本阶段先把 `homie-agents` 做成纯数据/纯函数的 Diri parity 基座：combined manifest -> catalog/readiness/detection/hook redaction tests；不触碰 runtime。

## 2. 问题清单

| 优先级 | 维度 | 问题 | 影响 | 整改建议 |
|---|---|---|---|---|
| P0 | 上下文一致性 | `specs/agent-adapter-contract` 声称承接 Reference 19-agent catalog，但现有 `assets/agent-descriptors` 只有简化 descriptor，没有 Diri combined manifest 的 `agent` block 与 detection rules | Catalog 数量看似完整，但 status detection 和 descriptor 投影不等价 | 将 asset 升级为 Diri combined manifest；`ManifestEngine` 与 catalog loader 从同一文件读取 |
| P0 | SDD/TDD 适配 | 当前 tests 只验证 manifest id、first-class 粗粒度 authority 和少量 reducer/hook 场景 | 无法证明 GoldenScreenTests、ManifestAndRegionTests 的 Diri 行为被承接 | 增加 golden screen fixture tests、strict bundled manifest decode tests、catalog/readiness tests |
| P0 | 安全 | hook redaction 已覆盖简单 `key=value`，但对 nested payload、Authorization/Cookie header、URL query secret 的约束不足 | safe summary 或 needs-input summary 可能泄漏 tool args 中的 secret | 增加 hostile nested payload tests，并增强结构化脱敏 |
| P1 | 语义清晰性 | Resume 只有 token，无 style/session id source/injection 语义 | Codex subcommand、Cursor latest、Claude flag 等行为会被混淆 | 增加 `ResumeStyle`、`sessionIDFlag`、`injection`、`can_resume` 规则 |
| P1 | 边界划分 | readiness 是 runtime/client 可见能力，但 `homie-agents` 当前没有 readiness projection | UI 或 runtime 可能各自硬编码 PATH 检测 | 在 `homie-agents` 提供 resolver-injected readiness pure function；真实 PATH 解析留给 runtime |
| P1 | 失败模式 | unknown agent/unknown hook/broken manifest 的 fail-closed/fail-open 规则分散 | 后续调用方可能误触发 quick action 或整 catalog 崩溃 | 在 spec 中明确：unknown agent fallback process/no quick action；unknown hook fail-open safe summary；bundled manifest strict fail test |
| P2 | 可扩展性 | alias、glyph、foreground exec、injection 等字段缺失 | 后续 CLI/MCP/adoption lane 会继续补洞 | 本阶段只加入字段和 tests，不实现 runtime usage |

## 3. 整改后的完善方案

### 3.1 目标与范围

- 目标：完成 Diri M16-F001/M16-F002 第一阶段数据合同和最小可验证实现。
- 范围：`specs/agent-adapter-contract`、`assets/agent-descriptors`、`crates/homie-agents`、本 change 的 PRD/OpenSpec/evidence。
- 非目标：runtime spawn、storage persistence、UI picker、MCP/CLI bridge。

### 3.2 设计原则

- 单一事实源：agent descriptor 和 detection rules 同文件。
- 纯函数优先：readiness 通过 resolver 注入，tests 不依赖真实 PATH。
- Conservative fallback：unknown agent 不崩溃，但只能作为 process/no binary/no quick action。
- 安全默认：任何 summary、diagnostic、safe payload 都先脱敏再输出。
- 不为旧简化 descriptor 增加兼容层；按仓库规则直接升级数据合同。

### 3.3 核心方案

| 模块 | 方案 |
|------|------|
| Manifest assets | 用 Diri combined manifest 形态替换现有简化 descriptor；保留 Homie 需要的 19-agent catalog |
| Catalog | 新增 `AgentCatalog`、`AgentReadinessItem`、`AgentReadinessResult`、`ResumeStyle`、`AgentInjection` |
| Detection | `ManifestEngine::load_dir` 继续读取同一目录；strict tests 保证 bundled full manifest 有 rules |
| Readiness | `readiness_with_resolver` 对 launchable agents 生成 binary/path/available/descriptor |
| Hook redaction | 增强 `detect::redact` 与 `hooks::safe_payload_summary`，覆盖 nested/URL/header secret |
| Spec | 将 Diri mapping、test mapping、failure modes、functional cases 写入 component spec |

### 3.4 兼容与风险控制

- 不保留旧 `resume: { token }` 简化语义；直接使用 Diri 风格 `resume: { style, token }`。
- 不在本阶段暴露 runtime feature flag；tests 证明纯 crate 行为。
- 如果某个 Diri manifest 因 JSON/regex 不兼容导致测试失败，先修正 parser/manifest，不降低 golden 断言。

## 4. 分层任务拆解

| 层级 | 任务 | 交付物 | 依赖 | 优先级 |
|---|---|---|---|---|
| Spec | 更新 agent-adapter-contract Diri parity mapping | component spec diff | PRD | P0 |
| Data | 升级 19 个 agent descriptor 为 combined manifest | `assets/agent-descriptors/*.json` | Diri manifests | P0 |
| Catalog API | 实现 catalog/readiness/resume pure functions | `crates/homie-agents/src/lib.rs` | Data | P0 |
| Detection tests | 增加 golden screen 和 strict manifest tests | `crates/homie-agents/tests/*` | Data | P0 |
| Security tests | 增加 hostile hook redaction tests | `hook_parser.rs` | Parser | P0 |
| Verification | 运行 focused quality gates 并记录 | `release-readiness-report.md` | Implementation | P0 |

## 5. 测试规划

| 类型 | 覆盖点 | 关键用例 | 执行阶段 |
|---|---|---|---|
| Contract | Catalog id/alias/resume/readiness | `bundled_catalog_projects_diri_manifest_fields`, `readiness_projects_launchable_agents_only` | RED/GREEN |
| Golden | Diri screen parity | Claude permission, Codex action required, Cursor confirm, Gemini working | RED/GREEN |
| Security | Redaction | nested authorization/cookie/token, URL query secret | RED/GREEN |
| Regression | Reducer/hook 既有行为 | subagent isolation, process-only exit, idle anti-flicker | GREEN |
| Quality | Rust gates | fmt/check/clippy/test/diff/pre-commit | 准出 |

## 6. 开发排期

| 阶段 | 顺序 | 工作项 | 风险与缓冲 | 验收物 |
|---|---|---|---|---|
| P0 文档门禁 | 1 | PRD/spec review/functional cases/OpenSpec | 若范围扩张到 runtime，立即停止 | 文档齐备 |
| TDD-1 | 2 | Catalog/readiness failing tests -> implementation | Diri resume style 字段需一次性对齐 | focused tests pass |
| TDD-2 | 3 | Golden failing tests -> combined manifest/data/parser fix | Manifest JSON 规则可能暴露 parser 缺字段 | golden tests pass |
| TDD-3 | 4 | Redaction failing tests -> implementation | 不应过度 mask 普通文本 | hostile tests pass |
| 准出 | 5 | quality gates + code review reports | 全仓未跟踪文件可能影响 hook | release report |

## 7. 待确认问题

- 无需用户补充即可进入第一阶段实现；Diri 源码和 tests 已在仓库 `diri/` 子树可读。
- 完整 runtime readiness command 和 UI picker 归后续 runtime/UI lane，不在本 change 内确认。
