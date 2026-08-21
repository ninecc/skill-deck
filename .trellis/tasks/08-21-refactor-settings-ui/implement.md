# Implementation plan

- [x] Map current Settings DOM/state to the Pencil matrix and behavior rail.
- [x] Refactor stable header/navigation/content/footer dialog composition.
- [x] Implement General and Appearance, including five theme tiles.
- [x] Implement Translation default and invalid-proxy states with Apply boundary.
- [x] Implement Installation automatic/explicit/no-match states and scrolling.
- [x] Implement About available/unavailable version presentation.
- [x] Preserve upstream Agent ordering and localization copy contracts.
- [x] Extend persistence, validation, focus, keyboard and filtering tests.
- [x] Compare all ten Settings proof states at approved sizes/themes/locales.
- [x] Run full frontend quality gate and Trellis check.
- [ ] Commit and archive this child before Child 5 starts.

## Rollback points

- Dialog chrome, each Settings section and Agent-list styling are separate
  checkpoints; no preference-schema migration is planned.
