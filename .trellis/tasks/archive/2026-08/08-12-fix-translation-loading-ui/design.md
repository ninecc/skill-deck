# Technical Design

## Boundaries

Keep the existing React-local state, CSS Grid shell, blocking reqwest client,
Tauri command signature, proxy preference and 15-second operation deadline.
No new component, state store, dependency or provider is needed.

## Skill-scoped translation UI

`chooseSkill` and `chooseFile` are the Installed Skill/document selection
boundaries. In the same synchronous transition that invalidates the translation
generation and clears Preview state, set `translationOn` to false and return
the mobile pane to `original`. The next Preview therefore renders one pane and
makes no translation command until the user explicitly enables translation
again.
Returning to the previous Skill does not restore its result or translated
layout.

Keep the existing translation key and generation checks. They continue to
reject late results from the old Skill/document; no per-Skill cache or state
map is introduced. File selection inside one Skill follows the same reset rule;
no newly selected document is sent automatically.

## Startup Grid

The app shell is one explicit column. Header, runtime screen and workspace all
occupy that column; runtime screen and workspace may share rows because the
workspace is inert/hidden while runtime is pending. This prevents auto-placement
from creating a second implicit column and keeps the Header full width.

Use CSS only. Do not conditionally mount a second startup shell or duplicate
Header markup. Keep Update All visible but disabled until runtime readiness;
Settings and locale remain enabled, so the Header does not reflow on success.

## Bounded translation retry

Keep the 5-second connect timeout and 15-second operation deadline. Each
provider request gets at most two attempts. Cap an attempt below half the
operation deadline (7 seconds), using the smaller of that cap and the remaining
deadline. Retry only reqwest connect/timeouts; status, decode, marker and
response-shape failures remain terminal.

The retry is an internal provider detail. React continues to render the existing
`translating` state until success or final failure; no retry DTO/state/copy is
added.

Ending a Translation Session invalidates publication but does not cancel the
already-running blocking Tauri command. Its existing shared deadline bounds the
remaining work, including any internal retry. Applying a new target language or
proxy is an explicit translation action: invalidate the old generation and
start the current document with the new parameters while keeping the translated
layout open.

The shared `Instant` is unchanged, so retries across chunks/batches cannot
extend the operation past 15 seconds. Markdown workers still accumulate
privately and publish only when every batch succeeds. Add a small endpoint
parameter at the existing provider-call seam so a stdlib local HTTP server can
deterministically prove first-attempt timeout followed by success without
contacting Google.

## Compatibility and rollback

No DTO or persisted preference changes. Update the backend command contract to
record the bounded retry while preserving the deadline. Rollback is the product
code/spec commit; no migration or cleanup is required.
