# Verification Report Templates

This directory contains lightweight report templates for Homie change verification.

Every meaningful change should write evidence under:

```text
docs/verification/<change-id>/
```

Recommended reports:

| Report | Purpose |
|--------|---------|
| `spec-review-report.md` | PRD and component spec review result |
| `openspec-alignment-report.md` | PRD/spec to OpenSpec task mapping |
| `sdd-tdd-task-report.md` | Red-green-refactor task execution evidence |
| `test-report.md` | Unit and integration test evidence |
| `e2e-report.md` | Local end-to-end or manual workflow verification |
| `security-review-report.md` | Credential, virtual key, logging, prompt, and proxy safety review |
| `code-review-report.md` | Code review findings and fixes |
| `release-readiness-report.md` | Final gate summary |

Reports must contain actual commands, results, risks, and file references. Do not mark a report as `pass` if the command was not run.

## Status Values

| Status | Meaning |
|--------|---------|
| `pass` | Evidence was collected and the gate passed |
| `blocked` | The gate could not pass because a required condition failed |
| `not_run` | The gate was not executed; reason must be recorded |
| `partial` | Some evidence exists, but scope is incomplete |

## Evidence Rules

1. Use concrete paths and commands.
2. Record exit codes for commands.
3. Record environment blockers plainly.
4. Keep real secrets, provider keys, raw prompts, cookies, and full tool arguments out of reports.
5. If a risk remains, link or create a Beads issue for it.
