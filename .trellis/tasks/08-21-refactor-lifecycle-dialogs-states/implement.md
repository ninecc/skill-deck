# Implementation plan

- [ ] Map current modal/state branches to Pencil References 14–19.
- [ ] Refactor shared dialog chrome and Find & Install content/results/actions.
- [ ] Add Search/From source local tabs and preserve existing command paths.
- [ ] Implement and test within-opening draft retention, normal reopen reset and
      unresolved retry-context preservation.
- [ ] Refactor Remove confirmation and destructive-action hierarchy.
- [ ] Refactor startup Loading, Runtime failure, Empty inventory and Preview
      failure compositions and recovery actions.
- [ ] Add the specified in-pane Find & Install and Retry/Reveal recovery actions.
- [ ] Align status success/partial/error/diagnostics presentation.
- [ ] Preserve and extend command, modal-focus and recovery tests.
- [ ] Compare every owned state at both approved sizes and across five themes.
- [ ] Run full frontend quality gate and Trellis check.
- [ ] Commit and archive this child before Child 4 starts.

## Rollback points

- Shared dialog chrome, lifecycle dialogs and shell operational states are
  separate logical checkpoints.
