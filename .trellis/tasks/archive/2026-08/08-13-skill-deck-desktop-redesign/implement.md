# Direction B implementation plan

## Preconditions

- Direction B, its UI scope and macOS/Windows/Linux scope are approved in
  `prd.md`.
- The user approved the final planning summary and `task.py start` activated the
  task on 2026-08-13.
- After activation, load `trellis-before-dev` before editing production code.
- Do not add a UI framework, remote font, remote asset or new product behavior.

## Ordered implementation checklist

### 1. Baseline and behavior protection

- Capture the current working-tree diff and avoid unrelated user changes.
- Run focused existing tests for commands, App, Settings, ModalShell,
  preferences and native menus before structural edits.
- Inventory all current localized labels used by the approved screens; add only
  necessary localized interface copy, keeping domain terms stable.

### 2. Semantic visual foundation

- Refactor `src/styles.css` tokens into primitive → semantic → component roles
  for all five persisted themes.
- Establish system typography, compact density, border/radius/elevation, focus,
  status and motion tokens.
- Update the existing local icon set/usage for distinct Refresh Inventory,
  Update All and Update Skill semantics without adding a dependency.
- Apply the restrained Direction B product mark and toolbar hierarchy.

Rollback point: token/chrome/icon changes can be reverted independently while
leaving component behavior unchanged.

### 3. Main workspace and Preview

- Keep the persistent Inventory + Preview two-column model at 1180×800 and
  720×520; remove any phone-style sub-layout from the approved range.
- Refine Inventory identity, filter focus, selection/focus separation, long
  names/source/path treatment and empty/no-match presentation.
- Recompose the selected-Skill header into identity/provenance plus separated
  content/navigation and lifecycle command groups.
- Preserve Preview viewer switching, readable Markdown/code width,
  original/translation comparison and unsupported/image handling.
- Restyle the existing file-tree transient as the approved anchored popover;
  preserve its keyboard navigation, Esc behavior and focus restoration.

Rollback point: workspace and popover presentation can revert without changing
command dispatch, Inventory data or Preview loading.

### 4. Settings information architecture

- Refactor `SettingsDialog` into compact internal sections while keeping one
  modal and one `Preferences` authority.
- Keep immediate-save fields on the existing `onChange` path.
- Preserve Proxy draft until Apply, including validation and draft retention
  across section switches; make Close-not-Cancel semantics explicit.
- Add searchable/summary treatment for the large Agent target set without
  changing stored agent values.
- Fix visible focus for theme radios and verify section navigation, tab order,
  focus trap, Esc and restoration.

Rollback point: Settings can revert as a unit because persistence and schema do
not change.

### 5. Dialogs, status and state grammar

- Apply the approved hierarchy and density to Find & Install and Remove
  confirmation while preserving flow, initial focus and command behavior.
- Normalize runtime loading/failure, empty/no-match, Preview and translation
  errors, unresolved install, neutral progress, success and partial feedback.
- Keep recovery actions and live regions in their current behavioral locations.
- Verify Google egress disclosure remains visible before translation.

### 6. Platform and accessibility pass

- Confirm the shared WebView changes do not alter native menu construction,
  shortcut mapping, command availability or Tauri window configuration.
- Verify pointer affordances, visible focus, keyboard-only traversal, disabled
  reasons, contrast and non-color state cues.
- Verify `prefers-reduced-motion` removes nonessential transitions and keyboard
  navigation remains immediate.

### 7. Tests and validation

- Update/add component tests for Settings sections and commit semantics, file
  popover focus restoration, command labels/icons, filter/theme focus hooks and
  state rendering. Preserve all existing behavior tests.
- Run:
  - `npm run format:check`
  - `npm run lint`
  - `npm run typecheck`
  - `npm test -- --run`
  - `npm run build`
- Run `trellis-check` after implementation and resolve spec drift.
- Visually verify 1180×800 and 720×520 in Light and Dark, plus representative
  Sand and Plum checks; test long text, English/Chinese, keyboard focus, Reduce
  Motion, runtime failure, empty/no-match, Preview/translation errors,
  unresolved install, success and partial states.
- Record native platform smoke separately. macOS can be checked locally;
  Windows/Linux native menu/window smoke remains explicitly outstanding unless
  those environments are available.

## Risk controls

- Prefer presentational extraction over an `App.tsx` business-logic rewrite.
- Do not change command IDs, availability reasons, preference keys or CLI
  request/response shapes.
- Do not use the planning prototype as generated production markup.
- Stop and request approval if implementation requires a new framework,
  preference migration, native window/menu change, resizable sidebar or scope
  beyond the approved screens.

## Definition of done

- Approved Direction B is visibly consistent across the entire authorized UI
  scope at both desktop sizes and all persisted themes.
- Existing behavior contracts and all required checks pass.
- The final report separates visual changes, behavior changes (expected: none),
  platform differences, validation evidence and unfinished native smoke tests.
