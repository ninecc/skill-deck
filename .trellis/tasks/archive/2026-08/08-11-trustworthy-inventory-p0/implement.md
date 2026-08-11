# Implementation plan: trustworthy inventory P0

1. Add the derived installation status, typed reconciliation evidence, Rust-owned installation actions, and package-level reconciliation/actions/Managed Library diagnostic DTOs to `inventory.rs` and `api.ts`; add DTO serialization/type regressions.
2. Refactor the existing lifecycle/configuration validation primitives only as far as needed for a single non-mutating classifier; cover all six states and precedence with temporary filesystem fixtures.
3. Extend Restore with explicit `recreate | replace` previews to recreate missing Copy Fallback and linked projections using existing install helpers, preserving plan/commit stale checks, overwrite confirmation only for replacement, and explicit missing-root confirmation.
4. Allow Detach for structurally valid drifted Copy Fallback content while keeping missing, retargeted, linked-broken, invalid-content, and library-broken cases blocked.
5. Add state-only Forget Installation plan/commit behavior for missing/retargeted/broken records, reject status-stale plans, and prove path, target, and configuration bytes remain unchanged.
6. Safely extend Remove from Library for absent or real app-owned broken zero-Installation package roots without weakening package-root topology checks.
7. Render distinct Broken Managed wording, installation status, package diagnostic, typed expected/observed evidence, provenance/deferred-check disclosure, and backend-provided installation/package actions in `App.tsx`; omit Healthy Restore, freeze expansion actions for abnormal packages, and add both locale catalogs and behavioral React tests.
8. Run focused tests while iterating, then the full backend and frontend quality gates.
9. Update `.trellis/spec/backend/command-contracts.md` and `CONTEXT.md` with the finalized reconciliation and lifecycle contracts.

## Validation

```bash
npm run format:check
npm run lint
npm run typecheck
npm test -- --run
npm run build
cargo fmt --check --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --all-features
```

## Risk and rollback points

- `src-tauri/src/inventory.rs`: classification must remain read-only and must not hide external attention entries beyond existing managed path matching.
- `src-tauri/src/lifecycle.rs` / `revision.rs`: relaxing Detach or expanding Restore must be status-specific; do not weaken generic mutation validation.
- `src-tauri/src/install.rs`: expose only the minimum projection helper needed by Restore; do not introduce an Agent adapter framework.
- `src/App.tsx`: unhealthy rows must not retain hidden destructive controls through old unconditional rendering.
- `src-tauri/src/diagnostics.rs`: exported reports must not inherit local resolved-target or full-fingerprint evidence.
- Persisted state remains unchanged, so rollback is code-only.

## Start gate

- PRD, design, and implementation plan reviewed.
- User explicitly approves this final plan in a subsequent message.
- Then run `task.py start` and load `trellis-before-dev` before product edits.
