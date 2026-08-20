# Parent execution plan

The parent is a coordination and integration task; do not start it for direct
implementation. Start and archive children in the order below.

## Ordered checklist

- [ ] Child 1: establish tokens, offline Iconify/Lucide integration and the
      Toolbar/Inventory/Preview/Status application shell.
- [ ] Establish the dev/review-only typed IPC scenario entry and initial shell
      states; verify production exclusion.
- [ ] Seed canonical Pencil data plus long/empty/Chinese stress fixtures and
      document how developers launch the review entry.
- [ ] Present Child 1 screenshots/runtime evidence and obtain user visual
      approval before Child 2.
- [ ] Apply requested corrections, then create Child 1's final commit/archive.
- [ ] Child 2: implement the anchored file tree and translation composition on
      top of Child 1.
- [ ] Add deterministic file-tree and translation review scenarios.
- [ ] Obtain user visual approval before Child 3.
- [ ] Apply requested corrections, then create Child 2's final commit/archive.
- [ ] Child 3: refactor Find & Install, Remove and operational states after the
      shared shell/transient patterns are stable.
- [ ] Add deterministic lifecycle and recovery review scenarios.
- [ ] Obtain user visual approval before Child 4.
- [ ] Apply requested corrections, then create Child 3's final commit/archive.
- [ ] Child 4: refactor all Settings sections and edge states using the shared
      modal and visual foundation.
- [ ] Add deterministic Settings matrix and localization proof scenarios.
- [ ] Obtain user visual approval before Child 5.
- [ ] Apply requested corrections, then create Child 4's final commit/archive.
- [ ] Child 5: run cross-theme, cross-size and native-desktop integration review;
      fix only regressions inside the approved parent scope.
- [ ] Confirm all child acceptance criteria and parent acceptance criteria.
- [ ] Run the final full-scope quality gate and visual comparison.
- [ ] Update project specs only for reusable contracts learned during delivery.
- [ ] Commit integration fixes, archive the children, then archive the parent.

## Shared validation

Run at every child boundary:

```bash
npm run format:check
npm run lint
npm run typecheck
npm test -- --run
npm run build
```

Visual review must cover 1180×800 and 720×520, all five themes, English and
Simplified Chinese, and every Pencil reference state owned by the child.

During Children 1–4, capture all child-owned Dark states at both sizes and use a
representative owned state to verify System, Light, Sand and Plum. Settings also
captures its Light and Chinese proof states. Child 5 runs the exhaustive matrix.
Screenshots are review artifacts only; record the matrix and findings in task
text rather than committing the image files.
The deterministic review entry owns the full matrix. Each child also runs a
representative native Tauri state; final integration repeats critical native
shell, popover, modal, focus and system-theme paths.
Present a labeled contact sheet plus the validation matrix and discrepancy list;
expand individual frames only when they need closer review.

## Review gates

- Do not start a dependent child until its predecessor is implemented, checked,
  committed and archived.
- Do not accept a runtime icon request, a mobile single-pane substitution,
  duplicated command logic or a change to Agent target ordering.
- Any new design gap returns to parent planning for a user decision.
- Accessibility and native platform behavior outrank pixel fidelity; disclose
  every resulting visual deviation.
- A later child that regresses an approved surface must fix it in the current
  child, rerun the earlier evidence and preserve existing commit history.
- Review corrections rerun affected frames plus representatives; changes to a
  shared token, layout or component rerun the full owning-child matrix.
- Final cross-platform evidence requires macOS native smoke and successful
  three-platform builds/shared UI validation. Record unavailable Windows/Linux
  native smoke explicitly; it remains part of the release checklist.
- Keep all children on `codex/refactor-ui-from-pencil-design`, with one reviewed
  commit and rollback boundary per child.
