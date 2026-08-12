# Implementation Plan

## 1. Lock down the regressions

- [ ] Add Rust tests for proxy validation, the shared operation deadline,
      fully sanitized provider errors and the
      off-event-loop command bridge.
- [ ] Add React tests for immediate startup chrome and translation Retry while
      original content remains available.

## 2. Move blocking work off the event loop

- [ ] Add the smallest shared `spawn_blocking` bridge in the Tauri command layer.
- [ ] Convert every blocking handler, including preview tree/read and Reveal, to
      async wrappers without changing existing DTOs.
- [ ] Add a tested Node toolchain resolver: inherited PATH first, then macOS
      `/opt/homebrew/bin` and `/usr/local/bin`; require sibling `node`/`npx` and
      keep absolute paths in the pinned CLI session.

## 3. Bound and configure translation networking

- [ ] Add validated optional proxy input to `translate_preview`.
- [ ] Use a 5-second connect timeout and one 15-second deadline shared by all
      chunks in a translation operation.
- [ ] Publish translated text only after all chunks succeed; test that timeout
      or a later chunk failure exposes no partial translation.
- [ ] Preserve automatic environment proxy behavior when override is blank.
- [ ] Sanitize connection, timeout, HTTP status, decode and response-shape
      failures so content/query URLs and reqwest details never reach the UI.

## 4. Complete responsive UX

- [ ] Render the full Header and startup progress before runtime completion;
      keep Settings/locale usable and disable runtime/Inventory controls.
- [ ] Add a dialog-local no-credentials proxy draft; persist only a valid value
      after Apply, while retaining Rust trust-boundary validation.
- [ ] Add translation Retry, latest-request-wins invalidation and localized
      English/Chinese copy.
- [ ] Add stable runtime error codes and localized actionable startup errors;
      never render raw spawn/PATH/OS error text.

## 5. Verify

- [ ] Frontend format, lint, typecheck, tests and production build.
- [ ] Rust fmt, Clippy with warnings denied and tests.
- [ ] macOS Computer Use smoke for delayed startup, failed translation, Retry
      a local unauthenticated proxy, and a release app launched with Finder-like
      PATH.

Rollback point: revert the task commit; no migration or external state changed.
