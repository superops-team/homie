# 从 diri 迁移核心引擎到 homie 设计文档

## 1. 概述

### 1.1 背景

Homie 当前的 `homie-runtime` 仅实现了文件级输出日志，没有真实的 PTY（伪终端）管理和进程监控。
`homie-term` 只有基于字符串的简单网格，不支持基于单元格的终端渲染。
`homie-proto` 缺少 diri 的 GridCell/RLE 编解码器、SessionRecord 等丰富模型。

diri 项目（`/Users/bytedance/workspace/github/diri`）已经完成了一个成熟的 Rust + GPUI 编码代理编排器，
其 Rust 引擎（`crates/diri-engine`）实现了：
- PTY 管理和进程生命周期
- 无头终端仿真（alacritty_terminal）
- 状态检测引擎（19 个 agent manifest、81 条规则、39 个模式）
- 会话注册表和持久化
- 控制 socket 协议
- Hook 和 notify 解析
- Git 分支/工作树检测

diri 的 `PORT.md` 记录了将这些组件从 Swift daemon 移植到 Rust 的全部工作，
且大部分已完成。本迁移将直接复用 diri 的 Rust 引擎代码，适配到 homie 的 crate 架构中。

### 1.2 目标

- **P0**: 将 diri-proto 的 GridCell/RLE 编解码器、SessionRecord、AgentDescriptor 等模型迁移到 homie-proto
- **P0**: 将 diri-engine 的 PTY 管理、会话生命周期、状态检测、注册表迁移到 homie-runtime
- **P0**: 将 diri-term 的基于单元格的终端渲染、回滚、选择、查找、主题迁移到 homie-term
- **P1**: 将 diri-app 的终端窗格、命令面板、会话切换器迁移到 homie-app
- **P1**: 将 diri 的 agent manifest 检测引擎迁移到 homie-agents

### 1.3 非目标

- **不迁移** Swift daemon（dirijord）：homie 使用嵌入式引擎（单进程架构）
- **不迁移** diri-updater：homie 有自己的更新机制
- **不迁移** diri-node（远程节点）：homie 有自己的 homie-remote
- **不迁移** MCP 集成：homie 有自己的 MCP 方案
- **不改变** homie 的 LLM 代理、虚拟 key、存储、context、memory、task、orchestrator 层
- **不改变** homie 的 GPUI 应用壳（窗口、侧边栏、检查器）——仅增强终端窗格

## 2. 用户场景

### 场景 1: 启动编码代理

**Given** 用户已配置 Claude Code 代理
**When** 用户在 homie 中点击"新建会话"并选择 Claude Code
**Then** homie 创建 PTY，启动 Claude Code 进程，在终端窗格中显示实时输出，
侧边栏显示会话状态为"working"

### 场景 2: 代理状态检测

**Given** Claude Code 正在运行并完成了一个任务
**When** 代理输出中显示 "needs your input" 提示
**Then** homie 检测引擎识别此状态，侧边栏状态变为"needs_input"，
终端窗格高亮提示区域

### 场景 3: 终端交互

**Given** 用户正在查看一个运行中的代理会话
**When** 用户在终端窗格中输入文本并回车
**Then** 输入通过 PTY 发送给代理进程，终端窗格实时更新输出

### 场景 4: 会话持久化

**Given** 用户有多个运行中的代理会话
**When** 用户关闭 homie 应用
**Then** 所有会话状态被保存，下次启动时恢复

## 3. 功能需求

### FR-1: PTY 管理 (homie-runtime)
- 创建 PTY master/slave 对
- 设置正确的终端大小（行/列）
- 处理 SIGWINCH 信号（窗口大小变化）
- 进程组管理（setsid、TIOCSCTTY）
- 子进程退出时清理
- 跨平台支持（unix 优先，Windows 使用 ConPTY 占位）

### FR-2: 无头终端仿真 (homie-runtime)
- 使用 alacritty_terminal 进行 VT 解析
- 维护终端网格状态（单元格、颜色、样式）
- 支持 OSC 9;4 进度检测
- 生成 RLE 编码的网格更新

### FR-3: 会话生命周期 (homie-runtime)
- 创建会话（指定 agent 类型、工作目录、终端大小）
- 启动代理进程
- 发送输入
- 调整终端大小
- 终止会话（kill 进程组）
- 休眠/唤醒会话
- 归档/取消归档会话

### FR-4: 状态检测引擎 (homie-agents)
- 加载 JSON agent manifest
- 根据 manifest 规则检测代理状态
- 反闪烁机制（防抖）
- 阻塞器仲裁
- 启动宽限期
- 子代理隔离
- 过期检测

### FR-5: 基于单元格的终端渲染 (homie-term)
- 从 diri-proto 解码 GridCell 行
- 维护终端缓冲区（包括回滚）
- 文本选择
- 查找
- 主题/颜色支持
- 键编码（ANSI 转义序列）

### FR-6: 终端窗格 (homie-app)
- 在 GPUI 中渲染终端网格
- 键盘输入处理
- 滚动
- 与侧边栏会话选择联动

## 4. 实现边界

### 4.1 涉及模块

| 模块 | 变更类型 | 说明 |
|------|----------|------|
| `homie-proto` | 增强 | 添加 GridCell、TermColor、TermStyle、RLE 编解码、SessionRecord、AgentDescriptor |
| `homie-runtime` | 重写 | 替换文件日志为真实 PTY + 会话管理 + 检测引擎 + 注册表 |
| `homie-term` | 重写 | 替换字符串网格为基于单元格的终端渲染 |
| `homie-agents` | 增强 | 添加检测引擎（manifest 加载、规则评估、状态 reducer） |
| `homie-app` | 增强 | 替换终端区域为真实终端窗格 |
| `Cargo.toml` | 更新 | 添加 alacritty_terminal 依赖 |

### 4.2 不涉及模块

| 模块 | 说明 |
|------|------|
| `homie-storage` | 已有完整的 SQLite 模式，保持不变 |
| `homie-llm` | 虚拟 key 管理已完整，保持不变 |
| `homie-context` | 上下文摘要已完整，保持不变 |
| `homie-memory` | 记忆候选已完整，保持不变 |
| `homie-task` | 任务管理已完整，保持不变 |
| `homie-orchestrator` | 编排器已完整，保持不变 |
| `homie-remote` | 远程管理已完整，保持不变 |
| `homie-cli` | CLI 已完整，保持不变 |
| `homie-updater` | 更新器已完整，保持不变 |

## 5. 组件 spec 影响

| 组件 | 是否影响 | 原因 | 需要更新 |
|------|----------|------|----------|
| `homie-proto` | 是 | 添加 GridCell/RLE 编解码器、SessionRecord、AgentDescriptor | 更新 specs/homie-proto/ |
| `homie-runtime` | 是 | 重写为真实 PTY + 会话管理 | 更新 specs/homie-runtime/ |
| `homie-term` | 是 | 重写为基于单元格的终端渲染 | 更新 specs/homie-term/ |
| `homie-agents` | 是 | 添加检测引擎 | 更新 specs/homie-agents/ |
| `homie-app` | 是 | 增强终端窗格 | 更新 specs/homie-app/ |
| `homie-storage` | 否 | 接口不变 | - |
| `homie-llm` | 否 | 接口不变 | - |

## 6. 测试计划

### 6.1 单元测试

- GridCell 编解码往返测试
- RLE 行编码/解码测试
- PTY 创建和信号处理测试
- 状态检测规则评估测试
- 状态 reducer 反闪烁测试
- 键编码测试
- 终端缓冲区测试

### 6.2 集成测试

- PTY → 仿真器 → 检测 → 状态 端到端管道测试
- 控制 socket 握手/生成/列出/发送文本/调整大小测试
- 会话生命周期测试（创建、运行、休眠、唤醒、终止）
- 注册表持久化测试

### 6.3 兼容性测试

- 现有 homie-storage 测试必须全部通过
- 现有 homie-llm 测试必须全部通过
- 现有 homie-app 必须能编译和启动

## 7. 验收标准

- [ ] `cargo build` 全部编译通过
- [ ] `cargo test` 全部测试通过
- [ ] homie-app 能启动并显示终端窗格
- [ ] 能创建 PTY 会话并显示实时输出
- [ ] 状态检测引擎能正确识别代理状态
- [ ] 现有存储和 LLM 功能不受影响

## 8. Beads 追踪

- Issue: homie-cj5 — 从 diri 迁移核心引擎到 homie
- Change ID: `diri-engine-migration`
- 优先级: P0
- 状态: open