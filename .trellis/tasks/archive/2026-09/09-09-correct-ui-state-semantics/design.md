# UI state semantics design

## Decision

Correct state derivation and presentation inside the existing two-pane composition. No new persistent state or broad component system is introduced.

## State Rules

- Empty Inventory owns both pane messages. The Inventory pane exposes Find & Install; Preview reinforces that same next action instead of asking for selection.
- A filter cannot suppress every empty-state explanation. Either disable/clear it for empty Inventory or render the empty Inventory recovery state regardless of filter.
- Shared status severity/class derives from startup, runtime failure, active mutation, active translation, feedback, then ready—in that precedence order.
- Narrow translation failure selects or visibly marks the failed translation pane while preserving the Original tab.
- Egress disclosure remains present at 720px; compact copy/wrapping may change, but disclosure cannot be hidden.

## Existing Seams

Reuse `runtime`, `operation`, translation request state, `mobilePane`, localized copy, and deterministic review scenarios. Extend `OperationKind` only if that preserves one truthful status derivation; do not force translation into mutation serialization.

## Compatibility

Keep the 1180x800 and 720x520 two-pane layout, command registry, DTOs, themes, locales, and session-only translation model.

## Risks

- Treating translation as a lifecycle mutation could incorrectly disable unrelated commands.
- Auto-switching tabs on every translation state change could steal user choice; switch only on a newly observed failure or expose a visible error indicator.
- CSS-only fixes cannot resolve contradictory state copy; derive content from Inventory state first.
