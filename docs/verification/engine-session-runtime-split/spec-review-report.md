# Spec Review Report — engine-session-runtime-split

- change_id: `engine-session-runtime-split`
- 对照 spec：`specs/engine-session-runtime.md`

## 边界评估

本次为纯职责搬迁 + resume spec 下沉，不改变：

- runtime authority 归属（`Session` 仍是单一公开类型，authority 语义不变）；
- PTY 环境合同（`shell_pty_environment`、`spawn_spec`/`remote_spawn_spec` 语义不变）；
- wire 协议（`ControlMessage`/`SessionSpawnParams`/`SessionResumeParams` 等 shape 与 method 名不变）；
- 磁盘持久化与恢复语义（`registry` 持久化模块未触碰）。

## spec 更新

`specs/engine-session-runtime.md` 已追加 Section 6，记录 `session/` 子模块拆分拓扑与
`session/launch.rs` 的 `LaunchContext` 入口，作为长存契约。

## 结论

边界契约保持不变；仅新增 Section 6 描述拆分后的模块边界，无既有契约弱化。
