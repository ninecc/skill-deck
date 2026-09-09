# Repair package CI gate

## Goal

Make the package workflow execute a real current upstream-CLI contract test and fail rather than silently passing when its selected test no longer exists.

## Background

`.github/workflows/ci.yml` filters Cargo tests by the deleted `projection_contract_smoke`, producing a successful zero-test run.

## Requirements

- R1: Replace the stale projection-era test filter with a current upstream-CLI contract or integration test target.
- R2: A missing or renamed selected test must make the CI step fail.
- R3: Keep the gate non-mutating and hermetic; it must not touch the user's global Skills or require the live catalog.
- R4: Keep the workflow portable across its declared runners.

## Acceptance Criteria

- [x] The package contract command executes at least one current test.
- [x] An intentionally nonexistent selector exits nonzero under the chosen mechanism.
- [x] The selected test covers a current accepted lifecycle/runtime contract, not superseded projection behavior.
- [x] Workflow syntax remains valid and the exact local command passes.

## Out of Scope

- Adding native installer smoke to CI.
- Restoring Managed Library/projection tests.
- Broad workflow restructuring.
