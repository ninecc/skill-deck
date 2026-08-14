# Skill Deck full-flow Pencil design

## Goal

Create a review-ready, high-fidelity and maintainable Pencil design-system
deliverable at `pen-design/skill-deck-2.pen`. It must translate the approved
Direction B visual contract and supplied interactive HTML reference into a
theme-first hierarchy rather than a loose gallery of full-page state captures.
The result should let product and engineering inspect each theme's tokens,
flows, components, icons, patterns/states and responsive composition without
changing application source code.

## Background and confirmed facts

- The supplied reference is
  `/Users/didi/.codex/visualizations/2026/08/13/019ff943-a5ca-7ee0-8846-0d284c1d0ab9/skill-deck-visual-directions.html`.
- Direction B is already approved in the archived desktop-redesign task: an
  extremely restrained professional desktop utility with low-chroma graphite
  surfaces, monochrome 1.5 px icons, sparse semantic accent, and compact native
  density.
- The product contract is a two-pane desktop layout at 1180×800 and 720×520.
  Inventory and Preview remain persistent; the file tree is a trigger-anchored
  popover, not a third persistent pane.
- The reference defines ten scenes: Preview, file-tree popover, Translation,
  Settings, Find & Install, Remove confirmation, startup loading, runtime
  failure, empty Inventory, and Preview failure.
- The existing empty target file is already active in Pencil at
  `pen-design/skill-deck-2.pen`.
- This task is a design-only deliverable. Production React, CSS, Tauri,
  localization, command, preference, and lifecycle code are out of scope.

## Requirements

### Deliverable information architecture

- Make the document's primary axis the five persisted themes:
  `system | light | dark | sand | plum`.
- Create a compact library index and a shared reusable-master area, then one
  ordered theme collection board per theme.
- Each theme collection must use the same internal sections, in this order:
  `Tokens`, `Page Flow`, `Components`, `Icons`, `Patterns & States`,
  `Responsive Composition`, and `Reference Compositions` where applicable.
- Keep root-level canvas content limited to the index, shared masters and the
  five theme collection boards. Full-page state screens must not remain as
  unrelated document roots.

### Theme token coverage

- Every theme must explicitly display the Primitive → Semantic → Component
  token chain, including the raw palette, purpose aliases and representative
  component assignments.
- Token output must cover color, typography, spacing, radii, borders/elevation,
  motion, icon sizing, focus, status roles, control density and layout metrics.
- `System` is a resolution theme, not an invented palette. Its token section
  must show both system-light → Light and system-dark → Dark branches.
- Light, Dark, Sand and Plum must use the values already defined by production
  theme contracts; do not invent replacement palettes.

### Structured design output

- Page Flow is a flow-oriented set of compact page/pattern thumbnails for the
  ten reference scenes, including Esc/dismiss and recovery transitions. It is
  not ten more full-size screens.
- Components are organized by family and show relevant variants/states:
  commands, inputs/search, Inventory/file navigation, tabs, status/feedback,
  dialog chrome and theme controls.
- File navigation must be specified as a real hierarchical tree component:
  anchored compact icon trigger, root Skill identity and count, file/folder
  icon semantics, disclosure controls, indentation, expanded/collapsed rows,
  selected/hover/focus states, and long-name/path handling. A flat stack of
  file labels is not acceptable.
- File rows must never show disclosure chevrons. Only folder rows may show
  collapsed/expanded disclosure controls. Nested files must display only their
  basename at the correct indentation level, not a slash-delimited path as the
  row label.
- The final file-tree composition must place the root header before all rows,
  use compact 28–30 px desktop rows, count files consistently without counting
  folders, avoid duplicate entries, and keep keyboard focus on the current
  selected row unless a state-specimen explicitly demonstrates otherwise.
- Long file names remain single-line with ellipsis; the complete path may appear
  in a tooltip specimen, never as a persistent footer/path preview or by
  wrapping the row onto a second line.
- File Tree documentation must separate `Anatomy`, `Row States`, `Indentation`,
  `Long-name Behavior`, and `Final Composition` so state examples are not
  mistaken for a real tree.
- The document identity header must not repeat owner/repository breadcrumbs or
  the installed filesystem path before the current-file trigger. The Skill
  title and local content label already establish context. Use only a compact
  icon button to open the tree: do not repeat the current filename or add a
  disclosure chevron inside the trigger. Place this button immediately before
  the Skill title so both read as one local identity/navigation group; do not
  isolate it on the opposite side of the page header like a global action.
- The File Tree popover must not repeat the installed root path in its header
  or show the selected file's full path in a persistent footer. Those paths are
  already present in the left Skill list. Keep only the root Skill name, file
  count, hierarchy, row labels and state styling inside the popover.
- Original and translated content are differentiated by their persistent
  two-column layout. Do not add an Original/Translation switcher or active-tab
  underline above the columns. Do not add a separate full-width comparison
  header above the two panes. Each pane may retain a small local content label
  inside its own column, but it must not look interactive or duplicate a
  second global heading row.
- Icons are organized by functional group: global commands, content/navigation,
  lifecycle/destructive, files, and status. Use one consistent icon family and
  document size/stroke roles.
- Patterns & States isolate popover, translation comparison, modal families,
  loading, empty, error and recovery patterns without repeating the entire app
  shell around each pattern.
- Responsive Composition shows only the representative 1180×800 and 720×520
  shell/composition proofs needed to communicate layout behavior.
- Preserve the existing validated full-page compositions without deleting
  them, but classify them by resolved theme: the Light proof belongs under
  Light Reference Compositions; only dark-resolved compositions belong under
  Dark Reference Compositions. System is the only collection allowed to show
  Light and Dark together as resolution branches.

### Visual and interaction contracts

- Preserve the reference hierarchy: native title bar, compact toolbar,
  Inventory/filter, selected-Skill provenance and commands, Preview content,
  persistent status bar, anchored file popover, and bounded modal dialogs.
- Use neutral large surfaces. Accent appears only for orientation, selection,
  active tabs, focus, code edge, install affordance, and state symbols; success,
  warning, and danger remain independent semantic roles.
- Use system-like sans typography and a monospaced face for paths/code. Keep
  controls approximately 28–32 px high and use a 4/8-point spacing rhythm.
- Use one consistent vector icon family and no emoji. Keep icons optically
  aligned at 14–16 px with restrained stroke weight.
- Represent visible focus, selected, disabled, loading, error, and destructive
  states without relying on color alone.
- Dialogs must have a strong enough scrim, clear default/secondary hierarchy,
  and visible close/cancel escape routes. File popover positioning must remain
  visibly anchored and bounded inside both target windows.
- Every added Pencil node must have a human-readable name; screen frames must
  use clipping and no screen may contain clipped or collapsed content.

## Acceptance criteria

- [ ] `pen-design/skill-deck-2.pen` opens through Pencil MCP and its primary
      root structure is a library index, shared masters, and five ordered theme
      collection boards rather than scattered full-page screens.
- [ ] System, Light, Dark, Sand and Plum each contain an explicit token board
      showing Primitive → Semantic → Component mappings; System shows both
      Light and Dark resolution branches.
- [ ] Every theme collection visibly contains Page Flow, Components, Icons,
      Patterns & States and Responsive Composition sections with consistent
      ordering and naming.
- [ ] The ten reference scenes are represented through structured flows and
      isolated patterns/states; only representative shells use full-page
      composition.
- [ ] Existing full-page compositions are preserved under the matching theme
      appendix: `Zraur · Wide Light Preview` is under Light and all dark-resolved
      screens are under Dark; no theme appendix contains a foreign theme.
- [ ] Light, Dark, Sand and Plum boards visibly demonstrate their production
      palette mappings while keeping component anatomy and hierarchy consistent.
- [ ] The file popover, Settings, Find & Install, Remove confirmation, loading,
      runtime failure, empty Inventory, and Preview failure states are visually
      complete and include their relevant recovery or dismissal actions.
- [ ] Repeated controls are implemented as reusable Pencil components and
      theme boards consume instances or systematic specimens instead of
      rebuilding complete pages.
- [ ] File-tree masters and themed specimens visibly demonstrate hierarchy,
      disclosure, indentation, selection, focus, hover and long-name behavior.
- [ ] In every final composition, chevrons appear only on folders, child rows
      use basenames and indentation, the root header comes first, file counts
      are correct, rows are compact and long names never wrap.
- [ ] No Original/Translation tab or segmented switch remains in masters,
      theme component boards, translation patterns or preserved compositions;
      two static column headings provide sufficient orientation.
- [ ] Pencil structural validation finds no collapsed layouts, overflow, or
      unresolved warnings; targeted screenshots confirm alignment, contrast,
      typography, selection/focus, and dialog composition.
- [ ] No production source file is modified as part of the design delivery.

## Out of scope

- Production implementation or behavior changes.
- Mobile or sub-720 layouts, touch-first navigation, custom window chrome, or
  a persistent third file pane.
- New product features, marketplace concepts, lifecycle changes, preference
  schema changes, or additional scenes absent from the approved reference.
- Remote fonts, stock photography, decorative illustration, gradients, Bento
  layouts, glass stacks, or marketing-style composition.

## Risks and deferred items

- Pencil cannot execute the reference interactions; interaction intent is
  documented through separate state screens and the flow map.
- A static design cannot prove runtime focus restoration or keyboard behavior;
  those contracts are annotated and remain production verification concerns.
- The exact system font renderer varies by platform; the design uses available
  system-like fonts and prioritizes hierarchy and metrics over font-brand
  imitation.
