# Design: Agent projection contract smoke tests

## Test boundary

Add `src-tauri/src/projection_contract_smoke.rs` behind `#[cfg(test)]` in the crate root. It orchestrates existing `LibraryManager`, `InstallManager`, inventory reconciliation, `ConfigurationManager`, `RevisionManager`, and state storage against one `TempDir` per test.

No new runtime abstraction or dependency is introduced. Existing internal root-injection seams are reused. Where module privacy prevents orchestration, add the smallest `#[cfg(test)] pub(crate)` wrapper so the callable surface does not exist in production builds; if a wrapper starts copying policy, expose one narrow injected-root domain seam instead.

## Fixture layout

```text
<temp>/
  app-data/
  source/<skill>/SKILL.md
  home/.agents/skills/                 # Codex user root
  codex-home/config.toml               # Codex configuration
  claude-home/skills/                  # Claude user root
  claude-home/settings.json            # Claude configuration
  retarget/<skill>/SKILL.md             # valid unknown target
```

`AgentRoots` points at these paths directly. Tests never set process environment variables.

## Scenarios

### Codex round trip

1. Seed unrelated TOML.
2. Import a Local Skill Source into Managed Library.
3. Install to Codex using the real preferred native link primitive.
4. Reconcile `healthy`; disable and verify the real `[[skills.config]]` shape plus unrelated TOML.
5. Modify the owned block externally; reconcile `configuration_drift`; Reapply; reconcile `healthy`.
6. Remove the logical entry; reconcile `missing`; Restore; reconcile `healthy`.

### Claude round trip

Repeat the same minimum complete lifecycle with `settings.json.skillOverrides`, preserving unrelated JSON: Import, native linked Install, Healthy, Configuration Drift, Resolve, Missing, Restore, Healthy. A couple of local setup/assertion helpers may be shared; do not build a generic fixture framework.

For both hosts, parse and assert semantic entries and ownership markers/shapes. Do not snapshot full TOML/JSON strings or require irrelevant whitespace/key ordering.

### Retargeting boundary

Use a separate short test: create a real linked Installation, redirect it to a valid external Skill, reconcile `retargeted`, Forget Installation, then observe the entry as External. Assert the external target bytes are unchanged throughout. Keeping this separate prevents the primary Healthy round trips from ending in a deliberately unmanaged state.

## CI

Before each package job's `npm run tauri build`, run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml --all-features projection_contract_smoke
```

This command must succeed before `tauri build`, so projection failures block artifact production. It uses each native runner's filesystem implementation but is not a substitute for installing and launching the produced artifact. On macOS it does not claim that the test binary ran as both architectures of the universal artifact.

The backend code-spec cites the official Codex Skills/config and Claude Skills/settings/environment documentation with a `last reviewed: 2026-08-11` note. Tests never fetch those sources; release preparation rechecks them manually.

## Safety and rollback

- Every path is below a `TempDir`; no global env mutation or real Agent root access.
- Existing focused tests remain responsible for root environment parsing, Copy Fallback drift, and Managed Library damage.
- Tests use the same plan/commit and reconciliation code as the app.
- Test-only wrappers do not exist in production compilation and do not alter serialized APIs or persistent state.
- Failure leaves only temporary files removed by `TempDir`.
