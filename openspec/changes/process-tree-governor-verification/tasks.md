# Tasks: process-tree-governor-verification

- [x] T1 新增 `tests/process_tree.rs`，覆盖 enumerate/SIGSTOP/SIGCONT/kill_tree
- [x] T2 新增身份安全（PID reuse）对抗测试
- [x] T3 补 governor `idle_since` eligibility 单测（三道闸）
- [x] T4 补三类休眠策略边界单测（含非 Idle 状态）
- [x] T5 编写失败模型 `failure-model.md`
- [x] T6 运行全量测试并记录证据到 `docs/verification/process-tree-governor-verification/`
- [x] T7 更新/关闭 Beads `homie-81i`
