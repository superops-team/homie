# Engine Control Wire/Runtime Split 发布就绪报告

## 1. 就绪结论

S1（wire codec）与 S2（codec 投影）切片就绪，可提交。

## 2. 交付物

- `homie/crates/homie-engine/src/control/wire.rs`（新增，170 行）：wire 编解码 + 错误映射
- `homie/crates/homie-engine/src/control/codec.rs`（新增，122 行）：proto↔domain 投影
- `homie/crates/homie-engine/src/control.rs`（抽取，3,802 → 3,714 行）

## 3. 验证汇总

| 门禁 | 结果 |
|------|------|
| `cargo test -p homie-engine` | 278 lib + 集成全绿，0 failed |
| `cargo fmt -p homie-engine -- --check` | clean |
| `cargo check -p homie-engine` | 无 warning |
| wire.rs 无重依赖 | 通过 |
| codec.rs 无重依赖 | 通过 |

## 4. 范围说明

本切片为 `engine-control-wire-runtime-split` 的 S1+S2，后续 S3/S4 由同一 change_id
继续切片交付，不另开 Beads。

## 5. 已知限制

- 未新增协议 golden fixture（属 `protocol-contract-golden-fixtures`）。
- S3 runtime 生命周期与 S4 handler 下沉尚未执行。
