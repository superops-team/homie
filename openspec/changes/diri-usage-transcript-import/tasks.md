# OpenSpec Tasks: Diri Usage Transcript Storage Import

| Task | Description | Acceptance | Cases |
|------|-------------|------------|-------|
| T-001 | Add RED storage importer tests | Tests fail before importer API exists | FC-DUTI-001, FC-DUTI-002 |
| T-002 | Add importer data types | `UsageImportDefaults` and `UsageImportResult` exist | FC-DUTI-001 |
| T-003 | Implement single/batch import | Events map to `RecordUsage` and call `record_usage` | FC-DUTI-001 |
| T-004 | Preserve dedupe | Reimported source events are skipped | FC-DUTI-002 |
| T-005 | Run quality gates and update parity lock | storage tests/check/clippy/fmt/diff/parity pass | FC-DUTI-003 |
