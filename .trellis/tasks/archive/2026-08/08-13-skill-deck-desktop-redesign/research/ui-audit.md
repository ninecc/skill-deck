# Current UI audit

## Evidence boundary

Audit date: 2026-08-13. Behavioral authority is the accepted ADR set, checked-in
React/Tauri implementation and tests. Historical roadmaps and prototypes were
used only to identify rejected models or failed directions.

The local running Tauri window exposed an older frontend bundle than the
checked-out source: it still showed a toolbar language picker and kept a
two-column layout near the minimum width, while current `App.tsx` has neither
behavior. Its screenshots are therefore useful evidence for visual crowding,
Settings length and truncation, but not an authoritative implementation map.

## Current structure and behavior

- `App.tsx` owns runtime, Inventory, selection, Preview, translation, discovery,
  mutation and feedback state. It derives command availability and dispatches
  the same command IDs across adapters.
- Default app chrome is Toolbar / Workspace / Status, with a 320 px Inventory
  pane and Preview detail. At 820 px, checked-in CSS switches between Inventory
  and detail rather than shrinking into a phone layout.
- Preview uses a file-path button to open an accessible tree popover. Markdown
  measure is capped at 74ch; translation becomes two columns and switches to
  tabs below 900 px.
- Settings uses one scrollable 520 px sheet. Appearance, target language, UI
  language, proxy and Apply, all Agent targets, install method and version are
  sequential siblings.
- Find & Install combines searchable results and a direct-source form in one
  modal. Remove is a separate confirmation modal whose initial focus is Cancel.
- Status is a persistent 32 px footer with operation/feedback on the left and
  Inventory/CLI facts on the right; diagnostics open above it.

## State coverage confirmed by code and tests

| Area | Confirmed states/contracts | Evidence |
|---|---|---|
| Startup | probe pending, localized runtime failure, Retry, inert workspace | `src/App.tsx:694`; `src/App.test.tsx:75`, `:136`, `:357`, `:382` |
| Inventory | empty, no filter match, selected/unselected, source/path matching | `src/App.tsx:801`; `src/App.test.tsx:89` |
| Commands | live availability, mutation/modal reasons, root vs document actions | `src/commands.ts`; `src/commands.test.ts:19` |
| Preview | loading, error+Retry, Markdown/Text/Code/Image/Unsupported, Reveal | `src/App.tsx:927`; `src/App.test.tsx:402` |
| Translation | egress notice, progress, timeout/unavailable, Retry, stale rejection | `src/App.tsx:969`; `src/App.test.tsx:467`, `:576` |
| Install | search, direct source, unresolved target, Retry, continued command | `src/App.tsx:1058`; `src/App.test.tsx:205` |
| Feedback | neutral, success, partial, error, details/diagnostics | `src/App.tsx:999` |
| Modal keyboard | focus trap, Esc, trigger/fallback restoration | `src/ModalShell.test.tsx:24`, `:55` |

## Priority findings

### P0 — preserve as non-visual contracts

1. The lifecycle boundary is already correct: Inventory is replaced from the
   pinned upstream CLI response. No redesign concept may infer lifecycle state
   from paths or reintroduce ownership/reconciliation.
2. Application Commands already separate availability and execution. A visual
   hierarchy change must project existing IDs, not add toolbar-local behavior.
3. Preview safety, translation generation invalidation, modal focus restoration
   and localized runtime messages have tests and must remain intact.

### P1 — visual/interaction defects to solve

1. **Settings hierarchy collapses.** The running sheet showed roughly 70 Agent
   entries consuming most of the viewport, with Install Method and CLI version
   below the fold. In source this is a two-column unsearchable grid at
   `src/SettingsDialog.tsx:146` and `src/styles.css:744`.
2. **Commit semantics are visually ambiguous.** Immediate-save fields and the
   proxy draft coexist, while only the latter needs Apply. The final Close
   button can be misread as a submit or cancel action even though it is neither.
3. **Installed filter focus is invisible.** The generic input focus rule is
   defeated by `outline: 0` on the nested input at `src/styles.css:329`; the
   wrapper receives no `:focus-within` treatment.
4. **Theme focus and selection are conflated.** Hidden radios at
   `src/styles.css:677` have no focus-visible projection; `.selected` at `:674`
   uses the same outline slot a focus indicator would need.
5. **Three update/refresh meanings look too similar.** Refresh Inventory,
   Update All and Update Skill all use the refresh icon at
   `src/App.tsx:687-688` and `:916`, raising recognition cost.
6. **Long identifiers lose distinguishing suffixes.** Inventory metadata,
   selected file path and toolbar actions all compete for narrow width. Current
   single-line ellipsis protects layout but can hide the part needed to
   distinguish sources and paths.
7. **State feedback is spatially fragmented.** Runtime blocks the workspace;
   empty/no-match sit inside Inventory; Preview and translation errors sit in
   content; install unresolved remains in its modal; mutation and final outcome
   sit in the footer. The locations are defensible but their visual grammar and
   recovery affordances are inconsistent.

### P2 — layout and implementation risks

1. The 900 px and 820 px breakpoints switch translation and master/detail in
   separate steps. Any new layout needs explicit behavior between 720 and
   1180, not only endpoint mockups.
2. Making the file tree persistent improves locality but costs Preview width
   and adds a new resizing/collapse seam. It should be approved as part of the
   direction rather than slipped in as CSS polish.
3. Platform-native visual materials cannot be reproduced identically in a
   shared WebView. Native feel should come from density, hierarchy, menus,
   pointer/keyboard behavior and system fonts, not simulated AppKit/WinUI chrome.

## Rejected inheritance

- No square-S, existing palette, existing radii or current token names are
  carried forward merely because they exist.
- No Managed Library, ownership, reconciliation, rollback or app marketplace.
- No marketing hero, CTA, Bento, glass stack, neon developer console, mobile
  bottom navigation or touch-first controls.
- No macOS-only visual choice silently applied to Windows/Linux.
