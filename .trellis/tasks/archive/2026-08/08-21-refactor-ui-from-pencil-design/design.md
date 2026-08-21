# Technical design

## Delivery boundary

This parent coordinates five independently verifiable frontend child tasks. It
owns the approved Pencil requirement set, sequencing, shared architectural
contracts and final cross-child review. It has no direct production-code work.

## Architecture

### Visual foundation

- Keep a three-layer token model in `styles.css`: primitive palette values,
  semantic surface/content/action/feedback roles, then component dimensions.
- Resolve System, Light, Dark, Sand and Plum through the existing preference
  and system-theme pipeline. Components consume semantic tokens only.
- Preserve the Toolbar/Content/Status grid and two-pane Inventory/Preview
  workspace at both approved window sizes.
- Keep Tauri's system decorations. The Pencil macOS title bar is comparison
  context only; React layout starts at the Application Toolbar.
- Use the same WebView UI on Windows, macOS and Linux. Reference dimensions are
  content viewport dimensions and exclude platform-native window chrome.
- Retain a 720×520 minimum content viewport; do not invent sub-minimum layouts.

### Icons

- Keep a typed application-level `Icon` wrapper so call sites use domain names
  and remain accessible.
- Replace handwritten paths with selected Lucide icon data from Iconify.
- Pass imported icon-data objects to the renderer or generate static SVG data at
  build time. Never pass unresolved `lucide:*` strings that trigger Iconify API
  requests at runtime.

### Component seams

- `App` retains orchestration, command dispatch and short-lived interaction
  state. Presentation can be extracted only where that reduces coupling without
  duplicating command availability or execution.
- Modal focus/dismissal stays centralized in `ModalShell`; Settings owns only
  section and draft-form state.
- File tree and translation remain Preview-adjacent transient/content surfaces,
  not a third persistent workspace pane.
- File-tree directories are interactive disclosures using standard accessible
  tree semantics. At compact width Translation preserves both pane states and
  switches the visible pane through local tabs.
- Directory expansion is a per-tree in-memory `Set` keyed by normalized directory
  path. Trees initialize fully expanded; reopening auto-expands selected-file
  ancestors without changing Preview selection or unrelated toggles.
- Find & Install uses local Search/From source tabs over the existing catalog
  and source-install capabilities.
- Both tab drafts persist while the modal is open. Normal reopen resets to a
  fresh Search task; unresolved retry context remains behavior-compatible.
- Lifecycle dialogs and status states render existing structured command results
  without inferring domain truth from paths or diagnostic strings.

### Review-state harness

- Add a separate development/review entry that installs typed mocked IPC before
  mounting the application. It is not reachable from the production UI.
- Child 1 establishes the entry and shared fixture contracts; Children 2–4 add
  only their owned scenarios; Child 5 verifies full coverage and production
  exclusion.
- Fixtures are deterministic, local and side-effect free. They do not call the
  network, invoke the real CLI or touch user Skill files.
- Base fixtures mirror Pencil's canonical sample content; separate fixtures
  exercise long text, empty values and Chinese expansion.
- Retain and document the review entry after delivery as a developer tool that
  remains outside the production graph.

## Compatibility and migration

- No backend DTO, command, preference-storage or localization-key change is
  required solely for visual styling.
- Preserve production copy and localization unless a task-local requirement
  explicitly approves wording changes.
- Existing tests remain the behavior baseline. New tests target semantics,
  focus, state transitions and offline icon behavior rather than decorative DOM.
- Do not add automated pixel baselines in this task. Capture fixed-size review
  screenshots, compare them to Pencil and record the matrix/results as text.
- Prove the production output contains no review entry, scenario identifiers or
  fixture payloads.
- Use the review entry for the exhaustive matrix and Tauri native windows for
  representative per-child smoke plus final critical-flow verification.
- When accessibility or native behavior conflicts with Pencil appearance,
  preserve the behavior and document the minimal visual variance for review.
- Compare production bundle size before/after and ensure the build contains only
  imported Lucide glyphs rather than the complete collection.
- Child tasks land sequentially on the same branch. Later children adapt to the
  foundation established by earlier children instead of reintroducing local
  styling systems.
- Migrate incrementally rather than rewriting the React application. Temporary
  old/new styling coexistence is acceptable only inside the active child.
- Use strict Pencil comparison as the default visual standard and document any
  unavoidable platform-rendering variance.

## Rollback

- Each child must finish with a clean quality gate and its own commit, providing
  a stable rollback boundary.
- After each child check, pause for user visual approval before starting the
  dependent child.
- Sequence each child as implementation/check, user review, corrections, then a
  final child commit and archive.
- Use one planned `codex/refactor-ui-from-pencil-design` branch with one stable
  commit per child. Do not create a branch per child.
- If a child reveals a product gap in Pencil, return to parent planning and ask
  for the missing decision before changing behavior.
- Resolve low-level unspecified states from shared masters, accessibility specs
  and platform conventions; escalate only meaningful alternatives.
- Final integration may make cross-child fixes but must not silently expand
  feature scope.
- Do not rewrite approved child history when a later child causes a regression.
  Fix it in the current child and rerun the affected earlier evidence.
- After review corrections, rerun affected states plus representative regression
  frames. Escalate to the full child matrix when shared tokens/layout/components
  changed.

## Task map

1. `08-21-build-visual-foundation-app-shell`
2. `08-21-refactor-content-navigation-translation`
3. `08-21-refactor-lifecycle-dialogs-states`
4. `08-21-refactor-settings-ui`
5. `08-21-integrate-verify-pencil-ui`
