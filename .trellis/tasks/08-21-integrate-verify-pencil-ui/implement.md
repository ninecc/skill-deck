# Implementation plan

- [ ] Confirm all predecessor tasks are archived with passing checks.
- [ ] Build a visual verification matrix for every Pencil reference and proof.
- [ ] Run production Vite and native Tauri smoke at 1180×800 and 720×520.
- [ ] Record successful three-platform builds and any unavailable Windows/Linux
      native smoke separately from the macOS native result.
- [ ] Exercise all themes, locales, transient surfaces, dialogs and state paths.
- [ ] Verify every state is reproducible through the review entry without real
      network, CLI or user-data mutation.
- [ ] Verify review fixtures use canonical/pressure data, remain documented and
      never enter production output.
- [ ] Verify keyboard/focus, reduced motion, scrolling, contrast and stacking.
- [ ] Inspect production output for Iconify API endpoints/runtime icon loading.
- [ ] Inspect production output for review entry, scenario IDs and fixture data.
- [ ] Compare bundle size to baseline and confirm only used glyphs ship.
- [ ] Fix verified integration regressions at their owning shared seams.
- [ ] Run `npm run format:check`, `npm run lint`, `npm run typecheck`,
      `npm test -- --run` and `npm run build`.
- [ ] Dispatch full-scope Trellis check and resolve verified findings.
- [ ] Record unavailable native checks, update specs if needed and commit.
- [ ] Archive this child, then complete and archive the parent task.

## Rollback points

- Each integration fix is an isolated checkpoint; no broad restyling is allowed
  during final verification.
