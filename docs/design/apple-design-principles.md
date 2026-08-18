# Homie Apple / Design-Engineering 视觉与动效规范

## 1. 目的

本规范为 Homie 的 GPUI 表面（尤其 Chat Session Surface）提供项目级视觉与动效一致性依据。
它把 Apple HIG 的核心原则落地为可执行的 design tokens 与行为规则，避免各表面各自手搓颜色、
间距、动效，造成设计漂移。

## 2. 设计原则（源自 Apple HIG）

| 原则 | 在 Homie 的落地 |
|------|-----------------|
| 清晰 Clarity | 文字清晰可读，层级分明，装饰不干扰信息本身 |
| 遵从 Deference | 界面退后、内容前置；chrome 用浅/低调色，内容用强调色 |
| 深度 Depth | 用层级（前景/背景/叠加）与 z-order 表达视觉层次，而非杂乱阴影 |
| 一致性 Consistency | 同一语义用同一 token、同一组件、同一交互，跨表面一致 |
| 直接操纵 Direct Manipulation | 拖拽、滚动、选中即时反馈，无迟滞 |
| 反馈 Feedback | 每次交互有可感知反馈（hover/pressed/loading/结果） |
| 隐喻 Metaphor | 用熟悉隐喻（如 turn、卡片、列表）降低学习成本 |
| 用户控制 User Control | 可撤销、可停止（steer/stop）、可退出，不劫持用户 |

## 3. Design Tokens（与 homie-ui 对齐）

### 3.1 颜色（语义色，不直接写 RGB）

| Token | 用途 |
|-------|------|
| `bg.base` / `bg.sunken` / `bg.elevated` | 背景层级（depth） |
| `fg.primary` / `fg.secondary` / `fg.muted` | 文字层级 |
| `accent` | 主强调（选中、主按钮、激活） |
| `danger` / `warning` / `success` | 状态语义（tool failed/needs-you/done） |
| `border` / `border.strong` | 分割与轮廓 |
| `focus.ring` | 焦点可见环 |

规则：UI 不得引入第二套颜色来源；新颜色必须进 `homie-ui` tokens 并说明语义。

### 3.2 字号阶梯（Typo）

- `caption` / `body` / `body.strong` / `title` / `title.large` 五档，按语义使用；
- 消息正文用 `body`，tool 卡片标题用 `body.strong`，turn 分隔用 `caption`；
- 不使用任意像素字号。

### 3.3 圆角与间距

- 圆角：`radius.small`（输入框/按钮）、`radius.medium`（卡片）、`radius.large`（弹层）；
- 间距：`space.1..6` 阶梯，卡片内 padding 用 `space.3`，卡片间距用 `space.2`。

## 4. 动效规范

### 4.1 原则

- 动效服务于**反馈与连续性**，不装饰；
- 默认使用短时长 + 单一缓动，避免连续动画帧占用；
- 尊重 `specs/gpui-interaction-contract.md` §7 平台偏好。

### 4.2 时长与缓动

| 场景 | 时长 | 缓动 |
|------|------|------|
| hover 状态切换 | 100–120ms | ease-out |
| 卡片/turn 进入 | 160–200ms | ease-out |
| 弹层出现/消失 | 200ms | ease-in-out |
| loading 指示 | 循环，但 reduce-motion 时静止 |

### 4.3 平台偏好

| 偏好 | 期望响应 |
|------|----------|
| reduce motion | 所有过渡 snap 或短 fade，无连续动画 |
| reduce transparency | 叠加层不透明，保持层级 |
| increase contrast | 加强文字/边框/焦点分离 |
| differentiate without color | tool status 加图标/标签/形状，不只靠颜色 |

### 4.4 Chat 表面专用

- **turn 进入**：短 fade + 上移（若 reduce motion 则 snap）；
- **streaming 文本**：按增量追加，不整体重排（避免滚动跳动）；
- **tool 状态迁移**：pending→running→completed/failed 用图标+标签表达，不只靠颜色；
- **审批卡片**：出现用 focus 引导，不打断输入焦点；
- **steer/stop**：stop 展示 stopping→stopped 过渡，steer 展示"已注入"反馈。

## 5. 一致性门禁

任何新 GPUI 表面在合并前需自查：

1. 颜色/字号/间距只来自 tokens；
2. 交互有 hover/pressed/disabled/loading/focus 状态（见 `specs/ui-components.md`）；
3. 动效遵循 §4 且尊重平台偏好；
4. 有键盘可达路径与 accessible name（见 `specs/gpui-interaction-contract.md`）；
5. 未引入第二套视觉语言。

违反上述任一则需在 spec review 中说明理由，否则退回。
