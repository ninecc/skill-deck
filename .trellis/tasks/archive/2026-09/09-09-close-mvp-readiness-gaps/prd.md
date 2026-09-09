# Close current MVP readiness gaps

## Goal

Remove the confirmed safety, contract, CI, and interaction-state defects identified by the functionality completion audit without expanding the accepted MVP or performing an incidental visual redesign.

## Background

The evidence baseline is `.trellis/tasks/09-03-audit-functionality-completion/evidence.md`. It records a conservative functional completion score of 64.5%, a current-implementation upper bound of 91.5%, and a NOT NOT READY release gate. The first remediation tranche contains four independently verifiable child tasks.

## In Scope

1. `09-09-restore-catalog-search-isolation`: catalog search remains usable without a resolved Skills CLI session.
2. `09-09-require-update-all-confirmation`: Update All requires explicit safe confirmation.
3. `09-09-repair-package-ci-gate`: the package contract gate executes a real current test and fails on zero matches.
4. `09-09-correct-ui-state-semantics`: empty, startup, translation, narrow-error, and egress states communicate truthfully and remain recoverable.

## Requirements

- R1: Preserve the accepted upstream-CLI ownership model and existing DTOs unless a child task explicitly proves a contract change is required.
- R2: Keep child ownership non-overlapping during parallel work. Search owns `src-tauri/src/cli.rs`; CI owns `.github/workflows/ci.yml`; Update All owns its/existing confirmation state in `src/App.tsx`; UI state semantics starts only after the Update All child completes because both touch `src/App.tsx`.
- R3: Each child must satisfy its own observable acceptance criteria and focused validation before integration review.
- R4: Final integration must run the complete frontend and backend quality gates and inspect the deterministic review states named by the UI child.
- R5: Product code must not add compatibility shims, duplicate command paths, or new persisted state.

## Acceptance Criteria

- [x] All four child tasks meet their acceptance criteria.
- [x] Catalog search no longer depends on runtime-session readiness.
- [x] Update All cannot invoke `update_skill` before explicit confirmation.
- [x] CI cannot pass its package contract gate with zero matching tests.
- [x] Empty, startup, and translation states are truthful and recoverable at 1180x800 and 720x520.
- [x] Frontend format, lint, typecheck, tests, and production build pass.
- [x] Rust format, Clippy with warnings denied, and tests pass.
- [x] Grok or another independent reviewer reports no blocking contract or UI findings.

## Key Decisions

- Execute search, Update All, and CI in parallel because their write ownership does not overlap.
- Execute UI state semantics after Update All to avoid concurrent `src/App.tsx` edits.
- Keep fake-CLI integration, three-platform native smoke, external-provider smoke, and broad visual redesign as later tasks; they are not hidden inside this tranche.

## Out of Scope

- New lifecycle features or restoration of the superseded Managed Library model.
- Real mutation of the user's global Skills.
- Native macOS, Windows, or Linux release smoke.
- Broad restyling without an Approved Visual Direction.
- Replacing the translation provider.
