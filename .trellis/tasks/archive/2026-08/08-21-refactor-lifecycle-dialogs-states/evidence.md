# Child 3 implementation evidence

## Behavior

- Find & Install now uses a 700×540 shared modal with Search and From source
  tabs. Search remains the default for a normal opening. Search query/results
  and source draft survive tab switches during one opening.
- A normal close/reopen resets both drafts and returns to Search. If an install
  result is unresolved, the active tab, drafts, last target and Retry context
  survive close/reopen.
- Both tabs reuse the existing typed `search_skills` and `add_skill` command
  paths. The modal footer explains pinned-CLI trust and retains the existing
  non-cancellation notice while an install is active. Search retains the
  `Install from source…` shortcut; From source intentionally has no reciprocal
  footer action because its local tabs already expose Search.
- Remove uses the shared focus trap and now identifies the Skill and path. It
  starts on Cancel, keeps the destructive action visually secondary to safety,
  and restores focus to the Inventory heading after successful removal deletes
  the original trigger.
- Startup Loading and Runtime failure match the approved centered operational
  compositions. Runtime failure exposes localized safe copy and Retry without
  rendering raw backend diagnostics.
- Empty Inventory includes the shared Find & Install command as an in-pane
  recovery action. The companion Preview pane explains what appears after a
  selection.
- Preview failure retains the requested path, retries that exact path and sends
  Reveal File through the existing Application Command dispatcher. The visible
  pane uses localized recovery copy; structured status feedback retains the
  normalized diagnostic behind Details.

## Deterministic review scenarios

- `lifecycle-loading`
- `lifecycle-runtime-failure`
- `lifecycle-empty`
- `lifecycle-preview-failure`
- `lifecycle-discovery-search`
- `lifecycle-discovery-source`
- `lifecycle-remove`

Review IPC uses only typed canonical fixtures. It does not invoke a real CLI,
network, Inventory or user filesystem. Locale-independent review automation
opens Search, From source and Remove states in both English and Simplified
Chinese.

## Real-layout checks

- Browser matrix: 7 owned scenarios × 5 themes × 2 sizes = 70 English frames.
- Simplified Chinese pressure matrix: 7 owned scenarios × 2 sizes = 14 frames.
- Every 1180×800 and 720×520 document scroll extent matched its viewport. No
  operational surface or dialog crossed the viewport boundary.
- Wide Find & Install measured 700×540 at (240, 130), matching Reference 14.
  Compact Find & Install measured 684×484 at 18px inset with its tabs, results
  and footer visible.
- Wide Remove measured 440×280 at (370, 260), matching Reference 15. Cancel was
  the initial focus and the canonical path remained visible.
- Loading and Runtime failure filled the workspace below Toolbar and above
  Status at both sizes. Empty Inventory and Preview failure remained centered
  inside their persistent panes.
- Preview failure exposed both Retry and Reveal File at both sizes. The fresh
  compact run produced no browser console warning or error.

Screenshots were reviewed transiently and are not committed.

The user approved the corrected wide and compact From source compositions on
2026-08-21. Deterministic lifecycle scenarios are development-only and cannot
be invoked safely in the production native bundle without real CLI, network or
user-filesystem effects, so native state smoke is recorded as unavailable; OS
window chrome remains covered by the preceding shell child.

## Quality gate

- `npm run format:check`: passed
- `npm run lint`: passed
- `npm run typecheck`: passed
- `npm test -- --run`: passed, 67 tests
- `npm run build`: passed
- `git diff --check`: passed
- Production `dist/` contains only `index.html`, one CSS asset and one JS asset
  (408 KiB total on disk).
- Production scan found no review entry, lifecycle scenario identifier,
  canonical fixture marker or Iconify API endpoint.

The final independent Trellis check passed after the footer correction, and the
user approved the resulting wide and compact visuals.
