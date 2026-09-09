# Correct UI state semantics

## Goal

Make empty, startup, and translation states truthful, visible, and recoverable at both supported desktop viewports.

## Background

The audit confirmed contradictory empty Inventory guidance, a blank empty-filter branch, Ready styling during startup and translation, hidden narrow-view egress disclosure, and translation failure/Retry behind an unselected pane.

## Requirements

- R1: With an empty Inventory, both panes present one coherent Find & Install next action; Preview must not ask users to select a nonexistent Installed Skill.
- R2: Empty Inventory plus a non-empty filter must show a recovery state or prevent the meaningless filter state.
- R3: Startup and translation activity must not use Ready status semantics; shared visible status or an equivalent live region must communicate active work.
- R4: At 720x520, external translation egress disclosure remains visible and readable.
- R5: At 720x520, translation failure and Retry are visible without requiring users to discover an unselected failing tab; original content remains accessible.
- R6: Preserve the approved two-pane desktop composition and existing theme/localization behavior.
- R7: Changes must follow a narrow usability correction, not an unapproved visual redesign.

## Acceptance Criteria

- [x] `shell-empty` at 1180x800 and `empty-720` present coherent, recoverable empty states.
- [x] Empty Inventory plus filter never produces an unexplained blank list.
- [x] `shell-loading` and `content-translation-loading` expose non-ready observable status semantics.
- [x] `content-translation-error` at 720x520 exposes the error and Retry on the initial task path while preserving access to original content.
- [x] Translation egress disclosure is visible at 720x520.
- [x] Focused behavior tests fail on the current defects and pass after the fix.
- [x] Deterministic browser review passes at 1180x800 and 720x520 in both locales represented by the affected scenarios.

## Out of Scope

- Changing the translation provider or persistence model.
- Replacing the two-pane layout with mobile navigation.
- Broad command hierarchy, typography, theme, or Settings redesign.
