# Alignment Report: Diri Host Prefs Sync

> Change ID: `diri-host-prefs-sync`  
> Beads: `homie-cue`

| Requirement | Task | Verification |
|-------------|------|--------------|
| Fixed include list | T-001 | FC-DHPS-001 |
| Credentials/transcripts/caches excluded | T-002 | FC-DHPS-001 |
| mkdir/rsync argv without delete | T-002 | FC-DHPS-001 |
| Clear rsync missing error | T-003 | FC-DHPS-001 |
| Quality/parity gates | T-004 | FC-DHPS-002..004 |

This change aligns with `specs/remote-node-handoff/README.md` and `specs/virtual-key-credentials/README.md`: prefs sync is secretless and does not copy provider credentials.
