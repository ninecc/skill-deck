# Audit project functionality completion

## Goal

Produce an evidence-backed assessment of how much of Skill Deck's current accepted MVP functionality is complete, so the remaining work can be prioritized confidently.

## Background

- The repository is a Tauri 2 desktop application with a React 19/TypeScript frontend and Rust backend.
- Accepted ADRs 0011–0016, `CONTEXT.md`, active Trellis specs, and the current implementation define the current product contract.
- The approved 2026-08-13 redesign established that the upstream Skills CLI owns lifecycle state while Skill Deck provides a single-window inventory management UI, bounded preview, preferences, and session-only translation.
- `docs/roadmap.md` predates that redesign and contains conflicting Managed Library/reconciliation language. It remains useful as historical long-term planning evidence, but it is not the authority for current-MVP behavior.
- Completion must not be inferred from file presence alone; each conclusion needs implementation, automated-test, build/static-check, or runnable-behavior evidence.

## Requirements

- R1: Inventory the major capabilities promised by the current accepted MVP contract.
- R2: Map every inventoried capability to concrete frontend, backend, command, persistence, and automated-test evidence where applicable.
- R3: Run the repository's supported frontend and backend formatting, lint/static-analysis, test, and production-build checks; record failures and environmental limitations reproducibly.
- R4: Exercise the deterministic review entry at the documented desktop viewports for representative startup, inventory, content/translation, lifecycle, settings, localization, theme, and failure states when the local environment supports it.
- R4a: Perform a read-only real-runtime check of Skills CLI readiness and Inventory when supported. Normal `npx skills@latest` network resolution and npm cache writes are permitted, but no Skill lifecycle mutation is allowed. Retain only readiness, version, count, and sanitized outcome evidence; do not record Skill names, paths, content, or other Inventory details.
- R5: Classify each capability as complete, partial, unimplemented, or not verifiable, with its reason, evidence confidence, and next action.
- R6: Calculate a conservative current-MVP completion score using the declared rubric, and report release readiness separately so automated coverage is not confused with packaged cross-platform validation.
- R7: Report the historical R1–R4 roadmap as a separate long-term progress view. Do not count removed Managed Library/reconciliation concepts as missing current-MVP work.
- R8: Identify the highest-impact gaps and recommend a prioritized next-work sequence without implementing fixes in this task.

## Scoring Contract

- The primary denominator is the current accepted MVP contract derived from accepted ADRs, active specs, current domain language, and approved redesign requirements.
- Capability groups are equal-weighted, then independently observable capabilities are equal-weighted within each group, preventing numerous small UI states from overwhelming a missing core workflow.
- Each capability receives `complete = 1`, `partial = 0.5`, or `unimplemented = 0`.
- Ordinary capabilities require implementation evidence plus a behavior-oriented automated test that passes in this audit to be complete.
- Integration-sensitive capabilities involving the real CLI, filesystem, native menus, OS helpers, or packaged applications also require matching integration/native evidence; without it they can score no higher than partial.
- `not verifiable` receives `0` in the conservative headline score and is also reported separately in an evidence-coverage ratio, so missing evidence cannot be mistaken for completion.
- Major capability groups are reported individually before aggregation. Any weighting beyond equal capability units must be disclosed and justified in the report.
- The six equal-weight groups are: Runtime/Inventory; lifecycle Search/Install/Update/Remove; file tree/Preview/Reveal; translation/network boundary; Application Commands/menu/shortcuts/status feedback; and Settings/theme/localization/accessibility/responsive behavior.
- The report presents one conservative completion score and a separate implemented upper bound. The upper bound is explicitly hypothetical and cannot be described as completed functionality.
- Release readiness is a qualitative gate based on automated checks plus available native packaging/smoke evidence; it is not derived from the functional percentage.
- Current HEAD is considered not release-ready until current-architecture native smoke exists for the required platforms. Missing or stale proof is reported as an evidence gap, not as proof that the platform behavior fails.
- Historical R1–R4 roadmap progress is reported independently as complete, partial, not started, or superseded; it receives no aggregate percentage and does not alter the current-MVP score.

## Acceptance Criteria

- [x] A capability matrix cites the relevant requirement source and implementation/test evidence for every major current-MVP feature area.
- [x] Frontend format, lint, type, test, and production build checks plus Rust format, Clippy, and test checks are run where supported; failures and environment limitations are recorded reproducibly.
- [x] Representative deterministic review scenarios are inspected at 1180×800 and 720×520 when supported, and the report states which states were and were not visually exercised.
- [x] The report gives a transparent current-MVP functional-completion estimate, an evidence-coverage ratio, and a separate release-readiness assessment.
- [x] Every partial, missing, or unverifiable capability has a concrete reason and suggested next action.
- [x] The report distinguishes confirmed facts from inferred judgments and does not count explicitly future or superseded roadmap behavior against the MVP.
- [x] Historical R1–R4 progress is summarized separately, with conflicts against the current accepted product model called out.
- [x] No product code, behavior, dependencies, or visual design are changed by the audit.

## Key Decisions

- The current accepted MVP is the headline completion denominator.
- The historical R1–R4 roadmap is an independent appendix/progress view.
- Accepted ADRs and active implementation/spec contracts override conflicting roadmap language.
- The audit uses conservative scoring and separates functional completeness, evidence coverage, and release readiness.
- Functionality completion is the primary report; release readiness is a separate hard-gate assessment.
- Functional scoring uses equal-weight groups with equal-weight capabilities inside each group; raw counts and group subtotals remain visible.
- The six equal-weight groups are fixed in the Scoring Contract, while native packaging remains outside the functional score.
- The headline is a conservative score; a clearly labeled implemented upper bound may show the maximum if missing integration evidence later passes.
- Real runtime and Inventory checks are read-only and privacy-minimized. Real install, update, and remove commands are prohibited.
- Normal `npx skills@latest` network resolution and npm cache writes are allowed for that read-only check; network/permission failure remains an environment limitation and is not bypassed.
- Visual evidence covers functional clarity, keyboard/focus behavior, responsive operability, and localization, not subjective aesthetic polish.
- Historical R1–R4 progress uses milestone states rather than an aggregate percentage.
- A severe finding is placed first in the report but does not stop other safe, read-only checks; only the affected or potentially mutating check is stopped.
- Current HEAD fails the release-readiness gate because native evidence is stale or missing, while the report must distinguish unproven behavior from demonstrated failure.
- The deliverable is a task-local `evidence.md` report; remediation is deferred to separately approved tasks.

## Out of Scope

- Implementing or fixing missing functionality during this audit.
- Changing product behavior, UI, architecture, dependencies, or persistent data.
- Restoring the superseded Managed Library, ownership, revision, reconciliation, or rollback model.
- Claiming production safety, security certification, or cross-platform readiness beyond available evidence.
