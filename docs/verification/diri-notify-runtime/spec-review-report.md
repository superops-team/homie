# Spec Review Report: Diri Codex Notify Runtime

## 1. 总体结论

- 可行性：高。
- 最大风险：notify 默认写入本机状态，破坏 fail-open。
- 推荐方向：只有显式 `--data-dir` 时写 runtime；默认 parse-only。

