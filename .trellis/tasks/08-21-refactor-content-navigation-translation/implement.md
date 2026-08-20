# Implementation plan

- [ ] Inspect current file-tree and translation behavior/tests against Pencil.
- [ ] Refactor file trigger, header, root/folder/file rows and stacking.
- [ ] Add localized icon-trigger label/tooltip and accessible folder disclosure
      state with full keyboard behavior.
- [ ] Implement all-expanded initialization, per-tree toggle memory,
      selected-ancestor reveal on reopen and visible-item focus fallback.
- [ ] Apply approved Lucide file/folder/image/disclosure icons.
- [ ] Preserve tree keyboard behavior, dismissal and focus restoration tests.
- [ ] Add file-activation trigger-focus restoration coverage.
- [ ] Refactor original/translation panes, divider and responsive behavior.
- [ ] Preserve compact local-tab behavior and persistent pane state at 720×520.
- [ ] Verify loading, error, retry, Markdown and plain-text states.
- [ ] Visually compare References 11 and 12 at 1180×800 and 720×520.
- [ ] Run the full frontend quality gate and Trellis check.
- [ ] Commit and archive this child before Child 3 starts.

## Rollback points

- File-popover markup/CSS and translation composition land as separate logical
  changes so either can be reverted without undoing the shared foundation.
