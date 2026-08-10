# Implementation Plan

## Ordered Checklist

1. Scaffold the minimum Tauri 2 + React + TypeScript application; add formatting, linting, Rust tests and frontend tests.
2. Define shared command DTOs and structured error codes; keep raw filesystem/Git payloads out of React.
3. Implement the Rust `SkillManager` inventory, global normalized-name uniqueness and Agent Skills validator with temporary-home fixtures for Windows/macOS/Linux path shapes.
4. Implement the versioned JSON state, single-writer mutation lock, Managed Library, Resource Boundary policy, content fingerprinting, current/previous revisions, symlink/junction inspection and cancellable-staging/transactional link/copy/rollback helpers.
5. Implement Codex and Claude adapters, including OS-home/CODEX_HOME/CLAUDE_CONFIG_DIR resolution, bounded path overrides, legacy discovery, structure-preserving enable/disable edits, Configuration Provenance and cleanup tests.
6. Implement zero-target Add to Library, local snapshot import/replacement/export, Link Preferred installation, explicit Copy Fallback, grouped third-party-link adoption, two-step Legacy Migration, drift restore/detach, Uninstall and protected Remove from Library end to end.
7. Add public HTTPS Git import/update with `git2`, explicit subpath selection, Source Unreachable/Missing/Diverged classification, immutable source coordinates, name invariants, staging validation and all-Installation rollback.
8. Build the bilingual inventory, single-package import, detail/actions, resource disclosure, revision rollback, configuration resolution, three-layer disclosure and local diagnostics export UI with accessible keyboard/focus/error behavior.
9. Add integration tests that exercise commands through temporary app-data/home directories; cover invalid input, collision, partial failure, external drift and recovery.
10. Add native CI jobs and packaging configuration for Windows x64 NSIS, macOS 12+ universal DMG and Ubuntu 22.04-baseline x86_64 AppImage.
11. Run the full quality gate and smoke-test artifacts on each target family before release claims.

## Validation Commands

- `npm run format:check`
- `npm run lint`
- `npm run typecheck`
- `npm test -- --run`
- `cargo fmt --check --manifest-path src-tauri/Cargo.toml`
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`
- `cargo test --manifest-path src-tauri/Cargo.toml --all-features`
- `npm run tauri build` on each native CI runner/target.

## Required Test Scenarios

- Valid and invalid Agent Skills frontmatter, mismatched directory names, oversized metadata, path traversal and symlink rejection.
- Codex/Claude discovery, config preservation, enable/disable round-trip and absence of one Agent directory.
- detected/override/resolved path precedence, fixed Codex user root, legacy Codex inventory and explicit no-WSL behavior.
- Pre-existing user/third-party configuration keeps its provenance through Adoption, Detach and Uninstall.
- Externally Controlled Configuration locks toggles; Configuration Drift requires explicit Reapply or Forget.
- Existing unmanaged Skill remains read-only; explicit adoption records the exact fingerprint without changing contents.
- Same-name/same-fingerprint External Installations can be jointly adopted only after confirmation; different fingerprints never merge.
- Managed Library rejects same-name packages across Sources; multi-Skill repositories import one selected subpath per transaction.
- Import starts with zero targets; Add to Library and each explicit target combination are covered.
- Healthy third-party symlink/junction topology is discovered without ownership inference; Adoption migrates content into Managed Library and never deletes an outside target.
- Two-target install succeeds atomically or rolls back all changes; same-name collision never overwrites.
- External edits block enable/update/uninstall writes that would destroy content.
- Restore overwrites only after diff confirmation; Detach preserves files; Uninstall and Remove from Library remain distinct.
- Linked Detach atomically becomes a standalone External Installation; link failure/fallback and broken/cyclic/outside links cover all three OS path shapes.
- Git equal/fast-forward/diverged/rewritten/unrelated/network-failure flows, including rollback after partial replacement.
- Deleted branch/subpath, renamed Skill and prohibited Source-change flows.
- Capability and change disclosures are deterministic and never emit safety labels or scores.
- All six fixed Resource Boundary thresholds, exact-boundary acceptance and one-unit-over rejection are displayed and tested; any breach aborts and cleans the transaction before Managed Library writes.
- No network request occurs during startup/inventory; explicit Git checks are the only network entry point.
- No telemetry/analytics/crash-upload dependency is present; local diagnostic export is explicit and previewable.
- Private Managed Library and state never mutate third-party lock files; successful writes emit the same restart fallback message.
- Missing Agent client/root is advisory; confirmed root creation and default-enabled installation are tested.
- `zh-CN` and `en` cover all core workflow/error keys, with system default and persisted override tests.
- Staging cancellation cleans up; cancellation is disabled during commit/rollback; concurrent mutations are rejected by the single-writer lock.
- Legacy Migration blocks current-root install until the external legacy entry disappears.
- Remove from Library requires zero Installations, preview/export, Local snapshot warning and exact-name confirmation, then removes current/previous/Source.
- Roll Back Revision atomically swaps current/previous across every Installation and supports one-step redo.
- Desktop uninstall packaging contains no hook that mutates Agent Installation, Agent configuration or app data.
- State version mismatch, valid-backup recovery, Read-only Recovery, Orphaned Package, stale staging and interrupted transaction recovery.

## Risky Files and Rollback Points

- Agent config writers: keep fixture-based round-trip tests and byte-preserving backups where practical.
- Transaction helper: centralize all directory replacement here; do not let commands perform ad-hoc copies/deletes.
- State schema: add `state_version` before the first persisted record and reject unknown future versions.
- Packaging/CI: change one target job at a time; application logic must remain testable without producing installers.

## Pre-start Checks

- Re-read current official Codex, Claude Code, Agent Skills and Tauri docs referenced by `research.md` if implementation begins substantially later.
- Confirm backend/frontend specs have been bootstrapped from actual code conventions before expanding beyond the initial scaffold.
- Do not add private Git authentication, a database, marketplace, auto-updater or generic Agent plugin system during MVP implementation.
