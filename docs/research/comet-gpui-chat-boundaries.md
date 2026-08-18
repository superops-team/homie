# Comet GPUI Chat 模块边界学习结论

## 1. 目的

学习 Zed 生态中 Comet（GPUI 聊天组件/参考实现）如何拆分 chat 界面的模块边界，提炼可复用的
**边界原则**，用于指导 Homie 的 Chat Session Surface 结构设计，而非直接拷贝代码。

> 说明：本仓库当前 pinned 的 gpui revision 不包含 Comet/gpui-component 源码，因此本结论基于
> 公开架构认知提炼原则，并标记为"待代码阶段以实际源码复核"。结论用于**设计边界**，不引入
> 未验证依赖。

## 2. Comet 的典型模块拆分（参考认知）

```text
chat/
├── message.rs        # 单条消息及其 blocks（text/thinking/tool/plan）
├── transcript.rs     # 消息列表 + 滚动 + 增量追加
├── composer.rs       # 输入区 + 发送/停止动作
├── input.rs          # 文本输入适配（复用编辑器原语）
├── tool_call.rs      # tool 调用卡片（状态 + 展开）
└── permission.rs     # 审批/权限请求卡片
```

## 3. 提炼的边界原则

1. **消息与 transcript 分离**：`message` 是纯数据/单条渲染单位，`transcript` 是容器（列表、
   滚动、增量），不把两者混在一个大实体里。
2. **composer 与 input 分离**：composer 管动作（send/steer/stop）与编排，input 复用编辑器
   文本原语（光标/选择/剪贴板），不新造文本系统。
3. **块级渲染（blocks）**：一条消息由多种 block（text/thinking/tool/plan）组成，block 是
   最小渲染单位，便于增量更新与复用。
4. **tool/permission 独立卡片**：tool call 与 permission 是有状态卡片（status/options），
   拥有自己的生命周期与渲染，不塞进 message 主体。
5. **projection 边界**：把结构化 event 投影为视觉元素的逻辑集中在 projection 层，render 只
   消费 prepared state（符合 `specs/gpui-shell.md` render contract）。
6. **稳定 ID**：消息/block/tool/permission 用 domain 稳定 ID，不用列表索引（符合
   `specs/gpui-interaction-contract.md` §2）。

## 4. 对 Homie 的落地映射

| Comet 边界 | Homie 对应 | 说明 |
|-----------|-----------|------|
| `message.rs` | `chat/transcript.rs` 内的 message 渲染单位 | 首阶段 message 即可承载 blocks |
| `transcript.rs` | `chat/transcript.rs` | 容器 + 滚动 + 增量 |
| `composer.rs` | `chat/composer.rs` | send/steer/stop |
| `input.rs` | 复用 `query_editor` 文本原语 | 不新造文本系统 |
| `tool_call.rs` | `chat/transcript.rs` 的 tool 卡片 | 首阶段并入 transcript，后续可拆 |
| `permission.rs` | `chat/approval_view.rs` | 审批四态卡片 |

## 5. 结论与待办

- **本阶段**：采纳第 3 节的 6 条边界原则，作为 `homie-app/src/chat/` 的结构依据。
- **后续代码阶段**：拉取 Comet 实际源码复核上述拆分，校正偏差并记录；本结论不替代真实源码
  学习，只提供设计起点。
