# Homie AI Coding Workflow — Evidence-Backed Analysis

**Date:** 2026-08-28
**Scope:** Analysis of how AI-assisted coding is governed, executed, and evidenced in this repository.
**Method:** Static inspection of the repository's workflow artifacts, cross-checked against hard counts and concrete samples. No code was modified.

---

## 1. Executive summary

Homie runs one of the most formally-governed AI-coding workflows found in a single-repo desktop project. It is not a chat-driven "agent does the work and describes it" loop — it is a **document → plan → implement → prove** pipeline enforced by repo-level contracts and a local issue database. The workflow is real (not aspirational): 160 tracked issues, 64 OpenSpec changes, 66 PRD/spec documents, 66 evidence directories, and 54 annotated release tags all line up against 106 commits on `main`.

The workflow is **strong where it matters most** — security-sensitive gateway/credential code and high-risk concurrency — and **thin where the change is a mechanical refactor**, which is a defensible tiering decision. Two concrete gaps stand out: (1) four security-sensitive LLM-gateway changes lack the Tier-3 `failure-model.md` evidence the project's own standard demands, and (2) evidence depth is bimodal, with ~40% of change directories holding a single evidence file.

Crucially, the workflow **governs its own evolution**: a PRD (`evidence-driven-gauntlet-gates`) explicitly identified five weaknesses in the prior workflow and hardened the standards in response. This meta-level self-correction is the strongest signal that the process is genuinely followed rather than cargo-culted.

---

## 2. What the workflow actually is

The governing contracts are, in authority order:

| Layer | Location | Role |
|-------|----------|------|
| Top-level authority | `AGENTS.md` | Required workflow, Beads linkage, security baseline, build-cache rules |
| Required dev workflow | `AGENTS.md` §"Required Development Workflow" | 10-step pipeline |
| Dev standards | `docs/development/standards.md` | Rust/GPUI/Swift/TDD/Tier rules |
| Quality gates | `docs/development/quality-gates.md` | Per-domain minimum evidence + fail conditions |
| Orientation | `docs/architecture/project-layout.md` | Module map, read-before-change |
| Package policy | `docs/research/rust-package-selection.md` | Dependency addition rules |
| Contributor contract | `CONTRIBUTING.md` | Human-facing build/test entrypoints |

The canonical pipeline (from `AGENTS.md`) is:

```
1. Beads issue with stable change_id
2. Chinese PRD/spec under prd-spec/
3. Durable contract updates under specs/ (when boundaries change)
4. Read project-layout / standards / quality-gates / package-selection
5. Spec review → docs/verification/<change-id>/
6. openspec/changes/<change-id>/plan.md + tasks.md
7. OpenSpec alignment report
8. SDD/TDD from OpenSpec tasks (RED→GREEN→REFACTOR)
9. Local verification → docs/verification/<change-id>/
10. Close Beads issue only after evidence matches delivery
```

Documentation boundaries are explicit and non-overlapping (see the table in `AGENTS.md`): product overview → `README.md`, requirements → `prd-spec/`, durable contracts → `specs/`, execution → `openspec/`, evidence → `docs/verification/`, issue state → Beads.

---

## 3. Hard evidence of adherence

### 3.1 Volume and consistency

| Artifact | Count |
|----------|-------|
| Beads issues (total / closed) | 160 / 160 |
| OpenSpec changes (`plan.md` / `tasks.md` / `alignment-report.md`) | 64 / 64 / 64 |
| PRD/spec documents | 66 |
| Evidence directories (`docs/verification/<id>/`) | 66 |
| Durable component specs (`specs/*.md`) | 7 |
| Git tags | 54 |
| Commits on `main` | 106 |

The three OpenSpec files (`plan.md`, `tasks.md`, `alignment-report.md`) are present in **all 64** change directories — no partial OpenSpec entries exist. This is a strong, mechanically-checkable adherence signal.

### 3.2 Change-id consistency

Cross-referencing the three identifier namespaces (OpenSpec change dirs, verification dirs, PRD/spec topic dirs) yields near-perfect alignment:

- **64 OpenSpec changes all have matching verification directories** (zero orphans).
- **62 of 66 PRD/spec topics have matching OpenSpec changes.** The four mismatches are explainable outliers:
  - `homie-v1-architecture` and `project-workflow-bootstrap` — verification dirs with no OpenSpec (bootstrap/architecture narrative docs, not implementation changes);
  - `architecture-audit-governance-2026-08` and `clean-dev-titlebar-chrome` — PRD topics with no OpenSpec (a governance document and one bugfix that appears not to have reached the OpenSpec stage).

### 3.3 Evidence depth distribution

Evidence volume is **bimodal**, which maps to the project's own Tier calibration:

**Thick changes (5–33 evidence files)** — substantive, high-risk, or cross-cutting work:

- `engine-control-wire-runtime-split` — 33 files (7 code-review rounds, 22 `fc-*.log` gates)
- `agent-manifest-single-source` — 31 files
- `startup-background-shell-invisibility` — 23 files
- `engine-registry-session-split` — 20 files (4 code-review rounds)
- `gpui-architecture-hardening` — 17 files
- `protocol-contract-golden-fixtures` — 13 files
- `llm-gateway-daemon-embed` — 5 files (the one gateway change with full functional-cases + code review)

**Thin changes (1–2 evidence files)** — ~42 of 66 directories hold 1–2 files. Almost all are the `*-module-split` mechanical refactors (Aug 20), each carrying only a `release-readiness-report.md`. The LLM-gateway security changes (`virtual-keys`, `policy-quota`, `credential-login`, `model-routing`) also sit in this thin band at 2 files each — this is the notable exception, flagged in §5.

Across all directories: 65 `release-readiness-report.md`, 40 `code-review-round-*.md`, 24 `functional-cases.md`, 24 `spec-review-report.md`, 118 `fc-*.log` gate logs, and only **2 `failure-model.md`**.

### 3.4 Versioning discipline

`AGENTS.md` requires a new SemVer tag per functional change with a descriptive message. The tag history shows this is followed:

- `v0.1.0` (Aug 16) → `v0.8.29` (Aug 25): 54 tags in ~9 days.
- Minor-version bumps (`v0.1.0 → v0.2.0 → … → v0.8.0`) map to new capabilities; patch bumps map to narrow fixes.
- The Aug 20 burst (`v0.8.5`–`v0.8.24`) corresponds to the `*-module-split` refactor series — each mechanical split receiving its own tag, consistent with "patch" classification.

---

## 4. Where the workflow is genuinely strong

1. **Fail-closed gates, not reports.** `quality-gates.md` §2.1 mandates that every gate state its failure condition, that home-grown checkers fail closed (no `|| true` / `2>/dev/null`), and that checkers prove they can fail via a negative control before being trusted. This is rare and materially raises evidence trustworthiness.

2. **Anti-gaming rules are explicit and enforceable.** `standards.md` §6.2 forbids weakening tests, editing test+impl in the same step, mocking the unit under test, chasing coverage numbers, and reporting unrun layers. These directly counter the failure modes of AI agents producing vacuous tests.

3. **Changed-line coverage over global percentage.** `quality-gates.md` §4.1 explicitly rejects the global-coverage vanity metric in favor of fail-under changed-line coverage — a principled stance against gaming.

4. **Tier calibration to blast radius.** §6.3 scales effort from "typo" (full suite + lint) to "high-stakes" (failure model + property tests + mutation + adversarial pass). This prevents both over- and under-engineering.

5. **Security is treated as a first-class boundary.** The pre-commit hook (`scripts/check.sh` + `.githooks/pre-commit`) blocks secret patterns and sensitive file paths; `AGENTS.md` security baseline; and the `homie` skill explicitly treats virtual-key/credential custody as the "red lines."

6. **Self-governing evolution.** The workflow documents are themselves produced and hardened *through* the workflow, with the `evidence-driven-gauntlet-gates` PRD (Aug 17) explicitly enumerating five prior gaps (G1–G5) and closing them.

---

## 5. Gaps and risks found

### GAP-1 — Tier-3 security work lacks its mandated failure-model evidence

`standards.md` §6.3 names "credential custody, virtual key issuance, LLM proxying, orchestration, concurrency, and data-loss-adjacent code" as **Tier 3 default**, requiring a `failure-model` + mutation + adversarial pass. Yet:

- Only **2 of 66** evidence directories contain a `failure-model.md` (`engine-session-runtime-split`, `process-tree-governor-verification`).
- The four gateway security changes — `llm-gateway-virtual-keys`, `llm-gateway-policy-quota`, `llm-gateway-credential-login`, `llm-gateway-model-routing` — each hold only `release-readiness-report.md` + `spec-review-report.md`, with **no failure model, no functional-cases, no code-review round**.

This is the single most material finding: the highest-stakes code (credential custody, virtual keys, the OpenAI-compatible proxy) has the *least* verification evidence, contradicting the project's own Tier-3 rule. (Partial mitigation: `llm-gateway-daemon-embed` is thicker and some gateway logic may be covered indirectly, but the named gate is missing.)

### GAP-2 — Bimodal evidence depth leaves ~40% of changes with a single file

~42 of 66 verification directories contain only a `release-readiness-report.md` (1 file) or two files. While the `*-module-split` refactors are legitimately low-risk (Tier 1–2), the fact that *every* such refactor carries an identical single-file footprint makes it hard to distinguish "properly minimal" from "under-verified" without opening each report.

### GAP-3 — Evidence quality is time-dependent (the standard tightened mid-stream)

`typed-agent-driver-capabilities` (Aug 13) reports only 8 gates with no numbers and a terse "Not Run" section, whereas post-Aug-17 changes (e.g. `engine-control-wire-runtime-split`, 33 files) paste actual pass counts. This is *correct* — the workflow improved — but it means historical evidence is not uniformly trustworthy, and "64 changes all followed the same standard" is false. An auditor must date-stamp which standard version each change was held to.

### GAP-4 — Two PRD topics have no OpenSpec/verification trail

`clean-dev-titlebar-chrome` (a bugfix) and `architecture-audit-governance-2026-08` have PRD documents but no corresponding `openspec/changes/` or `docs/verification/` directory, leaving their delivery status unverifiable from the repo alone.

---

## 6. Assessment

| Dimension | Rating | Evidence basis |
|-----------|--------|----------------|
| Process definition | **Excellent** | Layered, non-overlapping docs; fail-closed gates; anti-gaming rules |
| Process adherence | **Strong** | 64/64 OpenSpec complete; 160/160 Beads closed; near-perfect id alignment |
| Evidence trustworthiness | **Good, improving** | Fail-closed + negative-control + changed-line coverage + fresh-run rule |
| Security treatment | **Mixed** | Strong *rules* (Tier-3 default, secret hook) but **weak *evidence*** on gateway work (GAP-1) |
| Self-correction | **Excellent** | Workflow hardens itself through its own PRD/verification loop |

---

## 7. Recommendations (ranked)

1. **Backfill failure models for the four LLM-gateway security changes** (`virtual-keys`, `policy-quota`, `credential-login`, `model-routing`), or explicitly record why Tier 3 was not required. This is the highest-leverage gap — the rules are right, the evidence is missing.
2. **Enforce the Tier-3 gate at the tooling level** (a `check.sh`/CI step or a `bd` status check) so a security-sensitive change cannot close without a `failure-model.md`. A rule that only lives in prose is gamed by omission, not by malice.
3. **Version-stamp the workflow standard each change is held to** (e.g. reference the `evidence-driven-gauntlet-gates` change-id in `release-readiness-report.md`), so future audits can distinguish pre- vs post-hardening evidence.
4. **Resolve the two orphan PRD topics** (`clean-dev-titlebar-chrome`, `architecture-audit-governance-2026-08`) — either close them as intentionally-doc-only or produce their OpenSpec/verification trail.
5. **Add a mechanical drift check** (in the spirit of the existing `scripts/check-agent-manifest-drift.sh`) that asserts every `openspec/changes/<id>/` has a matching `docs/verification/<id>/` and a `prd-spec/**/<id>/` source. The current consistency is high but currently verified only by hand.
