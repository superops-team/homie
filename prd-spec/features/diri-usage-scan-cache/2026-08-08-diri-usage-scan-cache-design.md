# Diri Usage Scan File Offset Cache 设计文档

```yaml
change_id: diri-usage-scan-cache
beads: homie-xaz
target_rows:
  - USAGE-001
feature_atoms:
  - M19-F001
```

## 1. 概述

### 1.1 问题/背景

Diri usage watcher 使用 durable cache 保存 transcript 文件的 size、offset、modified_ns、tail_hash、model 等信息，以便只解析新增完整行并识别文件重写。Homie storage schema 已有 `usage_scan_files` 表，但没有 repository API，因此后续 watcher 不能安全持久化扫描进度。

该切片不依赖截图，不实现 watcher，只补 `usage_scan_files` 的读写 API 和测试。

### 1.2 目标

- 新增 usage scan file state 数据结构。
- 支持 upsert 一个 transcript 文件扫描状态。
- 支持按 path 读取状态。
- 支持按 provider/profile 查询状态。
- 不修改 schema。

## 2. 用户场景

### 场景 1：扫描后保存 offset

**Given** usage parser 已解析 transcript 的新增 bytes。  
**When** watcher 将文件状态写入 storage。  
**Then** 下次扫描可从 offset 继续。

### 场景 2：按 provider/profile 重建扫描集合

**Given** storage 中已有 Claude/Codex 多个 transcript 状态。  
**When** watcher 按 provider/profile 查询。  
**Then** 只返回匹配状态，供增量扫描使用。

## 3. 功能需求

### FR-1：UsageScanFileState

字段覆盖 `usage_scan_files` 表所有业务列：path、provider、profile_id、size、offset、modified_ns、device、inode、tail_hash、model、scanned_at。

### FR-2：Upsert

同 path 再次 upsert 应覆盖状态。

### FR-3：Query

支持 `usage_scan_file(path)` 和 `list_usage_scan_files(provider, profile_id)`。

## 4. 实现方案

在 `homie-storage` 增加：

- `UsageScanFileState`
- `UsageScanFileQuery`
- `Storage::upsert_usage_scan_file`
- `Storage::usage_scan_file`
- `Storage::list_usage_scan_files`

## 5. 非目标

- 不实现 watcher。
- 不计算 tail hash。
- 不扫描文件系统。
- 不写 usage records。

## 6. 涉及文件

- `crates/homie-storage/src/lib.rs`
- `crates/homie-storage/tests/usage_scan_cache.rs`
- `docs/research/diri-parity-lock.md`
- `docs/verification/diri-usage-scan-cache/`

## 7. 验收标准

- `cargo test -p homie-storage --test usage_scan_cache -- --nocapture`
- `cargo test -p homie-storage --test diri_storage_indexing`
- `cargo check -p homie-storage`
- `cargo clippy -p homie-storage --all-targets -- -D warnings`
- `cargo fmt --all -- --check`
- scoped `git diff --check`
- `make parity-lock`

## 8. Beads 跟踪

- Bead: `homie-xaz`
- 完成验证后关闭。
