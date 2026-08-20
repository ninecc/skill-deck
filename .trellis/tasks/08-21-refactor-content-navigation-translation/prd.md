# Refactor content navigation and translation

## Goal

Bring Preview content navigation and translation onto the shared Pencil visual
foundation: the leading-button anchored file-tree popover and the local-labeled,
divider-separated translation composition.

## Requirements

- Match Reference 11 (file popover) and Reference 12 (translation).
- Keep the file tree transient and anchored to the file trigger; it is not a
  persistent third pane. Preserve header, count, root name and five-file example
  hierarchy, while rendering real runtime data.
- Preserve tree semantics, full-path accessible metadata, unsupported-file
  selection and focus restoration on Escape/outside dismissal.
- Restore focus to the file-tree trigger after file activation closes the
  popover, not only after Escape dismissal.
- Upgrade folder rows to accessible disclosures: focusable, `aria-expanded`,
  Click/Enter toggle, Right expands/moves inward and Left collapses/moves to the
  parent.
- Initialize all folders expanded on load, Skill switch and tree refresh. Keep
  manual toggles only for the current Skill tree lifetime; do not persist them.
- Collapsing a selected file's ancestor leaves Preview unchanged. Reopening the
  popover expands selected ancestors, preserves unrelated toggles and focuses
  the selected file; if it disappeared, focus the first visible treeitem.
- Replace the visible path control with the Pencil icon-only trigger using
  `Open file tree` / `打开文件树` as `aria-label` and `Browse files` / `浏览文件`
  as tooltip.
- Preserve safe Markdown/plain-text Preview and remote-resource blocking.
- Preserve translation loading, success, failure, retry and compact-pane
  operability with localized pane labels and a stable central divider.
- At 720×520 keep the production local pane tabs: original and translation state
  remain persistent, while one readable pane is visible at a time.
- Reuse Child 1 tokens and Iconify icons; do not introduce local visual systems.
- Add deterministic dev/review scenarios for file-tree and translation states;
  keep them absent from production output.

## Dependency

- Requires `08-21-build-visual-foundation-app-shell` to be completed, checked,
  committed and archived.

## Acceptance Criteria

- [ ] File popover matches Pencil in Dark and remains usable in all themes and
      both approved window sizes.
- [ ] Keyboard tree navigation, dismissal and trigger-focus restoration work.
- [ ] File activation closes the popover, loads Preview and restores trigger
      focus.
- [ ] Folder disclosure and Left/Right tree behavior are tested with correct
      `aria-expanded` state.
- [ ] Expansion reset/reopen/selected-descendant/fallback-focus behavior is
      tested without changing backend FileEntry DTOs or ordering.
- [ ] Translation success/loading/error/retry states preserve current behavior
      and do not clip English or Simplified Chinese content.
- [ ] Preview cannot navigate or load remote Markdown resources.
- [ ] Full frontend quality gate passes.

## Out of scope

- Lifecycle dialogs, Settings, backend translation behavior or a third pane.
