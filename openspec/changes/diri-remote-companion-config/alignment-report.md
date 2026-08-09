# Alignment Report: Diri Remote Companion Config

> Change ID: `diri-remote-companion-config`  
> Beads: `homie-0gh`

| Requirement | Task | Verification |
|-------------|------|--------------|
| Remote config schema | T-001 | FC-DRCC-001 |
| Owner-only save/remove | T-002 | FC-DRCC-001 |
| Endpoint/pairing helpers and no debug token leak | T-003 | FC-DRCC-001 |
| Quality/parity gates | T-004 | FC-DRCC-002..004 |

This change aligns with the remote-node-handoff security contract: companion token may be stored in owner-only config and must not leak through Debug/display paths.
