# Integrate and verify Pencil UI refactor

## Goal

Perform the final cross-child integration, native-desktop visual comparison and
regression hardening needed to accept the complete Pencil UI refactor.

## Requirements

- Review the running desktop App against every owned Pencil reference, not only
  isolated browser components.
- Treat native OS title bars/window controls as platform context and compare the
  shared React UI from Application Toolbar downward.
- Interpret both approved dimensions as WebView content viewport sizes and
  exercise the same shared UI on all three supported desktop platforms.
- Cover System, Light, Dark, Sand and Plum at 1180×800 and 720×520.
- For System, explicitly exercise OS Light and OS Dark at both sizes rather than
  relying only on the two drawn System examples.
- Cover English and Simplified Chinese, long Agent lists/names, loading, empty,
  error, partial, modal, popover, translation and Settings states.
- Verify keyboard-only operation, visible focus, focus restoration, reduced
  motion, contrast, scrolling, stacking and no clipped/overlapping content.
- Verify no icon-related network request and no unresolved Iconify API endpoint
  in the production bundle.
- Require macOS native smoke and successful three-platform build/shared-UI
  evidence. Record unavailable Windows/Linux native smoke without blocking this
  task; those checks remain in the release smoke workflow.
- Preserve accessibility and native behavior over pixel fidelity, documenting
  every necessary visual deviation.
- Fix only integration regressions within the approved parent scope.
- Store the verification matrix and conclusions as text; do not commit review
  screenshots or add automated pixel-baseline infrastructure.
- Use the deterministic review entry to reproduce every Pencil state and prove
  the production bundle excludes all harness code and fixture data.
- Retain the documented review entry as long-lived developer tooling. Use it for
  the full matrix and native Tauri for representative/critical flows.

## Dependency

- Requires Children 1–4 to be completed, checked, committed and archived.

## Acceptance Criteria

- [x] Every parent acceptance criterion is backed by test or visual/native-smoke
      evidence, with any unavailable platform check recorded explicitly.
- [x] All Pencil screens have been compared at their approved size/theme/state.
- [x] Full frontend quality gate passes from a clean checkout state.
- [x] No behavior, localization, accessibility or offline regression remains.
- [x] Production bundle size is compared to baseline, only used icons ship and
      no material startup regression is observed.
- [x] Parent artifacts accurately record final scope and verification.

## Out of scope

- New features or design changes not required to correct an integration defect.
