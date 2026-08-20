# Implementation plan

## Ordered checklist

- [x] Re-read the live Pencil app state, source Settings composition, Dark
      appendix containers, theme variables, and shared dialog/control masters.
- [x] Mark the existing Settings source and any resized appendix containers as
      placeholders while they are actively modified.
- [x] Build the canonical 1180 × 800 Wide Dark full-context reference with
      General active and remove the non-production `Saved` badge.
- [x] Build a labeled modal-only matrix containing all eight Dark states,
      including Appearance for side-by-side comparison.
- [x] Build General and Translation default modal states by deriving the source
      dialog and replacing only section navigation/content/footer descendants.
- [x] Build Translation invalid-proxy state with the production error copy and
      masked sample value plus the apply-only persistence notice.
- [x] Build Installation automatic, explicit-target, and no-match states with
      authentic method, target, filtering, disabled/enabled, count, and empty
      result treatments; keep the list viewport fixed-height and scrollable.
- [x] Alphabetize visible Agent targets in the design; use `cod` with `codex`
      selected in the explicit state and record the production-order delta.
- [x] Build About with the Skills CLI version row and compact `—` alternative.
- [x] Build the separate Light Appearance and Dark Chinese Installation /
      Automatic proof states.
- [x] Add the compact behavior rail for default entry, persistence, scrolling,
      and the alphabetical-order recommendation/current implementation delta.
- [x] Arrange and label the context anchor, matrix, proofs, and behavior rail in
      the existing reference/documentation hierarchy; resize ancestors without
      overlapping other references or creating new roots.
- [x] Clear placeholders state-by-state after structural inspection.
- [x] Capture and review the context anchor, matrix, Light proof, and Chinese
      proof; directly repair clipping, hierarchy, copy, contrast, scrolling,
      localization, or alignment defects.
- [x] Run a final changed-scope Pencil scan for placeholders, names, bounds,
      clipping, warnings, and broken refs.
- [x] Run `git status --short` and confirm no production source changed.
- [x] Dispatch the Trellis quality check against the PRD and design contract.

## Validation gates

- Pencil MCP structural queries for changed Settings and appendix nodes.
- Pencil screenshots for the context anchor, matrix, and both isolated proofs.
- Visual checklist: active nav, modal/scrim bounds, 4/8 rhythm, readable copy,
  control state semantics, proxy error visibility, disabled targets, selected
  count, no-match empty state, and consistent footer behavior.
- Design-delta checklist: no `Saved` badge; General is default; Agent rows are
  alphabetical in the design and explicitly marked as not yet shipped.
- Repository-scope check: only the target `.pen` file and Trellis artifacts.

## Risk and rollback points

- Copy/replace one logical state per Pencil operation so failures roll back only
  that state.
- Resolve source descendants by current names before every copy batch because
  Pencil is collaborative and node IDs may change.
- Customize copied descendants inside `Copy`; do not update regenerated copied
  descendant IDs afterward.
- Resize the appendix from inner canvas outward and verify sibling positions
  before clearing placeholders.
- Do not alter the shared component masters unless a genuine reusable contract
  is missing; state-local controls are safer than unnecessary new masters.

## Implementation results

- Updated `Reference 13` to the production default-open General state while
  retaining the full 1180 × 800 application context.
- Added the eight-state Dark modal matrix, Light Appearance proof, Dark Chinese
  Installation proof, and compact desktop behavior rail inside the existing
  Dark reference appendix.
- Reused semantic theme variables plus the shared dialog header, footer,
  buttons, and search masters. Shared masters themselves were not modified.
- Screenshot review repaired scroll-row clipping and fractional-width clipping
  in the behavior rail. Focused structural scans report no placeholders,
  unnamed nodes, zero bounds, clipping warnings, or broken references in the
  changed Settings roots.
- The final Trellis quality check repaired the blank-proxy specimen, exact
  `System (Default)` labels, the alphabetical `cod` result set, and the default
  General keyboard-focus treatment directly in Pencil. The live active canvas
  is saved and `git status --short` reports `pen-design/skill-deck.pen` modified.
  No production source file changed.
