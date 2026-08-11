# macOS installer smoke and recognition

## Goal

Prove on the current Mac that a packaged Skill Deck DMG can be installed and
launched, and that Skills projected by the packaged application are discovered
by the installed Codex and Claude Code runtimes. Preserve an honest R1 status:
macOS may be recorded as passed, while Windows NSIS and Linux AppImage remain
open until they are exercised on those operating systems.

## Background

- `docs/roadmap.md` leaves three-platform installer smoke and Agent Runtime
  Recognition as the only unfinished R1 item.
- CI already runs `projection_contract_smoke` before producing Windows NSIS,
  macOS universal DMG, and Linux AppImage artifacts.
- ADR 0010 separates filesystem projection, runtime discovery, and
  model-dependent invocation/effectiveness. This task tests discovery only.
- The current machine is Apple Silicon on macOS 26.5.2. Codex CLI 0.147.0 and
  Claude Code 2.1.220 are installed. Skill Deck is not currently installed in
  `/Applications` or `~/Applications`.

## Requirements

### R1. Test the packaged macOS application

- Test the exact universal DMG produced by CI from the current commit. A local
  universal build may diagnose a missing or broken CI artifact but cannot satisfy
  the macOS pass criterion.
- Record the commit, artifact filename, SHA-256, bundle architectures, macOS
  version, and Skill Deck launch result.
- Mount the DMG, copy its application bundle to `/Applications`, and launch that
  installed bundle without disabling Gatekeeper or weakening host security
  settings. Refuse to overwrite an existing same-name application.
- Confirm that the packaged UI can load inventory and complete the smoke flow;
  a Vite development server or unpackaged Tauri binary does not satisfy this
  requirement.

### R2. Prove real Agent Runtime Recognition

- Create one uniquely named, inert local Skill fixture containing only a
  `SKILL.md`; it must not execute commands, use the network, or request secrets.
- Through the packaged Skill Deck UI, import the fixture and install it for both
  Codex and Claude Code using the default Linked Installation mode. Copy Fallback
  is not part of this manual smoke.
- Launch each installed Agent runtime and observe that its native Skill listing
  or completion UI exposes the unique Skill name. A model response claiming to
  see the Skill is not evidence. Do not ask a model to execute the Skill and do
  not treat invocation or task success as evidence.
- Uninstall the fixture from both Agent Targets through Skill Deck and confirm
  that fresh Agent runtimes no longer expose it, then Remove from Library so no
  test package remains in the Managed Library.
- Use a unique name and touch only test-owned entries. Preserve unrelated Skill
  directories and unrelated Codex/Claude configuration.

### R3. Preserve and report platform boundaries

- Add one concise, reusable release-smoke checklist covering NSIS, DMG, and
  AppImage packaging, application launch, Skill projection, runtime discovery,
  removal, cleanup, and evidence recording.
- Record macOS observations in task evidence, including failures or security
  prompts rather than bypassing them silently.
- Split the combined roadmap item into macOS, Windows, and Linux subitems. Check
  macOS only if the packaged application, Codex recognition, and Claude Code
  recognition all pass; keep Windows and Linux unchecked.
- Do not claim Windows or Linux results from CI projection tests or from macOS.

### R4. Cleanup and ownership safety

- Capture relevant pre-smoke paths/configuration before mutation and compare them
  after cleanup.
- Real user-level Codex and Claude Skill roots/configuration may be changed only
  through the uniquely named test fixture and must be returned to their prior
  unrelated state.
- Unmount the DMG and remove only the application bundle and Skill fixture made
  by this task.
- If cleanup cannot be proven safe, stop and report the exact residual state;
  never replace an entire user configuration file to remove a test-owned entry.

## Acceptance Criteria

- [x] The tested packaged DMG is tied to a commit and SHA-256 and its bundle
      architectures are recorded.
- [x] The installed packaged application launches and loads inventory on the
      current Mac.
- [x] Codex and Claude Code each expose the unique Skill after installation by
      packaged Skill Deck.
- [x] After removal, fresh Codex and Claude Code runtimes no longer expose the
      unique Skill.
- [x] Unrelated Agent Skills and configuration survive unchanged, and all
      test-owned files/mounts/application bundles are removed or explicitly
      reported as residual.
- [x] A three-platform manual release-smoke checklist exists.
- [x] Task evidence records the environment, artifact, observations, cleanup,
      and the fact that Windows/Linux remain unverified.
- [x] `docs/roadmap.md` exposes separate platform status; macOS is checked only
      when the application and both Agent runtimes pass, while Windows and Linux
      remain unchecked.

## Out of Scope

- Windows NSIS or Linux AppImage execution on this Mac.
- Automating GUI smoke tests or adding a general release-test framework.
- Skill invocation, trigger quality, task success, cost, latency, or any R3 Eval
  claim.
- Code signing, notarization, publishing a release, changing Agent discovery
  contracts, or changing product behavior unless the smoke exposes a blocking
  defect that is separately planned. A standard macOS user confirmation may
  allow the test to proceed, but disabling Gatekeeper or removing quarantine is
  prohibited; an unsigned/unnotarized artifact is recorded as a separate
  distribution-security gap rather than an application smoke failure.
- Fixing a product defect discovered by the smoke. Record the failure, stop, and
  plan a separate defect task; after its fix is packaged by CI, rerun this smoke
  against the new immutable artifact.
- Manual Copy Fallback recognition coverage; existing deterministic coverage
  remains authoritative unless a separate runtime risk is demonstrated.

## Technical Notes

- Prefer native runtime listing/completion evidence over a model response.
- The installed app and real user-level Agent roots are required for recognition,
  so execution will need explicit system/UI approvals when requested by the host.
- After verification, remove the installed application, test Skill, and mounted
  image; retain only redacted textual evidence.
- No product or scope questions remain. The user chose macOS execution plus a
  Windows/Linux checklist, with those platforms deliberately left incomplete.
