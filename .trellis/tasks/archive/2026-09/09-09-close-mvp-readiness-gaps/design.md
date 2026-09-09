# MVP readiness remediation design

## Boundary

The parent coordinates four child tasks and owns integration review only. Product changes belong to the children. The accepted ADR/spec hierarchy and upstream Skills CLI ownership model remain unchanged.

## Work Graph

```text
restore-catalog-search-isolation ─┐
repair-package-ci-gate ───────────┼─> integration quality gate
require-update-all-confirmation ──┤
                                 └─> correct-ui-state-semantics ─> visual/runtime review
```

Search, CI, and Update All may execute concurrently. UI state semantics waits for Update All because both modify `src/App.tsx` and related frontend tests.

## Ownership

- Search: `src-tauri/src/cli.rs` and focused Rust tests/seam.
- CI: `.github/workflows/ci.yml`; may consume a test added by Search but must not edit Search-owned Rust while that child is active.
- Update All: confirmation state/rendering and its focused tests in `src/App.tsx` / `src/App.test.tsx`.
- UI states: subsequent edits to `src/App.tsx`, `src/styles.css`, review scenarios, and focused tests.
- Parent: integration reconciliation and final evidence only.

## Contracts

- Search remains a read-only HTTP catalog operation independent of CLI session state.
- All lifecycle mutations remain serialized through the existing backend path.
- Update All crosses the frontend confirmation boundary exactly once.
- UI state fixes derive from existing runtime/translation/inventory state; no persisted flags are introduced.
- The CI gate must establish both nonzero selected-test count and test success.

## Rollback

Each child is independently reversible. If UI integration exposes a conflict, retain completed backend/CI work and roll back only the owning frontend child. No migration or persisted-data rollback is required.
