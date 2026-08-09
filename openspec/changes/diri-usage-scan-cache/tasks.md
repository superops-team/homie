# OpenSpec Tasks: Diri Usage Scan File Offset Cache

| Task | Description | Acceptance | Cases |
|------|-------------|------------|-------|
| T-001 | Add RED scan cache tests | Tests fail before repository API exists | FC-DUSC-001, FC-DUSC-002 |
| T-002 | Add state/query structs | `UsageScanFileState` and `UsageScanFileQuery` exist | FC-DUSC-001, FC-DUSC-002 |
| T-003 | Implement upsert/get/list | API reads and overwrites `usage_scan_files` rows | FC-DUSC-001, FC-DUSC-002 |
| T-004 | Run quality gates and update parity lock | storage tests/check/clippy/fmt/diff/parity pass | FC-DUSC-003 |
