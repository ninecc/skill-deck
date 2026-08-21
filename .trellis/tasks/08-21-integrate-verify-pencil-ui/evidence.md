# Final Pencil UI integration evidence

## Predecessors

All four implementation children are archived under
`.trellis/tasks/archive/2026-08/` with passing independent checks and user
visual approval:

- Visual foundation/app shell: feature commit `4214383`
- Content navigation/translation: feature commit `2a99b0b`
- Lifecycle dialogs/states: feature commit `9e40ddf`
- Settings UI: feature commit `54d0dd8`

## Exhaustive deterministic matrix

The review entry currently declares 26 canonical states: five shell/pressure
states, four content/translation states, seven lifecycle/dialog states and ten
Settings proof states.

Completed matrix:

- Four explicit themes (Light, Dark, Sand, Plum) × 26 states × two sizes × two
  locales: 416 frames.
- System with emulated OS Light and OS Dark × 26 states × two sizes × two
  locales: 208 frames.
- Total primary matrix: 624 frames, all passed.
- Compact Translation success/loading/error was additionally activated on its
  local Translation tab for six theme modes (four explicit plus System Light
  and System Dark) × two locales: 36 focused frames, all passed.

Every primary frame verified:

- requested review scenario ID;
- resolved theme and effective document language;
- exact 1180×800 or 720×520 App/document scroll extent;
- all dialogs and file-tree popovers fully inside the viewport;
- expected dialog/popover/translation/Settings surface presence.

No integration regression was found in this matrix. Long-name pressure retained
the 50-character canonical title, compact provenance stayed hidden, and both
persistent panes remained present.

## Accessibility and interaction

- File tree: one roving tab stop, selected file focused, two directories
  expanded, z-index 5 above the detail toolbar at z-index 3. ArrowDown moved
  `SKILL.md` -> `references`; ArrowRight entered
  `references/checklist.md`; Escape closed the tree and restored `.path-button`
  focus.
- Settings dialog: actual focus trap wrapped backward from header Close to
  footer Close and forward from footer Close to header Close. Keyboard focus
  rendered a 2px solid visible outline.
- Existing 69-test suite covers Inventory navigation, modal focus/dismissal,
  removal fallback focus, compact Translation tabs, Settings section keys,
  preference boundaries and stale async result rejection.
- With `prefers-reduced-motion: reduce`, spinner animation name was `none`,
  animation/transition duration `0s`, and scroll behavior `auto`.

## Contrast

Measured semantic-token contrast against actual resolved theme surfaces:

| Theme mode | Text/document | Muted/document | Muted/chrome | Emphasis |
| --- | ---: | ---: | ---: | ---: |
| Light | 15.30 | 5.03 | 4.73 | 11.97 |
| Dark | 13.00 | 6.31 | 6.70 | 12.25 |
| Sand | 15.16 | 5.75 | 5.14 | 11.97 |
| Plum | 14.33 | 7.63 | 6.74 | 12.25 |
| System / OS Light | 15.30 | 5.03 | 4.73 | 11.97 |
| System / OS Dark | 13.00 | 6.31 | 6.70 | 12.25 |

All sampled normal/muted text pairs exceed 4.5:1.

## Review isolation and icons

- CDP captured a fresh `content-translation` review load. Every request used
  `127.0.0.1:1420`; there was no external host, Iconify endpoint, translation
  provider request or user filesystem request.
- The review entry loaded only its typed mock boundary and canonical fixture
  modules. `installReviewIpc.ts` handles runtime, Preview, Reveal, catalog and
  Translation calls locally and rejects undeclared product commands.
- CDP observed exactly 18 local Lucide virtual-glyph modules. The 19 typed
  application icon names intentionally alias `download` and `update-skill` to
  the same compiled glyph.
- A fresh matrix run produced no browser warning or error.

## Production Vite

`npm run build` passed and emitted:

| Asset | Raw | Gzip |
| --- | ---: | ---: |
| CSS | 28.45 kB | 6.15 kB |
| JavaScript | 385.31 kB | 117.67 kB |

Compared with the pre-refactor baseline recorded by Child 1:

- CSS: +5.10 kB raw / +0.77 kB gzip
- JavaScript: +13.43 kB raw / +3.30 kB gzip

The production preview filled both 1180×800 and 720×520 exactly, loaded one
hashed JS asset, exposed no review scenario dataset and produced no console
warning/error. Direct navigation to `/review.html?scenario=...` served only the
production App fallback; no review harness or requested scenario was present.

## Three-platform evidence

The unchanged `.github/workflows/ci.yml` defines quality plus Windows NSIS,
macOS universal DMG and Linux AppImage package jobs. Public GitHub Actions run
`31475914431` completed successfully for all four jobs on commit
`a473f566ff2a73aa82db626a1c2c20f4e9ac1224`. The current branch has not been
pushed, so this is platform-pipeline evidence, not a claim that the current UI
commit already has remote package artifacts. Current shared UI evidence is the
local platform-neutral Vite build and the 624-frame WebView-size matrix.

Windows and Linux native application smoke remain unavailable on this macOS
host and must stay separate in `docs/release-smoke.md`.

## Native regression found by main-session smoke

A clean current macOS Tauri shell reached Ready with 53 real Skills and CLI
1.5.23. Selecting `ask-matt` loaded `SKILL.md` without mutation. Opening its
file tree exposed a regression that deterministic fixtures had not reproduced:
the Preview remained on `SKILL.md`, but keyboard focus reopened on the first
`PHASE-BOUNDARIES.md` row. The approved contract requires every open to reveal
and focus the current file, even when a different row was focused in an earlier
tree session.

The owning `openFileTree` seam now synchronously resets `treeFocusPath` to the
current `file.path`, or to the first visible treeitem only when the current path
is absent, before mounting/focusing the popover. Expansion still adds only the
current file's ancestors and preserves every unrelated folder toggle.

A focused regression test starts with `PHASE-BOUNDARIES.md` before `SKILL.md`,
previews `SKILL.md`, focuses the stale first row, closes and reopens the tree,
then proves `SKILL.md` is the sole `tabIndex=0` treeitem and owns DOM focus.

The first post-fix native Escape retest exposed a second event-order regression:
the tree handler closed the popover, then the same bubbling Escape reached the
window handler and returned to Inventory, clearing the selected Preview. The
tree Escape contract is to close only the transient surface, restore its trigger
and preserve the selected Skill/file; the event must stop before the global
back-navigation branch.

The local `moveTreeFocus` Escape branch now calls `stopPropagation()` alongside
its existing `preventDefault()`, close callback and synchronous trigger focus.
The focused regression test additionally proves Escape removes only
`.file-tree`, keeps the `demo` Skill heading and current `SKILL.md` Preview, and
restores `.path-button` focus before the reopen assertions run.

## macOS native result

The main-session post-fix smoke reached Ready in the current macOS Tauri app
with 53 real Skills and CLI 1.5.23. Selecting `ask-matt` loaded the current
`SKILL.md` Preview without mutation. After both integration fixes:

- Escape inside the file tree closed only the popover;
- focus returned to the file-tree trigger;
- the selected `ask-matt` Skill and `SKILL.md` Preview remained intact;
- reopening uses the current file as the sole roving tab stop, backed by the
  focused close/reopen regression test.

The current wide native shell and the archived Settings-child compact native
smoke together cover the approved 1180×800 and 720×520 WebView sizes with
system-provided macOS title-bar controls. The deterministic review entry remains
the safe Translation evidence: clicking native Translate would transmit real
local Skill content to Google, so that action was intentionally not triggered.

Windows and Linux native smoke remain unavailable and are explicitly deferred
to the platform-owned release checklist; they are not inferred from macOS or
the shared WebView matrix.

## Post-fix quality gate

- Focused `src/App.test.tsx`: passed, 23 tests.
- `npm run format:check`: passed.
- `npm run lint`: passed.
- `npm run typecheck`: passed.
- `npm test -- --run`: passed, 70 tests.
- `npm run build`: passed.
- `git diff --check`: passed.
- Final `dist/` contains only production HTML/CSS/JS (412 KiB on disk).
- Final scan found no Iconify API endpoint, review HTML, scenario identifier,
  fixture marker/name or canonical review payload.
- Independent Trellis full-scope review passed after the native fixes; it also
  synchronized the reusable tree-local Escape propagation contract into the
  frontend component guidelines.

The Tauri dev session received both source fixes through its Vite dev server and
was terminated after the successful native retest.

Remaining work:

1. Commit and archive this integration child, then complete the parent task.

The user approved the final integrated Pencil UI result on 2026-08-21.
