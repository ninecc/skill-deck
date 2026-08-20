# Quality check

## Findings fixed

- `Translation · Default` displayed `Optional HTTP(S) proxy` as an input value,
  which contradicted the production blank draft and the task contract. Removed
  the value node so the field is genuinely empty while its help text remains.
- Dark and Light Appearance proofs labeled the System theme as `System` rather
  than the production `System (Default)`. Restored the exact label and adjusted
  its compact type size to fit the existing desktop tile.
- The explicit `cod` filter omitted `codearts-agent` and included the later
  `forgecode`, so the visible result slice was not alphabetical. Added
  `codearts-agent` first and kept the visible sequence through `command-code`,
  with `codex` selected.
- The default-open General states documented initial focus in the behavior rail
  but did not show it. Added the semantic focus stroke to General in both the
  full application context and General matrix state.

## Structural and visual verification

- Pencil active canvas: `pen-design/skill-deck.pen`.
- Reviewed screenshots for the full General context, both Dark matrix rows,
  each individual Dark state, Light Appearance proof, Chinese Installation
  proof, and desktop behavior rail.
- Resolved-instance scans of the Dark reference appendix and changed Settings
  roots report zero placeholders, unnamed nodes, zero bounds, clipping
  problems, or broken references.
- Verified fixed 720 × 548 modal chrome, independent 112px Agent list viewport
  with scrollbar, active section indicators, initial focus, header/footer Close
  affordances, disabled automatic targets, selected `codex`, no-match result,
  masked invalid proxy, About `1.5.22` and `—`, and the production-order delta.
- Repository scope contains only `pen-design/skill-deck.pen` plus this task's
  Trellis artifacts; no production source changed.

## Commands

- `npm run format:check` — pass
- `npm run lint` — pass
- `npm run typecheck` — pass
- `npm test -- --run` — pass (7 files, 34 tests)
- `npm run build` — pass

No `.trellis/spec/` update is needed: the fixes enforce this task's existing
design and production-copy contracts without introducing a reusable convention.
