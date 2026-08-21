# Implementation plan

- [x] Confirm all predecessor tasks are archived with passing checks.
- [x] Build a visual verification matrix for every Pencil reference and proof.
- [x] Run production Vite and native Tauri smoke at 1180×800 and 720×520.
- [x] Record the prior successful three-platform packaging pipeline, the
      current shared-UI build evidence and the unavailable Windows/Linux native
      smoke separately from the current macOS native result.
- [x] Exercise all themes, locales, transient surfaces, dialogs and state paths.
- [x] Verify every state is reproducible through the review entry without real
      network, CLI or user-data mutation.
- [x] Verify review fixtures use canonical/pressure data, remain documented and
      never enter production output.
- [x] Verify keyboard/focus, reduced motion, scrolling, contrast and stacking.
- [x] Inspect production output for Iconify API endpoints/runtime icon loading.
- [x] Inspect production output for review entry, scenario IDs and fixture data.
- [x] Compare bundle size to baseline and confirm only used glyphs ship.
- [x] Fix verified integration regressions at their owning shared seams.
  - [x] On every file-tree open, reset the roving focus path to the currently
        previewed file before the popover mounts; native WebView smoke exposed
        stale focus reopening on a previously focused first row.
  - [x] Stop the tree Escape event before it reaches the window-level Escape
        handler; closing the popover must preserve the selected Skill/Preview.
- [x] Run `npm run format:check`, `npm run lint`, `npm run typecheck`,
      `npm test -- --run` and `npm run build`.
- [x] Dispatch full-scope Trellis check and resolve verified findings.
- [x] Record unavailable native checks and update reusable specs.
- [x] Commit this integration child.
- [ ] Archive this child, then complete and archive the parent task.

## Rollback points

- Each integration fix is an isolated checkpoint; no broad restyling is allowed
  during final verification.
