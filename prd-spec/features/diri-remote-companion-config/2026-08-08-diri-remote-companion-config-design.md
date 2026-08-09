# Diri Remote Companion Config 对齐设计文档

```yaml
change_id: diri-remote-companion-config
beads: homie-0gh
target_rows:
  - REM-002
feature_atoms:
  - M06-F002
```

## 1. 背景

`REM-002` 仍为 `missing`。Diri 的 RemoteConfig/remote_access 负责 iOS companion/Tailscale remote access 配置：port、bind host、pairing token、forwardAnyPort，并要求 token config 文件 owner-only。

Homie 目前只有 app settings 的布尔偏好和 remote host config，没有 companion config model。

## 2. 目标

- 在 `homie-remote` 增加 `RemoteCompanionConfig`。
- 支持 load/save/remove。
- 保存文件在 Unix 上必须 owner-only `0600`。
- 生成 endpoint label 和 pairing URL。
- pairing URL 可用，但普通 display/debug 不泄漏 token。

## 3. 非目标

- 不启动 TCP listener。
- 不执行 Tailscale discovery。
- 不把 `REM-002` 标为 implemented；完整 parity 仍需 app settings UI + listener E2E。

## 4. 验收

- `cargo test -p homie-remote --test companion_config -- --nocapture`
- `cargo check -p homie-remote`
- `cargo clippy -p homie-remote --all-targets -- -D warnings`
- scoped `git diff --check`
- `make parity-lock`
