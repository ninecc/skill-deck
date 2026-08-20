# Refactor lifecycle dialogs and states

## Goal

Implement the Pencil presentation for Find & Install, Remove confirmation and
all operational recovery states without changing the shared Application Command
model or upstream CLI lifecycle authority.

## Requirements

- Match References 14–19: Find & Install, Remove, Loading, Runtime failure,
  Empty inventory and Preview failure.
- Split Find & Install into local Search and From source tabs, defaulting to
  Search and reusing the existing source field/hint/install action.
- Preserve both drafts/results while switching tabs during one opening. Normal
  close/reopen resets both and returns to Search; unresolved retry context keeps
  the existing recovery behavior.
- Reuse shared dialog chrome, tokens, icons and status patterns.
- Preserve search install, direct-source install, cancellation, destructive
  confirmation, retry, diagnostics and Reveal File behavior.
- Preserve one availability calculation and execution path per Application
  Command across toolbar, menu, shortcut and context-menu adapters.
- Render structured outcomes and refreshed Inventory; do not infer truth from
  paths or diagnostic strings.
- Put the existing Find & Install command inside Empty recovery and expose both
  Retry and Reveal File inside Preview-failure recovery as Pencil specifies.
- Keep core error/recovery copy localized in English and Simplified Chinese.
- Add deterministic dev/review scenarios for every owned lifecycle/recovery
  state; keep them absent from production output.

## Dependency

- Requires the visual foundation and content-navigation children to be
  completed, checked, committed and archived.

## Acceptance Criteria

- [x] References 14–19 are reproducible in the running application and match
      Pencil at 1180×800 without clipping at 720×520.
- [x] Dialog focus trap, Escape/cancel, destructive confirmation and trigger
      focus restoration remain correct.
- [x] Loading/error/empty/partial/success states expose textual meaning and
      recovery actions without relying on color.
- [x] Existing command and lifecycle tests pass with added state coverage.
- [x] Search/source tab switching and normal/unresolved reopen lifecycles are
      behaviorally tested.
- [x] Full frontend quality gate passes.

## Out of scope

- New providers, CLI behavior, automatic repair or Agent ordering changes.
