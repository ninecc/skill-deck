# Implementation plan: Agent projection contract smoke tests

1. Add the test-only crate module and minimal `#[cfg(test)]` wrappers or existing root-aware seams required to orchestrate managers without environment mutation or production visibility expansion.
2. Implement Codex and Claude projection lifecycle smoke tests with real Local import, native linked installation, semantic configuration/ownership assertions, unrelated-field preservation, drift detection/resolution, missing detection/Restore, and final healthy reconciliation.
3. Add one separate retargeting/Forget/External transition test with external-target byte preservation; rely on existing P0 regressions for host-neutral Copy Drift and Managed Library damage.
4. Add the focused contract-smoke command to every native package CI job before packaging.
5. Update the backend command contract spec with projection-smoke/native-CI requirements, official source links, and last-reviewed date.
6. Run focused smoke tests, then all existing frontend/backend quality gates.

## Validation

```bash
cargo test --manifest-path src-tauri/Cargo.toml --all-features projection_contract_smoke
cargo fmt --check --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --all-features
npm run format:check
npm run lint
npm run typecheck
npm test -- --run
npm run build
```

## Risks

- Tests must not call production methods that resolve the real OS home; use injected roots consistently through plan, commit, resolve, and inventory refresh.
- Windows junction assertions execute only on the Windows CI runner; local macOS/Linux cannot claim that result.
- macOS native-runner success does not prove both architectures inside the universal bundle.
- Avoid sleeps, global test serialization, environment locks, snapshot fixtures, or new dependencies.
- Do not turn Agent Runtime Recognition or model-dependent invocation into this task's acceptance criteria.

## Start gate

- User reviews this scoped roadmap step and explicitly approves implementation in a subsequent message.
