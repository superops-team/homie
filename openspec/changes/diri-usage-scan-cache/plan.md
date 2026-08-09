# OpenSpec Plan: Diri Usage Scan File Offset Cache

```yaml
change_id: diri-usage-scan-cache
beads: homie-xaz
source_prd: prd-spec/features/diri-usage-scan-cache/2026-08-08-diri-usage-scan-cache-design.md
```

## 1. Scope

Add repository APIs for the existing `usage_scan_files` table. This is the durable offset cache prerequisite for a later usage watcher.

## 2. Current State

- Schema has `usage_scan_files`.
- No public storage API can upsert/read/list scan file state.

## 3. Target State

```text
upsert_usage_scan_file(state)
usage_scan_file(path)
list_usage_scan_files(query)
```

## 4. Verification

- FC-DUSC-001 upsert/read overwrite.
- FC-DUSC-002 provider/profile list filters.
- FC-DUSC-003 quality gates.
