# Local Shell TERM Shortcuts Alignment Report

| Requirement | Task | Case | Status |
|-------------|------|------|--------|
| PRD root cause and scope | T1 | FC-01 | Covered |
| OpenSpec exists and references cases | T1 | FC-02 | Covered |
| helper sets TERM and removes NO_COLOR | T2 | FC-03 | Covered |
| local shell/generic path uses helper | T3 | FC-04 | Covered |
| remote shell/generic TERM policy stays aligned | T4 | FC-04, FC-06 | Covered |
| durable Engine runtime contract is recorded | T5 | FC-01, FC-02 | Covered |
| real shell session reports TERM | T6 | FC-05 | Covered |
| static gates | T7 | FC-06 | Covered |

No TerminalPane key-adapter work is included because diagnosis proved the input
bytes already reach Engine.
