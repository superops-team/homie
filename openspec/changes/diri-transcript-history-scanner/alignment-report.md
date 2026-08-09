# Alignment Report: Diri Transcript History Scanner

> Change ID: `diri-transcript-history-scanner`  
> Beads: `homie-941`

| Requirement | Task | Verification |
|-------------|------|--------------|
| Claude transcript fixture scan | T-001, T-002 | FC-DTHS-001 |
| Codex transcript fixture scan | T-001, T-002 | FC-DTHS-001 |
| Tracked id dedupe | T-002 | FC-DTHS-001 |
| Storage history write | T-003 | FC-DTHS-001 |
| Resume command projection | T-004 | FC-DTHS-001 |
| Quality/parity evidence | T-005 | FC-DTHS-002..004 |

The change is aligned with `specs/session-context-store/README.md` and `specs/storage-indexing/README.md`: scanner logic belongs in runtime/session-context, while durable history facts are written through storage APIs.
