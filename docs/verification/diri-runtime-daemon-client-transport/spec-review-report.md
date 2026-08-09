# Spec Review Report: diri-runtime-daemon-client-transport

```yaml
change_id: diri-runtime-daemon-client-transport
status: pass
implementation_status: not_run
beads: homie-nep
review_date: 2026-08-08
baseline_commit: 7ba3407
```

## 评审范围

- 中文 PRD
- 6 份受影响长期组件规格
- OpenSpec proposal/design/4 capability specs/plan/tasks/alignment
- Bead `homie-nep`

本报告只判定规格与实施计划可进入开发。Rust 实现、E2E、安全扫描和发布准出尚未执行，状态为 `not_run`。

## 评审概览

| 维度 | 判定 | 未解决问题数 |
|------|------|-------------|
| 1. 上下文逻辑连贯性 | 通过 | 0 |
| 2. 内容空洞排查 | 通过 | 0 |
| 3. 歧义点识别 | 通过 | 0 |
| 4. 大模型语义理解偏差 | 通过 | 0 |
| 5. SDD/TDD 开发模式适配 | 通过 | 0 |
| 6. 最小化实现原则 | 通过 | 0 |
| 7. 向下兼容隐患 | 通过 | 0 |
| 8. 存量业务影响 | 通过 | 0 |
| 9. 功能失效风险预判 | 通过 | 0 |
| 10. 落地可行性评估 | 通过 | 0 |
| 11. 任务拆解与排期 | 通过 | 0 |
| 12. 可扩展性影响 | 通过 | 0 |
| 13. 过度设计警惕 | 通过 | 0 |
| 14. 小而高效改动 | 通过 | 0 |
| 15. 代码优雅性 | 通过 | 0 |
| 16. 架构统一性 | 通过 | 0 |

总评：16/16 通过，0 个未解决阻断项。

## 逐维度结论

### 1. 上下文逻辑连贯性

Proposal 的 4 个 capabilities 均有同名 spec。PRD 16 个 FR 全部映射到 OpenSpec requirement、task 和 verification。术语统一为 daemon、RuntimeActor、TerminalSource、event cursor 和 terminal offset。

### 2. 内容空洞排查

未发现 TBD/TODO、视情况、适当、合理或无量化性能承诺。33 个 requirements 均有可执行 scenarios；计划明确文件、RED/GREEN 命令和完成结果。

### 3. 歧义点识别

frame 长度、payload 上限、队列、连接、stream、pending request、heartbeat、backoff、event ring、tail frequency 和 shutdown 顺序均已量化。

### 4. 大模型语义理解偏差

关键分支使用 WHEN/THEN；explicit launcher、no fallback、handler capability truth、T-102/T-103 边界均显式重复，没有依赖“同上”或聊天上下文。

### 5. SDD/TDD 开发模式适配

90 个 scenarios 可映射自动化测试；102 个 checkbox tasks 按 RED、最小实现、GREEN 排序。fake backend 仅为 server library 内部注入，real daemon 另有 subprocess gate。

### 6. 最小化实现原则

仅新增 Tokio、SHA-256；不引入 RPC/codec/async-trait/channel 包。remote、agent runtime、完整 storage ownership、styled terminal 和 release signing 均推迟给既定 owning changes。

### 7. 向下兼容隐患

本 change 有意删除 embedded/sync API，符合仓库不保留兼容层规则。迁移要求 app/CLI/MCP 和 daemon 作为一个原子 change/package closure 更新，回滚方式是完整 git revert。

### 8. 存量业务影响

已列出 proto/runtime/client/CLI/MCP/app/package 影响。评审补齐了原计划遗漏的 artifacts、ports、status、lineage、hook report 和 worktree overview 现存 typed client 行为。

### 9. 功能失效风险预判

覆盖 partial frame、hostile length、caller cancellation、timeout、disconnect、late response、connection/stream/pending/actor queue limits、slow consumer、server high queue overflow、signal shutdown 和 stale socket。

### 10. 落地可行性评估

RuntimeActor 修订为专用 OS thread，避免 rusqlite/PTY 阻塞 Tokio worker。git/history/output scan 进入单 worker/32-job LongRunningLane，防止长任务阻塞所有 runtime mutation。TerminalSource 修订为每 session 一个共享 tailer，避免 attachment 数放大 actor/file I/O。依赖均已在 package research 或 Diri baseline 验证。

### 11. 任务拆解与排期

20 个 task groups、102 个可追踪步骤、5 个 checkpoint。预计 21.25 engineering days；T-102 holder 修复不隐含计入 Wave 1A。

### 12. 可扩展性影响

固定 frame + typed stream metadata 允许 T-202 增加 styled grid diff，不改变 wire。handler registry 和 stream opener 支持按实际实现增加 capability，不由 `Method::ALL` 自动扩张。

### 13. 过度设计警惕

RuntimeBackend 仅服务 production adapter 与 deterministic transport tests；TerminalSource 是多 client 共享实时输出的必要 owner。没有引入通用 service container、插件协议或未来 remote transport 抽象。

### 14. 小而高效改动

优先复用现有 RuntimeSupervisor、holder output log、HeadlessScreen、GridUpdate、Serde、libc 和 package scripts。DTO 仅移动到 proto 以解除 client 的 runtime/storage 依赖。

### 15. 代码优雅性

职责边界明确：codec/DTO 在 proto，blocking ownership 在 actor，async stream 在 hub，transport lifecycle 在 client，GPUI 只消费 bridge projection。错误使用固定 code，不泄漏实现错误。

### 16. 架构统一性

方案符合项目 UDS runtime/client 硬边界、app 不依赖 runtime、daemon 单 owner、SDD/TDD 和 package dependency closure 规则，并采用项目已批准的 Tokio/Serde/rusqlite/alacritty/libc 技术栈。

## 评审中已修复的问题

| 问题 | 严重度 | 修复 |
|------|--------|------|
| capability 只列 dispatcher 分支，遗漏 app/CLI/MCP 现存 typed 方法 | High | 补入 status/artifacts/ports/lineage/hook/overview handlers、DTO 和 tasks |
| RuntimeActor 可能被理解为 Tokio task | High | 固定为 named dedicated OS thread |
| git/history/output scan 若进入 RuntimeActor 会造成队头阻塞 | High | 新增 single-worker/32-job LongRunningLane、hard deadlines 和 no-partial-commit |
| 每 attachment 独立 tail 会放大 actor/file I/O | High | 改为每 attached session 一个共享 TerminalSource |
| request future cancellation 未定义 | Medium | 增加 pending waiter cleanup 和 late-response scenario |
| connection/high queue/launcher incompatible endpoint 边界未量化 | Medium | 增加 64 connections、high overflow close、anti-restart scenarios |
| task 没有排期 | Low | 增加 20-task estimate，总计 20.5 engineering days |

## 风险清单

| 风险 | 严重度 | 缓解措施 |
|------|--------|---------|
| 单连接断开影响所有 streams | Medium | cursor/offset 持久恢复，单 stream overflow 优先 reset |
| handler 迁移面较大 | High | exact inventory tests，先 server/client 后消费者，原子删除旧 API |
| terminal full grid 初建成本 | Medium | session 级共享 source、64 KiB chunk、后台 tail |
| daemon/process test flake | Medium | absolute temp paths、serial process tests、fixed readiness probe |
| holder adoption 继续失败 | High | 保持 T-102 blocker，不改变 Wave 1A transport 判定 |
| app async bridge回归首帧 | High | two-worker runtime、bridge library test、first-frame gate |
| 当前未提交 `.gitignore` 忽略 canonical docs/spec 目录 | Medium | 提交前由用户批准恢复 ignore 规则或对本 change 文件显式 force-stage；本轮不擅自修改 |

## 门禁证据

| Command/check | Result |
|---------------|--------|
| `openspec status --change diri-runtime-daemon-client-transport` | pass, 4/4 |
| `openspec validate diri-runtime-daemon-client-transport --strict` | pass |
| requirement/scenario structural scan | pass, 34 requirements / 90 scenarios |
| OpenSpec task checkbox scan | pass, 102 tasks |
| PRD FR scan | pass, 16 FR |
| placeholder/vague-language scan | pass, no matches |
| `git diff --check` for change docs/specs | pass |

## 结论

规格与实施计划可进入 Wave 1A 开发。开始产品代码前仍需用户批准本轮 component/OpenSpec/spec-review 产物。实施必须按 `tasks.md` 增量更新，不得跳过 RED evidence、cross-entry E2E 或 T-102 blocker 诚实披露。

当前工作区的未提交 `.gitignore` 会隐藏新 PRD/OpenSpec/spec/evidence 文件；这不影响本地 OpenSpec validation，但在创建 commit 前必须解决纳管方式。
