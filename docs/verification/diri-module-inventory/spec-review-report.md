# Spec Review Report

```yaml
change_id: diri-module-inventory
reviewed_doc: docs/research/diri-module-inventory.md
beads: homie-729
status: needs_revision
review_type: module_abstraction_feasibility
```

## 1. 总体结论

- 可行性：中。
- 最大风险：当前清单能作为“方向索引”，但还不能直接驱动真实实现；缺少逐模块验收边界、Diri 行为契约、数据/协议接口、跨模块依赖和 E2E 证据设计。
- 推荐方向：保留 20 个模块的一级拆分，但必须补一层“功能原子项矩阵”和“模块间依赖/验证矩阵”，再进入各模块 PRD/spec。否则后续仍会重复出现“实现了局部代码，但没有 Diri 等价行为”的问题。

## 2. 问题清单

| 优先级 | 维度 | 问题 | 影响 | 整改建议 |
|---|---|---|---|---|
| P0 | 可落地性 | 每个模块只有功能 bullet，没有拆到可开发的原子功能、输入输出、状态机和验收口径 | 无法判断一个功能是否真实对齐 Diri，后续 OpenSpec task 容易继续写成泛泛实现项 | 为每个 Mxx 增加 `Feature ID` 级矩阵：功能名、Diri 源文件/测试、用户可见行为、数据/协议接口、Homie owner、验收命令、证据路径 |
| P0 | 验证策略 | 模块级后续入口没有绑定 Diri 测试或 Homie 验证 case | 不能防止“代码有了但行为没对齐” | 增加 `Diri test -> Homie test/E2E` 映射，尤其是 `diri/Tests/*` 中每个 test suite 的覆盖归属 |
| P0 | 依赖顺序 | 20 个模块顺序是线性建议，但未标注阻塞依赖 | 可能先做 UI 或 CLI，却缺 protocol/runtime/storage 前置能力，导致再次做静态 UI | 增加 DAG：Core/Protocol/Storage/Runtime/Client 是基础层；UI/CLI/MCP/Remote/Release 是消费层；每个模块列 `blocks`/`blocked_by` |
| P1 | 范围边界 | 部分模块边界过宽：M14 Runtime 同时覆盖 registry/status/resource/migration；M04 Inspector 同时覆盖 diff/artifact/browser/PR/ports | 单个 PRD 会过大，开发无法 TDD 收口 | 将超大模块拆成子模块，例如 M14a session lifecycle、M14b status/reducer integration、M14c resource governor、M14d migration/checkpoint |
| P1 | 产品设计 | UI 模块列了功能，但没有记录 Diri 真实视觉/交互基线、布局 token、截图目标 | 后续仍可能“功能有但不像 Diri” | 每个 UI 模块增加 Diri screenshot/fixture requirement：目标视图、交互状态、视觉 tokens、必测截图 |
| P1 | 数据模型 | Storage 改造面只写了表/索引方向，没有列具体表、字段和查询路径 | 后续 storage schema 可能反复变化，影响 runtime/client/app | 对 M05/M06/M07/M17/M19 增加 storage schema inventory：表、字段、唯一约束、查询 API、迁移测试 |
| P1 | 协议接口 | M10/M11/M12/M13 未列 method-by-method 对齐表 | CLI/MCP/client 很容易遗漏方法或事件 | 增加 protocol method catalog：Diri method/event、request/response DTO、Homie method、测试 fixture |
| P1 | 安全 | Remote、credential、notification、hook/notify、MCP lineage 缺少安全准入条款 | 可能泄露 token 或绕过权限边界 | 每个安全敏感模块增加 redaction、credential custody、permission scope、failure mode 验收 |
| P1 | 运行风险 | 没有标注需要真机/PTY/GUI/macOS 权限/网络的模块 | CI/unit 通过但本机 app 或系统集成失败 | 为每个模块标注验证环境：unit、integration、real PTY、macOS GUI、network、packaged app |
| P2 | 文档状态 | `status: draft` 但 Beads 已 closed | 状态不一致，后续 agent 可能误判清单还未完成或已可实现 | 改为 `status: reviewed_needs_revision` 或 `baseline_complete_needs_feature_matrix` |
| P2 | 命名一致性 | 中英文混排和 “Diri module” 粒度不完全一致 | 后续生成 PRD 时可能产生重复 change-id | 固定 change-id 词表和模块命名规范 |
| P2 | 兼容策略 | 文档说 Homie 不追求旧兼容，但没有说明 Diri parity 与 Homie 自有 LLM custody/virtual key 架构如何合并 | 可能机械复刻 Diri，破坏 Homie 长期架构 | 每个模块增加 “Diri behavior parity / Homie architecture adaptation” 双栏 |

## 3. 整改后的完善方案

### 3.1 目标与范围

`docs/research/diri-module-inventory.md` 应定位为总索引和规划入口，不应直接作为实现规格。下一版需要补充两层：

- 模块层：保持 M01..M20，负责说明产品/技术边界。
- 功能原子层：每个模块下拆 `Mxx-F###`，作为 PRD、OpenSpec task 和验证 case 的最小单位。

### 3.2 非目标

- 不在总清单里写完整实现方案。
- 不把模块存在视为实现证据。
- 不让 UI screenshot 或 source-text tests 单独关闭模块。

### 3.3 设计原则

- 每个功能原子项必须能回答：Diri 哪个文件/测试证明它存在、用户怎么感知、Homie 哪个模块承接、如何验证。
- UI 模块必须同时有行为验收和视觉验收。
- Runtime/Client/Protocol 模块必须先有 contract tests，再写 app/CLI 消费层。
- Remote/MCP/Credential/Hook 模块必须先定义安全边界和 redaction 规则。

### 3.4 核心方案

在 `docs/research/diri-module-inventory.md` 后追加三个矩阵：

1. Feature Atom Matrix：
   - `Feature ID`
   - Diri source
   - Diri tests
   - User-visible behavior
   - Homie owner modules
   - Component spec path
   - PRD path
   - OpenSpec change id
   - Verification gate
   - Current parity status

2. Dependency Matrix：
   - Module
   - Blocks
   - Blocked by
   - Required data contract
   - Required protocol contract

3. Verification Matrix：
   - Unit
   - Integration
   - Real PTY
   - macOS GUI
   - Packaged app
   - Remote/network
   - Screenshot/E2E

### 3.5 兼容与风险控制

- 不需要兼容 Homie 旧静态实现；旧 preview-only 行为应被替换或降级为测试 fixture。
- 不直接复制 Diri 内部实现；Diri 行为是验收标准，Homie 架构仍以 `homie-client`、runtime supervisor、LLM custody 为边界。
- 每次实现前必须检查 `docs/research/diri-parity-lock.md`，实现后只更新有真实证据的 row。

### 3.6 验收标准

整改后的 inventory 通过：

- 所有 Diri top-level source group 有 Mxx 映射。
- 所有 `diri/Tests/*` test suite 有模块归属。
- 每个 Mxx 至少有一个 `Mxx-F###` 功能原子项。
- 每个功能原子项有 Homie owner、spec path、OpenSpec path、verification gate。
- 无 `TBD/TODO/待补`。

## 4. 分层任务拆解

| 层级 | 任务 | 交付物 | 依赖 | 优先级 |
|---|---|---|---|---|
| Inventory | 将 M01..M20 扩展为 Feature Atom Matrix | `docs/research/diri-module-inventory.md` v2 | 当前清单 | P0 |
| Test Mapping | 映射 `diri/Tests/*` 到模块和功能原子项 | 同文档 `Diri Test Coverage` section | Diri tests scan | P0 |
| Dependency | 增加模块 DAG 和 blocked_by/blocks | 同文档 `Dependency Matrix` | Feature Atom Matrix | P0 |
| Verification | 增加验证环境矩阵 | 同文档 `Verification Matrix` | Feature Atom Matrix | P0 |
| Spec Planning | 为前 5 个基础模块生成 PRD/spec 骨架 | `prd-spec/features/diri-core-models/*` 等 | inventory v2 | P1 |
| Gate | 增加 inventory 校验脚本 | `scripts/quality/check-diri-module-inventory.py` | inventory v2 | P1 |

## 5. 测试规划

| 类型 | 覆盖点 | 关键用例 | 执行阶段 |
|---|---|---|---|
| 文档校验 | M01..M20 存在 | 脚本检查所有 module id | inventory v2 后 |
| 源码覆盖 | Diri source dirs 映射 | `find diri/Sources diri/diri/crates -maxdepth 2 -type d` 与清单对比 | inventory v2 后 |
| 测试覆盖 | Diri tests 映射 | `find diri/Tests -maxdepth 2 -type f` 与功能原子项对比 | inventory v2 后 |
| 状态一致 | Beads/doc status | Beads open/closed 与 YAML status 一致 | 收尾 |
| PRD 入口 | 每个模块 PRD path 合法 | path exists or planned marker consistent | 生成 PRD 前 |

## 6. 开发排期

| 阶段 | 顺序 | 工作项 | 风险与缓冲 | 验收物 |
|---|---|---|---|---|
| A | 1 | inventory v2：Feature Atom Matrix | 功能项较多，先覆盖 Diri tests 中出现的行为 | 更新后的 inventory |
| B | 2 | Diri tests mapping | Swift/Rust 双测试体系可能重叠 | Test Coverage section |
| C | 3 | Dependency/Verification Matrix | 依赖关系可能需要二次修正 | DAG + gate matrix |
| D | 4 | inventory 校验脚本 | 文档结构需稳定 | `make module-inventory-check` |
| E | 5 | 前 5 个基础模块 PRD skeleton | 不进入实现 | PRD/spec 骨架 |

## 7. 待确认问题

- 是否要求每个 Diri test case 都映射到单独功能原子项，还是 test suite 粒度即可？
- UI 视觉基线是否以已安装 `/Users/bytedance/Applications/diri.app` 截图为准，还是以源码 fixtures 为准？
- Remote/node 功能是否要求本机搭建真实 node E2E，还是先以 protocol + mocked node integration 作为第一阶段？

