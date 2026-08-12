# Implementation Plan

1. Add a focused React regression that selects Skill A, enables translation,
   switches to Skill B before A settles, and asserts single-pane/off state with
   no stale result or automatic B translation. Run it red.
2. Reset translation mode and mobile pane at the existing `chooseSkill` and
   `chooseFile` boundaries. Cover switching back without restoration and run
   the focused React tests green. Preserve automatic retranslation after an
   explicit target-language/proxy Apply and generation rejection for old
   background requests.
3. Make the app shell a single explicit CSS Grid column and smoke the runtime
   pending view at desktop width to confirm the Header/loading area span the
   window.
4. Add a deterministic Rust regression using a stdlib local HTTP server that
   makes the first attempt time out and the second succeed within the same
   deadline. Run it red.
5. Add one bounded retry at the existing provider request loop: two attempts,
   7-second per-attempt cap, retry only connect/timeout, keep the 15-second
   shared deadline and atomic publication. Run translation tests green.
6. Update `.trellis/spec/backend/command-contracts.md` with the bounded retry
   contract.
7. Run frontend format, lint, typecheck, Vitest and build; run Rust fmt, clippy
   with warnings denied and tests. Rebuild and repeat the current Brand/proxy
   smoke plus cross-Skill and startup-layout checks.

## Risk and rollback points

- `src/App.tsx`: reset at both Skill and file selection; do not introduce a
  per-Skill/document result cache.
- `src/styles.css`: explicit column placement must not disturb status-row layout.
- `src-tauri/src/translation.rs`: retry must share the existing deadline and
  never retry parsing/validation failures or publish partial batches.
- If the local HTTP test shows the retry seam needs broad client abstraction,
  stop and keep the smaller request-level seam.

## Validation commands

```bash
npm test -- --run src/App.test.tsx
npm run format:check
npm run lint
npm run typecheck
npm test -- --run
npm run build
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
```
