# Implementation Plan

1. Update the Rust startup DTO and manager so one validated list result is
   returned by `runtime_status`/`retry_runtime`; remove the unused `list_skills`
   command boundary and add a focused required-Inventory serialization check.
2. Update the TypeScript DTO and initial effect to publish runtime and Inventory
   together, including Retry; remove the redundant frontend list wrapper.
3. Split startup/translation loading copy, remove Inventory Agent tags, compact
   row styles, add a reduced-motion-safe CSS spinner, and replace the centered
   Preview placeholder with a small top hint.
4. Update React tests to assert one-command startup and Retry, atomic Inventory
   publication, distinct loading/empty/filter-empty semantics, the decided
   visible filtering behavior, compact placeholder, and translation-specific
   loading copy. Avoid adding process-injection infrastructure solely to count
   Rust child processes.
5. Run frontend format, lint, typecheck, tests, build; run Rust fmt, clippy and
   tests; update the desktop command contract; perform a full-scope Trellis
   check.
6. Move existing catalog search and source install markup into a reusable-style
   right sheet opened from the Inventory heading; add only the local open state,
   close control, responsive CSS, i18n copy, and focused behavioral tests needed
   for this placement.
7. Build/launch the current app, apply `http://127.0.0.1:7890`, and run a real
   minimal Installed Skill translation. If it still times out, preserve the
   red-capable loop, rank/test hypotheses, add a focused regression test, and
   change only the proven root cause. Re-run affected gates and full check.

## Runtime verification result

See `verification.md`. The proxy was proven end-to-end with a short Installed
Skill; the long-document scheduling defect was reproduced and fixed. Final
post-fix GUI completion is blocked because the local proxy subsequently began
failing even minimal `hello` TLS connections.
8. Batch escaped Markdown fragments as strictly indexed inert spans within the
   provider-size bound, run at most four batches on scoped workers sharing the
   existing deadline, validate/decode markers, and cover packing, order,
   concurrency, malformed responses, and atomic failure without network access.

## Validation commands

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

## Rollback point

The cross-layer DTO must be changed atomically in Rust, `src/api.ts`, and
`src/App.tsx`. If validation fails, revert that product-code batch; no user data
or installed Skill state is mutated by this task.
