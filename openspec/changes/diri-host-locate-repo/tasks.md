# OpenSpec Tasks: Diri host.locate_repo

| Task | Description | Cases |
|------|-------------|-------|
| T-001 | Add Diri-compatible `HostLocateRepoParams` and `HostLocateRepoResult` DTOs | FC-DHLR-001 |
| T-002 | Add read-only repo origin discovery from `.git/config` and linked worktree gitdir files | FC-DHLR-002, FC-DHLR-004 |
| T-003 | Add candidate checkout matching and no-origin/not-cloned result projection | FC-DHLR-002, FC-DHLR-003, FC-DHLR-004 |
| T-004 | Wire `host.locate_repo` through `HomieClient::handle_request` | FC-DHLR-005 |
| T-005 | Add `homie host locate-repo` CLI fixture E2E | FC-DHLR-005 |
| T-006 | Update remote-node spec, parity lock, Beads, and evidence | FC-DHLR-006 |

## Task to Requirement Mapping

| PRD Requirement | Tasks |
|-----------------|-------|
| FR-1 | T-001 |
| FR-2 | T-002 |
| FR-3 | T-003 |
| FR-4 | T-004 |
| FR-5 | T-005 |
