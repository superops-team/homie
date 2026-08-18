# OpenSpec Alignment Report — homie-cli-config-ops

## 1. 目的

证明 PRD 功能需求（FR）、组件 spec、OpenSpec tasks 三者一一对应，无孤儿需求/任务。

## 2. PRD FR → OpenSpec Task 映射

| PRD 需求 | spec 合同（specs/homie-cli-config-ops.md） | OpenSpec Task |
|----------|--------------------------------------------|---------------|
| FR-1 config show/get | §4.1 Command Contract | T2 |
| FR-2 config set | §4.2 Command Contract | T3 |
| FR-3 config agent | §4.3、§5 Injection Parity | T4 |
| FR-4 增强 doctor | §4.4 Command Contract | T5 |
| FR-5 fix | §4.5 Command Contract | T6 |
| FR-6 homie skill | §8 Skill Contract | T7 |
| FR-7 安全边界 | §3 Config File、§7 Security | T1、T3、T9 |

## 3. 验收标准映射

| 验收标准 | 覆盖 Task | 覆盖验证 Case |
|----------|-----------|---------------|
| config show/get/set/agent 可用且脱敏 | T2、T3、T4 | FC-1、FC-2、FC-3、FC-4 |
| config agent 与注入一致 | T4 | FC-4 |
| doctor 新增 4 项且失败非零 | T5 | FC-5 |
| fix 幂等修复 | T6 | FC-6 |
| homie skill 存在且不泄露 key | T7 | FC-7 |
| homie.local.json 忽略 + 0600 + key 不泄露 | T1、T9 | FC-1、FC-9 |
| OpenSpec 对齐 + Beads 关闭 | T9 | FC-9 |

## 4. 无漂移声明

- 每条 PRD FR 均有对应 spec 合同与 task，无孤儿需求。
- 每条 task 均可回溯到 FR 或验收标准，无孤儿任务。
- 决策 A（JSON 统一）是对 PRD1 `gateway.local.toml` 的必要调整，已在本 PRD §1.4 记录，并同步
  映射到 T1；不改变 `specs/llm-gateway.md` 的协议/虚拟 key/用量合同本身，仅统一其本地配置
  文件格式（§6 Upstream Forwarding 的 `gateway.local.toml` 措辞需在实现时同步为
  `homie.local.json`，作为本变更 T1 的一部分，已在 plan.md §5 说明）。
- child Bead（model-routing UI / policy-quota / credential-login / TUI）在本变更仅声明，不计入
  任务范围。
