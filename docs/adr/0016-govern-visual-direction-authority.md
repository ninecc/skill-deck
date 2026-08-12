# Govern visual direction authority
status: accepted

## Context

Prototype G, the historical `ux-design.md`, earlier UI Audits, archived design
tasks, and the production UI capture choices that followed them record useful
project history. They no longer have default authority over future visual work.
The current theme names and Graphite/Azure/Sand/Plum palette family, token names
and values, typography, density, spacing, radii, shadows, brand marks, and
component shapes are current implementation rather than a required future
visual baseline.

This includes the square `S`, the published App Icon, and concepts under
`design/app-icon-concepts/`. Those assets remain untouched and usable by the
current product, but have no default authority over a future brand direction.
The `Skill Deck` product name and its domain language remain active contracts.

## Decision

We call a prior visual choice that has lost default authority but remains in
project history a **Historical Visual Direction**. Prototype G, historical
design documents and audits, archived task designs, unpromoted screenshots and
prototypes, and the visual assets listed above have this status. They may still
support a precisely cited interaction, accessibility, functional, or failed-
direction conclusion; citing one does not restore the whole artifact's visual
authority. `Historical Visual Direction` is distinct from `Legacy App State`,
which names the removed lifecycle data model.

Only an **Approved Visual Direction** may authorize new visual implementation.
It exists when the user unambiguously approves a reviewable, task-local design
direction and identifies both its UI scope and platform scope. Vague positive
feedback, current production code, platform HIG guidance, prototypes, agent
self-assessment, passing tests, and design-skill output do not create one.
This rule does not prescribe how many alternatives, screenshots, or skills a
future task must use.

An Approved Visual Direction applies only to the states, components, screens,
and platforms it explicitly covers. Uncovered areas retain their current
implementation. A later approval replaces an earlier one only where their
declared scopes overlap. Approval for macOS does not approve a visual direction
for Windows or Linux; changes to shared implementation on unapproved platforms
must be only those necessary to support the approved scope without reducing
existing usability.

Visual authority is subordinate to product requirements, domain language,
accessibility, safety boundaries, platform behavior, and accepted architecture
ADRs. A direction cannot implicitly revise those contracts. Any such change
must first be explicitly scoped and accepted in the appropriate task. Platform
guidelines remain authoritative for platform behavior, native-control
compatibility, and usability safeguards, but their aesthetic advice becomes
visual authority only when incorporated into an Approved Visual Direction.

During a task, its approved artifacts are the visual authority for that task.
A decision intended to constrain later tasks must be distilled into an active
`.trellis/spec/` document or ADR before completion. Any exploration, screenshot,
or prototype not promoted this way becomes Historical Visual Direction after
archival. If the user explicitly withdraws an Approved Visual Direction, its
artifacts and history remain intact but the withdrawn scope is reclassified as
Historical Visual Direction.

Archived design documents and prototypes must not be added wholesale to a
future visual implementation or visual-review context manifest. When one is
needed for a non-visual contract, the manifest `reason` must identify the exact
behavioral, accessibility, or counterexample evidence required.

## Preserved contracts

Theme Accent remains separate from native System Accent as established by
ADR-0014, and success, warning, and danger remain independent semantic roles.
Semantic styling must continue to deliver readable contrast, distinguishable
selection and focus, `focus-visible`, reduced-motion behavior, keyboard
operation, recognizable status, and system-control compatibility. These are
required outcomes, not a commitment to the current token names, count, layers,
or values; a future Approved Visual Direction may safely migrate that taxonomy
while preserving the outcomes.

The current Theme Preference runtime values remain
`system | light | dark | sand | plum`. Removing their visual authority does not
change that product behavior or its persistence schema. Any runtime change
requires a separately planned migration. ADR-0012's single-window utility
model, ADR-0013's cross-platform Command Model, and ADR-0015's single Application
Command authority also remain fully effective. Command discoverability, status
feedback, keyboard paths, focus restoration, responsive operability, current
domain terms, and localization are not visual prescriptions and remain intact.

## Partial supersession and current state

This ADR partially supersedes ADR-0014 only where ADR-0014 required preserving
five complete themes and their then-current visual identity. It does not
supersede the Theme Accent/System Accent separation or independent semantic
roles.

This decision governs the source, scope, priority, and lifecycle of visual
authority. It introduces no palette, style, layout, brand, or component
prescription. On acceptance of this ADR, the project has no Approved Visual
Direction. One can be established only by a later, independent design task that
meets the approval requirements above.
