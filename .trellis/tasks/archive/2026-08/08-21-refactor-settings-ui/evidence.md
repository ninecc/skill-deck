# Child 4 implementation evidence

## Structure and behavior

- Wide Settings uses the approved 720×548 dialog. Header, navigation and footer
  are fixed at 52px, 42px and 48px; only the 404px content viewport scrolls.
- At the 720×520 application boundary the dialog remains inset at 684×496. Its
  chrome retains the same row heights and the content viewport becomes 352px.
- Settings always opens on General and initially focuses its first navigation
  button. Left/Right wrap through sections; Home/End move to the first/last
  section while preserving ordinary Tab and Shift+Tab order.
- Escape, header Close and footer Close dismiss through `ModalShell` and restore
  focus to the Settings command.
- General exposes System Default, English and Simplified Chinese UI intent.
  The effective locale also updates the document `lang` attribute.
- Appearance exposes a visible Theme field and five build-time-token previews:
  System (Default), Light, Dark, Sand and Plum.
- Translation target persists immediately. Proxy input remains a local draft
  until Apply; invalid credential/path values render inline and never enter
  persisted preferences.
- Installation method and Agent choices persist immediately. Automatic,
  explicit, filtered and no-match states reuse the upstream `agentOptions`
  order without sorting. The Agent result viewport remains bounded while the
  owning Settings content viewport is independently scrollable.
- About presents the current CLI version, or a muted em dash when unavailable.

## Deterministic review scenarios

- `settings-proof-general`
- `settings-proof-appearance`
- `settings-proof-translation`
- `settings-proof-translation-invalid`
- `settings-proof-installation-auto`
- `settings-proof-installation-explicit`
- `settings-proof-installation-no-match`
- `settings-proof-about`
- `settings-proof-appearance-light`
- `settings-proof-installation-zh`

The invalid proxy uses a non-secret masked canonical value. Explicit targets
use the approved `cod` pressure query while retaining upstream result order.
Review locale fixtures override only the development browser language and never
enter the production graph.

## Real-layout checks

- All ten proof states were exercised at 1180×800 and 720×520: 20 frames.
- The eight production states resolved Dark, the isolated Appearance proof
  resolved Light, and the Chinese Installation proof resolved Dark/zh-CN.
- Every frame kept 52/42/48px fixed chrome, stayed inside the viewport and had
  document scroll extents exactly matching its requested size.
- Wide states measured exactly 720×548. Compact states measured 684×496 with an
  18px horizontal and 12px vertical inset.
- Installation content measured 404/430px wide-state client/scroll height and
  352/410px compact client/scroll height, proving the content viewport scrolls
  without moving dialog chrome.
- A fresh review tab produced no browser console warning or error. Warnings
  observed during live source edits came only from development hot reload and
  were absent from the fresh run.

Screenshots were reviewed transiently and are not committed.

## Automated coverage

- Immediate persistence: UI locale, theme, target language, install method and
  Agent target mode.
- Proxy draft retention across section switches, invalid rejection and valid
  Apply persistence.
- Navigation ArrowLeft/ArrowRight/Home/End, initial focus, Escape, header/footer
  dismissal and focus restoration.
- Upstream Agent order, explicit selection, no-match filter and unavailable CLI
  version presentation.
- English/Simplified Chinese catalog parity and document language publication.

## Quality gate

- `npm run format:check`: passed
- `npm run lint`: passed
- `npm run typecheck`: passed
- `npm test -- --run`: passed, 69 tests
- `npm run build`: passed
- `git diff --check`: passed
- Production `dist/` contains only `index.html`, one CSS asset and one JS asset
  (412 KiB total on disk).
- Production scan found no review entry, `settings-proof-*` identifier,
  canonical fixture marker or Iconify API endpoint.

## Independent and native checks

- The final independent Trellis check fixed focused-but-unselected section
  navigation so Left/Right/Home/End calculate from `event.currentTarget`, and
  added regression coverage for all four keys plus individual Agent-target
  persistence. The full gate then passed with 69 tests.
- The macOS Tauri WebView loaded the current UI with native system title-bar
  controls and the real CLI Inventory. Settings rendered correctly at the
  configured wide size and after resizing the client area to its compact
  minimum. General and the long Installation section retained fixed chrome;
  Escape and the header Close action dismissed the dialog and restored the
  toolbar Settings command. No install, translation, removal or preference
  mutation was triggered during this smoke test.

The user approved the representative wide, Light Appearance, invalid-proxy and
native compact Installation visuals on 2026-08-21.
