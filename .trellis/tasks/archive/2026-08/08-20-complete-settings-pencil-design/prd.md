# Complete Settings Pencil design

## Goal

Complete the Settings portion of `pen-design/skill-deck.pen` so the design
artifact documents the real desktop Settings information architecture and its
important interaction states, rather than only the existing Appearance view.

The result should be implementation-ready for the current Skill Deck desktop
application and remain visually consistent with the reviewed Dark reference
compositions and shared Pencil component library.

## Background

- Pencil currently contains `Reference 13 · Wide Dark Settings`, whose visible
  content represents the Appearance section.
- The production `SettingsDialog` defines five sections: General, Appearance,
  Translation, Installation, and About (`src/SettingsDialog.tsx`).
- General changes the interface language.
- Appearance selects System, Light, Dark, Sand, or Plum.
- Translation selects a target language and edits an optional HTTP(S) proxy;
  the target language saves immediately, while the proxy changes only after
  Apply and can show a validation error.
- Installation selects automatic/copy install behavior and automatic/explicit
  Agent targets; explicit targets support filtering, selected counts, disabled
  automatic mode, and a no-match result.
- About displays the resolved Skills CLI version or an unavailable placeholder.
- The existing dialog uses a 720px-wide, bounded desktop modal with a horizontal
  five-item section navigation and fixed header/footer.

## Requirements

- R1. Preserve the existing Settings modal chrome, dimensions, navigation,
  backdrop treatment, and reference visual language while allowing deliberate
  improvements to spacing, grouping, and information hierarchy.
- R2. Represent every production Settings section with authentic labels,
  controls, help text, save behavior, and footer messaging.
- R3. Reuse existing shared components and theme variables wherever applicable;
  do not introduce an unrelated visual system.
- R4. Keep all screen content fully visible, aligned, unclipped, and readable at
  the existing Wide Dark reference size.
- R5. Name every new Pencil node clearly and keep root/reference hierarchy clean.
- R6. Do not change React, Rust, CSS, tests, or other production behavior; this
  task changes only the Pencil design artifact and Trellis planning records.
- R7. Include the agreed key interaction states: Translation proxy validation
  error, Installation explicit Agent targets, and Installation filter with no
  matching targets.
- R8. Deliver one 1180 × 800 full-application Settings context showing the real
  default-open General section, plus one complete eight-state Dark modal matrix.
- R9. Add two isolated proof states: Light Appearance with Light selected, and
  Dark Chinese Installation automatic mode for localization stress testing.
- R10. Keep Installation at the production dialog height and visibly express a
  scrolling content region instead of expanding the modal or truncating the
  existence of additional Agent targets.
- R11. Show Agent targets alphabetically in the design recommendation and label
  the current implementation difference; production sorting is deferred.
- R12. Add a compact behavior rail covering immediate saves, proxy Apply-only
  persistence, content scrolling, default-open General, and alphabetical Agent
  ordering as a design recommendation.
- R13. Remove the non-production `Saved` badge from Settings states.

## Acceptance Criteria

- [ ] One full-application Wide Dark reference shows Settings opening on General.
- [ ] The Dark Settings matrix contains eight reviewable modal states: General,
  Appearance, Translation default, Translation proxy error, Installation
  automatic, Installation explicit targets, Installation no matches, and About.
- [ ] One Light Appearance proof and one Dark Chinese Installation proof isolate
  theme and localization variables respectively.
- [ ] Active navigation, headings, control values, save/apply copy, and footer
  copy match the production Settings contract.
- [ ] Translation error uses the masked invalid value
  `http://user:••••@proxy.example` and the production validation message.
- [ ] Installation uses a realistic fixed-height scrolling target list; the
  explicit state filters with `cod`, selects `codex`, and shows the selected count.
- [ ] Agent ordering is alphabetical in the design, and the behavior rail states
  that current production code still preserves upstream registration order.
- [ ] About includes the normal CLI version and a compact `—` unavailable-value
  specimen without duplicating the full modal.
- [ ] No Settings state contains the unimplemented `Saved` badge.
- [ ] Existing reviewed references and shared components are not regressed.
- [ ] Pencil structural validation reports no placeholders, unnamed new nodes,
  collapsed layouts, clipping, or unresolved warnings in the changed scope.
- [ ] Final screenshots are reviewed for hierarchy, spacing, contrast,
  alignment, and consistency with the existing Dark theme.

## Out of Scope

- Changing application code or Settings behavior.
- Redesigning the shared theme/token system.
- Adding new Settings options not present in the production application.
- Reworking non-Settings reference compositions.
- Changing production Agent ordering in this task; capture it as a follow-up.

## Key Decisions

- Use the existing Wide Dark Settings composition as the visual source, but make
  General the full-context default-open reference and retain Appearance in the
  modal matrix.
- Add the agreed validation/filter states rather than limiting the artifact to
  one canonical view per section.
- Keep the task design-only and place the completed state family inside the
  existing Dark reference appendix rather than adding unrelated document roots.
- Use English for the canonical states, plus one Chinese Installation automatic
  proof; keep Light Appearance separate from localization testing.
- Use behavior-accurate production copy and controls, allow layout refinement,
  and explicitly mark the alphabetical-order recommendation as not yet shipped.
