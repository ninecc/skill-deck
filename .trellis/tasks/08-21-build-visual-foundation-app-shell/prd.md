# Build visual foundation and app shell

## Goal

Deliver the first runnable UI increment: shared Pencil-aligned visual tokens,
offline Iconify/Lucide icons and the Toolbar/Inventory/Preview/Status two-pane
shell. This child establishes the contracts every later UI child consumes.

## Requirements

- Implement primitive, semantic and component tokens for System, Light, Dark,
  Sand and Plum without page-local palette duplication.
- Replace handwritten SVG paths with build-time bundled Lucide icons selected
  through Iconify; preserve typed domain icon names and accessible labels.
- The approved build integration may add `unplugin-icons` and
  `@iconify-json/lucide` as development dependencies.
- Refactor the application toolbar, inventory list, Preview frame and status bar
  to match the Pencil wide Preview and responsive proofs.
- Establish a deterministic development/review-only typed IPC scenario entry
  with initial shell states. It must be absent from production output.
- Seed it with Pencil canonical samples plus long/empty/Chinese stress data,
  keep it as documented developer tooling after this task and never read user
  data for review.
- Preserve command dispatch, selection, loading and preference behavior.
- Refactor incrementally rather than rewriting `App`; match Pencil strictly
  except for explicitly reviewed platform-controlled differences.
- Keep OS-native window decorations; do not render Pencil's macOS title bar or
  traffic-light controls in React.
- Apply the shared shell to Windows, macOS and Linux and interpret both Pencil
  reference sizes as WebView content viewport sizes.
- Enforce 720×520 as the content minimum; do not add smaller-window behavior.
- Preserve current localized product copy unless a wording change is separately
  approved.
- Preserve the two-pane composition at 1180×800 and 720×520; compact mode may
  hide secondary labels but may not become a mobile single-pane flow.
- Do not implement feature-specific file-tree, translation, modal or Settings
  restyling beyond compatibility needed for the new foundation.

## Dependency

- None. This is the first implementation child.

## Acceptance Criteria

- [ ] The default Preview state matches the Pencil Dark and Light wide proofs.
- [ ] All five themes resolve through shared semantic tokens and match their
      responsive shell proofs at 1180×800 and 720×520.
- [ ] Required Lucide icons are bundled at build time; built output contains no
      Iconify API endpoint or icon-related runtime request.
- [ ] Production size is measured before/after, no complete Lucide collection is
      bundled and startup shows no material regression.
- [ ] Initial shell review states render without real CLI/network/user-data
      access, and production contains no harness identifiers or fixture data.
- [ ] Full shell matrices use the review entry and representative shell states
      pass native Tauri smoke.
- [ ] Inventory selection/focus, toolbar command availability, Preview reading
      and status feedback behavior remain covered by tests.
- [ ] Format, lint, typecheck, Vitest and production build pass.
- [ ] User reviews and approves this child's visual evidence before Child 2.
- [ ] Corrections are incorporated before the child's final commit/archive.
- [ ] Unfinished feature surfaces remain visually coherent through the shared
      tokens and base controls even before their dedicated child task.
- [ ] Accessibility/native behavior deviations from Pencil are minimized,
      documented and included in user review.

## Out of scope

- File-tree and translation composition.
- Lifecycle dialogs, Settings and operational-state redesigns.
- Agent target ordering or backend behavior changes.
