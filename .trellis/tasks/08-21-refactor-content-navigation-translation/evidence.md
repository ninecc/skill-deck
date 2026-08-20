# Child 2 implementation evidence

## Behavior

- File trigger is an icon-only Lucide folder control with localized
  `aria-label` and tooltip copy at both approved sizes. Compact mode retains the
  full install path only as hidden DOM/title metadata.
- File folders are real treeitem buttons with `aria-expanded`, Click/Enter,
  Left/Right, Up/Down, Home/End and roving-tab-stop behavior.
- Every tree load starts expanded. Manual folder state remains local to the
  current loaded tree; reopening reveals the selected file's ancestors while
  preserving unrelated toggles.
- Escape, non-control outside dismissal and file activation restore the file
  trigger focus. Unsupported files remain selectable.
- Translation preserves the wide equal-pane composition and the 720×520 local
  Original/Translation tabs. Switching tabs changes visibility only and does
  not restart translation or discard either pane.
- Markdown links/images remain inert through the unchanged safe renderer.

## Deterministic review scenarios

- `content-tree`
- `content-translation`
- `content-translation-loading`
- `content-translation-error`

The canonical tree contains five files across root, folder, nested Markdown,
image and plain-text examples. Review IPC stays typed and does not use real CLI,
filesystem, inventory or network data.

## Real-layout checks

- Post-review identity correction: at 1180×800 the title and location rows each
  used the full 456px identity width; the long Skill name remained readable,
  source truncated independently, path owned the remaining location width and
  the leading 27px file trigger stayed fixed before the path. At 720×520
  source/path were hidden while the long Skill name and trigger remained
  visible on one row, with no page overflow. The popover opens to the right
  from the wide leading trigger and inward from the compact window edge.
- Final compact-header correction: at 720×520 the header is one balanced row.
  Its left side holds a truncating Skill name and fixed icon-only file trigger;
  the right side holds fixed icon-only Translate, Reveal, Update and Remove.
  Source, visible path and egress are hidden, while `.skill-path` retains the
  full install path and title as non-visual metadata. Reveal uses `folder-open`,
  distinct from the file-tree `folder` glyph. The popover remains anchored
  inward at the compact edge; wide source/path/labels remain unchanged.
- Final browser geometry: compact header 576×54px; identity 406×27px with a
  370px name area and 30px file trigger; command group 138×30px with four 30px
  controls. The 304px popover aligned to the trigger's right edge at x=562,
  full path/title metadata remained in the hidden DOM, and the document stayed
  exactly 720×520 with no console warnings. At 1180×800 the header remained
  70px, source/path/egress/action labels were visible and the trigger remained
  27px.
- Dark 1180×800 file tree: 304px vertical popover, all rows aligned, selected
  row focused, no page overflow.
- Dark 720×520 file tree: popover remains inside the content viewport and all
  rows remain reachable without page overflow.
- Dark 1180×800 translation: two equal 449px panes, stable separator, hidden
  compact tabs, no pane overflow.
- Dark 720×520 translation: one readable 576px pane, visible local tabs;
  switching to Translation preserved translated content and produced no page
  overflow.
- Compact deterministic loading and error states rendered both pane shells;
  error exposed localized alert copy and Retry.
- A fresh review-page load produced no browser console warning or error.

Screenshots were reviewed transiently and are not committed.

## Quality gate

- `npm run format:check`: passed
- `npm run lint`: passed
- `npm run typecheck`: passed
- `npm test -- --run`: passed, 58 tests
- `npm run build`: passed
- `git diff --check`: passed
- Production `dist/` scan: no review entry, scenario identifiers, canonical
  fixture marker or Iconify API endpoint.

Independent Trellis check passed after correcting the tree's controlled roving
tab stop and adding regression coverage for outside dismissal, refresh reset,
and selected-file disappearance. Native Tauri visual smoke and user visual
approval remain for the parent session's verification phase.

## Main-session visual matrix

After the independent fix, all four owned Dark scenarios (`content-tree`,
translation success, loading and error) were rerun at both 1180×800 and 720×520.
Every document/body scroll extent exactly matched its requested viewport and the
fresh console contained no warning or error. Compact Translation was also
activated on its local tab to verify success, loading and error content rather
than only the default Original pane. System, Light, Sand and Plum compact
file-tree representatives rendered without overflow.

## Native smoke limitation

The macOS Tauri window launched with system-provided title bar/window controls
and responded correctly at the 720×520 compact minimum. However, the available
same-bundle-id application instance remained in the CLI-unavailable state and
did not expose a real file-tree trigger. A temporary review `devUrl` was not
observable in that selected instance. No real translation request was sent, so
local Skill content was not transmitted externally. Feature-state native smoke
is therefore unavailable for this child; deterministic Chromium/WebView-size
evidence and automated focus/keyboard contracts cover References 11 and 12.
`tauri.conf.json` was restored and has no diff.

## User visual approval

The user reviewed the wide name/source and leading file-trigger/path layout,
then iterated the compact header to remove unnecessary visible provenance. The
approved final compact contract is a balanced 54px row: truncating Skill name
plus icon-only file-tree trigger on the left and four distinct icon-only actions
on the right. Source, visible path and egress are hidden; full path remains in
non-visual metadata. The user explicitly confirmed the final translation,
file-tree-open and long-name screenshots without further correction.
