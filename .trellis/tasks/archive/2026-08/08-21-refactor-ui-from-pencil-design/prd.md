# Refactor UI from Skill Deck Pencil design

## Goal

Refactor the Skill Deck desktop UI to faithfully implement the production-aligned
design system and complete application-state references in
`pen-design/skill-deck.pen`, while preserving existing lifecycle behavior,
accessibility, localization and offline operation. Deliver the work in ordered,
independently verifiable child tasks so the overall shell lands first and
feature-specific surfaces follow incrementally.

## Background and confirmed facts

- The authoritative design source is `pen-design/skill-deck.pen` (Pencil 2.17).
- The document defines System, Light, Dark, Sand and Plum theme contracts, a
  shared master component library, page flows, icon rules, patterns/states,
  responsive proofs and high-fidelity reference compositions.
- The visual direction is a restrained professional desktop utility: compact
  native density, low-chroma surfaces, sparse semantic accent, stable chrome
  and a persistent two-pane Inventory/Preview workspace.
- Reference compositions cover wide Preview, anchored file-tree popover,
  Translation, Settings, Find & Install, Remove confirmation, startup Loading,
  Runtime failure, Empty inventory and Preview failure.
- Responsive proofs cover 1180×800 and 720×520 in every theme. The compact
  contract remains two-pane rather than changing to a single-pane mobile app.
- Settings covers General, Appearance, Translation, Installation and About,
  including default, invalid proxy, automatic targets, explicit targets, no
  matches, Light-theme and Chinese text-expansion proofs.
- Settings opens on General; its header/navigation/footer remain stable while
  the content viewport scrolls independently. Immediate preferences persist
  immediately; the proxy remains draft-only until Apply. Close, Escape and
  keyboard focus restoration are part of the interaction contract.
- The design icon system is Lucide at 14/16 px with a 1.5 stroke. Named icons
  include package-plus, rotate-cw, download, sliders-horizontal, languages,
  folder-open, file-text, files, arrow-down-to-line, trash-2, x, refresh-cw,
  folder, image, chevron-right/down, check, info, triangle-alert and
  square-terminal.
- Icons must be sourced through the Iconify ecosystem. Iconify's name-based
  React usage calls the public API at runtime, which conflicts with this
  application's offline/no-implicit-network contract. The implementation must
  therefore bundle selected Lucide icon data at build time and never fetch
  icons at runtime.
- The current React UI already implements the underlying commands, state and
  most interaction flows. This task changes presentation and component
  structure without moving lifecycle authority out of the upstream CLI.
- The repository has no reusable real-layout state fixture harness. A
  development/review-only typed IPC fixture entry is approved so every Pencil
  state can be reproduced deterministically without network or user-data
  mutation. Production builds must completely exclude it.
- The approved migration strategy is incremental: preserve working state,
  command and test contracts while replacing structure and styling by child.
- Pencil fidelity is strict by default. Only genuinely platform-controlled
  rendering differences are tolerated; intentional deviations require review.
- Pencil's macOS title bar is contextual native-window chrome, not React UI.
  Keep system-provided decorations and window controls on every platform; the
  shared visual implementation begins at the Application Toolbar.
- Apply the shared WebView visual direction to Windows, macOS and Linux; only
  native system chrome is platform-specific.
- Treat 1180×800 and 720×520 as React/WebView content viewport sizes, excluding
  system title bars and outer window borders.
- Keep 720×520 as the enforced minimum content size; smaller-window navigation
  or layout is not part of this design.
- Preserve current production terminology and English/Simplified Chinese copy.
  Design-only wording does not override UX Writing unless it is an explicit
  correction or receives separate approval.

## Requirements

### R1. Design fidelity and hierarchy

- Reproduce the Pencil design's visual hierarchy, spacing, typography, theme
  tokens, component states and desktop density for all in-scope surfaces.
- Treat Pencil reference compositions and state matrices as the visual source
  of truth. When the design does not specify behavior or content, record the
  gap and obtain a product decision before implementing it.
- Shared component masters, established accessibility requirements and native
  platform conventions may fill low-level hover/focus/disabled/motion details.
  Ask before choosing among meaningful product, hierarchy, copy, interaction or
  visual alternatives.

### R2. Incremental task delivery

- Use a parent task with sequential child tasks. Each child must have its own
  requirements, implementation plan, checks and acceptance boundary.
- Land the shared visual foundation and overall application shell before
  feature-specific UI work.
- Preserve a runnable application and passing quality gate at every child-task
  boundary.
- Every intermediate child must remain visually coherent: unfinished surfaces
  may keep their old structure but must consume the new shared tokens and base
  control styling instead of showing an obvious split skin.
- Stop after every child for user review of screenshots and runtime evidence;
  the next child requires explicit visual approval.

### R3. Iconify/Lucide integration

- Replace hand-authored icon paths with the matching Lucide icons selected from
  Iconify.
- Bundle only the required icon data and retain currentColor, accessible button
  labels and the design's 14/16 px and 1.5-stroke appearance.
- No icon may depend on Iconify API availability at runtime.

### R4. Themes and responsive behavior

- Implement System, Light, Dark, Sand and Plum through shared semantic tokens,
  not duplicated page-specific colors.
- Match the 1180×800 and 720×520 proofs without collapsed, clipped or
  overlapping content.
- Preserve reduced-motion, keyboard focus visibility and long localized-text
  resilience.
- Accessibility, understandable state and native platform behavior take
  precedence if strict visual replication conflicts with them; document and
  review the smallest necessary visual deviation.

### R5. Feature surfaces and states

- Main shell: native-window context, application toolbar, inventory list,
  preview workspace and status bar.
- Content navigation: anchored file-tree popover and translation panes.
- Lifecycle flows: Find & Install, Remove confirmation, startup loading,
  runtime failure, empty inventory, preview failure and status feedback.
- Settings: all five sections, persistence boundaries, validation, independent
  content scrolling, keyboard/dismissal behavior and localization proofs.

### R6. Behavior preservation

- Preserve the existing command DTOs, upstream CLI lifecycle authority,
  preferences semantics, diagnostics and localization contracts unless a
  separately approved requirement explicitly changes them.
- Do not add implicit telemetry or runtime network activity for UI assets.

### R7. Deterministic visual review

- Establish a development/review-only scenario entry in Child 1 and let each
  later child add its owned typed states.
- The harness may mock IPC only in the dedicated review entry. It must not add a
  production menu, route, behavior branch, user-data mutation or network call.
- Use canonical Pencil sample data for direct comparison, plus explicit long
  text, empty-value and Simplified-Chinese stress fixtures. Never read real user
  Skills or environment data for review scenarios.
- Keep the review tool after this task as maintained developer capability, with
  brief usage documentation and production-exclusion checks.
- Production verification must prove review code, fixture identifiers and data
  are absent from the shipped bundle.

## Acceptance Criteria

- [x] Every in-scope Pencil reference composition has a corresponding runnable
      application state whose layout and styling match the design at 1180×800.
- [x] System, Light, Dark, Sand and Plum all match their token and responsive
      proofs at 1180×800 and 720×520.
- [x] System resolves correctly for OS Light and OS Dark at both approved sizes.
- [x] All production icons use bundled Iconify Lucide data; the app performs no
      icon-related runtime request.
- [x] File-tree, translation, lifecycle dialogs/states and Settings interactions
      retain their existing functional behavior and documented focus/dismissal
      contracts.
- [x] File folders support real accessible disclosure and the compact
      Translation view retains persistent pane state behind local tabs.
- [x] English, Simplified Chinese, long target names, empty/error/loading states
      and reduced-motion mode do not clip or break the layout.
- [x] Existing and added component/integration tests pass, as do formatting,
      ESLint, TypeScript, Vitest and the production Vite build.
- [x] Record pre/post production bundle size, bundle only used Lucide icons and
      show no material startup regression; investigate significant growth rather
      than inventing an unsupported fixed KB/ms budget.
- [x] Visual acceptance uses fixed-size screenshots and Pencil comparison;
      behavior remains automated without adding brittle pixel-baseline tests.
- [x] Every Pencil state is reproducible through typed review fixtures, while
      the production bundle contains no review harness or fixture data.
- [x] The complete deterministic matrix is captured through the review entry;
      representative states per child and final critical flows receive native
      Tauri smoke coverage.
- [x] macOS native smoke and three-platform builds/shared-UI checks pass;
      unavailable Windows/Linux native smoke is recorded for release follow-up.
- [x] Each child task is independently reviewed and archived before the next
      dependent child starts; the parent finishes with a cross-child visual and
      behavioral integration review.
- [x] Each child review covers all owned Dark states at both approved sizes plus
      representative checks in the other themes; the final child covers the
      exhaustive theme × size × state matrix.

## Out of scope

- Changes to upstream CLI lifecycle ownership or command semantics.
- New backend capabilities that are not required to render the approved design.
- Editing the Pencil source design itself.
- Runtime retrieval of icons or other UI assets from third-party services.
- Additional breakpoints or mobile-navigation patterns not shown in the Pencil
  responsive proofs.
- Layout or navigation behavior below the enforced 720×520 content minimum.
- Replacing native Windows, macOS or Linux window chrome with a custom React
  title bar.
- Committing generated review screenshots or introducing screenshot-regression
  infrastructure; task documents retain the validation matrix and conclusions.

## Deferred-item policy

- Meaningful deferred product/behavior work receives a separate Trellis task
  after the current design tree is approved. Low-priority visual observations
  remain in this parent's Deferred Items rather than creating task noise.
- Alphabetizing Agent targets is the currently identified behavior follow-up.

## Approved behavior additions

- At 720×520, Translation uses local Original/Translation tabs while retaining
  both pane states; simultaneous narrow panes are not required.
- File-tree folders become real accessible disclosures with focus, Click/Enter,
  Left/Right and `aria-expanded` behavior.
- Find & Install separates Search and From source into two local tabs, with
  Search as the default.
- Search/source drafts survive tab switches within one opening. Normal close and
  reopen resets both and returns to Search; unresolved retry context preserves
  the current production recovery behavior.
- The icon-only file-tree trigger uses `Open file tree` / `打开文件树` as its
  accessible name and `Browse files` / `浏览文件` as its tooltip.
- File-tree directories start expanded on load/switch/refresh. Manual expansion
  is in-memory for the current Skill tree only. Reopening reveals/focuses the
  selected file by expanding its ancestors while preserving unrelated toggles.
