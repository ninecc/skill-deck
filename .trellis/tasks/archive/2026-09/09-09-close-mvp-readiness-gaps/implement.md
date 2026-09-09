# MVP readiness remediation execution plan

## Ordered checklist

1. Start Search, CI, and Update All children with non-overlapping ownership.
2. For each child, reproduce the current defect with a focused behavior check before changing source.
3. Implement the smallest contract-compliant fix and run the child-specific verification.
4. Complete Update All before starting UI state semantics; rebase the UI child's assumptions on the resulting `App.tsx`.
5. Run UI deterministic review for empty, loading, translation-loading, and narrow translation-error states.
6. Integrate all children and run frontend format, lint, typecheck, tests, build; then Rust format, Clippy, and tests.
7. Request independent cross-review against the parent/child acceptance criteria.
8. Record remaining fake-CLI, native-smoke, external-provider, and visual-direction work as deferred rather than claiming release readiness.

## Validation commands

- `npm run format:check`
- `npm run lint`
- `npm run typecheck`
- `npm test -- --run`
- `npm run build`
- `cargo fmt --check --manifest-path src-tauri/Cargo.toml`
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`
- `cargo test --manifest-path src-tauri/Cargo.toml --all-features`

## Review gates

- No child may modify another active child's owned files.
- UI state semantics starts only after Update All is integrated.
- No real add/update/remove is run against the user's global Skills.
- Browser review proves WebView behavior only; native evidence remains open.
