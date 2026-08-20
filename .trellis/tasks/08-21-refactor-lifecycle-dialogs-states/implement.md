# Implementation plan

- [x] Map current modal/state branches to Pencil References 14–19.
- [x] Refactor shared dialog chrome and Find & Install content/results/actions.
- [x] Add Search/From source local tabs and preserve existing command paths.
- [x] Remove the redundant reciprocal Search footer action from From source.
- [x] Implement and test within-opening draft retention, normal reopen reset and
      unresolved retry-context preservation.
- [x] Refactor Remove confirmation and destructive-action hierarchy.
- [x] Refactor startup Loading, Runtime failure, Empty inventory and Preview
      failure compositions and recovery actions.
- [x] Add the specified in-pane Find & Install and Retry/Reveal recovery actions.
- [x] Align status success/partial/error/diagnostics presentation.
- [x] Preserve and extend command, modal-focus and recovery tests.
- [x] Compare every owned state at both approved sizes and across five themes.
- [x] Run full frontend quality gate and Trellis check.
- [ ] Commit and archive this child before Child 4 starts.

## Rollback points

- Shared dialog chrome, lifecycle dialogs and shell operational states are
  separate logical checkpoints.
