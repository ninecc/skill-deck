# Native release evidence baseline

## Confirmed evidence

- The only recorded native installation, launch, Inventory, and Codex/Claude runtime-recognition smoke is macOS universal DMG evidence in `.trellis/tasks/archive/2026-08/08-11-macos-installer-smoke/evidence.md`.
- That smoke ran on 2026-08-11 against artifact commit `b9373b373c6e5bade8da4df84c633e654da086fe`, using Actions run `31466973404` and `Skill Deck_0.1.0_universal.dmg`.
- The evidence covers installed launch, Inventory, positive recognition in Codex/Claude, negative recognition after uninstall, and cleanup. It does not prove signing/notarization/default Gatekeeper behavior: the bundle was unsigned and the host had Gatekeeper disabled.
- Windows NSIS and Linux AppImage native installation/runtime-recognition smoke remain unverified.

## Applicability to current HEAD

- Current HEAD at planning time is `7d983f35979ded0551ed036eaa82ea3450dea669`.
- The tested macOS artifact commit is an ancestor but predates roughly 60 commits.
- Commit `7c69e26301f59a07de359ae5ce366d8180ae68cc` replaced the lifecycle backend with the upstream Skills CLI and deleted the old projection smoke/module structure. Therefore the 2026-08-11 macOS result is historical evidence, not current-HEAD native integration proof.

## CI boundary and stale gate

- `.github/workflows/ci.yml` defines Ubuntu format/lint/typecheck/frontend-test/build/Rust fmt/Clippy/test gates and a Windows/macOS/Linux packaging matrix. A green run can prove those exact automated jobs and artifact production for its checkout, but not native install, launch, Inventory, runtime recognition, removal recognition, cleanup, signing, notarization, or Gatekeeper behavior.
- The packaging workflow still filters Rust tests by `projection_contract_smoke`, but the matching module was deleted by the lifecycle rewrite. Running the exact filter on current HEAD exits successfully with `0 passed, 26 filtered out`, so this gate currently proves nothing about Agent Projection Contract behavior.
- `docs/release-smoke.md` explicitly requires independent native smoke on every platform and does not accept CI or another platform as a substitute.

## Audit consequence

Current-HEAD native release readiness is not established on any platform. The stale zero-test CI gate is a release-infrastructure defect and the absence of current integration evidence caps relevant integration-sensitive capability rows at partial under the agreed scoring contract.

