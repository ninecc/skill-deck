# Settings Pencil design

## Authority and boundary

Production behavior and copy come from `src/SettingsDialog.tsx`,
`src/preferences.ts`, and `src/i18n.ts`. The existing
`Reference 13 · Wide Dark Settings` composition and shared Pencil masters are
the visual authority. Only `pen-design/skill-deck.pen` and this task's Trellis
artifacts may change.

## Deliverable architecture

### Context anchor

One 1180 × 800 Wide Dark composition shows Settings immediately after opening:
General is active, System language is selected, the scrim and underlying app are
visible, and the non-production `Saved` badge is absent. This replaces
Appearance as the canonical full-context reference without deleting the
Appearance design source.

### Dark modal matrix

One focused matrix contains eight complete 720px-wide Settings modal states:

1. General — System language selected.
2. Appearance — Dark selected with the established five theme tiles.
3. Translation — target language, blank optional proxy, and Apply proxy.
4. Translation / Invalid proxy — masked
   `http://user:••••@proxy.example`, inline error, and Apply-only notice.
5. Installation / Automatic — automatic method and Agent detection, with target
   options visibly disabled inside a fixed-height scrolling content region.
6. Installation / Explicit targets — alphabetical targets filtered by `cod`,
   `codex` selected, selected count visible, and checkboxes enabled.
7. Installation / No matches — explicit mode with a clearly non-matching query
   and the production empty-result message.
8. About — current sample Skills CLI version plus a compact `—` unavailable
   value specimen within the same state.

The matrix includes Appearance even though a full application source already
exists because the matrix's job is side-by-side state comparison.

### Isolated proofs

- Light Appearance — Light theme resolved and Light selected, English copy.
- Dark Chinese Installation / Automatic — Chinese navigation, labels, helper
  copy, and scrolling list at the same geometry as the English modal.

Theme and localization proofs remain separate so contrast and text-expansion
failures have one changed variable each.

### Behavior rail

A compact handoff rail states: default open is General; non-proxy changes save
immediately; proxy changes require Apply proxy; Settings content scrolls inside
fixed chrome; Agent targets are alphabetized in the design recommendation while
current production preserves upstream registration order.

Place the context anchor, matrix, proofs, and behavior rail inside the existing
Dark reference appendix/adjacent Settings documentation region, extending
containers only as required. Do not create unrelated document roots.

## Construction strategy

- Query the source Settings composition by stable human-readable node names at
  execution time; do not depend on stale descendant IDs.
- Copy the existing full composition with `placeholder: true` for the General
  context anchor. Derive modal-only states from the existing dialog geometry and
  customize copied descendants within `Copy`; replace the settings-content
  subtree with a complete state-specific tree when the section differs.
- Reuse the existing dialog header/footer component instances, nav geometry,
  theme variables, typography, radii, separators, controls, and scrim tokens.
- Active section is conveyed by the established accent underline/wash. Inputs,
  radios, checkboxes, helper text, validation error, counts, and empty text use
  the production hierarchy without extra cards or decorative containers.
- Clear each copied root placeholder immediately after that state is complete
  and verified.
- Preserve fixed header, navigation, and footer chrome while Installation's
  content viewport visibly scrolls.

## Content and interaction contracts

- All non-proxy controls say that changes save immediately.
- Translation states say proxy changes apply only through Apply proxy.
- Invalid proxy uses the production rule and error copy; no credential is
  persisted or presented as a successful value.
- Automatic Agent mode visibly disables target checkboxes.
- Explicit mode enables target selection and shows a selected count.
- No-match mode keeps the filter query visible and shows the empty-result copy.
- About displays `Skills CLI version` and the same sample CLI version already
  used elsewhere in the design artifact.
- General is the default-open state. No static `Saved` badge appears.
- Canonical states are English; one separate Chinese Installation proof tests
  localization length and wrapping.
- Agent rows use alphabetical order in the design. The behavior rail makes the
  current implementation delta explicit rather than silently claiming it ships.

## Verification and rollback

After every state batch, inspect the changed subtree for placeholders, unnamed
nodes, zero or collapsed bounds, overflow/clipping, and broken references.
Capture screenshots of the context anchor, eight-state matrix, Light proof, and
Chinese proof. Review active navigation, copy, alignment, contrast,
enabled/disabled treatment, error treatment, scrolling, localization expansion,
theme resolution, and dialog consistency.

The rollback boundary is the changed Settings family and any appendix sizing
updates inside the single `.pen` file. Fix reviewed nodes directly; do not
delete and recreate completed state frames.
