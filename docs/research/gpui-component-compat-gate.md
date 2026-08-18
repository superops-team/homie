# gpui-base / gpui-component 兼容性门禁结论

## 1. 目的

评估 Zed 拆出的 gpui-base / gpui-component 组件库与 Homie 当前 pinned gpui revision 的兼容性，
判断 Chat Session Surface 是否能复用其组件（Button/ListRow/Dialog/TextField 等），而不引入
版本冲突或 GPL 污染。结论作为**决策记录**，先于任何依赖引入。

> 说明：本仓库当前未引入 gpui-component/gpui-base，且 pinned 的 gpui revision 不包含其拆分
> 版本。以下为基于公开认知的评估，标记"待代码阶段以实际 crate 元数据复核"。

## 2. 背景

- Homie pinned gpui：`git rev dc2a339d5d043da448a3f7ddc7c0a85c63864aad`（Zed 仓库）。
- Zed 后续将 GPUI 拆分为 gpui-base / gpui-component / gpui-derive / gpui-macros 等子 crate。
- Homie 已有 `homie-ui` 语义 tokens 与 primitive 组件（见 `specs/ui-components.md`）。

## 3. 兼容性评估维度

| 维度 | 结论 | 说明 |
|------|------|------|
| 版本/revision 兼容 | 待复核 | 当前 pinned rev 与 gpui-component 发布版本需逐一对齐，直接混用大概率版本冲突 |
| license 兼容 | 风险低 | gpui/gpui-component 同为 Apache-2.0/MIT 系，但需复核 Zed 仓库内个别 GPL profiler 引用（Homie 已有 ztracing 补丁先例） |
| 组件能力覆盖 | 部分可复用 | Button/ListRow/Dialog/TextField 与 `specs/ui-components.md` 重叠，但 API 形态需适配 |
| 引入成本 | 高 | 拉取新 Zed rev 会连带 gpui 升级，影响现有 vendored `gpui_macos` 补丁 |

## 4. 门禁结论

**首阶段不引入 gpui-base / gpui-component 作为运行时依赖**。理由：

1. 当前 pinned gpui revision 未包含该拆分，引入会触发 gpui 大版本升级，动摇现有
   `gpui_macos`/`ztracing` vendored 补丁与 release 稳定性。
2. Homie 已有 `homie-ui` + `specs/ui-components.md` 定义的 primitive 契约，Chat 表面可基于
   现有 primitive 自研 chat 专用组件（message/tool/permission 卡片），复用度足够。
3. 引入新 Zed rev 属独立的重基/升级变更，不应捆绑在 Chat 表面设计里。

**门禁动作**：

- 后续若决定升级 gpui 或引入 gpui-component，必须独立 PRD/OpenSpec，先过 license 审计、
  版本对齐、`gpui_macos` 补丁回归三关。
- 在升级前，Chat 表面组件遵循 `specs/ui-components.md` 自研，复用 `homie-ui` tokens。

## 5. 结论记录

- change_id: `codex-acp-harness-runtime`（本设计）
- 决定：不引入 gpui-base/gpui-component，Chat 表面自研组件 + 复用 homie-ui tokens。
- 状态：结论待代码阶段以实际 crate 元数据复核，但设计层面不依赖该依赖。
