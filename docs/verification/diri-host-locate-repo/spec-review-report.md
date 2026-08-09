# Spec Review Report: Diri host.locate_repo

## 1. 总体结论

- 可行性：高。
- 最大风险：把 remote host 行为伪装成已完成真实 SSH/node E2E。
- 推荐方向：只实现本地可验证的 Diri 等价协议和 repo 匹配基础能力，parity lock 保持 `partial`。

## 2. 问题清单

| 优先级 | 维度 | 问题 | 影响 | 整改建议 |
|---|---|---|---|---|
| P1 | 范围边界 | `host.locate_repo` 容易被误写成真实远端扫描完成 | 会错误关闭 REM-003 | PRD 和 readiness 明确 no real SSH/node E2E |
| P1 | 协议一致性 | JSON 字段若使用 camelCase 默认会得到 `originUrl/sessionId` | 与 Diri MCP/client 不兼容 | DTO 显式 rename 为 `originURL/sessionID` |
| P2 | 可测试性 | 依赖真实 git/ssh 会让测试不稳定 | 无法持续跑本地门禁 | 读取 `.git/config` fixture，避免 shell |
| P2 | 安全 | origin URL 可能含敏感片段 | evidence/log 泄漏 | 测试使用 example.invalid，报告不记录真实 URL |

## 3. 整改后的完善方案

实现协议 DTO、remote 纯逻辑、client dispatch 和 CLI fixture E2E。候选路径匹配是只读行为；无 origin 返回空结果，有 origin 未命中返回 origin-only。真实远端扫描、host 账号和 node E2E 留给后续 `REM-001/REM-003` lane。

## 4. 分层任务拆解

| 层级 | 任务 | 交付物 | 依赖 | 优先级 |
|------|------|--------|------|--------|
| Protocol | Diri-compatible DTO | `homie-proto` tests | none | P1 |
| Remote | repo origin discovery/match | `host_locate_repo` tests | none | P1 |
| Client | request handler | `HomieClient::locate_repo` | storage/runtime | P1 |
| CLI | fixture E2E | `homie host locate-repo` | client | P1 |
| Evidence | parity lock and readiness | docs + Beads | tests | P1 |

## 5. 测试规划

| 类型 | 覆盖点 | 关键用例 | 执行阶段 |
|------|--------|----------|----------|
| Unit | DTO spelling | FC-DHLR-001 | RED/GREEN |
| Unit | origin discovery | FC-DHLR-002..004 | RED/GREEN |
| Integration | CLI real binary | FC-DHLR-005 | GREEN |
| Quality | check/clippy/diff/parity | FC-DHLR-006 | GAUNTLET |

## 6. 开发排期

| 阶段 | 顺序 | 工作项 | 风险与缓冲 | 验收物 |
|------|------|--------|------------|--------|
| Spec | 1 | PRD/OpenSpec/Case | 无 | 文档齐备 |
| TDD | 2 | RED tests | DTO/API 未存在 | 失败测试 |
| Impl | 3 | remote/proto/client/CLI | storage API 需轻量补充 | 测试通过 |
| Verify | 4 | 门禁与证据 | parity 只能 partial | readiness report |

## 7. 待确认问题

- 真实远端 host 扫描路径和 node 协议仍需后续 lane 设计，不在本 change 中解决。
