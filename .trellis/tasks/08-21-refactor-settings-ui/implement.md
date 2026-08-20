# Implementation plan

- [ ] Map current Settings DOM/state to the Pencil matrix and behavior rail.
- [ ] Refactor stable header/navigation/content/footer dialog composition.
- [ ] Implement General and Appearance, including five theme tiles.
- [ ] Implement Translation default and invalid-proxy states with Apply boundary.
- [ ] Implement Installation automatic/explicit/no-match states and scrolling.
- [ ] Implement About available/unavailable version presentation.
- [ ] Preserve upstream Agent ordering and localization copy contracts.
- [ ] Extend persistence, validation, focus, keyboard and filtering tests.
- [ ] Compare all ten Settings proof states at approved sizes/themes/locales.
- [ ] Run full frontend quality gate and Trellis check.
- [ ] Commit and archive this child before Child 5 starts.

## Rollback points

- Dialog chrome, each Settings section and Agent-list styling are separate
  checkpoints; no preference-schema migration is planned.
