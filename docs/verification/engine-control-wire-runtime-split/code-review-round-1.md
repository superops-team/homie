# Engine Control Wire/Runtime Split 首轮代码评审报告

## 1. 评审范围

`control/wire.rs` 新增 + `control.rs` 抽取，纯机械搬迁。

## 2. 显性问题（语法/运行/接口/命名）

| # | 问题 | 处理 |
|---|------|------|
| 1 | `wire.rs` 测试 `Probe` 缺 `Serialize` derive，`encode` 测试编译失败 | 已修：补 `Serialize` derive |
| 2 | `decode_defaults_missing_params_to_empty_object` 测试断言错误（decode(None) 对必填字段类型返回 bad_request，不是空对象） | 已修：改为断言 `decode::<JsonValue>(None) == json!({})`，准确反映空对象 fallback 语义 |
| 3 | `control.rs` 移走 `write_message` 后 `Write` import 成为 unused | 已修：移除 `Write` |
| 4 | 抽取后 `shell_pty_environment` 前多余空行 | 已修：`cargo fmt` |

## 3. 隐性问题（边界/资源/规范）

- 无。函数体一字未改，仅移动位置 + 加 `pub(super)`；可见性限定在 `control` 模块内，无 API 泄漏。
- `resolve_on_path`/`migrate_control_error`/`io_control_error` 保持原实现，无逻辑漂移。

## 4. 功能验证覆盖审查

- FC-03 新增 8 个 focused tests 直接覆盖 decode/encode/io/migrate/resolve 纯函数行为。
- FC-04 全量测试（含集成）保障 wire shape 与 daemon 行为不变。

## 5. 结论

显性问题全部修复，无残留 P0/P1。可进入二次复核。
