# OpenSpec Tasks: Diri MCP release_agent Owned-child Guard

| Task | Description | Acceptance | Cases |
|------|-------------|------------|-------|
| T-001 | Update MCP automation spec with release owned-child contract | Spec states only direct child may be released; all other relations denied or covered by existing guards | FC-DMRO-004 |
| T-002 | Add RED integration test for sibling release denial | Test fails before implementation because sibling release is allowed or target is terminated | FC-DMRO-001 |
| T-003 | Add RED integration test for unrelated release denial | Test fails before implementation because unrelated release is allowed or target is terminated | FC-DMRO-002 |
| T-004 | Implement `release_agent` allow-list | `child` is the only relation that reaches `terminate_session`; self/parent/ancestor retain existing messages | FC-DMRO-001, FC-DMRO-002, FC-DMRO-003 |
| T-005 | Run regression and quality gates | All listed commands pass and evidence is recorded | FC-DMRO-003, FC-DMRO-004 |
| T-006 | Update parity/evidence and close Bead | `docs/research/diri-parity-lock.md` records owned-child guard evidence while API-005 remains partial | FC-DMRO-004 |
