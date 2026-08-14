# Implementation plan

## Ordered checklist

### Theme-first restructuring pass — 2026-08-14

- [x] Redesign File Trigger/File Row masters into a hierarchical File Tree
      family and propagate it to every theme, file-popover pattern, flow and
      preserved composition.
- [x] Correct File Tree semantics and presentation: chevrons only on folders,
      basenames plus indentation for children, header-first order, consistent
      file-only counts, compact single-line rows, no duplicates, focus follows
      selection in final compositions, and clearly separated documentation
      groups for Anatomy/States/Indentation/Long-name/Final Composition.
- [x] Repurpose the existing active Tab master as a static Translation Column
      Header and remove Original/Translation switcher styling from all masters,
      theme boards, patterns and preserved screens.
- [x] Remove repeated owner/repository and installed-path metadata from every
      page-level document identity header; retain only the Skill title and
      compact current-file trigger.
- [x] Remove the separate full-width Original/Translation comparison header
      from masters, theme patterns/flows and preserved screens; retain only the
      two panes, divider and local pane labels where needed.
- [x] Replace the filename-and-chevron File Tree trigger with a compact
      icon-only button in masters, all themes, flows, responsive proofs and
      preserved screens.
- [x] Remove installed root paths and persistent selected-file path footers from
      all File Tree compositions; keep Skill name, file count and hierarchy,
      with long-path disclosure documented as a tooltip-only behavior.
- [x] Reposition every page-level File Tree trigger immediately before the Skill
      title as one left-aligned identity/navigation group, and propagate the
      composition across themes, flows, responsive proofs and preserved screens.

- [x] Correct reference classification: move `Zraur · Wide Light Preview`
      from Dark to Light Reference Compositions and verify every remaining
      Dark reference resolves to Dark.

- [x] Define the final root hierarchy: Library Index, Shared Masters, System,
      Light, Dark, Sand and Plum theme collections.
- [x] Expand Pencil theme variables to explicit System/Light/Dark/Sand/Plum
      values using production theme contracts.
- [x] Build the Library Index and Shared Masters boards.
- [x] Build a complete Primitive → Semantic → Component token board for each
      theme; System must show both resolution branches.
- [x] Build consistent Page Flow, Components, Icons, Patterns & States and
      Responsive Composition sections inside every theme collection.
- [x] Reparent the existing foundations, flow and fourteen full-page screens;
      keep the screens only as the Dark Reference Compositions appendix.
- [x] Verify no former full-page state remains as an unrelated root and no
      user-reviewed work was deleted.
- [x] Run structural and screenshot QA for each theme collection and repair
      overflow, contrast, naming, instance and ordering issues directly.

### Initial full-page validation pass — complete

- [x] Load the approved reference evidence and Pencil schema/guidelines.
- [x] Define semantic variables for dark/light surfaces, content, actions,
      feedback, spacing, typography, and component metrics.
- [x] Build and validate the foundations/components board.
- [x] Build and validate the flow-map board.
- [x] Build the shared 1180×800 dark application shell and main Preview screen.
- [x] Derive and validate the remaining nine wide-dark state screens in small
      batches: file popover/translation; Settings/install/remove; loading/
      runtime/empty/Preview error.
- [x] Create and validate the wide-light Preview proof.
- [x] Create and validate the three 720×520 compact-desktop proofs.
- [x] Run a full structural inspection for root hierarchy, placeholder flags,
      bounds, clipping, warnings, and reusable component usage.
- [x] Capture final representative screenshots and perform visual QA against
      the PRD checklist; patch issues directly.
- [x] Confirm that only the target design and Trellis artifacts changed.

## Validation tools

- Pencil `Get`/bounds inspection through MCP for structural checks.
- Pencil screenshots for visual fidelity checks at section-level nodes.
- `git status --short` to verify production code remains untouched.
- Trellis check phase for final spec/acceptance review.

## Risk and rollback points

- Batch screens so a Pencil execute error rolls back only one logical section.
- Keep each new root screen as a placeholder until its subtree is complete,
  then clear the placeholder immediately.
- Validate the first wide shell before copying it; otherwise structural errors
  would multiply across every state.
- Validate component descendant IDs before relying on instance overrides.
- If a repeated structure is too stateful for reliable instance overrides,
  keep the component as the documented source of truth and use systematic
  screen-local structure rather than unstable deep overrides.

## Check results — 2026-08-13

- Fixed four Pencil layout warnings: the foundation theme tile overflow and
  the fifth Inventory row in each of the three compact screens.
- Added reusable `Dialog Chrome/Header` and `Dialog Chrome/Footer` components;
  the wide and compact Settings screens now consume both through instances.
- Revalidated all 1,218 nodes: no unnamed nodes, placeholders, zero-size
  bounds, missing path view boxes, or clipped/overflowing descendants remain.
- Verified 16 root frames: foundations, flow map, ten 1180×800 dark scenes,
  one 1180×800 light proof, and three 720×520 dark proofs. Every root is clipped
  and uses the intended theme.
- Reviewed exported images for all 16 roots, including popover anchoring,
  scrims/dialog bounds, state recovery actions, light-theme mapping, and the
  persistent two-pane compact layout.
- `npm run lint`, `npm run typecheck`, and `npm test -- --run` pass (34 tests).
- Git shows no tracked production source changes; only the untracked task
  artifacts and `pen-design/` directory are present.

## Theme-first restructuring results — 2026-08-14

- Reorganized the live Pencil document into exactly seven ordered primary
  roots: Library Index, Shared Masters, System, Light, Dark, Sand and Plum.
- Expanded semantic Pencil variables to production-backed Light, Dark, Sand,
  Plum, system-light and system-dark values sourced from `src/styles.css`.
- Added the same ordered Tokens, Page Flow, Components, Icons, Patterns &
  States and Responsive Composition sections to every theme collection.
- Preserved the original foundations and flow sources under Shared Masters;
  thirteen dark-resolved screens remain under Dark Reference Compositions and
  `Zraur` is preserved under Light Reference Compositions.
- Repaired ten clipped component-navigation specimens by increasing the five
  themed component sections from 250 to 290 px.
- Final Pencil scan reports zero unnamed nodes, placeholders, zero-size bounds,
  clipped descendants or broken component references; 332 instances resolve
  to 16 reusable masters.

## Theme-first check results — 2026-08-14

- Confirmed the document has exactly seven ordered roots: Library Index,
  Shared Masters, System, Light, Dark, Sand and Plum. Every theme collection
  contains Tokens, Page Flow, Components, Icons, Patterns & States and
  Responsive Composition in the required order.
- Fixed the scrim token to match production inheritance: Light/Sand use the
  42% light overlay and Dark/Plum use the 58% dark overlay. Added production
  graphite, code-border, danger-soft, control-state, spacing, radius, motion,
  layout, icon and elevation contract variables without changing source CSS.
- Expanded each concrete theme's visible primitive palette with the inherited
  `graphite-500` and `graphite-700` values, and added visible typography,
  overlay, code-border and elevation details to the three-layer token chain.
- Repaired copied component metadata and visible lane labels so Light, Dark,
  Sand and Plum show theme-specific core/state specimens rather than stale
  System or Light/Dark resolution labels. Added destructive and disabled
  state specimens to the second lane.
- Split combined state cards and added the missing Remove pattern. Every theme
  now shows nine isolated patterns: popover, translation, Settings, install,
  remove, loading, empty, runtime failure and Preview failure.
- Verified all fourteen original full-page compositions remain unchanged and
  are classified by resolved theme: one Light proof under Light Reference
  Compositions and thirteen dark-resolved screens under Dark Reference
  Compositions; none remain at document root.
- Final Pencil scan: 2,453 nodes, seven roots, 16 reusable masters and 332
  resolving instances; zero unnamed nodes, placeholders, zero-size bounds,
  clipped descendants, broken refs or misleading cross-theme names.
- Captured and reviewed screenshots for the Index, Shared Masters, all five
  theme boards, all five token sections, themed component/state sections,
  Page Flow, Icons, Responsive Composition and the Dark reference appendix.
- `npm run lint`, `npm run typecheck` and `npm test -- --run` pass (34 tests).
  Git reports no tracked production-file modifications.

## Reference-classification correction — 2026-08-14

- Moved preserved node `Zraur` without replacement or content changes from the
  Dark appendix into `20 Theme Light / 07 Reference Compositions`.
- Updated visible appendix evidence to `1` Light reference and `13` Dark
  references; compact Dark proofs were repositioned to close the vacated gap.
- Audited every direct child of both reference canvases: the Light canvas has
  one explicit `mode: light` child and the Dark canvas has thirteen explicit
  `mode: dark` children. No foreign-theme composition remains.
- Revalidated seven ordered roots, 2,453 named nodes, 332 resolving instances,
  16 reusable masters, clipping, placeholders and bounds with no findings.
- Reviewed updated screenshots for the Light board, Light appendix, Dark board
  and Dark appendix. Production source remains untouched.

### Independent classification check

- Confirmed `Zraur` occurs exactly once and its ancestry is
  `20 Theme Light / 07 Reference Compositions / Light Reference Composition
  Canvas`; it has explicit `mode: light`.
- Confirmed the Light canvas has exactly one light child and the Dark canvas
  has exactly thirteen dark children. No Light composition descends from the
  Dark collection.
- Confirmed visible appendix labels report `1` Light and `13` Dark references.
  The three compact Dark proofs occupy x=0/760/1520 with consistent 40 px
  gaps, closing the former Light-proof slot.
- Confirmed System is the only theme collection containing two resolution
  modes (`system-light` and `system-dark`); all concrete theme collections are
  single-theme.
- Independent global scan: 2,453 named nodes, seven clipped roots, 16 reusable
  masters and 332 resolving instances; zero placeholders, zero-size bounds,
  clipping problems, missing path view boxes or broken refs.

## File-tree and translation-header results — 2026-08-14

- Preserved and redesigned `LTKTz` as `File Tree/Trigger`, `pkaw8` as
  `File Tree Row/Default`, and `Rz8m0` as `File Tree Row/Selected`. Added
  reusable Hover (`KotDs`), Focus (`FFRtS`), Folder (`lBllu`), Root Header
  (`V5I4D`) and Popover (`BIna5`) masters.
- The reusable popover now documents Skill identity and file count, expanded
  and collapsed folders, disclosure and file/folder icons, 16 px nested
  indentation, selected/hover/focus states, a truncated long filename and its
  full path. Selection uses an accent wash/rail while keyboard focus uses an
  independent focus ring.
- Repurposed `q7hO4` in place as `Translation Column Header`. Detached the
  three legitimate Settings/install section-navigation instances first, then
  replaced every Original/Translation switcher with static document headers.
  The translation reference retains both persistent columns separated by a
  vertical divider and has no active underline or segmented-control styling.
- Propagated the new contracts through Shared Masters; all System, Light,
  Dark, Sand and Plum Components, Icons, Patterns & States and Page Flow
  sections; both preserved file-popover compositions; and all preserved
  document-header compositions.
- Screenshot review covered Shared Masters (`aBICA` and `lm6dI`), all five
  themed Components and Patterns & States sections, the wide and compact Dark
  file-popover references (`n5X3t`, `fn2aQ`), and the Dark translation
  reference (`z6HOSV`). The only visual repair was shortening themed compact
  focus labels; Shared Masters was expanded to eliminate bottom clipping.
- Final Pencil scan: 2,545 named nodes, seven ordered/clipped roots, 21
  reusable masters and 393 resolving instances, including 76 File Tree and
  30 Translation Column Header instances. Zero broken refs, placeholders,
  unnamed nodes, zero-size bounds, clipped descendants, Pencil warnings,
  stale `Tab`/`Active` names, or visible `tab` labels remain.

### Independent File Tree and translation-header check

- Inspected all nine File Tree masters in resolved form. The hierarchy is
  Trigger → Popover → Root Header → rows; the root visibly carries Skill name,
  file count and path. Row specimens cover default, hover, focus, selected,
  expanded and collapsed behavior with Lucide disclosure/file/folder icons.
- Verified the 16 px hierarchy increments in the reusable popover: base rows
  use 8 px left padding, the focused child uses 24 px and the nested file uses
  40 px. Selected state uses an accent wash and 3 px rail; focus instead uses
  a 2 px `$focus` ring. The long filename is bounded and accompanied by its
  full path annotation.
- Confirmed symmetric propagation through System, Light, Dark, Sand and Plum:
  every Components section has six File Tree row/trigger instances, every
  Icons section shows the file/folder/disclosure set, every Page Flow names
  the File Tree route, and every Patterns & States section contains the
  hierarchical tree and paired static translation headers.
- Confirmed `q7hO4` is a non-interactive `Translation Column Header`. Seven
  paired-header containers (Shared Masters, all five theme patterns and the
  preserved Dark translation screen) each contain two header instances plus
  one 1 px vertical divider. No Tab/Active/Underline/Segmented/Switcher names
  or visible `tab` labels remain; the nine retained selected-section nodes are
  legitimate Settings/Install navigation selectors and do not reference
  `q7hO4`.
- Re-reviewed Shared Masters, all ten themed Components/Patterns sections,
  wide and compact Dark file popovers, and the Dark translation reference.
  Popovers remain visibly trigger-anchored and bounded; the translation screen
  keeps both columns visible without a selected-tab treatment.
- Independent structural scan again reports 2,545 named nodes, 21 reusable
  masters and 393 resolving refs (76 File Tree, 30 translation-header), with
  zero broken refs, placeholders, unnamed nodes, zero-size bounds, clipped
  direct children, stale interaction names or visible `tab` labels. No Pencil
  changes were required by this verification pass.

## File Tree semantic correction results — 2026-08-14

- Corrected all four reusable file-row variants so they reserve a blank leading
  alignment slot and never render a disclosure icon: Default `pkaw8`, Hover
  `KotDs`, Focus `FFRtS` and Selected `Rz8m0`. Preserved `lBllu` as the distinct
  Folder Expanded master and added Folder Collapsed master `uh5Wt`; only those
  two folder variants contain chevrons.
- Rebuilt reusable final composition `BIna5` to 320×272 with compact 28 px
  rows. Its exact order is Root Header, selected+focused `SKILL.md`,
  `README.md`, expanded `references`, one depth-1 ellipsized basename,
  collapsed `scripts`, `icon.png`, then the separate full-path footer. The
  header reports `5 files`; a documentation note records four visible files
  plus hidden `scripts/install.sh`, with folders excluded.
- Reorganized Shared Masters File Tree documentation into five visible areas:
  Anatomy, Row States, Indentation, Long-name Behavior and Final Composition.
  The indentation specimen records the exact 8 px base and 24 px depth-1
  insets; the Row States matrix isolates default, hover, focus, selected and
  both folder states rather than presenting them as a real tree.
- Propagated Folder Expanded/Collapsed variants to every theme Components
  board. Replaced every themed file-popover pattern with a readable instance
  of the corrected final composition, updated all five Page Flow summaries,
  and rebuilt preserved wide `YYTJk` and compact `kwd7I` popovers in place.
- Screenshot review covered the full Shared Masters File Tree section
  `lm6dI`, reusable final composition `BIna5`, System/Light/Dark/Sand/Plum
  final-composition cards, both preserved full screens and their anchored
  popovers. A transient blank Light screenshot was recaptured successfully;
  no design repair was required.
- Final semantic scan: 2,579 named nodes, 22 reusable masters and 398 resolving
  refs. File masters contain zero chevrons; final compositions contain one
  `README.md`, no slash-delimited row label, one 16 px-indented long-name row,
  the correct five-file count and no wrapped long filename. Structural checks
  report zero broken refs, placeholders, unnamed nodes, warnings or clipped
  direct children.

### Independent semantic-correction check

- Resolved and audited `BIna5`, its System/Light/Dark/Sand/Plum instances, and
  preserved wide/compact popovers `YYTJk` and `kwd7I`. Every real composition
  has the root header first, followed by six 28 px rows in the same order:
  selected+focused `SKILL.md`, `README.md`, expanded `references`, its depth-1
  long basename, collapsed `scripts`, and `icon.png`, then the path footer.
- Confirmed all four reusable file-row masters contain a 14 px alignment spacer
  and no disclosure icon. The two folder masters alone contain one chevron each
  (`chevron-down` expanded, `chevron-right` collapsed). Real-tree file rows
  contain no chevron, slash-delimited label, duplicate `README.md`, or second
  current/focused row.
- Verified every final tree reports `5 files`: four visible file rows plus the
  documented hidden `scripts/install.sh`; `references` and `scripts` are not
  counted. The long basename is a 15 px-high single-line fixed box with an
  explicit ellipsis, while the complete path appears only in the footer/detail.
- Confirmed Shared Masters visibly separates the five requested documentation
  areas—Anatomy, Row States, Indentation, Long-name Behavior, and Final
  Composition—so synthetic state rows do not read as children of the real tree.
- Cross-checked all five Page Flow sections (`header → 5 files`), Components
  sections (one instance of every file/folder state), and Patterns & States
  sections (one corrected final-composition instance each).
- Screenshot review covered Shared Masters, the reusable final composition,
  all five themed final compositions, and the complete wide/compact preserved
  screens. The Light specimen was recaptured after one transient blank render;
  the successful image matches the other resolved compositions. Popovers are
  trigger-anchored, bounded and visually unclipped in full-screen context.
- Independent global scan: 2,579 named nodes, 22 reusable masters and 398
  resolving refs, with zero broken refs, placeholders, unnamed nodes, zero-size
  bounds or clipped direct children. No Pencil changes were required.

## Document identity and translation-pane simplification — 2026-08-14

- Simplified all eight wide page-level document identities and all three
  compact variants to the Skill title plus the existing `LTKTz` current-file
  trigger. Removed the repeated owner/repository breadcrumb, provenance
  separator and installed filesystem path; compacted wide headers from 76 to
  50 px and compact headers from 94 to 80 px so the removed metadata leaves no
  dead row or accidental whitespace.
- Removed the obsolete reusable comparison-header master `q7hO4`, all 30 of
  its resolved instances and eleven full-width comparison-header containers.
  Shared Masters and all System/Light/Dark/Sand/Plum pattern specimens now use
  two persistent panes, a 1 px central divider and small labels local to each
  pane. All five Page Flow summaries were updated; Responsive Composition
  sections contain no global comparison header.
- Preserved the corrected File Tree family unchanged, including reusable Root
  Header `V5I4D`, final composition `BIna5`, and the root-header-first wide and
  compact popovers. The File Tree root still displays `ask-matt`, the full
  `~/.agents/skills/ask-matt` path and the correct `5 files` count.
- Screenshot QA covered Shared Masters `lm6dI`, wide preview `paqAO`, wide
  translation reference `z6HOSV`, compact preview `gsCQ8`, compact file-popover
  screen `fn2aQ`, Responsive Composition `Kvhyr`, and the System/Light/Dark/
  Sand/Plum translation specimens `a1UmMl`, `J4MLb6`, `rBAzP`, `b1aKGb` and
  `P5efFg`. The only repair found was compacting the themed local pane labels;
  a follow-up scan also corrected historical narrow detail/icon-card clipping.
- Final Pencil scan reports seven ordered roots, 2,528 named nodes, 21 reusable
  masters and 368 resolving refs. There are zero broken refs, placeholders,
  unnamed or zero-size nodes, clipped descendants, stale comparison-header or
  `Tab/Active` names, and zero references to `q7hO4`. The only remaining
  `Skill Source` labels belong to inventory-row masters, not page identities.
- Repository checks pass: `npm run lint` and `npm run typecheck` both exit 0.

### Independent full-scope Phase 2.2 check

- Read the latest PRD/design/implementation context and both frontend/backend
  Quality Check layers. Independently resolved eleven page-level document
  identity groups across eight wide and three compact variants. Every identity
  contains only the `ask-matt` title and compact `SKILL.md` trigger; repository
  and installed-path metadata occurs only in Inventory-row masters, the File
  Tree root header, or the Remove confirmation detail.
- Confirmed the obsolete `q7hO4` master and all references are absent. The
  preserved translation screen contains no full-width comparison header: its
  two persistent content panes begin immediately below the sparse document
  header, use one central separator, and keep `ORIGINAL · SKILL.MD` and
  `TRANSLATION · 简体中文` as local labels inside their respective panes.
- Cross-checked all five theme Page Flow, Components, Patterns & States and
  Responsive Composition sections. Page Flow consistently documents local
  labels plus a central divider; every translation pattern contains exactly
  two local panes and one divider; responsive proofs retain the two-pane shell
  without repository/path metadata or a global translation heading.
- Reconfirmed the corrected File Tree contract remains intact. `V5I4D` stays
  inside `BIna5` and both preserved anchored popovers; page-level document
  headers do not absorb the Skill count/path. File rows have no chevrons,
  folders alone disclose, the header reports five files, and final rows remain
  compact, ordered and unclipped.
- Reviewed screenshots for Shared Masters; all five themed Components and
  translation patterns; Page Flow; Light responsive composition; wide Dark
  Preview and Translation; compact Dark Preview and file popover; the Light
  reference proof; and representative Settings, Install, Remove and runtime
  failure states. A transient partial Light-pattern render was recaptured and
  resolved normally. No visual repair was required.
- Final Pencil scan: seven ordered clipped roots, 2,528 named nodes, 21 reusable
  masters and 368 resolving refs. Zero broken refs, placeholders, unnamed or
  zero-size nodes, clipped direct children, `q7hO4` refs, stale comparison
  headers or page-identity provenance nodes.
- Full repository quality gate passes: Prettier, ESLint, TypeScript, production
  Vite build, 34 Vitest tests, Cargo fmt, Clippy with warnings denied, and 26
  Rust tests. Git reports no tracked production-source modification.

## Icon-only File Tree trigger and path disclosure — 2026-08-14

- Redesigned reusable trigger `LTKTz` in place as a 28×28 square icon-only
  button using the existing Lucide `folder-tree` icon. Removed the `SKILL.md`
  label and disclosure chevron, documented accessible label `Open file tree`
  and tooltip `Browse files`, and renamed all 22 instances to File Tree Button
  terminology. Every resolved instance measures exactly 28×28.
- Simplified reusable Root Header `V5I4D` from 58 to 38 px and removed the
  installed root path. It now contains only the `ask-matt` Skill identity and
  correct `5 files` count. The five theme flow summaries and ten responsive
  proof descriptions explicitly describe the icon-triggered hierarchy.
- Removed the persistent full-path footer from reusable final composition
  `BIna5` and from preserved wide/compact popovers `YYTJk` and `kwd7I`.
  All three compositions now measure 320×206 and contain only the root header
  plus six compact ordered rows. Updated all five themed `BIna5` instances and
  wrappers to the same 206 px height, closing the vacated space.
- Reworked Shared Masters Long-name Behavior so the complete path appears only
  in tooltip specimen `O9aNh`, explicitly scoped to hover or keyboard focus.
  Actual final/preserved compositions contain no slash-delimited text, root
  path, footer or path-preview node. File rows retain no chevrons; only folder
  masters `lBllu` and `uh5Wt` retain expanded/collapsed disclosure icons.
- Screenshot QA covered Shared Masters `lm6dI`, final composition `BIna5`,
  tooltip documentation `PedLV`, wide/compact previews `paqAO` and `gsCQ8`,
  preserved wide/compact popover screens `n5X3t` and `fn2aQ`, Page Flow
  `id3Iz`, Responsive Composition `Kvhyr`, and System/Light/Dark/Sand/Plum
  final specimens `L3a1I5`, `mlXL3`, `f4Hp3p`, `t356YJ`, `E6bXkP`.
  A transient blank Light capture was immediately recaptured successfully.
- Final Pencil validation reports seven ordered roots, 2,519 named nodes,
  21 reusable masters and 368 resolving refs. There are zero broken refs,
  placeholders, unnamed or zero-size nodes, clipped descendants, stale
  filename-trigger/path-footer names or layout warnings. `npm run lint` and
  `npm run typecheck` both pass.

### Independent icon-trigger and path-disclosure check

- Resolved reusable `LTKTz` and all 22 instances. Every instance is exactly
  28×28, contains only the Lucide `folder-tree` icon, and inherits the component
  context `Accessible label: Open file tree. Tooltip: Browse files.` No current
  filename, trigger chevron, or stale Current File Trigger naming remains.
- Confirmed `V5I4D` and the root headers in reusable, themed, wide and compact
  final compositions are 38 px high and contain only `ask-matt` plus `5 files`.
  None contains an installed root path.
- Resolved `BIna5`, all five themed instances, `YYTJk` and `kwd7I`: each is
  206 px high with exactly the root header plus six 28 px rows. No footer,
  path-preview node, long full-path text or dead vertical space remains.
- Confirmed the only complete long-file path occurs in isolated tooltip
  specimen `O9aNh`, whose visible label and component context explicitly scope
  it to hover/keyboard focus. Other exact installed-root paths remain only in
  the Inventory-row master and Remove confirmation, outside File Tree content.
- Revalidated prior semantics: file rows have no chevrons; expanded/collapsed
  folders each have one; the long row remains an ellipsized basename at 16 px
  indentation; selection and focus share `SKILL.md`; translation retains two
  panes and no global comparison-header or `q7hO4` reference.
- Screenshot review covered Shared Masters, final composition, tooltip, Page
  Flow, responsive proof, all five themed trees, wide/compact Preview and
  popover screens, and the preserved translation view. The transient blank
  Light specimen was recaptured successfully. Popovers are tightly reflowed,
  visibly anchored, bounded and unclipped.
- Independent scan reports 2,519 named nodes, 21 reusable masters and 368
  resolving refs, with zero broken refs, placeholders, unnamed or zero-size
  nodes, clipped direct children, stale trigger/footer names or translation
  headers. ESLint, TypeScript and all 34 Vitest tests pass; no tracked
  production source is modified.

## Leading File Tree identity placement — 2026-08-14

- Reordered all eight wide and three compact page-level identity groups so the
  `LTKTz` instance is the first child and the Skill title is second. Every
  group uses a compact 10 px gap, start alignment and optical center alignment;
  the former space-between/right-side trigger placement is gone.
- Re-anchored preserved open trees from the new leading controls. Wide popover
  `YYTJk` now starts at x=295 and compact popover `kwd7I` at x=228, immediately
  inside their Preview panes. Both remain y=120, 320×206, bounded and clear of
  the left Inventory pane while retaining root-header-first content.
- Expanded Shared/component documentation systematically: Shared navigation
  overview now shows button → Skill title; all ten themed component lanes show
  a dedicated 10 px leading identity specimen beside the retained Inventory
  row; all ten System/Light/Dark/Sand/Plum wide and compact responsive proofs
  include the same button/title group. Five Page Flow and Pattern summaries
  now explicitly describe the leading-button anchor.
- Screenshot QA covered wide Preview `paqAO`, compact Preview `gsCQ8`, open
  popover screens `n5X3t` and `fn2aQ`, Shared components `aBICA`, Light/Dark
  Components `rXOZt` and `YTFuf`, Light Responsive Composition `Kvhyr`, System
  Page Flow `id3Iz`, Light Pattern popover `hReKH`, and preserved translation
  screen `z6HOSV`. Identity grouping, header command spacing and popover
  stacking remain visually clear at both approved sizes.
- Final Pencil scan reports seven ordered roots, 2,570 named nodes, 21 reusable
  masters and 378 resolving refs, including 32 correctly sized `LTKTz`
  instances. Structural checks find eight wide, three compact, ten themed
  component and ten responsive leading groups, all with trigger first and
  10 px gaps. There are zero broken refs, placeholders, unnamed or zero-size
  nodes, clipping/layout warnings, stale identity placements, persistent path
  UI, invalid file chevrons, comparison headers or `q7hO4` refs.
- `npm run lint` and `npm run typecheck` both pass. Production source remains
  untouched.

### Independent leading-identity placement check

- Resolved all 32 `LTKTz` references and their direct parents. Every trigger is
  the first child of its local leading identity group, followed immediately by
  the Skill title (or the compact title wrapper); every group uses a 10 px gap
  and optical center alignment. Coverage is complete across the Shared
  overview, eight wide and three compact preserved pages, ten themed component
  specimens and ten themed responsive proofs, with no residual right-side
  trigger.
- Verified the reusable `LTKTz` remains a 28×28 icon-only control with the
  Lucide `folder-tree` icon and component context `Accessible label: Open file
  tree. Tooltip: Browse files.` Page identities contain no repository, owner,
  installed-path, filename or disclosure metadata.
- Checked preserved open states structurally and visually. The wide trigger
  resolves at preview x=13 and its 320×206 popover starts at screen x=295,
  exactly the 282 px Inventory boundary plus 13 px; the compact trigger is at
  preview x=10 and its popover starts at x=228, exactly the 218 px boundary
  plus 10 px. Their vertical gaps are 4 px and 6 px respectively. Both are
  fully bounded, remain inside Preview and leave Inventory unobscured.
- Screenshot review covered Shared Masters, all five themed Components, System
  Page Flow, Light Patterns and Responsive Composition, preserved wide and
  compact popovers, and the wide translation screen. The leading control/title
  grouping, command separation, popover anchoring and local translation-pane
  labels are legible and unclipped; no Pencil repair was required.
- Independent global scan reports seven ordered clipped roots, 2,570 named
  nodes, 21 reusable masters and 378 resolving references. All 32 trigger
  placements pass; there are zero broken refs, placeholders, unnamed or
  zero-size nodes, stale comparison-header/`q7hO4` nodes, or invalid trigger
  placements. The only complete long file path remains in the isolated tooltip
  specimen; other installed paths are confined to the typography/Inventory
  documentation and Remove confirmation, outside File Tree content.
- `npm run lint`, `npm run typecheck`, and all 34 Vitest tests pass. Git status
  shows no tracked production-source modification.
