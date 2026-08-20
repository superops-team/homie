# OpenSpec Alignment — engine-control-handlers-split

## PRD → OpenSpec 对齐

| PRD 需求 | OpenSpec 任务 | 状态 |
|---------|--------------|------|
| hook/readiness 下沉 agent.rs | T2 | ✅ |
| governor/hibernation 下沉 governor.rs | T2 | ✅ |
| hello/capabilities 下沉 handshake.rs | T2 | ✅ |
| host 操作下沉 host.rs | T2 | ✅ |
| migrate 下沉 migrate.rs | T2 | ✅ |
| resume/reopen 下沉 resume.rs | T2 | ✅ |
| session 系列 + publish_updated/session_history 下沉 session.rs | T2 | ✅ |
| spawn/browser 下沉 spawn.rs | T2 | ✅ |
| worktree 操作下沉 worktree.rs | T2 | ✅ |
| 方法逐字迁移 | T1/T2（C5） | ✅ |
| pub(super) → pub(crate) | T2（C5） | ✅ |
| 公共 API 不变 | T3（C2） | ✅ |
| 单文件 < 800 行 | T4（C3） | ✅ |
| 全量验证 | T4（C4） | ✅ |
| 证据 | T5（C6） | ✅ |

## 验收覆盖

- C1 目录化：T2（1173 行拆为 10 子模块 + facade，最大 session.rs 229 行）。
- C2 引用方零改动：T3 code review。
- C3 行数阈值：T4。
- C4 编译/静态/测试：T4。
- C5 逐字迁移 + 可见性：T1/T2。
- C6 证据：T5。

无未映射需求，无漂移。
