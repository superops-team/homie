# OpenSpec Tasks: Diri Proto Host Catalog and Remote Config

| Task | Description | Acceptance | Cases |
|------|-------------|------------|-------|
| T-001 | Add RED host/remote serde test | Test fails before DTOs exist | FC-DPHR-001 |
| T-002 | Implement host catalog DTOs | HostNodeConfig, HostEntry, HostsConfig match Diri wire | FC-DPHR-001 |
| T-003 | Implement remote config DTO | RemoteConfig matches current and legacy Diri wire | FC-DPHR-001 |
| T-004 | Run quality gates and update parity lock | homie-proto tests/check/clippy/fmt/diff/parity pass | FC-DPHR-002 |
