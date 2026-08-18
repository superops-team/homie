# Spec Review Report — codex-acp-host-runtime

## 1. 审查对象

- `prd-spec/features/codex-acp-host-runtime/2026-08-18-codex-acp-host-runtime-design.md`
- `homie/crates/homie-engine/src/acp/{protocol,frame,host,approval,driver}.rs`
- `homie/crates/homie-engine/tests/acp_host.rs`
- `specs/engine-session-runtime.md`（ACP/PTY 边界记录）

## 2. 16 维度合规分析

| 维度 | 结论 | 说明 |
|------|------|------|
| 上下文逻辑连贯性 | PASS | 背景→目标→非目标→场景→需求→方案→验证 逻辑闭环 |
| 空洞表述 | PASS | 各需求落到具体 DTO/方法/常量，无空泛表述 |
| 歧义点 | PASS | stop/cancel 语义、approval 四态、steer 映射均明确 |
| 语义偏差 | PASS | ACP host 定义为 host 侧、provider 侧边界清晰 |
| SDD/TDD 适配 | PASS | 协议/framing/approval/driver 均有单测，E2E 有集成测试 |
| 最小化实现 | PASS | 不引 `codex-acp` crate、不接 session.spawn、不接 GPUI |
| 向下兼容 | PASS | PTY/holder 保持权威，非 ACP agent 行为不变 |
| 存量业务影响 | PASS | 仅新增 `pub mod acp;`，既有 driver/session 行为不变 |
| 风险预判 | PASS | 子进程 spawn 失败、id 不匹配、未知 kind、非法 JSON 均处理 |
| 落地可行性 | PASS | std-only 同步模型，与 engine 现有设计一致，无 tokio |
| 可扩展性 | PASS | `AcpClient` trait 抽象可注入 mock；协议 DTO 可扩展 provider |
| 过度设计 | PASS | 首阶段不引入 Transport trait 层级，用 `AcpStream` 精简 |
| 最小变更 | PASS | 新增 `acp/` 模块 + 一个集成测试 + 一个 spec 段落 |
| 代码优雅性 | PASS | 模块边界清晰、`Drop` 正确回收子进程/reader 线程 |
| 架构一致性 | PASS | 对齐 `typed-agent-driver-capabilities`、`specs/engine-session-runtime.md` |

## 3. 问题清单与严重等级

| 级别 | 数量 | 状态 |
|------|------|------|
| P0 | 0 | — |
| P1 | 0 | — |
| P2 | 0 | — |
| P3 | 0 | — |

## 4. 结论

16 维度全部 PASS，无 P0-P3 问题，spec 审查通过。
