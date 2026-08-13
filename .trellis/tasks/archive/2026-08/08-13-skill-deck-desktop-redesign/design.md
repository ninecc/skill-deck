# Direction B implementation design

## Approved authority

This document translates the Direction B visual approval recorded in `prd.md`
into the existing React 19, CSS and Tauri 2 architecture. Accepted ADRs and
current behavior remain authoritative. The review prototype is visual evidence,
not production code and must not be copied as a replacement application shell.

## Architecture and boundaries

### Preserve behavior ownership

- `src/commands.ts` remains the only Application Command definition and
  availability authority. React controls, native menus, shortcuts and context
  menus continue to dispatch the same command IDs.
- `src/App.tsx` continues to own runtime, Inventory, selection, Preview,
  translation, discovery, mutation and feedback state. The redesign may extract
  presentational components, but it must not create parallel business state.
- The pinned upstream Skills CLI remains lifecycle truth. UI state must never
  infer ownership, reconciliation or update truth from paths.
- `src/preferences.ts` keeps its schema and persisted values, including
  `system | light | dark | sand | plum`.
- `src/nativeMenu.ts` and Tauri window/menu configuration retain platform
  behavior. No custom title bar or traffic-light simulation is introduced.

### Component shape

1. **Application chrome** — compact product mark, count, scope-separated
   toolbar commands and persistent status bar. Labels stay visible when the
   720×520 contract permits; icon-only controls retain accessible names and
   disabled reasons.
2. **Inventory pane** — two-line Skill identity, visible focus, restrained
   selected rail/wash, long-text truncation with full accessible/title context,
   and stable empty/no-match states.
3. **Preview pane** — Skill identity and provenance above the primary document
   area; content/navigation commands are separated from lifecycle commands.
4. **File tree popover** — the existing transient tree remains anchored to a
   compact current-file control. It gains clear file/folder/disclosure semantics,
   bounded desktop geometry and selected-file treatment. Existing arrow keys,
   Esc close and focus restoration remain intact.
5. **Settings** — one modal with compact section navigation: General,
   Appearance, Translation, Installation and About. Sections are presentation
   views over the same `Preferences`; they do not introduce a second settings
   store. Agent targets receive search/filter and summary controls inside the
   Installation section.
6. **Find & Install and Remove** — retain current flows and command dispatch;
   reorganize hierarchy, density, field grouping and state feedback only.
7. **State grammar** — runtime/loading/empty/no-match/Preview/translation/
   unresolved install/success/partial/error states use a consistent symbol,
   severity label, message and recovery-action pattern while remaining in their
   behaviorally appropriate locations.

## Token system

Implement tokens in `src/styles.css` using three layers without introducing a
framework.

- **Primitive:** neutral graphite ramps, compact spacing steps, system font
  stacks, 1 px borders, 4/6/8 px radii and short duration curves.
- **Semantic:** window, chrome, panel, document, text, muted text, separator,
  control, Theme Accent, focus, success, warning, danger, overlay and code.
- **Component:** toolbar height, control height (28–32 px), Inventory row,
  file-tree row, dialog chrome, status bar and document measure.

Light and dark remain low-chroma. Sand and Plum retain their persisted identity
but map through the same semantic roles. Theme Accent is used sparsely on the
brand detail, install affordance, selection rail, active tab, code edge and
state symbols. Native System Accent is not sampled or overwritten. Focus uses a
separate high-contrast token; semantic status colors never derive from Theme
Accent.

Icons use the existing local icon mechanism, a consistent 16 px optical size
and approximately 1.5 px stroke. Refresh Inventory uses a circular refresh
symbol; Update All uses a collection/download meaning; Update Skill uses a
single-item download/update meaning. No remote icon package is added.

## Layout contracts

### 1180×800

- Two persistent columns: approximately 280–300 px Inventory and remaining
  Preview.
- Preview is the dominant area; Markdown remains near 68–74 characters.
- File tree is a roughly 244 px trigger-anchored popover inside the window,
  collision bounded without becoming a third pane.
- Settings and Find & Install are bounded dialogs with internal section/content
  scrolling rather than one document-length sheet.

### 720×520

- Still a two-pane desktop workspace, approximately 210–230 px Inventory plus
  Preview; no phone-style master/detail navigation.
- Header commands may occupy a compact second row but must not overlap, clip or
  use touch-first sizing.
- The file popover stays within the application window. Dialog section
  navigation may compress horizontally, while content remains keyboard
  reachable.
- Layouts below 720×520 are explicitly unsupported by this task.

## Settings commit semantics

- Theme, UI language, target language, Agent targets and install method call the
  existing immediate `onChange` path.
- Translation Proxy remains local draft state. Only Apply validates and commits
  it. Its section shows both the exceptional Apply behavior and validation
  result.
- Close dismisses the modal and is never labeled or styled as Cancel. A compact
  “saved immediately” explanation appears where applicable.
- Section switching must preserve the Proxy draft and validation state for the
  lifetime of the dialog.

## Accessibility and interaction

- Preserve native tab order, list/tree arrow navigation, modal focus trap, Esc,
  initial focus and trigger/fallback restoration.
- Installed filter uses `:focus-within`; theme tiles project radio
  `:focus-visible` independently from selection.
- Selection never relies on color alone. Icons have localized labels when the
  adjacent visible text is absent. Disabled command reasons remain exposed.
- Status announcements retain appropriate live-region behavior without moving
  keyboard focus.
- Motion is limited to feedback or hierarchy transitions, completes within
  300 ms, is immediate on keyboard paths and is removed under
  `prefers-reduced-motion`.

## Platform adaptation

- **Shared:** React structure, semantic tokens, two-pane layout, dialogs,
  file-tree popover, state grammar, command IDs and accessibility semantics.
- **macOS:** native application menu and Command shortcut labels; system window
  chrome and pointer behavior remain native.
- **Windows:** application menu and Control shortcut labels; native system font
  resolution and window behavior remain intact.
- **Linux:** application menu and Control shortcut labels; tolerate distribution
  font/rendering variation without fixed font metrics.

Platform adaptation must not fork business logic or duplicate command state.

## Compatibility, risk and rollback

- No preference migration, command migration, lifecycle migration or native
  configuration rewrite is required.
- Highest risks are Settings refactoring, 720×520 density, long localized text,
  file-popover focus behavior and broad CSS regressions across five themes.
- Keep changes staged by seam: tokens/chrome, main workspace, file popover,
  Settings, dialogs/states, tests. Each seam can be reverted without reverting
  CLI or command behavior.
- The rollback boundary is the production UI/component/CSS change set. Planning
  artifacts and accepted ADRs remain; upstream lifecycle behavior is untouched.
