# Security Review Report

```yaml
change_id: diri-runtime-daemon-client-transport
beads: homie-nep
date: 2026-08-08
status: pass
reportable_findings: 0
```

## 1. 范围

审查本 change 引入或修改的 production surface：

- owner-only runtime directory、socket、lock、boot log；
- peer effective UID validation；
- launcher canonical executable、hash/version compatibility；
- fixed-frame decode、JSON/raw payload limits；
- terminal input/resize、event/terminal stream recovery；
- MCP capability filtering 与 atomic child spawn；
- daemon/process/log safe-field behavior；
- package daemon/holder dependency closure。

只报告能够证明 source 到 sink、由本 change 引入或恶化、且产生实际权限增益的漏洞。DoS、测试代码、既有 permission debt 和泛化 hardening 不计 finding。

## 2. 基线与结果

- runtime directory 为 current UID、mode `0700`。
- socket/lock/boot log 为 owner-only，server 在协议读取前校验 peer UID。
- stale socket 删除前重验 type、UID、device 和 inode。
- frame decoder 在复制 payload 前校验 fixed length 与 payload 上限。
- malformed kind/flags/length、65th connection 和 unsafe peer 均 fail closed。
- daemon startup error 不回显 data path；hostile payload 不进入 boot log、daemon log 或 process arguments。
- real provider credential 不进入 managed agent configuration 或 transport evidence。

结论：

> No exploitable issues found in the reviewed change set.

## 3. 候选项复核

| 候选 | 处置 | 依据 |
|---|---|---|
| same-UID process 可自报 executable hash | dropped | 本地认证边界就是 effective UID；同 UID 主体已经能直接读取/修改该用户 Homie data/config/output，没有新增 privilege gain。hash 用于版本/identity compatibility，不是跨 UID credential。 |
| MCP `read_output`/`send_prompt` 未执行完整 permission profile | dropped for this change | 权限债务真实存在且 API-005 保持 partial，但这些工具在 Wave 1A 前已交付；本 change 只迁移 async transport、capability truth 和 atomic spawn，没有新增或放宽 read/send 权限。 |

## 4. 动态证据

`crates/homie-cli/tests/shared_daemon_e2e.rs` 使用真实 daemon 验证：

- malformed/oversized frame 被拒绝；
- 65th active connection 被拒绝；
- hostile payload marker 不出现在 logs 或 process arguments；
- cleanup 后 test daemon/holder 为 0。

相关 focused server、launcher、daemon process、frame codec 和 MCP capability suites 均通过。T-102 holder adoption 与 API-005 permission completion 作为已知 scope boundary 保留，不影响本 change 的 security finding 判定。
