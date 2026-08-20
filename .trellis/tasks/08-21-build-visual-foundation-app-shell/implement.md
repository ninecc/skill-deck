# Implementation plan

- [x] Load frontend specs and inspect every current icon call site and token use.
- [x] Add build-time Iconify/Lucide dependencies and Vite integration.
- [x] Replace `src/icons.tsx` path data with typed static icon components.
- [x] Add icon tests and verify no runtime Iconify endpoint remains in `dist`.
- [x] Record pre/post production bundle size and confirm only used icons ship.
- [x] Normalize the five-theme token architecture in `src/styles.css`.
- [x] Add the dev/review-only typed IPC entry and initial shell scenarios.
- [x] Add canonical Pencil and long/empty/Chinese fixtures plus usage docs.
- [x] Verify the normal production graph excludes review code and fixture data.
- [x] Refactor Toolbar/Inventory/Preview/Status markup and styling to Pencil.
- [x] Add or update behavior tests without snapshotting decorative markup.
- [x] Review Dark and Light at 1180×800, then all themes at 720×520.
- [x] Capture the owned Dark shell states at both sizes and one representative
      shell state in System, Light, Sand and Plum for user review.
- [x] Present a labeled contact sheet, matrix and discrepancy list; expand only
      disputed frames.
- [x] Verify native OS title bars remain system-provided and outside React.
- [x] Run representative native Tauri shell smoke in addition to the review
      entry's complete deterministic matrix.
- [x] Run `npm run format:check`, `npm run lint`, `npm run typecheck`,
      `npm test -- --run` and `npm run build`.
- [x] Run Trellis check and fix verified findings before Child
      2 starts.
- [x] Present screenshots and runtime evidence to the user and obtain explicit
      visual approval before starting Child 2.
- [x] Apply review corrections; no additional correction was requested after
      visual approval.
- [ ] Create the child's final commit and archive.
- [x] Rerun affected frames plus representatives; run the complete child matrix
      when shared tokens/layout/components changed.

## Risk and rollback points

- Dependency/config changes: revert independently if bundle generation fails.
- Global tokens: compare all five themes before landing them.
- Shell CSS: explicitly check grid columns and overflow at both approved sizes.
