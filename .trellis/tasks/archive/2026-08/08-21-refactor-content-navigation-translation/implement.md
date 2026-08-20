# Implementation plan

- [x] Inspect current file-tree and translation behavior/tests against Pencil.
- [x] Refactor file trigger, header, root/folder/file rows and stacking.
- [x] Move source beside Skill name and isolate path + file trigger on the
      second row with wide/compact truncation priorities.
- [x] Move the file-tree trigger before the install path and keep popover
      anchoring inside both wide and compact viewports.
- [x] Replace the compact path prototype with a balanced single-row header:
      name + icon-only file-tree trigger, then distinct icon-only core actions.
- [x] Add localized icon-trigger label/tooltip and accessible folder disclosure
      state with full keyboard behavior.
- [x] Implement all-expanded initialization, per-tree toggle memory,
      selected-ancestor reveal on reopen and visible-item focus fallback.
- [x] Apply approved Lucide file/folder/image/disclosure icons.
- [x] Preserve tree keyboard behavior, dismissal and focus restoration tests.
- [x] Add file-activation trigger-focus restoration coverage.
- [x] Refactor original/translation panes, divider and responsive behavior.
- [x] Preserve compact local-tab behavior and persistent pane state at 720×520.
- [x] Verify loading, error, retry, Markdown and plain-text states.
- [x] Visually compare References 11 and 12 at 1180×800 and 720×520.
- [x] Run the full frontend quality gate and Trellis check.
- [x] Rerun the post-check Dark dual-size matrix and representative themes.
- [x] Present screenshots and native-smoke limitation for explicit user visual
      approval.
- [x] Create final work commit `2a99b0b`.
- [x] Archive this child through the Trellis finish workflow before Child 3.

## Rollback points

- File-popover markup/CSS and translation composition land as separate logical
  changes so either can be reverted without undoing the shared foundation.
