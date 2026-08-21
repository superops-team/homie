# 修复 CI 回归检查脚本引用已拆分的 control/handlers.rs 路径

## 背景

`homie/scripts/verify-remote-refactor.sh` 是 Remote PTY Holder refactor 的回归检查脚本，在 CI `engine`
job 的 "Verify reviewed regressions stay fixed" 步骤中执行。其中两项检查通过 `grep` 在
`crates/homie-engine/src/control/handlers.rs` 中查找两个回归锚点：

- `self.remote.is_none()`（"session.migrate refuses only without a transport"）
- `Evicting the corpse`（"resume relaunches a session whose Agent died"）

在 `5e6349f refactor(engine): split control/handlers.rs into focused submodules` 之后，
`control/handlers.rs` 已拆分为 `control/handlers/` 目录（含 `migrate.rs`、`resume.rs` 等），上述两个
锚点分别迁入：

- `self.remote.is_none()` → `control/handlers/migrate.rs:28`
- `Evicting the corpse` → `control/handlers/resume.rs:59`

脚本仍指向已不存在的 `handlers.rs`，导致 `grep` 失败，两项回归检查失败，`verify-remote-refactor.sh`
以非零退出，从而使 CI `engine` job 每次推送都失败。

## 目标

- 更新 `verify-remote-refactor.sh` 中两处 `grep` 的文件路径，指向拆分后的正确文件。
- 使 `verify-remote-refactor.sh` 全部检查通过，CI `engine` job 回归检查恢复绿色。

## 非目标

- 不改变任何生产代码行为。
- 不改变回归检查的语义（锚点字符串不变，仅更新路径）。
- 不新增/删除检查项。

## 需求

### FR-1: migrate 检查指向 migrate.rs

`grep -q 'self.remote.is_none()' crates/homie-engine/src/control/handlers/migrate.rs`。

### FR-2: resume 检查指向 resume.rs

`grep -q 'Evicting the corpse' crates/homie-engine/src/control/handlers/resume.rs`。

## 涉及文件

- `homie/scripts/verify-remote-refactor.sh`

## 验证计划

```bash
bash homie/scripts/verify-remote-refactor.sh   # 全部 PASS
bash -n homie/scripts/*.sh                     # 语法通过
```

## 验收标准

- C1：`verify-remote-refactor.sh` 13 项检查全部 PASS。
- C2：脚本语法校验通过。
- C3：除脚本路径修正外无其它改动。

## Beads

- `homie-5t6`
