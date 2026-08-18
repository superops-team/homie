# OpenSpec Tasks — homie-cli-config-ops

本变更跨 Swift CLI 与 Rust 网关，覆盖配置存储、config 子命令、doctor 增强、fix、skill 与验证。

## T1: 统一配置存储（HomieConfigStore.swift + 网关 config.rs 对齐 JSON）

- 交付：`HomieConfigStore.swift` 实现 `homie.local.json` 原子读写、0600 权限、脱敏函数；
  `homie-gateway/src/config.rs` 改为读同一 JSON（对齐决策 A，取代 TOML）。
- 验收：Swift 写入后 Rust 网关可读同一文件启动；脱敏单测；损坏文件检测。
- 关联验证 Case：FC-1。

## T2: `homie config show` / `config get`

- 交付：`ConfigCommand.swift` 的 show/get；show 汇总网关/上游/模型/虚拟 key（只读 SQLite）；
  get 读单字段；全部脱敏。
- 验收：show/get 输出脱敏；无配置时输出空配置提示。
- 关联验证 Case：FC-2。

## T3: `homie config set`

- 交付：设置 baseUrl/apiKey/listen/masterKey/models；`--api-key-from-stdin` 与 env 录入；原子写。
- 验收：写入后 show 可见；key 不进 history；参数校验。
- 关联验证 Case：FC-3。

## T4: `homie config agent <codex|claude>` + 注入一致性

- 交付：`homie-gateway/src/inject.rs` 暴露 `inject --agent <agent> --json`，内部调用
  `homie-engine::inject::injection_args()`；Swift 转发 stdout 并格式化。
- 验收：`config agent` 输出与 spawn 注入 argv/env 形状一致（单测锁定）。
- 关联验证 Case：FC-4。

## T5: 增强 `homie doctor`

- 交付：在原有 3 项基础上新增网关可达、上游凭证、虚拟 key 生效、agent 配置指向正确 4 项；
  失败返回非零。
- 验收：健康环境全 `✓`；构造故障触发对应 `✗`。
- 关联验证 Case：FC-5。

## T6: `homie fix`

- 交付：端口冲突/凭证缺失/配置漂移/网关未运行 4 个幂等修复动作；探测→跳过或修复→输出。
- 验收：各动作幂等；不静默填真实 key；不自动拉起守护进程。
- 关联验证 Case：FC-6。

## T7: `homie` skill（SKILL.md）

- 交付：`homie/.agents/skills/homie/SKILL.md`，记录 config/doctor/fix 用法、脱敏红线、stdin/env
  录入。
- 验收：skill 可被 agent 读取并正确指导操作，不泄露真实 key。
- 关联验证 Case：FC-7。

## T8: 集成测试与门禁

- 交付：`config set`→网关读同文件；`config agent` 与真实 spawn 一致；`doctor`/`fix` 集成测试；
  `cargo check`/`fmt`/`test` + `swift build` 绿。
- 关联验证 Case：FC-8。

## T9: 证据 + 关闭

- 交付：spec review、覆盖率/mutation 证据、OpenSpec alignment；Beads `homie-ys0` 关闭。
- 关联验证 Case：FC-9。
