# Skill Deck UI/UX design exploration

> Planning artifact. The prototype is THROWAWAY and must not be promoted directly into product code.

## Decision status

### Round 1: Rejected

Round 1 used A/B/C layouts and recommended B. That recommendation is **Rejected**.

The failure was structural, not visual:

- A website-style top navigation treated Installed, Discover and Settings as pages.
- Large page headings and centered/max-width task surfaces wasted desktop window space.
- Card containers and page sections made the app resemble a B2B admin website.
- Discover and Settings became destinations instead of temporary utility surfaces.
- Runtime unavailable looked like a marketing empty state instead of a window state.
- The layouts did not establish a true desktop master/detail, navigation stack or unified window toolbar.

Do not use the Round 1 recommendation during implementation. Its only retained value is evidence of what to avoid.

## Round 2 Design Read

Reading this as: a local desktop developer utility for repeatedly scanning, previewing and maintaining global Installed Skills, using a compact unified toolbar, full-window content and native-feeling sheets/popovers.

- `DESIGN_VARIANCE: 2` - stable desktop placement and platform familiarity win.
- `MOTION_INTENSITY: 1` - only pressed, selection, sheet and progress feedback.
- `VISUAL_DENSITY: 8` - source lists and tables are compact; document reading remains comfortable.

The `design-taste-frontend` rules are not used as a page-layout method in Round 2. Only its accessibility, state completeness and existing-token audit remain applicable. Desktop structure follows window-level toolbar, split view, navigation stack, utility sheet, keyboard command and resize behavior.

## Round 3: ui-ux-pro-max audit and convergence

Round 3 uses the user-requested `ui-ux-pro-max` skill as a searchable audit source, not as an authority over the desktop product context. The required design-system query produced several high-confidence false matches, so every recommendation was filtered against the confirmed platform, task model and Round 2 desktop direction.

### Searches performed

```bash
python3 /Users/didi/.agents/skills/ui-ux-pro-max/scripts/search.py \
  "desktop developer utility skill manager compact master detail dense local tool" \
  --design-system --variance 2 --motion 1 --density 8 -p "Skill Deck"

python3 /Users/didi/.agents/skills/ui-ux-pro-max/scripts/search.py \
  "desktop keyboard focus dialog focus trap return focus inline error recovery disabled loading z-index reduced motion" \
  --domain ux -n 18

python3 /Users/didi/.agents/skills/ui-ux-pro-max/scripts/search.py \
  "desktop list stable keys focus management dialog state loading performance" \
  --stack react

python3 /Users/didi/.agents/skills/ui-ux-pro-max/scripts/search.py \
  "desktop developer tool restrained forest green cobalt blue warm copper light dark accessible" \
  --domain color -n 12

python3 /Users/didi/.agents/skills/ui-ux-pro-max/scripts/search.py \
  "desktop utility restrained monochrome single accent native dense no gradient" \
  --domain style -n 12

python3 /Users/didi/.agents/skills/ui-ux-pro-max/scripts/search.py \
  "desktop developer tool graphite azure slate teal refined stone forest light dark calm compact native" \
  --domain color -n 15

python3 /Users/didi/.agents/skills/ui-ux-pro-max/scripts/search.py \
  "desktop utility cold neutral restrained native dense no gradient no bento light dark surfaces" \
  --domain style -n 15

python3 /Users/didi/.agents/skills/ui-ux-pro-max/scripts/search.py \
  "desktop local developer utility skill manager compact native graphite blue light dark" \
  --design-system --variance 2 --motion 1 --density 8 -p "Skill Deck"

python3 /Users/didi/.agents/skills/ui-ux-pro-max/scripts/search.py \
  "desktop developer utility graphite cool neutral restrained blue azure light dark selected hover focus surfaces" \
  --domain color -n 15

python3 /Users/didi/.agents/skills/ui-ux-pro-max/scripts/search.py \
  "desktop master detail tree selection color highlight keyboard aria-selected focus contrast light dark" \
  --domain ux -n 15

python3 /Users/didi/.agents/skills/ui-ux-pro-max/scripts/search.py \
  "desktop dense list selection focus theme" --stack react
```

No `--persist` flag was used. This planning round does not create a parallel design-system package or new source of truth.

### Retrieval error

The comprehensive design-system result recommended:

- Horizontal Scroll Journey as the product pattern.
- Exaggerated Minimalism and oversized `3rem-12rem` typography.
- Immersive product discovery and floating sticky CTA.
- Dark navy plus bright run-green palette.
- Inter loaded from Google Fonts.
- GSAP Scroll Reveal.

These are marketing/landing matches despite a desktop-tool query. They are explicitly rejected. The mismatch is recorded so future agents do not treat database rank as product authority.

The later color/style retrieval was similarly noisy: it mixed AI-purple themes, neon developer palettes, gradients, mobile neumorphism, dashboard chart palettes, black-and-white editorial systems and landing-page styles into a desktop utility query. The usable signal was limited to a restrained single accent, semantic role tokens and rational contrast. Purple-as-AI shorthand, neon, gradients and multi-accent palettes are rejected.

The surface-system follow-up again returned code-dark + bright-run-green, violet neumorphism, bento cards, cinematic mobile blur, OLED neon and chart-dashboard matches. None determines G. The only adopted evidence is that developer tools benefit from clear neutral surface levels, a restrained single interaction hue and explicit destructive semantics. The actual Light/Dark systems below were calibrated as desktop window chrome, not copied from a database row.

### Adopt / reject matrix

| Database recommendation | Decision | Skill Deck application |
| --- | --- | --- |
| Visible focus indicator | Adopt | 2 px semantic focus ring on every keyboard control |
| Keyboard order matches visual order | Adopt | Toolbar, source list, detail, sheet controls follow DOM/visual order |
| Dialog focus management | Adopt | Focus first task control, trap Tab, Escape close, return to trigger |
| Inline errors with recovery | Adopt | Error next to failed search/refresh plus Retry; source install remains available |
| Disabled/loading clarity | Adopt | Native `disabled`, reduced emphasis, row/viewer skeletons and stable status text |
| Error announcements | Adopt | `role="alert"` for newly appearing runtime and refresh/search errors |
| Semantic color tokens | Adopt | Window/surface/text/muted/border/accent/danger/focus tokens only |
| 4.5:1 text contrast | Adopt | Body/control text targets WCAG AA on light surfaces |
| Reduced motion | Adopt | No decorative motion; media query removes optional transitions |
| Finite z-index scale | Adopt | Popover 10, sheet 20, prototype marker 30 |
| Stable React list keys | Adopt for implementation | Use CLI Skill identity/path, never array index, for dynamic Inventory/tree rows |
| Consistent outline icons, no emoji | Adopt | G uses text controls; production may reuse one existing icon family only |
| Horizontal Scroll Journey | Reject | No scroll storytelling in a desktop utility |
| Bento Grid / Landing structures | Reject | No marketing sections, feature cards or conversion CTA hierarchy |
| Exaggerated Minimalism | Reject | Conflicts with compact source-list/detail scanning |
| Huge typography / massive whitespace | Reject | Wastes adjustable desktop window space |
| GSAP / scroll reveal / stagger | Reject | No justified scroll animation and no new dependency |
| Light-only MVP | Superseded by user decision | G now ships Light and Dark visual modes, with Follow system as the default preference |
| Google Fonts / Inter import | Reject | Use platform system fonts; no network font or dependency |
| Mobile-first 375 px checklist | Reject | Product minimum is a resizable desktop window, not a phone viewport |
| 44/48 px touch targets | Reject mechanically | Mouse/keyboard desktop controls use 28-32 px visual height with clear focus |
| Safe areas, haptics, mobile keyboard | Reject | Mobile-native concerns do not apply to this Tauri desktop app |
| Mobile bottom navigation / gestures | Reject | Unified toolbar plus master/detail remains the correct hierarchy |
| Virtualize every list | Defer | Add only if measured Inventory size makes native scrolling insufficient |

### Round 3 candidate

Only **G** is a current candidate. Its window model is:

- Wide windows use compact Installed source list plus Preview detail.
- At 820 px and below, the same state becomes an actual list-to-Preview navigation stack with Back.
- The toolbar, status area, Add sheet, Settings sheet and Preview popover remain window-level desktop patterns.
- Selection uses a tinted row plus accent edge without visible `Selected` copy. The native control still exposes `aria-selected`, and keyboard focus has its own ring rather than reusing the selection treatment.
- Startup has no selected Skill. Wide detail shows a compact “Select a Skill” state with no file or translation actions; narrow startup remains on the list.
- Source list supports Up/Down focus movement and Enter confirmation. Arrow navigation alone does not create selection.
- Sheets demonstrate autofocus, Tab-loop focus trap, Escape close and focus return.
- The Preview path trigger already represents the Skill root, so the tree has no visible or focusable `/` row. Top-level `SKILL.md`, `references` and `preview.png` use `aria-level="1"`; the two `references` children use level 2. Phosphor folder/file icons plus indentation express type and hierarchy.
- Translation uses wide split and operable narrow Original/Translation tabs with correct ARIA state.
- A toolbar-only prototype-state selector exposes loading, runtime unavailable, partial outcome and refresh error without adding product navigation.
- Pressed feedback changes background only and never moves layout bounds.

Round 2 D/E/F remain historical reasoning below. They are not implementation alternatives after Round 3.

## Existing visual tokens worth preserving

- System sans for controls and body; system monospace for paths, sources and versions.
- Warm-neutral window chrome and near-white content.
- Forest green (`#315c47`) was the established accent baseline; the later complete surface-system exploration supersedes this exact value without changing information architecture.
- Rust only for destructive/error semantics.
- 6-8 px control radii, compact borders and visible semantic focus rings.
- The square green `S` mark, used once in window chrome.

Retire serif headings, oversized empty-state graphics, floating card stacks and non-semantic status dots.

## Desktop information architecture

Skill Deck has too few top-level areas to justify a permanent navigation sidebar or website navigation.

```text
Window
├── Unified toolbar
│   ├── leading: app identity or Back
│   ├── center: current title or filter/search
│   └── trailing: Update All, Add, Settings, overflow
├── Window content
│   ├── Installed source list/table
│   ├── Preview detail
│   └── Runtime unavailable replacement
└── Status area
    ├── readiness / command result
    └── session CLI version
```

Temporary utilities:

- **Add Skill** opens a sheet/secondary utility surface from the toolbar. It contains catalog search and direct source install.
- **Settings** opens a trailing sheet or small preferences window. It is not a navigation destination.
- **File tree** opens only from the Preview path control as a compact popover. It never becomes a persistent sidebar.
- **Remove / Update All confirmations** use short sheets or native dialogs.
- **Command diagnostics** expand near the status/result that produced them; they do not become a dashboard.

Durable content:

- Installed is the window's home state.
- Preview is detail for one Installed Skill, not a new top-level product area.
- Runtime unavailable replaces CLI-dependent window content while preserving toolbar/window identity.

## Window toolbar behavior

The toolbar is part of the window chrome and stays one compact row where space allows.

### Leading zone

- App identity in list/home state.
- Back to Skills when a single-stack Preview is pushed.
- In master/detail, no Back is needed because the source list remains visible.

### Center zone

- Local Installed filter on list/table states.
- Selected Skill title in a pushed Preview.
- G wide mode may use a simple `Installed Skills` title because its source list has a local list header.

### Trailing zone

Priority from highest to lowest:

1. Add Skill
2. Update All or selected Skill Update
3. Settings
4. Less frequent actions in overflow at narrow widths

Remove stays near the selected Skill or in a context menu; it should never receive primary toolbar emphasis.

Toolbar controls do not reflow into a website-style second navigation row. When width is constrained, low-priority labels move into overflow or become accessible icon controls.

## Status area behavior

Use a 22-24 px bottom status area because it carries real operational context:

- Ready, loading Inventory, running command, refresh failed or runtime unavailable.
- Count of global Installed Skills where useful.
- Exact `skills@<session-version>`.
- Neutral completion text such as `Command completed, Inventory refreshed`.
- Partial Agent results or diagnostics can expand upward from the status area.

Do not use transient toast-only feedback for CLI mutations. Users need a stable place to confirm what happened. The status area must not claim an update revision Inventory cannot prove.

## Primary workflows

### Launch

1. Render window chrome immediately.
2. Use row-shaped loading placeholders in the content area while checking Node/npx and resolving `skills@latest`.
3. On success, show Inventory and the exact session version in the status area.
4. On failure, replace content with compact runtime guidance and Retry. Preserve Settings access.

### Scan and Preview

1. Installed list/table is the initial focus target.
2. Arrow keys move the focus cursor in the source list; Enter confirms selection.
3. Enter or Preview opens/selects detail.
4. Preview starts with `SKILL.md` where present.
5. The path control opens the temporary file-tree popover.
6. File selection closes the popover and restores focus to the path control.

### Translate

1. Eligible Markdown/plain text shows Translate and persistent Google egress disclosure in the Preview toolbar.
2. Translation on splits only the detail viewer, original left and translation right.
3. Both panes scroll independently at wide widths.
4. Below the split threshold, the same detail viewer uses Original/Translation tabs.
5. The source list, window toolbar and status area do not split.
6. Ineligible files send nothing and show no active translation control.

### Add Skill

1. Toolbar Add or `Cmd/Ctrl+N` opens Add Skill sheet.
2. Search input receives focus.
3. Search results use compact rows with one Install action.
4. Direct source input remains in the same sheet even if catalog search fails.
5. Source copy states that every Skill found in the source may be installed.
6. After the command, close or keep the sheet based on outcome, refresh Inventory, select the observed result and announce status.

### Update and Remove

- Row/context Update affects one Skill.
- Update All belongs in the window toolbar and asks for confirmation.
- Remove is destructive, confirms whole global Skill removal across Agent Targets, then refreshes Inventory.
- Mutations preserve selection and scroll position when the selected Skill still exists.

### Settings

Toolbar Settings or `Cmd/Ctrl+,` opens a trailing sheet/preferences window with only:

- Translation target, with system language behavior marked `(Default)`.
- Agent targets, automatic detection `(Default)` or verified list.
- Install mode, automatic link/copy `(Default)` or Always copy.
- Read-only exact CLI session version.

## Menu commands and keyboard map

Production should expose the same commands through the platform application menu and keyboard shortcuts.

| Command | macOS | Windows/Linux | Scope |
| --- | --- | --- | --- |
| Filter Installed | `Cmd+F` | `Ctrl+F` | Focus local filter |
| Add Skill | `Cmd+N` | `Ctrl+N` | Open Add sheet |
| Settings | `Cmd+,` | `Ctrl+,` | Open Settings |
| Refresh Inventory | `Cmd+R` | `Ctrl+R` | Refresh without mutation |
| Preview selection | `Return` | `Enter` | Open selected detail |
| Close popover/sheet | `Esc` | `Esc` | Close top temporary surface |
| Back from pushed Preview | `Cmd+[` | `Alt+Left` | G narrow stack |
| Reveal selected file | `Cmd+Shift+R` | `Ctrl+Shift+R` | Preview only |

Do not assign a one-key shortcut to Update or Remove. Destructive and network/mutation commands should remain deliberate.

## Context menu

Right-clicking an Installed Skill may expose:

1. Preview
2. Update
3. Reveal Skill Folder
4. Remove

The context menu mirrors visible commands and is never the only route to an essential action. Per-Agent Remove is absent because it is outside MVP.

## Window resizing

Recommended minimum window size: approximately 720 x 520.

### G wide, 1000 px and above

- G uses a 240-280 px source list and flexible detail viewer.
- Translation uses two readable panes.
- Toolbar labels remain visible.

### G medium, 821-999 px

- G source list narrows to 180-220 px.
- Optional toolbar labels enter overflow.
- Translation can remain split until the defined 900 px viewer threshold, then switches to tabs.

### G narrow stack, 720-820 px

- Installed list and Preview occupy the window one at a time.
- Selecting a Skill pushes Preview; Back restores the list, selection and scroll position.
- The file tree stays inside Preview as a path popover.

Resize must preserve selection, chosen file, translation mode and scroll positions. Layout change must not retrigger translation unless the selected eligible file changed.

## State matrix

| State | Toolbar | Content | Status area |
| --- | --- | --- | --- |
| Runtime checking | Identity, actions unavailable | Row/viewer skeletons | `Checking Node and skills CLI` |
| Runtime unavailable | Identity, Settings, Retry if appropriate | Compact replacement guidance | Actual missing/incompatible condition |
| Inventory ready, no selection | Filter/title, Add, Update All, Settings | Source list plus compact “Select a Skill” detail state; no Preview actions | Ready, count, exact CLI version |
| Inventory empty | Add remains primary | Compact empty list instruction, no illustration | Ready, 0 global Skills |
| Inventory refresh failed | Mutations unavailable | Last safe session Inventory only if already loaded | Persistent failure plus Retry |
| Add sheet searching | Add sheet owns focus | Existing content remains behind dim layer | Search progress may stay in sheet |
| Search failed | Sheet remains open | Direct source remains usable | No global failure state |
| Mutation running | Conflicting mutations disabled | Selection and content remain stable | Command and target shown |
| Partial add/remove outcome | Controls re-enabled after refresh | Observed Agents/rows updated | Partial result and expandable diagnostics |
| Preview loading | Preview-specific actions unavailable | Current file may remain until replacement is ready | Reading selected file |
| Unsupported file | Reveal actions available | Type, size, unsupported reason | No error toast |
| Translation loading | Translate pressed, file navigation available | Original stable, translation pane skeleton | Provider work scoped to Preview |
| Translation failed | Retry translation available | Original stable, pane-local error | CLI status unaffected |

## Round 2 variants (historical)

### D. Compact master-detail

Structure:

- Unified window toolbar.
- 240-280 px Installed Skill source list on the left.
- Selected Skill Preview detail on the right.
- File tree remains a Preview path popover.
- Bottom operational status area.

Strengths:

- Most desktop-native scan/select/read loop.
- Installed context stays visible while browsing files.
- Makes strong use of wide desktop windows without adding top-level navigation.
- Translation splits only the detail region as required.

Risks:

- At narrow widths the source list competes with readable document width.
- Selection versus explicit Preview opening needs one consistent behavior.

### E. Navigation stack with unified toolbar

Structure:

- Full-window compact Installed table.
- Preview pushes onto the same content region.
- Toolbar leading control becomes Back; trailing commands update for context.
- Add and Settings remain sheets.

Strengths:

- Simplest state model and best narrow-window behavior.
- Full Preview width and clear task focus.
- No permanent sidebar or inspector.

Risks:

- Repeated scan-preview requires back navigation.
- Users lose list context while reading.

### F. Command utility with inspector

Structure:

- Unified command/search field in toolbar.
- Compact table fills the window.
- Selection inspector holds metadata and actions.
- Preview can replace the table while inspector keeps actions/context.

Strengths:

- Efficient for keyboard-heavy users and known Skill names/sources.
- Selection actions are stable and do not inflate every table row.
- Source metadata is visible without opening Preview.

Risks:

- Combined filter/find/source semantics can be ambiguous.
- Inspector consumes space for low-volume metadata.
- Adds command interpretation complexity before usage proves the need.

## Round 2 recommendation (refined by G)

Recommend **D, compact master-detail**, with **E as its narrow-window collapse**.

This is not a generic navigation sidebar. The left side is the source list for the selected Preview detail, which is a justified master/detail relationship. Add and Settings stay temporary sheets, and the window uses all available space. At widths where the master/detail relationship stops being readable, push Preview using E's Back behavior.

Do not implement F's combined command field in MVP. A plain local Installed filter plus explicit Add sheet is easier to understand and maps directly to existing CLI operations.

## Final candidate G

G is the only implementation candidate. Its invariant layout contract is:

```text
Wide window
[ unified toolbar ]
[ compact Installed source list | Preview detail                  ]
[ stable status area                                               ]

Narrow desktop window
[ unified toolbar + contextual Back ]
[ Installed list ] -> [ Preview detail ]
[ stable status area                    ]
```

The source list is content-specific master context, not a top-level navigation sidebar. Add and Settings remain temporary sheets. The file tree remains a Preview path popover. Translation never changes the master or window chrome.

### G interaction contract

- Source list: one tab stop per Skill row; Up/Down changes focus only; click or Enter establishes selection and opens Preview where appropriate.
- Wide selection: selecting a row updates detail without hiding the list.
- Narrow selection: selecting a row pushes Preview; Back restores the list and its selected row.
- Sheet open: record trigger, render dialog, focus the first task control.
- Sheet Tab/Shift+Tab: loop within the current dialog.
- Sheet Escape/Close: dismiss and return focus to the recorded trigger.
- File tree: open on current file, Up/Down moves, Enter selects, Escape returns focus to the path trigger.
- Translation tabs: roving selected state is reflected by `aria-selected` and `tabindex`; only the chosen pane is visible below 900 px.
- Mutation/error feedback: stable status area plus contextual inline recovery, never color alone or toast alone.

### G tokens and density

- 4/8 px base spacing, with 12/16/24 px only as composed multiples.
- Mouse/keyboard control height: 28-32 px, not mobile 44/48 px.
- Semantic color roles: window, chrome, surface, subtle surface, text, muted, border, accent, danger and focus.
- Z-index: popover 10, sheet 20, prototype-only marker 30.
- System sans and system monospace only.
- Two actual visual modes, Light and Dark; the former light-only MVP decision is superseded by the user's explicit theme decision.
- Pressed states change fill/contrast, not transform or dimensions.

### G theme contract

Appearance is one theme preference, not separate mode and palette settings. Five options are the confirmed upper bound: `system` (Default), `light`, `dark`, `sand` and `plum`. A compact horizontal tile combines three representative token swatches, a readable name and native checked state; it remains a keyboard-operable radio group. The System tile always shows three explicit Light/Dark split pairs (chrome, content, interaction), independent of the currently resolved OS mode, so it cannot be mistaken for fixed Dark. Do not expand this into a theme editor or expose internal token-family names.

`system` resolves to Graphite Light or Graphite Dark and responds immediately to `prefers-color-scheme` changes. `light` and `dark` pin those same Graphite/Azure maps. `sand` is a distinctly warm fixed **light** theme with amber/rust interaction tokens. `plum` is a deep aubergine fixed **dark** theme with lavender interaction tokens. Manual choices ignore later system changes until System is selected again. Production persists this single preference; the throwaway prototype stores it only in the URL.

Applying a theme is a visual update, not a navigation or data event. It must preserve selected Skill, list/detail view, current file, translation enabled state and pane, sheet state, document/list scroll positions, and cached command/translation results. It must not refresh Inventory or retranslate content.

The earlier Forest/Cobalt/Copper accent-only comparison is **superseded**. Copper is removed. Changing only an accent over the same yellow-grey neutral base did not answer the visual feedback. The current exploration changes the entire surface system while preserving G's information architecture and theme contract.

The later Teal Dark and Forest Light options are also superseded by Sand and Plum because they remained too close to the default Graphite temperature. They are not implementation options.

### Complete theme maps

Each row lists `window / chrome / content surface / subtle / border / text / muted / selected / focus / accent / danger / warning / code / scrim`. Danger and warning remain independent semantics, never theme derivatives.

| Visible option | Resolution | Tokens |
| --- | --- | --- |
| System (Default) | Live Graphite Light or Graphite Dark | Uses the next two maps and follows the OS |
| Light | Fixed Graphite + restrained Azure Light | `#f5f6f7 / #edeff1 / #fcfcfd / #f4f5f6 / #aeb5bd / #23282e / #606a74 / #e2eefb / #1f6fb2 / #2d669d / #9a3f46 / #7a581f / #f0f2f4 / 34% graphite` |
| Dark | Fixed Graphite + restrained Azure Dark | `#1a1c1f / #222529 / #1e2125 / #25292e / #69717a / #eef0f2 / #adb4bc / #243b53 / #86c1ef / #78a8d2 / #f09a9e / #e3b761 / #16181b / 60% black` |
| Sand | Fixed warm Sand Light | `#f5f0e7 / #ebe1d2 / #fffaf2 / #f4eadc / #b9aa98 / #322a22 / #6f6255 / #f4dfbf / #a85e2f / #9a552c / #973f4d / #76531b / #f2e8da / 35% brown` |
| Plum | Fixed deep Aubergine Dark | `#211820 / #2b1f2a / #261c25 / #30232f / #766576 / #f2edf2 / #c0b2bf / #493653 / #d8b4e5 / #c6a0d4 / #f1a0a5 / #e3bd72 / #1c151c / 62% black` |

Detail roles use the same small semantic set rather than component hex values:

| Resolved map | Selected fill | Tag surface / border | Control surface / border | Code surface / border |
| --- | --- | --- | --- | --- |
| Light | `#e2eefb` | `#f4f5f6 / #c8d0d8` | `#fafbfc / #aeb5bd` | `#f0f2f4 / #d2d8df` |
| Dark | `#243b53` | `#292d31 / #46515c` | `#292d32 / #69717a` | `#16181b / #39434d` |
| Sand | `#f4dfbf` | `#f6ecdf / #d1b995` | `#fcf6ed / #b9aa98` | `#f2e8da / #d8c2a4` |
| Plum | `#493653` | `#322631 / #67516b` | `#342833 / #766576` | `#1c151c / #59435a` |

`selected` is shared by Installed rows and current tree items. `tag` styles Agent labels. `control` plus the main border token styles buttons and the Preview path trigger. `code` styles Markdown code blocks. This keeps each theme coherent without adding per-component color APIs.

Graphite + Azure feels like a crisp native developer utility: the final Light map moves window, chrome and content toward neutral whites instead of blue-grey, and uses soft separators for window structure while retaining stronger borders on controls and popovers. Azure is concentrated in selection, focus and primary actions. Dark mode uses neutral graphite layers rather than blue-black surfaces; only interaction states carry blue. Its main risk is generic platform-tool character, not semantic collision.

Sand is intentionally warmer than Graphite Light: cream content, tan chrome and rust interaction color make the tile and window immediately distinguishable. Rust sits nearer warning/danger than Azure, so danger remains cranberry, warning remains gold-brown, and both keep labels and borders.

Plum is intentionally darker and more chromatic than Graphite Dark: aubergine surfaces and lavender focus/primary tokens create a clear alternative without neon or gradient effects. Lavender remains separate from coral danger and gold warning.

Recommendation: **System (Default)** still resolves to Graphite Light/Dark with restrained Azure interaction tokens. Sand and Plum are deliberately different alternatives, not near-duplicate palette nudges. Five visible choices remain the MVP maximum.

Across all four unique maps, tested text-role pairs remain above WCAG AA. The overall text-role minimum is 5.32:1; the lowest focus/window pair is Sand at 4.30:1. Graphite body text remains 14.48:1 in Light and 14.15:1 in Dark. Icons inherit the same foreground roles and receive equivalent hover, focus, disabled and selected treatment in every theme.

### Icon system

Use Iconify as the catalog and data source, not as a runtime service. G uses only the Iconify `ph` collection: Phosphor Regular 2.1.1. Iconify collection metadata currently records the collection as MIT and links to the Phosphor core license; implementation must retain the applicable attribution/license record and re-check it when icon data is updated.

Reference points: [Iconify](https://iconify.design/), [Iconify React component behavior](https://iconify.design/docs/icon-components/react/), [Phosphor collection browser](https://icon-sets.iconify.design/ph/), and [Phosphor MIT license](https://github.com/phosphor-icons/core/blob/main/LICENSE).

The prototype vendors the verified Iconify bodies for `plus`, `arrows-clockwise`, `gear`, `arrow-left`, `translate`, `folder-open`, `trash`, `magnifying-glass`, `file` and `dots-three`. SVGs use `viewBox="0 0 256 256"`, `currentColor` and the collection's Regular weight. No CDN, API request, runtime loader, emoji or mixed collection is present.

Production integration options:

| Option | Trade-off | Decision |
| --- | --- | --- |
| `@iconify/react` with imported local icon-data objects | Can remain offline when every icon object is statically imported, but adds a rendering dependency and string names can accidentally fall back to runtime API fetching | Do not use for this small fixed set |
| Vendored generated SVG components/data | Smallest auditable bundle, no runtime fetch path, tree-shaking or registry required | **Recommend**: vendor only the selected SVG bodies/components |

Production rules:

- Bundle only actually used icons; do not ship the collection or build an icon registry.
- Never pass string icon names to a runtime Iconify component and never contact a third-party icon API from the desktop app.
- Use one Phosphor Regular collection/weight, `currentColor`, 16 px default glyphs and 18 px only where document hierarchy requires it.
- Keep the desktop hit target 28-32 px even when the visible glyph is 16 px.
- Primary and destructive actions use icon + label. `Remove` remains visibly destructive and never collapses to an unlabeled trash glyph.
- Familiar secondary actions may be icon-only only with an accessible name and native `title` tooltip. Focus, hover and disabled states apply to the whole button, not the SVG path.
- File-tree rows reuse local `folder-open` and `file` SVG data. Icons are `aria-hidden`; each tree item exposes its full relative path as its accessible name and keeps explicit `aria-level`.
- Theme changes recolor icons through semantic foreground tokens without replacing SVGs or resetting selection, view, translation or scroll.

## Prototype

Open [`prototype/index.html`](./prototype/index.html) directly or serve its directory with a static server.

- `?variant=G` - the only current candidate and the default
- Optional `&view=list|preview`
- Optional `&state=ready|loading|runtime|partial|error`
- Optional `&sheet=add|settings`
- Optional `&theme=system|light|dark|sand|plum`; this is the single Appearance preference, and `system` is the default.
- No `palette` setting or URL parameter exists. Azure remains an internal interaction-token family for Light/Dark, not a user-facing choice.
- `Cmd/Ctrl+F`, `Cmd/Ctrl+N`, `Cmd/Ctrl+,` and Escape demonstrate representative desktop commands.
- Preview path opens the temporary file-tree popover.
- The tree begins directly with `SKILL.md`, `references` and `preview.png`; it has no selectable root row. Local folder/file icons and indentation express type and level, while complete paths such as `references/` remain in `data-path` and accessible names.
- Translate toggles wide split view; at 900 px and below, operable Original/Translation tabs choose one pane.
- At 820 px and below, the source list and Preview are mutually exclusive stack states rather than a squeezed split.

All data is sample content. Buttons never invoke a real command.

## Final G pre-flight

- No website navigation, centered max-width page, hero, large page heading or card dashboard.
- G is a full-window desktop skeleton with compact controls and system fonts.
- Startup has no selected Skill: wide mode shows a compact detail empty state, narrow mode stays on Inventory, and all row `aria-selected` values are false.
- Add and Settings are sheets, not navigation destinations.
- Runtime unavailable replaces window content.
- Wide master/detail and narrow navigation-stack behavior are one responsive G contract.
- File tree is temporary and only enters through the Preview toolbar path.
- File-tree rows use local Phosphor icons, indentation and `aria-level`; Installed selection uses row tint plus accent edge without visible `Selected` copy.
- Translation splits only Preview detail and has narrow-window tabs.
- Installed is global scope; Update, Remove, search/source install, Node/npx and all required Settings are represented.
- Toolbar priority, status behavior, context menu, application-menu commands and keyboard shortcuts are documented.
- G wide, medium and narrow-stack resize behavior is explicit.
- Light/Dark have semantic parity across window, chrome, surface, border, focus, selected, danger, warning, code, Markdown, sheet, scrim and skeleton roles.
- Theme changes preserve task state and scroll; Follow system reacts to system changes without a data refresh.
- Appearance is one five-option horizontal theme selector: System, Light, Dark, Sand Light and Plum Dark. It is not a theme editor and has no separate palette control.
- One local Phosphor Regular icon set covers Add, Update, Settings, Back, Translate, Reveal, Remove, Search, file/path and overflow; the DOM has no Iconify API/CDN dependency.
- Every icon-only button has an accessible name and tooltip; destructive actions retain visible text.
- Radius and focus treatment remain consistent.
- No new framework, dependency, product route, persistence or backend was added.

## Desktop reference basis

Round 2 internalizes these official desktop interaction principles without copying platform-specific materials:

- [Apple: Designing for macOS](https://developer.apple.com/design/human-interface-guidelines/designing-for-macos/)
- [Apple: Toolbars](https://developer.apple.com/design/human-interface-guidelines/toolbars)
- [Apple: Sidebars](https://developer.apple.com/design/human-interface-guidelines/sidebars)
- [Microsoft: Command bar](https://learn.microsoft.com/windows/apps/design/controls/command-bar)
- [Microsoft: Navigation view](https://learn.microsoft.com/windows/apps/design/controls/navigationview)

The relevant conclusion for Skill Deck is: use a unified toolbar and master/detail for the repeated task; avoid heavyweight top-level navigation because the app has very few peer destinations.

## Verification

Static and DOM interaction checks completed against the self-contained prototype:

```text
G theme-detail syntax passed 47772 bytes
G theme-detail jsdom passed: stable split System preview, 4 distinct maps,
selected/tag/control/code consumers, tree keyboard/icons, offline SVGs
G theme-detail contrast passed: text 5.32:1 minimum (Light danger);
focus 4.30:1 minimum (Sand)
```

Checks cover JavaScript parsing; System preview stability under simulated OS Light/Dark; visual distinction from fixed Dark; four unique token signatures; and the selected/tag/control/code CSS consumers. Rootless tree keyboard/icon behavior and offline SVG constraints remain verified. Contrast checks cover body, muted, selected, on-accent, danger, warning, code and focus pairs across all four resolved maps.

The available in-app browser could not reach the sandboxed localhost server in Round 1, and browser policy blocks `file://` navigation. That environment limitation still prevents honest screenshot sign-off. Before implementation, manually open G at approximately 1280 x 800, 900 x 650 and 720 x 520, including sheet focus, toolbar overflow and translation tabs.

## Known prototype limits

- Sheets approximate native desktop sheets using HTML/CSS; production should use the app's existing dialog/window primitives.
- The file tree demonstrates Up/Down, Enter and Escape, but does not simulate directory expand/collapse or type-ahead.
- Menu-bar integration, context menu, command overflow and confirmation sheets are specified but not fully simulated.
- Runtime, mutation and translation data are static representative states.
