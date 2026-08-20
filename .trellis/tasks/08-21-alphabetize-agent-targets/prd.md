# Alphabetize Agent targets

## Goal

Follow up the Pencil UI refactor with the separately approved behavior change
that displays explicit Agent targets in stable alphabetical order instead of
upstream registration order.

## Requirements

- Sort the presented explicit Agent target list alphabetically using a stable,
  locale-independent comparison appropriate for agent identifiers.
- Do not change stored preference values, automatic detection, filtering,
  selection semantics or upstream CLI registration order.
- Apply sorting at the presentation boundary so backend/domain order remains
  authoritative elsewhere.
- Preserve keyboard navigation, selected-count behavior and English/Chinese UI.

## Acceptance Criteria

- [ ] The full explicit target list and filtered results are alphabetized.
- [ ] Existing selections remain selected and persist unchanged.
- [ ] Automatic detection and install command payloads are unchanged.
- [ ] Settings behavior tests and the full frontend quality gate pass.

## Out of scope

- The active Pencil UI refactor, backend target ordering or preference migration.
