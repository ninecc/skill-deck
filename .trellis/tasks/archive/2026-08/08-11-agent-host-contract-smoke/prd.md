# Add Agent projection contract smoke tests

## Goal

Prove that Skill Deck's Codex and Claude Code Agent Projection Contracts work end to end across domain managers and native filesystem semantics, without claiming that either Agent runtime loaded or invoked a Skill.

## Background

- `docs/roadmap.md:21-30` leaves Codex/Claude Code contract smoke tests and three-platform manual installer smoke tests as the remaining R1 completion work. This task sharpens the former as projection compatibility; runtime recognition belongs to the latter.
- The preceding P0 task delivered active reconciliation and explicitly deferred host contract and installer smoke coverage.
- Existing Rust tests heavily exercise individual managers with `TempDir` and injected `AgentRoots`, but no single test follows Library import through install, inventory, configuration drift, external breakage, recovery, and refreshed reconciliation.
- `.github/workflows/ci.yml` runs the full Rust suite on Ubuntu, while native package jobs build Windows NSIS, macOS universal DMG, and Linux AppImage without running a focused native filesystem contract test.

## Requirements

### R1. Real host-shaped fixtures

- Codex fixture uses a temporary home shaped as `$HOME/.agents/skills`, a temporary `CODEX_HOME`, and the real Codex `config.toml` entry format.
- Claude fixture uses a temporary `CLAUDE_CONFIG_DIR/skills` root and the real `settings.json.skillOverrides` format.
- Tests inject roots directly; they do not mutate process-global `HOME`, `CODEX_HOME`, or `CLAUDE_CONFIG_DIR`, so parallel tests remain deterministic.
- Fixtures contain unrelated valid configuration and assert it survives every Skill Deck write.
- Configuration assertions are semantic: expected target entry, enabled/override value, Skill Deck-owned shape/marker, and preservation of unrelated fields. Tests do not snapshot whole TOML/JSON byte layout.
- Backend code-spec records the official Codex/Claude documentation sources and last-reviewed date for these encoded contracts; CI stays offline and release preparation owns source review.

### R2. Cross-module lifecycle smoke

- Exercise real managers and persisted state in this order: Local Skill Source import, linked installation, inventory reconciliation, configuration mutation, external breakage, recovery, and refreshed inventory.
- Both Agent Targets must reach `healthy`, detect externally changed Skill Deck-owned configuration as `configuration_drift`, resolve it, detect a missing Installation, Restore it, and return to `healthy`.
- Both Agent Targets run that same minimum complete lifecycle; coverage is not split so that one Agent tests only configuration while the other tests only installation recovery.
- Across the smoke suite, also prove linked retargeting is detected without touching the observed target.
- Recovery assertions inspect the resulting filesystem/configuration/state and final reconciliation, not only command success.
- Existing focused P0 regressions remain authoritative for host-neutral Copy Fallback Content Drift and Managed Library damage; this task does not duplicate them.

### R3. Native CI execution

- Add one focused `cargo test` command for the projection-contract smoke module to every native package matrix job before packaging; failure blocks artifact production.
- Windows executes the existing junction implementation; macOS and Linux execute symlink behavior.
- macOS proves the native runner's filesystem contract only; it does not claim that the test binary exercised both architectures in the universal bundle.
- The existing full Ubuntu quality suite remains unchanged and continues to run all Rust tests.

## Acceptance Criteria

- [x] Codex smoke uses `.agents/skills` plus `CODEX_HOME/config.toml`, preserves unrelated TOML, and completes install/config-drift/reapply/missing/restore/healthy round trips.
- [x] Claude smoke uses `CLAUDE_CONFIG_DIR/skills` plus `settings.json`, preserves unrelated JSON, and completes the equivalent round trip.
- [x] The suite detects resolvable link retargeting and proves the observed target remains unchanged.
- [x] Every repair is followed by inventory refresh asserting the expected reconciliation state.
- [x] No test changes global environment variables or writes outside its temporary directory.
- [x] Tests assert host configuration semantics without whole-file formatting snapshots.
- [x] Code-spec cites official host contract sources with a review date and requires release-time manual revalidation; smoke CI performs no network access.
- [x] Native Windows, macOS, and Ubuntu package jobs run the focused smoke test before `tauri build`.
- [x] Test orchestration uses only test-compiled wrappers or an existing narrow injected-root seam; production compilation gains no test-only callable surface.
- [x] Full frontend/backend quality gates pass without new dependencies.

## Out of Scope

- Installing, launching, or uninstalling NSIS/DMG/AppImage artifacts; those are the next three-platform manual smoke task.
- Proving that Codex or Claude Code actually discovers, loads, or invokes the projected Skill.
- Proving reliable Skill triggering or task success; those belong to R3 Revision-level Eval, not the next Agent Runtime Recognition smoke.
- GUI automation, a general fixture framework, third Agent Targets, R2 Capability Cards, or runtime execution of third-party Skill scripts.
- Revalidating official host documentation in CI or making network calls.

## Technical Notes

- Prefer one test-only `projection_contract_smoke` module and the smallest test-only seams needed to reuse existing managers.
- Production host resolution and persisted state formats remain unchanged.
- Do not mutate process-global environment or add subprocess/serialization machinery merely to retest root resolution; the following installer smoke task owns real desktop-process environment verification.
- The next installer smoke task proves Agent Runtime Recognition only. Model-dependent invocation and effectiveness remain R3 concerns.
- No blocking product decisions remain; this is the next independently verifiable roadmap item established in the prior task/session.
