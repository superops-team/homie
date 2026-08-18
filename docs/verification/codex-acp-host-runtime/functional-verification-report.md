# 功能验证 Case 执行报告 — codex-acp-host-runtime

## 执行摘要

| Case | 断言要点 | 结果 |
|------|----------|------|
| FC-1 | ACP JSON-RPC DTO 往返 + classify + 未知 kind 容忍 + 非法 JSON | PASS |
| FC-2 | framing 编解码 + 空行/空白行跳过 + CRLF + EOF | PASS |
| FC-3 | host 循环 + id 关联 + 通知派发（经 E2E 覆盖） | PASS |
| FC-4 | approval 四态记忆 | PASS |
| FC-5 | AcpDriver 方法映射（cancel/steer/respond/model_options） | PASS |
| FC-6 | fake ACP server 端到端（真实子进程路径） | PASS |
| FC-7 | 模块注册 + 规范记录 + 对齐 + 门禁全绿 | PASS |

## 执行证据

- `cargo test -p homie-engine --lib acp`：18 项单测全部通过（0 failed）。
- `cargo test -p homie-engine --test acp_host`：E2E 通过，输出
  `acp_host: end-to-end host loop passed`。
- `cargo test -p homie-engine`：全量 296+0 failed（含既有 281 项 + 新增 18 项 lib + 1 项 E2E）。
- `cargo check --workspace`：通过。
- `cargo fmt --all --check`：通过（无 diff）。

## 通过率

7 / 7 通过，0 失败。

## 结论

所有功能验证 Case 通过，可进入提交、打 tag 与关闭流程。
